//! Characterization tests over `tests/fixtures/`.
//!
//! These replace the rca parity harness. While `rust-code-analysis` was still a
//! dependency, every metric was verified against it; the values committed in
//! `tests/fixtures/golden.json` are those verified values, plus the deliberate
//! corrections where ratchet stopped reproducing rca's defects.
//!
//! With rca gone there is no external oracle left, so this is the regression net:
//! any unintended change to a metric shows up as a diff against the golden file.
//! To re-bless after an intended change, run:
//!
//! ```sh
//! RATCHET_BLESS_GOLDEN=1 cargo test golden
//! ```
//!
//! and review the resulting diff before committing it.

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::path::Path;

    use serde::{Deserialize, Serialize};

    use crate::language::Language;
    use crate::native;

    /// Every metric ratchet computes for one fixture.
    #[derive(Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
    struct FixtureMetrics {
        file_lines: u64,
        file_functions: u64,
        function_lines: BTreeMap<String, u64>,
        function_args: BTreeMap<String, u64>,
        function_cyclomatic: BTreeMap<String, u64>,
        function_cognitive: BTreeMap<String, u64>,
    }

    type Golden = BTreeMap<String, FixtureMetrics>;

    fn collect(entries: Vec<(String, u64)>) -> BTreeMap<String, u64> {
        entries.into_iter().collect()
    }

    /// Compute every metric for each fixture, keyed by file name.
    fn measure_fixtures() -> Golden {
        let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
        let mut out = Golden::new();
        for entry in std::fs::read_dir(&dir).expect("fixtures dir").filter_map(Result::ok) {
            let path = entry.path();
            let Some(lang) = path.extension().and_then(|e| e.to_str()).and_then(Language::from_extension) else {
                continue;
            };
            let source = std::fs::read(&path).expect("fixture readable");
            let analysis = native::analyze(lang, &source).unwrap_or_else(|| panic!("{} should parse natively", path.display()));
            let name = path.file_name().expect("file name").to_string_lossy().into_owned();
            out.insert(
                name,
                FixtureMetrics {
                    file_lines: analysis.file_lines(),
                    file_functions: analysis.file_functions(),
                    function_lines: collect(analysis.function_lines()),
                    function_args: collect(analysis.function_nargs()),
                    function_cyclomatic: collect(analysis.function_cyclomatic()),
                    function_cognitive: collect(analysis.function_cognitive()),
                },
            );
        }
        out
    }

    /// Pins the places ratchet deliberately departs from `rust-code-analysis`.
    ///
    /// Parity against rca was how each language was verified; where rca turned out
    /// to be self-inconsistent, ratchet fixed it instead of copying it. These are
    /// those decisions, recorded as behaviour.
    mod deliberate_divergences_from_rca {
        use super::*;

        /// rca leaves anonymous TS/TSX functions `"<anonymous>"` only because its
        /// naming fallback compares kind ids against the *Mozjs* enum. Ratchet
        /// names them from their `variable_declarator`, as it does in JavaScript.
        #[test]
        fn test_typescript_names_anonymous_functions_like_javascript() {
            let src = b"const arrow = (x: number): number => x * 2;\nconst expr = function (a: number) { return a; };\n";
            let names = native::analyze(Language::TypeScript, src).expect("native").function_entities();
            assert_eq!(names, vec!["arrow", "expr"]);
        }

        /// rca never detects an `else if` in TSX (it looks for an `IfStatement`
        /// parent, but the parent is always an `else_clause`), so each one takes a
        /// full nesting increment. Ratchet charges the flat `+1` the rest of the
        /// family gets.
        #[test]
        fn test_tsx_scores_else_if_flat_like_the_rest_of_the_family() {
            let src = b"function badge(n: number): string {\n  if (n < 0) {\n    return \"a\";\n  } else if (n === 0) {\n    return \"b\";\n  }\n  return \"c\";\n}\n";
            let cognitive = |lang| native::analyze(lang, src).expect("native").function_cognitive();
            assert_eq!(cognitive(Language::Tsx), vec![("badge".to_string(), 2)]);
            assert_eq!(cognitive(Language::TypeScript), vec![("badge".to_string(), 2)]);
        }

        /// rca counts every Python `else` toward cyclomatic — its loop-else guard
        /// never engages. Only a loop-`else` is a genuine second exit.
        #[test]
        fn test_python_counts_only_loop_else_toward_cyclomatic() {
            let if_else = b"def f(n):\n    if n < 0:\n        return 1\n    elif n == 0:\n        return 2\n    else:\n        return 3\n";
            let loop_else = b"def f(xs):\n    for x in xs:\n        pass\n    else:\n        pass\n";
            let cyclo = |src| native::analyze(Language::Python, src).expect("native").function_cyclomatic();
            assert_eq!(cyclo(if_else), vec![("f".to_string(), 3)]);
            assert_eq!(cyclo(loop_else), vec![("f".to_string(), 3)]);
        }

        /// rca's Java `is_non_arg` always returns `false`, so parentheses and
        /// commas count as arguments and a two-argument method scores 5.
        #[test]
        fn test_java_counts_only_real_arguments() {
            let src = b"class A { int add(int a, int b) { return a + b; } void none() {} }";
            let nargs = native::analyze(Language::Java, src).expect("native").function_nargs();
            assert_eq!(nargs, vec![("add".to_string(), 2), ("none".to_string(), 0)]);
        }

        /// rca detects neither an `else if` nor the enhanced `for` in Java's
        /// cognitive rules; ratchet scores both as every other language does.
        #[test]
        fn test_java_scores_else_if_and_enhanced_for_like_other_languages() {
            let else_if = b"class A { int f(int n) { if (n < 0) { return 1; } else if (n == 0) { return 2; } return 3; } }";
            let enhanced = b"class A { void f(int[] xs) { for (int x : xs) { if (x > 0) { return; } } } }";
            let cog = |src| native::analyze(Language::Java, src).expect("native").function_cognitive();
            assert_eq!(cog(else_if), vec![("f".to_string(), 2)]);
            assert_eq!(cog(enhanced), vec![("f".to_string(), 3)]);
        }
    }

    #[test]
    fn test_fixture_metrics_match_the_golden_file() {
        let golden_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/golden.json");
        let measured = measure_fixtures();
        assert!(!measured.is_empty(), "expected fixtures to measure");

        if std::env::var_os("RATCHET_BLESS_GOLDEN").is_some() {
            let json = serde_json::to_string_pretty(&measured).expect("serializable");
            std::fs::write(&golden_path, json + "\n").expect("writable");
            return;
        }

        let raw = std::fs::read_to_string(&golden_path)
            .unwrap_or_else(|e| panic!("reading {}: {e}\nrun with RATCHET_BLESS_GOLDEN=1 to create it", golden_path.display()));
        let expected: Golden = serde_json::from_str(&raw).expect("golden.json parses");

        for (name, expect) in &expected {
            let got = measured.get(name).unwrap_or_else(|| panic!("fixture {name} disappeared"));
            assert_eq!(got, expect, "metrics changed for {name}");
        }
        let missing: Vec<_> = measured.keys().filter(|k| !expected.contains_key(*k)).collect();
        assert!(missing.is_empty(), "fixtures not in golden.json: {missing:?} — re-bless with RATCHET_BLESS_GOLDEN=1");
    }
}
