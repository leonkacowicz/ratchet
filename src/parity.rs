//! Production dispatch for metrics migrating off `rust-code-analysis`, plus a
//! test-only rca-parity harness (see the migration epic).
//!
//! A metric can be computed two ways — natively over a raw tree-sitter tree, or
//! via rca. [`MIGRATED`] and `native::supports` decide which the production report uses
//! for a given language; [`file_level_metrics`] and [`function_metric_values`] are
//! the production entry points. Fully-migrated languages (Rust today) never parse
//! through rca — their `top` is `None`. A metric only joins `MIGRATED` once the
//! native and rca results are proven identical on a corpus — that proof lives in
//! the `#[cfg(test)]` parity oracle at the bottom of this file.

use rust_code_analysis::FuncSpace;

use crate::collectors::structural::{args_for, cognitive_for, cyclomatic_for, function_count_for, function_entity_name, sloc_for, visit_function_spaces};
use crate::native::Analysis;

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
    /// Source lines per function — rca `loc.sloc()` on the function space.
    FunctionLines,
    /// Cognitive complexity per function — rca `cognitive_sum` (subtree sum).
    FunctionCognitive,
}

/// Which implementation computes a metric.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Backend {
    Rca,
    Native,
}

/// Metrics whose native implementation has reached rca parity and drives the
/// production report. Grows as each migration lands.
pub const MIGRATED: &[Metric] =
    &[Metric::FileLines, Metric::FileFunctions, Metric::FunctionArgs, Metric::FunctionCyclomatic, Metric::FunctionLines, Metric::FunctionCognitive];

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

/// File-level metric values `(file_lines, file_functions)` for one file. `native`
/// is the language's parsed [`Analysis`] when it has a native implementation, and
/// `top` the rca parse otherwise — exactly one of the two is `Some`. Note this
/// takes no `Language`: the dispatch already happened.
pub fn file_level_metrics(native: Option<&Analysis>, top: Option<&FuncSpace>) -> (u64, u64) {
    let file_lines = match native {
        Some(a) if Metric::FileLines.backend() == Backend::Native => a.file_lines(),
        _ => top.map_or(0, sloc_for),
    };
    let file_functions = match native {
        Some(a) if Metric::FileFunctions.backend() == Backend::Native => a.file_functions(),
        _ => top.map_or(0, function_count_for),
    };
    (file_lines, file_functions)
}

/// Per-function values `(entity_name, value)` in walk order for a function-level
/// `metric` — from the native [`Analysis`] when the metric is migrated, rca
/// otherwise. Entity names are bare (the caller prefixes the file path), matching
/// the rca function loop in `structural.rs`.
pub fn function_metric_values(metric: Metric, native: Option<&Analysis>, top: Option<&FuncSpace>) -> Vec<(String, u64)> {
    match native {
        Some(a) if metric.backend() == Backend::Native => native_function_metric(metric, a),
        _ => top.map_or_else(Vec::new, |t| rca_function_metric(metric, t)),
    }
}

/// Native per-function values for a function-level `metric`, off an already
/// parsed [`Analysis`].
fn native_function_metric(metric: Metric, native: &Analysis) -> Vec<(String, u64)> {
    match metric {
        Metric::FunctionArgs => native.function_nargs(),
        Metric::FunctionCyclomatic => native.function_cyclomatic(),
        Metric::FunctionLines => native.function_lines(),
        Metric::FunctionCognitive => native.function_cognitive(),
        other => panic!("{other:?} is not a native function-level metric"),
    }
}

/// rca per-function values for a function-level `metric`, in walk order.
fn rca_function_metric(metric: Metric, top: &FuncSpace) -> Vec<(String, u64)> {
    let per_space: fn(&FuncSpace) -> u64 = match metric {
        Metric::FunctionArgs => args_for,
        Metric::FunctionCyclomatic => cyclomatic_for,
        Metric::FunctionLines => sloc_for,
        Metric::FunctionCognitive => cognitive_for,
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
    use crate::language::Language;
    use crate::native;

    /// Raw (pre-threshold) metric values keyed by entity (a file path for
    /// file-level metrics; `path::name` for function-level ones).
    type EntityValues = BTreeMap<String, u64>;

    /// One file under test: its language, bytes and display path.
    #[derive(Clone, Copy)]
    struct Unit<'a> {
        lang: Language,
        source: &'a [u8],
        path: &'a Path,
    }

    /// Compute `metric` for one file via `backend`, as an entity→value map.
    /// Function-level metrics key each entity as `path::name` (colliding closures
    /// collapse, exactly as the production report's map does).
    fn compute(metric: Metric, backend: Backend, unit: Unit) -> EntityValues {
        let (source, path) = (unit.source, unit.path);
        let rca_top = || unit.lang.parse_metrics(source.to_vec(), path);
        match metric {
            Metric::FileLines => single(
                path,
                match backend {
                    Backend::Rca => rca_top().map(|t| t.metrics.loc.sloc().round() as u64),
                    Backend::Native => native::analyze(unit.lang, source).map(|a| a.file_lines()),
                },
            ),
            Metric::FileFunctions => single(
                path,
                match backend {
                    Backend::Rca => rca_top().map(|t| t.metrics.nom.total().round() as u64),
                    Backend::Native => native::analyze(unit.lang, source).map(|a| a.file_functions()),
                },
            ),
            Metric::FunctionArgs | Metric::FunctionCyclomatic | Metric::FunctionLines | Metric::FunctionCognitive => keyed_by_path(
                path,
                match backend {
                    Backend::Rca => rca_top().map(|t| rca_function_metric(metric, &t)).unwrap_or_default(),
                    Backend::Native => native::analyze(unit.lang, source).map(|a| native_function_metric(metric, &a)).unwrap_or_default(),
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
    fn check_parity(metric: Metric, unit: Unit) -> Result<(), String> {
        let rca = compute(metric, Backend::Rca, unit);
        let native = compute(metric, Backend::Native, unit);
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

    /// Every metric the native path implements.
    const ALL_METRICS: [Metric; 6] =
        [Metric::FileLines, Metric::FileFunctions, Metric::FunctionLines, Metric::FunctionArgs, Metric::FunctionCyclomatic, Metric::FunctionCognitive];

    /// Every file in the repo corpus with extension `ext`: ratchet's own `src/`
    /// plus `tests/fixtures/`.
    fn corpus(ext: &str) -> Vec<(std::path::PathBuf, Vec<u8>)> {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let dirs = [root.join("src"), root.join("tests/fixtures")];
        let mut out = Vec::new();
        for entry in dirs.iter().flat_map(walkdir::WalkDir::new).filter_map(Result::ok) {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some(ext) {
                let source = std::fs::read(path).unwrap_or_else(|e| panic!("reading {}: {e}", path.display()));
                out.push((path.to_path_buf(), source));
            }
        }
        out
    }

    /// Run `check_parity(metric, ..)` over every corpus file for `lang`,
    /// panicking on the first divergence.
    fn assert_metric_parity_over_corpus(metric: Metric, lang: Language, ext: &str, least: usize) {
        let files = corpus(ext);
        assert!(files.len() >= least, "expected at least {least} .{ext} files, found {}", files.len());
        for (path, source) in &files {
            if let Err(report) = check_parity(metric, Unit { lang, source, path }) {
                panic!("{report}");
            }
        }
    }

    /// Ordered function entity names rca produces — the walk-parity oracle.
    fn rca_function_entities(top: &FuncSpace) -> Vec<String> {
        let mut out = Vec::new();
        let mut closure_counter: u32 = 0;
        visit_function_spaces(top, &mut |space| out.push(function_entity_name(space, &mut closure_counter)));
        out
    }

    /// Assert the native function walk matches rca's function list on `source`.
    fn assert_function_walk_parity(lang: Language, source: &[u8], path: &Path) {
        let rca = lang.parse_metrics(source.to_vec(), path).map(|top| rca_function_entities(&top)).unwrap_or_default();
        let native = native::analyze(lang, source).map(|a| a.function_entities()).unwrap_or_default();
        assert_eq!(native, rca, "function-walk divergence in {}", path.display());
    }

    #[test]
    fn test_every_metric_and_language_now_resolves_to_the_native_path() {
        for metric in ALL_METRICS {
            assert_eq!(metric.backend(), Backend::Native, "{metric:?} still routes through rca");
        }
        assert!(native::supports(Language::Cpp));
    }

    #[test]
    fn test_native_file_lines_matches_rca_on_a_snippet() {
        let src = b"fn add(a: i32, b: i32) -> i32 {\n    a + b\n}\n";
        let unit = Unit { lang: Language::Rust, source: src, path: Path::new("snippet.rs") };
        let rca = compute(Metric::FileLines, Backend::Rca, unit);
        assert!(!rca.is_empty(), "rca should produce a file_lines value");
        assert_eq!(compute(Metric::FileLines, Backend::Native, unit), rca);
    }

    #[test]
    fn test_native_file_functions_matches_rca_on_a_snippet() {
        let src = b"fn a() {}\nfn b() { let c = || 0; }\nstruct S;\nimpl S { fn m(&self) {} }\n";
        let unit = Unit { lang: Language::Rust, source: src, path: Path::new("snippet.rs") };
        let rca = compute(Metric::FileFunctions, Backend::Rca, unit);
        assert!(!rca.is_empty(), "rca should produce a file_functions value");
        assert_eq!(compute(Metric::FileFunctions, Backend::Native, unit), rca);
    }

    #[test]
    fn test_function_cyclomatic_parity_over_repo_corpus() {
        assert_metric_parity_over_corpus(Metric::FunctionCyclomatic, Language::Rust, "rs", 6);
    }

    #[test]
    fn test_function_lines_parity_over_repo_corpus() {
        assert_metric_parity_over_corpus(Metric::FunctionLines, Language::Rust, "rs", 6);
    }

    #[test]
    fn test_function_cognitive_parity_over_repo_corpus() {
        assert_metric_parity_over_corpus(Metric::FunctionCognitive, Language::Rust, "rs", 6);
    }

    /// Cognitive parity on constructs the repo corpus may under-exercise, each
    /// checked against rca: else-if chains, nested functions (depth), labeled
    /// break/continue, closures inside control flow (lambda), boolean sequences
    /// (same vs mixed operators, unary `!`), match, and `let ... else`.
    #[test]
    fn test_function_cognitive_parity_on_tricky_constructs() {
        let snippets: &[&[u8]] = &[
            b"fn f(a: i32) { if a > 0 {} else if a < 0 {} else {} }",
            b"fn f() { fn g() { for _ in 0..3 { if true {} } } }",
            b"fn f() { 'outer: for _ in 0..3 { for _ in 0..3 { break 'outer; continue 'outer; } } }",
            b"fn f(v: Vec<i32>) { if !v.is_empty() { let c = |x: i32| { if x > 0 { x } else { 0 } }; c(1); } }",
            b"fn f(a: bool, b: bool, c: bool) { let _ = a && b && c; let _ = a && b || c; let _ = !a && b; }",
            b"fn f(x: Option<i32>) -> i32 { let Some(n) = x else { return 0 }; match n { 0 => 1, _ => 2 } }",
            b"fn f() { while let Some(_) = None::<i32> { loop { break; } } }",
        ];
        for src in snippets {
            let unit = Unit { lang: Language::Rust, source: src, path: Path::new("tricky.rs") };
            if let Err(report) = check_parity(Metric::FunctionCognitive, unit) {
                panic!("{report}\n  source: {}", std::str::from_utf8(src).unwrap());
            }
        }
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
        assert_metric_parity_over_corpus(Metric::FileLines, Language::Rust, "rs", 6);
    }

    #[test]
    fn test_file_functions_parity_over_repo_corpus() {
        assert_metric_parity_over_corpus(Metric::FileFunctions, Language::Rust, "rs", 6);
    }

    #[test]
    fn test_function_args_parity_over_repo_corpus() {
        assert_metric_parity_over_corpus(Metric::FunctionArgs, Language::Rust, "rs", 6);
    }

    /// The native JavaScript path (vendored mozjs grammar + JS rules) must agree
    /// with rca on the JS corpus, for the walk and for every metric.
    #[test]
    fn test_javascript_function_walk_parity_over_corpus() {
        let files = corpus("js");
        assert!(files.len() >= 2, "expected JS fixtures, found {}", files.len());
        for (path, source) in &files {
            assert_function_walk_parity(Language::JavaScript, source, path);
        }
    }

    #[test]
    fn test_javascript_metric_parity_over_corpus() {
        for metric in ALL_METRICS {
            assert_metric_parity_over_corpus(metric, Language::JavaScript, "js", 2);
            assert_metric_parity_over_corpus(metric, Language::JavaScript, "jsx", 1);
        }
    }

    /// TypeScript and TSX keep rca parity for the **file-level** metrics, which do
    /// not depend on function naming.
    ///
    /// The function-level metrics deliberately diverge: ratchet does not reproduce
    /// rca's two TS/TSX bugs (see `rules::JS_FAMILY`), so entity names and TSX
    /// cognitive values differ by design. Those are pinned by golden tests below
    /// instead of by parity.
    #[test]
    fn test_typescript_and_tsx_file_level_parity_over_corpus() {
        for (lang, ext) in [(Language::TypeScript, "ts"), (Language::Tsx, "tsx")] {
            for metric in [Metric::FileLines, Metric::FileFunctions] {
                assert_metric_parity_over_corpus(metric, lang, ext, 2);
            }
        }
    }

    /// Golden: anonymous TS/TSX functions take their name from the enclosing
    /// `variable_declarator`, exactly as JavaScript's do. rca leaves these
    /// `"<anonymous>"` in TS/TSX only because its fallback compares against the
    /// wrong grammar's enum.
    #[test]
    fn test_typescript_names_anonymous_functions_like_javascript() {
        let src = b"const arrow = (x: number): number => x * 2;\nconst expr = function (a: number) { return a; };\n";
        let names = native::analyze(Language::TypeScript, src).expect("TS is native").function_entities();
        assert_eq!(names, vec!["arrow", "expr"]);

        let tsx = b"const arrow = (x: number) => <b>{x}</b>;\n";
        let tsx_names = native::analyze(Language::Tsx, tsx).expect("TSX is native").function_entities();
        assert_eq!(tsx_names, vec!["arrow"]);
    }

    /// Golden: an `else if` costs a flat `+1` in TSX, as it does in TS and JS.
    /// rca charges it a full nesting increment there (cognitive 4 rather than 2)
    /// because its TSX else-if test looks for the wrong parent kind.
    #[test]
    fn test_tsx_scores_else_if_flat_like_the_rest_of_the_family() {
        let src =
            b"function badge(n: number): string {\n  if (n < 0) {\n    return \"a\";\n  } else if (n === 0) {\n    return \"b\";\n  }\n  return \"c\";\n}\n";
        let cognitive = |lang| native::analyze(lang, src).expect("native").function_cognitive();
        assert_eq!(cognitive(Language::Tsx), vec![("badge".to_string(), 2)]);
        // …and identical to TypeScript and JavaScript for the same source.
        assert_eq!(cognitive(Language::TypeScript), vec![("badge".to_string(), 2)]);
    }

    /// Python must agree with rca on the walk and on every metric.
    #[test]
    fn test_python_parity_over_corpus() {
        let files = corpus("py");
        assert!(files.len() >= 2, "expected .py fixtures, found {}", files.len());
        for (path, source) in &files {
            assert_function_walk_parity(Language::Python, source, path);
        }
        for metric in ALL_METRICS {
            assert_metric_parity_over_corpus(metric, Language::Python, "py", 2);
        }
    }

    /// Java must agree with rca on the walk and on every metric.
    #[test]
    fn test_java_parity_over_corpus() {
        let files = corpus("java");
        assert!(files.len() >= 2, "expected .java fixtures, found {}", files.len());
        for (path, source) in &files {
            assert_function_walk_parity(Language::Java, source, path);
        }
        for metric in ALL_METRICS {
            assert_metric_parity_over_corpus(metric, Language::Java, "java", 2);
        }
    }

    /// C and C++ share one grammar; both extensions must agree with rca.
    #[test]
    fn test_cpp_parity_over_corpus() {
        for ext in ["c", "cpp"] {
            let files = corpus(ext);
            assert!(!files.is_empty(), "expected .{ext} fixtures");
            for (path, source) in &files {
                assert_function_walk_parity(Language::Cpp, source, path);
            }
            for metric in ALL_METRICS {
                assert_metric_parity_over_corpus(metric, Language::Cpp, ext, 1);
            }
        }
    }

    #[test]
    fn test_function_walk_parity_on_tricky_constructs() {
        // Trait signature (excluded) vs defaulted method (included), module, async, generic.
        let src = b"trait T {\n    fn required(&self);\n    fn defaulted(&self) { let c = || 0; }\n}\nmod inner { pub fn nested() {} }\nasync fn a() {}\nfn generic<X>(x: X) -> X { x }\n";
        assert_function_walk_parity(Language::Rust, src, Path::new("tricky.rs"));
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
            assert_function_walk_parity(Language::Rust, &source, path);
            checked += 1;
        }
        assert!(checked > 5, "expected to check several Rust files, checked {checked}");
    }
}
