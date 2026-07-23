//! Production dispatch for metrics migrating off `rust-code-analysis`, plus a
//! test-only rca-parity harness (see the migration epic).
//!
//! A metric can be computed two ways — natively over a raw tree-sitter tree, or
//! via rca. [`use_native`] / [`MIGRATED`] decide which the production report uses
//! for a given language; [`file_level_metrics`] and [`function_args_values`] are
//! the production entry points. A metric only joins `MIGRATED` once the native
//! and rca results are proven identical on a corpus — that proof lives in the
//! `#[cfg(test)]` parity oracle at the bottom of this file.

use rust_code_analysis::FuncSpace;

use crate::collectors::structural::{args_for, cyclomatic_for, function_count_for, function_entity_name, sloc_for, visit_function_spaces};
use crate::language::Language;
use crate::native;

/// A structural metric being migrated from rca to the native path.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Metric {
    /// Source lines of code for a whole file — rca `loc.sloc()` on the unit.
    FileLines,
    /// Number of functions in a file — rca `nom.total()` (functions + closures).
    FileFunctions,
    /// Argument count per function — rca `max(fn_args, closure_args)`.
    FunctionArgs,
    /// Cyclomatic complexity per function — rca `cyclomatic_sum` (subtree sum).
    FunctionCyclomatic,
}

/// Which implementation computes a metric.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Backend {
    Rca,
    Native,
}

/// Metrics whose native implementation has reached rca parity and drives the
/// production report. Grows as each migration lands.
pub const MIGRATED: &[Metric] = &[Metric::FileLines, Metric::FileFunctions, Metric::FunctionArgs, Metric::FunctionCyclomatic];

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

/// Whether production should compute `metric` for `lang` via the native path: it
/// must be migrated *and* the language natively supported (Rust only today —
/// other languages still route through rca until their grammars are vendored).
pub fn use_native(metric: Metric, lang: Language) -> bool {
    metric.backend() == Backend::Native && lang == Language::Rust
}

/// File-level metric values `(file_lines, file_functions)` for one already-parsed
/// file, dispatching each metric to the native path when migrated for `lang`
/// (Rust) and to rca otherwise. `source` is the same (test-stripped) bytes rca
/// parsed into `top`; native falls back to rca only on a native parse failure.
pub fn file_level_metrics(lang: Language, source: &[u8], top: &FuncSpace) -> (u64, u64) {
    let file_lines = if use_native(Metric::FileLines, lang) { native::rust_file_lines(source).unwrap_or_else(|| sloc_for(top)) } else { sloc_for(top) };
    let file_functions = if use_native(Metric::FileFunctions, lang) {
        native::rust_file_functions(source).unwrap_or_else(|| function_count_for(top))
    } else {
        function_count_for(top)
    };
    (file_lines, file_functions)
}

/// Per-function values `(entity_name, value)` in walk order for a function-level
/// `metric` on one already-parsed file — via the native path when the metric is
/// migrated for `lang`, rca otherwise. Entity names are bare (the caller prefixes
/// the file path), matching the rca function loop in `structural.rs`.
pub fn function_metric_values(metric: Metric, lang: Language, source: &[u8], top: &FuncSpace) -> Vec<(String, u64)> {
    if use_native(metric, lang) {
        native_function_metric(metric, source)
    } else {
        rca_function_metric(metric, top)
    }
}

/// Native per-function values for a function-level `metric`.
fn native_function_metric(metric: Metric, source: &[u8]) -> Vec<(String, u64)> {
    match metric {
        Metric::FunctionArgs => native::rust_function_nargs(source),
        Metric::FunctionCyclomatic => native::rust_function_cyclomatic(source),
        other => panic!("{other:?} is not a native function-level metric"),
    }
}

/// rca per-function values for a function-level `metric`, in walk order.
fn rca_function_metric(metric: Metric, top: &FuncSpace) -> Vec<(String, u64)> {
    let per_space: fn(&FuncSpace) -> u64 = match metric {
        Metric::FunctionArgs => args_for,
        Metric::FunctionCyclomatic => cyclomatic_for,
        other => panic!("{other:?} is not a function-level metric"),
    };
    let mut out = Vec::new();
    let mut closure_counter: u32 = 0;
    visit_function_spaces(top, &mut |space| out.push((function_entity_name(space, &mut closure_counter), per_space(space))));
    out
}

#[cfg(test)]
mod tests {
    //! The rca-parity oracle: computes each metric both ways and asserts they
    //! agree. Test-only — production never computes rca and native side by side.
    use std::collections::BTreeMap;
    use std::path::Path;

    use super::*;

    /// Raw (pre-threshold) metric values keyed by entity (a file path for
    /// file-level metrics; `path::name` for function-level ones).
    type EntityValues = BTreeMap<String, u64>;

    /// Compute `metric` for a single Rust `source` file via `backend`, as an
    /// entity→value map. Function-level metrics key each entity as `path::name`
    /// (colliding closures collapse, exactly as the production report's map does).
    fn compute(metric: Metric, backend: Backend, source: &[u8], path: &Path) -> EntityValues {
        let rca_top = || Language::Rust.parse_metrics(source.to_vec(), path);
        match metric {
            Metric::FileLines => single(
                path,
                match backend {
                    Backend::Rca => rca_top().map(|t| t.metrics.loc.sloc().round() as u64),
                    Backend::Native => native::rust_file_lines(source),
                },
            ),
            Metric::FileFunctions => single(
                path,
                match backend {
                    Backend::Rca => rca_top().map(|t| t.metrics.nom.total().round() as u64),
                    Backend::Native => native::rust_file_functions(source),
                },
            ),
            Metric::FunctionArgs | Metric::FunctionCyclomatic => keyed_by_path(
                path,
                match backend {
                    Backend::Rca => rca_top().map(|t| rca_function_metric(metric, &t)).unwrap_or_default(),
                    Backend::Native => native_function_metric(metric, source),
                },
            ),
        }
    }

    /// A one-entry map `{path: value}`, or empty when the source failed to parse.
    fn single(path: &Path, value: Option<u64>) -> EntityValues {
        value.into_iter().map(|v| (path.display().to_string(), v)).collect()
    }

    /// Turn a walk-ordered `(name, value)` list into a map keyed `path::name`.
    fn keyed_by_path(path: &Path, entities: Vec<(String, u64)>) -> EntityValues {
        entities.into_iter().map(|(name, value)| (format!("{}::{name}", path.display()), value)).collect()
    }

    /// Compare native and rca for `metric` on one file; `Ok` if they agree, else
    /// `Err` with a per-entity divergence report.
    fn check_parity(metric: Metric, source: &[u8], path: &Path) -> Result<(), String> {
        let rca = compute(metric, Backend::Rca, source, path);
        let native = compute(metric, Backend::Native, source, path);
        match diff(metric, &rca, &native) {
            None => Ok(()),
            Some(report) => Err(report),
        }
    }

    /// Report the entities where `rca` and `native` disagree, or `None` if equal.
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

    /// Run `check_parity(metric, ..)` over every Rust file in the repo (ratchet's
    /// own `src/` plus the fixtures), panicking on the first divergence.
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

    /// Ordered function entity names rca produces — the walk-parity oracle.
    fn rca_function_entities(top: &FuncSpace) -> Vec<String> {
        let mut out = Vec::new();
        let mut closure_counter: u32 = 0;
        visit_function_spaces(top, &mut |space| out.push(function_entity_name(space, &mut closure_counter)));
        out
    }

    /// Assert the native function walk matches rca's function list on `source`.
    fn assert_function_walk_parity(source: &[u8], path: &Path) {
        let rca = Language::Rust.parse_metrics(source.to_vec(), path).map(|top| rca_function_entities(&top)).unwrap_or_default();
        let native = native::rust_function_entities(source);
        assert_eq!(native, rca, "function-walk divergence in {}", path.display());
    }

    #[test]
    fn test_use_native_requires_migrated_metric_and_supported_language() {
        assert!(use_native(Metric::FileLines, Language::Rust));
        // Never native for a language whose grammar isn't vendored yet.
        assert!(!use_native(Metric::FileLines, Language::Python));
    }

    #[test]
    fn test_native_file_lines_matches_rca_on_a_snippet() {
        let src = b"fn add(a: i32, b: i32) -> i32 {\n    a + b\n}\n";
        let path = Path::new("snippet.rs");
        let rca = compute(Metric::FileLines, Backend::Rca, src, path);
        assert!(!rca.is_empty(), "rca should produce a file_lines value");
        assert_eq!(compute(Metric::FileLines, Backend::Native, src, path), rca);
    }

    #[test]
    fn test_native_file_functions_matches_rca_on_a_snippet() {
        let src = b"fn a() {}\nfn b() { let c = || 0; }\nstruct S;\nimpl S { fn m(&self) {} }\n";
        let path = Path::new("snippet.rs");
        let rca = compute(Metric::FileFunctions, Backend::Rca, src, path);
        assert!(!rca.is_empty(), "rca should produce a file_functions value");
        assert_eq!(compute(Metric::FileFunctions, Backend::Native, src, path), rca);
    }

    #[test]
    fn test_native_nargs_counts_self_params_and_closure_args() {
        // `self` counts; a 2-arg method is 3; the closure `|x|` is 1; no-arg fn is 0.
        let src = b"struct S;\nimpl S { fn m(&self, a: i32, b: u8) { let c = |x: i32| x; } }\nfn n() {}\n";
        assert_eq!(native::rust_function_nargs(src), vec![("m".to_string(), 3), ("<anonymous>".to_string(), 1), ("n".to_string(), 0)]);
    }

    #[test]
    fn test_native_cyclomatic_sums_subtree_including_nested_closures() {
        // base 1 each; `&&` counts; `nested` folds in its closure (2) → 4.
        let src = b"fn simple() {}\nfn one_if(x: bool) { if x {} }\nfn two(a: bool, b: bool) { if a && b {} }\nfn nested() { let c = || { if true {} }; if false {} }\n";
        assert_eq!(
            native::rust_function_cyclomatic(src),
            vec![("simple".to_string(), 1), ("one_if".to_string(), 2), ("two".to_string(), 3), ("nested".to_string(), 4), ("<anonymous>".to_string(), 2)]
        );
    }

    #[test]
    fn test_function_cyclomatic_parity_over_repo_corpus() {
        assert_metric_parity_over_corpus(Metric::FunctionCyclomatic);
    }

    #[test]
    fn test_diff_reports_each_divergent_entity_with_both_values() {
        let rca = EntityValues::from([("a.rs".to_string(), 10), ("b.rs".to_string(), 20)]);
        let native = EntityValues::from([("a.rs".to_string(), 10), ("b.rs".to_string(), 22)]);
        let report = diff(Metric::FileLines, &rca, &native).expect("should report a divergence");
        assert!(report.contains("b.rs") && report.contains("rca=Some(20)") && report.contains("native=Some(22)"));
        assert!(!report.contains("a.rs"), "agreeing entities are omitted");
    }

    #[test]
    fn test_diff_returns_none_when_maps_agree() {
        let m = EntityValues::from([("a.rs".to_string(), 10)]);
        assert!(diff(Metric::FileLines, &m, &m).is_none());
    }

    #[test]
    fn test_file_lines_parity_over_repo_corpus() {
        assert_metric_parity_over_corpus(Metric::FileLines);
    }

    #[test]
    fn test_file_functions_parity_over_repo_corpus() {
        assert_metric_parity_over_corpus(Metric::FileFunctions);
    }

    #[test]
    fn test_function_args_parity_over_repo_corpus() {
        assert_metric_parity_over_corpus(Metric::FunctionArgs);
    }

    #[test]
    fn test_function_walk_parity_on_tricky_constructs() {
        // Trait signature (excluded) vs defaulted method (included), module, async, generic.
        let src = b"trait T {\n    fn required(&self);\n    fn defaulted(&self) { let c = || 0; }\n}\nmod inner { pub fn nested() {} }\nasync fn a() {}\nfn generic<X>(x: X) -> X { x }\n";
        assert_function_walk_parity(src, Path::new("tricky.rs"));
    }

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
            assert_function_walk_parity(&source, path);
            checked += 1;
        }
        assert!(checked > 5, "expected to check several Rust files, checked {checked}");
    }
}
