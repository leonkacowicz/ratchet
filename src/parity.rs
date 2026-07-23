//! Dual-path metric computation and an rca-parity harness for migrating metrics
//! off `rust-code-analysis` one at a time (see the migration epic).
//!
//! During migration a metric can be computed two ways — natively over a raw
//! tree-sitter tree, or via rca — and it is only switched to the native path in
//! production once the two agree on a corpus ([`check_parity`]). The production
//! report does not consult the selector yet; that wiring lands with the Rust
//! cutover, so these items are currently exercised only by tests.
#![allow(dead_code)]

use std::collections::BTreeMap;
use std::path::Path;

use rust_code_analysis::FuncSpace;

use crate::collectors::structural::{function_entity_name, visit_function_spaces};
use crate::language::Language;
use crate::native;

/// Raw (pre-threshold) metric values keyed by entity (a file path for
/// file-level metrics; later `path::fn` for function-level ones).
pub type EntityValues = BTreeMap<String, u64>;

/// A structural metric being migrated from rca to the native path.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Metric {
    /// Source lines of code for a whole file — rca `loc.sloc()` on the unit.
    FileLines,
    /// Number of functions in a file — rca `nom.total()` (functions + closures).
    FileFunctions,
}

/// Which implementation computes a metric.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Backend {
    Rca,
    Native,
}

/// Metrics whose native implementation has reached rca parity and may drive the
/// production report. Grows as each migration lands.
pub const MIGRATED: &[Metric] = &[];

impl Metric {
    /// The backend that should compute this metric in production: the native
    /// path once migrated, otherwise rca.
    pub fn backend(self) -> Backend {
        if MIGRATED.contains(&self) {
            Backend::Native
        } else {
            Backend::Rca
        }
    }
}

/// Compute `metric` for a single Rust `source` file via `backend`, as an
/// entity→value map. Empty when the source fails to parse.
pub fn compute(metric: Metric, backend: Backend, source: &[u8], path: &Path) -> EntityValues {
    match (metric, backend) {
        (Metric::FileLines, Backend::Rca) => rca_file_lines(source, path),
        (Metric::FileLines, Backend::Native) => native_file_lines(source, path),
        (Metric::FileFunctions, Backend::Rca) => rca_file_functions(source, path),
        (Metric::FileFunctions, Backend::Native) => native_file_functions(source, path),
    }
}

/// Compare the native and rca implementations of `metric` for one Rust file.
/// Returns `Ok(())` when they agree, otherwise `Err` with a per-entity report.
pub fn check_parity(metric: Metric, source: &[u8], path: &Path) -> Result<(), String> {
    let rca = compute(metric, Backend::Rca, source, path);
    let native = compute(metric, Backend::Native, source, path);
    match diff(metric, &rca, &native) {
        None => Ok(()),
        Some(report) => Err(report),
    }
}

/// Build a human-readable report of the entities where `rca` and `native`
/// disagree, or `None` if they are identical. Agreeing entities are omitted.
fn diff(metric: Metric, rca: &EntityValues, native: &EntityValues) -> Option<String> {
    if rca == native {
        return None;
    }
    let mut lines = Vec::new();
    let mut entities: Vec<&String> = rca.keys().chain(native.keys()).collect();
    entities.sort_unstable();
    entities.dedup();
    for entity in entities {
        let (r, n) = (rca.get(entity), native.get(entity));
        if r != n {
            lines.push(format!("  {entity}: rca={r:?} native={n:?}"));
        }
    }
    Some(format!("{metric:?} parity divergence:\n{}", lines.join("\n")))
}

/// Compare the native tree-sitter function walk against rca's for one Rust
/// file: same functions, same names, same order. `Ok(())` when identical,
/// otherwise `Err` with both ordered lists for debugging.
pub fn check_function_walk_parity(source: &[u8], path: &Path) -> Result<(), String> {
    let rca = Language::Rust.parse_metrics(source.to_vec(), path).map(|top| rca_function_entities(&top)).unwrap_or_default();
    let native = native::rust_function_entities(source);
    if rca == native {
        Ok(())
    } else {
        Err(format!("function-walk divergence in {}:\n  rca   ={rca:?}\n  native={native:?}", path.display()))
    }
}

/// rca's ordered function entity names for a parsed file — the oracle the native
/// walk is checked against. Reuses rca's production naming (`visit_function_spaces`
/// + `function_entity_name`) so the two implementations cannot drift.
fn rca_function_entities(top: &FuncSpace) -> Vec<String> {
    let mut out = Vec::new();
    let mut closure_counter: u32 = 0;
    visit_function_spaces(top, &mut |space| out.push(function_entity_name(space, &mut closure_counter)));
    out
}

fn rca_file_lines(source: &[u8], path: &Path) -> EntityValues {
    let mut out = EntityValues::new();
    if let Some(top) = Language::Rust.parse_metrics(source.to_vec(), path) {
        out.insert(path.display().to_string(), top.metrics.loc.sloc().round() as u64);
    }
    out
}

fn native_file_lines(source: &[u8], path: &Path) -> EntityValues {
    let mut out = EntityValues::new();
    if let Some(tree) = native::parse_rust(source) {
        let root = tree.root_node();
        // Mirrors rca's file SLOC: `end_row - start_row` of the root node
        // (rca `Sloc` unit branch). Same grammar + runtime → same rows.
        let lines = (root.end_position().row - root.start_position().row) as u64;
        out.insert(path.display().to_string(), lines);
    }
    out
}

fn rca_file_functions(source: &[u8], path: &Path) -> EntityValues {
    let mut out = EntityValues::new();
    if let Some(top) = Language::Rust.parse_metrics(source.to_vec(), path) {
        out.insert(path.display().to_string(), top.metrics.nom.total().round() as u64);
    }
    out
}

fn native_file_functions(source: &[u8], path: &Path) -> EntityValues {
    let mut out = EntityValues::new();
    if let Some(tree) = native::parse_rust(source) {
        // rca's `nom.total()` counts exactly the Function-kind spaces
        // (function_item + closure_expression) — the ones the walk emits.
        let mut count = 0u64;
        native::visit_rust_functions(&tree, source, &mut |_name, _node| count += 1);
        out.insert(path.display().to_string(), count);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_backend_defaults_to_rca_until_metric_is_migrated() {
        // MIGRATED is empty, so production still resolves every metric to rca;
        // a metric moves to Native by being added there once its parity is green.
        assert_eq!(Metric::FileLines.backend(), Backend::Rca);
    }

    #[test]
    fn test_native_file_lines_matches_rca_on_a_snippet() {
        let src = b"fn add(a: i32, b: i32) -> i32 {\n    a + b\n}\n";
        let path = Path::new("snippet.rs");
        let rca = compute(Metric::FileLines, Backend::Rca, src, path);
        let native = compute(Metric::FileLines, Backend::Native, src, path);
        assert!(!rca.is_empty(), "rca should produce a file_lines value");
        assert_eq!(native, rca, "native file_lines must match rca");
    }

    #[test]
    fn test_native_file_functions_matches_rca_on_a_snippet() {
        let src = b"fn a() {}\nfn b() { let c = || 0; }\nstruct S;\nimpl S { fn m(&self) {} }\n";
        let path = Path::new("snippet.rs");
        let rca = compute(Metric::FileFunctions, Backend::Rca, src, path);
        let native = compute(Metric::FileFunctions, Backend::Native, src, path);
        assert!(!rca.is_empty(), "rca should produce a file_functions value");
        assert_eq!(native, rca, "native file_functions must match rca");
    }

    #[test]
    fn test_diff_reports_each_divergent_entity_with_both_values() {
        let rca = EntityValues::from([("a.rs".to_string(), 10), ("b.rs".to_string(), 20)]);
        let native = EntityValues::from([("a.rs".to_string(), 10), ("b.rs".to_string(), 22)]);
        let report = diff(Metric::FileLines, &rca, &native).expect("should report a divergence");
        assert!(report.contains("b.rs"), "names the divergent entity");
        assert!(report.contains("rca=Some(20)"), "shows the rca value");
        assert!(report.contains("native=Some(22)"), "shows the native value");
        assert!(!report.contains("a.rs"), "agreeing entities are omitted");
    }

    #[test]
    fn test_diff_returns_none_when_maps_agree() {
        let m = EntityValues::from([("a.rs".to_string(), 10)]);
        assert!(diff(Metric::FileLines, &m, &m).is_none());
    }

    /// Run `check_parity(metric, ..)` over every Rust file in the repo (ratchet's
    /// own `src/` plus the fixtures), panicking on the first divergence report.
    fn assert_metric_parity_over_corpus(metric: Metric) {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let mut checked = 0;
        for entry in walkdir::WalkDir::new(root.join("src")).into_iter().chain(walkdir::WalkDir::new(root.join("tests/fixtures"))).filter_map(Result::ok) {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("rs") {
                continue;
            }
            let source = std::fs::read(path).unwrap_or_else(|e| panic!("reading {}: {e}", path.display()));
            if let Err(report) = check_parity(metric, &source, path) {
                panic!("{report}");
            }
            checked += 1;
        }
        assert!(checked > 5, "expected to check several Rust files, checked {checked}");
    }

    /// The parity guarantee that lets `file_lines` flip to the native path:
    /// native matches rca on every real Rust file in the repo.
    #[test]
    fn test_file_lines_parity_over_repo_corpus() {
        assert_metric_parity_over_corpus(Metric::FileLines);
    }

    /// Same guarantee for `file_functions` (function count per file).
    #[test]
    fn test_file_functions_parity_over_repo_corpus() {
        assert_metric_parity_over_corpus(Metric::FileFunctions);
    }

    /// Parity on constructs the repo corpus may not exercise: a trait with a
    /// body-less signature (excluded) plus a defaulted method (included), a
    /// module, an async fn, and a generic fn. Checked against the rca oracle.
    #[test]
    fn test_function_walk_parity_on_tricky_constructs() {
        let src = b"trait T {\n    fn required(&self);\n    fn defaulted(&self) { let c = || 0; }\n}\nmod inner { pub fn nested() {} }\nasync fn a() {}\nfn generic<X>(x: X) -> X { x }\n";
        if let Err(report) = check_function_walk_parity(src, Path::new("tricky.rs")) {
            panic!("{report}");
        }
    }

    /// The native function-space walk must enumerate and name the same functions
    /// rca does, in the same order, on every real Rust file in the repo.
    #[test]
    fn test_function_walk_parity_over_repo_corpus() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let mut checked = 0;
        for entry in walkdir::WalkDir::new(root.join("src")).into_iter().chain(walkdir::WalkDir::new(root.join("tests/fixtures"))).filter_map(Result::ok) {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("rs") {
                continue;
            }
            let source = std::fs::read(path).unwrap_or_else(|e| panic!("reading {}: {e}", path.display()));
            if let Err(report) = check_function_walk_parity(&source, path) {
                panic!("{report}");
            }
            checked += 1;
        }
        assert!(checked > 5, "expected to check several Rust files, checked {checked}");
    }
}
