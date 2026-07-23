use rust_code_analysis::{metrics, CppParser, FuncSpace, JavaParser, MozjsParser, ParserTrait, PythonParser, RustParser, TsxParser, TypescriptParser};
use std::path::Path;

/// A source language ratchet can parse and measure.
///
/// The set is intentionally narrow: only languages whose metrics are fully
/// implemented in `rust-code-analysis` are enabled. Widening it is a deliberate
/// step (e.g. Kotlin parses but has no complexity metrics), tracked separately
/// in the multi-language roadmap.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    Rust,
    TypeScript,
    Tsx,
    /// C and C++ (and C-family headers), both handled by rca's `tree-sitter-cpp`.
    Cpp,
    Python,
    Java,
    /// JavaScript (including JSX), via rca's Mozjs grammar — the parser rca itself
    /// routes `.js`/`.mjs`/`.jsx` files to.
    JavaScript,
}

impl Language {
    /// Map a file extension (without the leading dot) to a supported language,
    /// or `None` when ratchet does not analyze that extension.
    ///
    /// Extension sets mirror `rust-code-analysis`'s own file-extension routing so
    /// ratchet measures each file with the grammar rca would pick for it.
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext {
            "rs" => Some(Self::Rust),
            "ts" => Some(Self::TypeScript),
            "tsx" => Some(Self::Tsx),
            "c" | "cc" | "cpp" | "cxx" | "h" | "hh" | "hpp" | "hxx" => Some(Self::Cpp),
            "py" => Some(Self::Python),
            "java" => Some(Self::Java),
            "js" | "mjs" | "cjs" | "jsx" => Some(Self::JavaScript),
            _ => None,
        }
    }

    /// Parse `source` with this language's grammar and return the metric tree,
    /// or `None` when the parser produced nothing usable.
    pub fn parse_metrics(self, source: Vec<u8>, path: &Path) -> Option<FuncSpace> {
        match self {
            Self::Rust => metrics(&RustParser::new(source, path, None), path),
            Self::TypeScript => metrics(&TypescriptParser::new(source, path, None), path),
            Self::Tsx => metrics(&TsxParser::new(source, path, None), path),
            Self::Cpp => metrics(&CppParser::new(source, path, None), path),
            Self::Python => metrics(&PythonParser::new(source, path, None), path),
            Self::Java => metrics(&JavaParser::new(source, path, None), path),
            Self::JavaScript => metrics(&MozjsParser::new(source, path, None), path),
        }
    }

    /// Whether trailing `#[cfg(test)]` module stripping applies before parsing.
    /// Only Rust uses that convention.
    pub fn strips_rust_test_modules(self) -> bool {
        matches!(self, Self::Rust)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_from_extension_maps_supported_languages() {
        assert_eq!(Language::from_extension("rs"), Some(Language::Rust));
        assert_eq!(Language::from_extension("ts"), Some(Language::TypeScript));
        assert_eq!(Language::from_extension("tsx"), Some(Language::Tsx));
    }

    #[test]
    fn test_from_extension_maps_c_and_cpp_to_cpp() {
        for ext in ["c", "cc", "cpp", "cxx", "h", "hh", "hpp", "hxx"] {
            assert_eq!(Language::from_extension(ext), Some(Language::Cpp), "extension {ext}");
        }
    }

    #[test]
    fn test_from_extension_maps_python_java_javascript() {
        assert_eq!(Language::from_extension("py"), Some(Language::Python));
        assert_eq!(Language::from_extension("java"), Some(Language::Java));
        for ext in ["js", "mjs", "cjs", "jsx"] {
            assert_eq!(Language::from_extension(ext), Some(Language::JavaScript), "extension {ext}");
        }
    }

    #[test]
    fn test_from_extension_rejects_unknown() {
        assert_eq!(Language::from_extension("md"), None);
        assert_eq!(Language::from_extension("go"), None);
        assert_eq!(Language::from_extension(""), None);
    }

    #[test]
    fn test_parse_metrics_rust_finds_a_function() {
        let src = b"fn add(a: i32, b: i32) -> i32 { a + b }".to_vec();
        let space = Language::Rust.parse_metrics(src, Path::new("x.rs")).unwrap();
        assert!(space.metrics.nom.total() >= 1.0);
    }

    #[test]
    fn test_parse_metrics_typescript_finds_a_function() {
        let src = b"function add(a: number, b: number): number { return a + b; }".to_vec();
        let space = Language::TypeScript.parse_metrics(src, Path::new("x.ts")).unwrap();
        assert!(space.metrics.nom.total() >= 1.0);
    }

    #[test]
    fn test_parse_metrics_tsx_parses_jsx() {
        let src = b"const App = () => <div>hi</div>;".to_vec();
        assert!(Language::Tsx.parse_metrics(src, Path::new("x.tsx")).is_some());
    }

    #[test]
    fn test_parse_metrics_cpp_finds_a_function() {
        let src = b"int add(int a, int b) { return a + b; }".to_vec();
        let space = Language::Cpp.parse_metrics(src, Path::new("x.cpp")).unwrap();
        assert!(space.metrics.nom.total() >= 1.0);
    }

    #[test]
    fn test_parse_metrics_python_finds_a_function() {
        let src = b"def add(a, b):\n    return a + b\n".to_vec();
        let space = Language::Python.parse_metrics(src, Path::new("x.py")).unwrap();
        assert!(space.metrics.nom.total() >= 1.0);
    }

    #[test]
    fn test_parse_metrics_java_finds_a_method() {
        let src = b"class X { int add(int a, int b) { return a + b; } }".to_vec();
        let space = Language::Java.parse_metrics(src, Path::new("X.java")).unwrap();
        assert!(space.metrics.nom.total() >= 1.0);
    }

    #[test]
    fn test_parse_metrics_javascript_finds_a_function() {
        let src = b"function add(a, b) { return a + b; }".to_vec();
        let space = Language::JavaScript.parse_metrics(src, Path::new("x.js")).unwrap();
        assert!(space.metrics.nom.total() >= 1.0);
    }

    /// Every example fixture must dispatch to the expected language and yield
    /// at least one measured function — the per-language coverage guarantee.
    #[test]
    fn test_example_fixtures_parse_and_yield_metrics() {
        let cases: &[(&str, &str, Language)] = &[
            ("example.rs", "rs", Language::Rust),
            ("example.ts", "ts", Language::TypeScript),
            ("example.tsx", "tsx", Language::Tsx),
            ("example.c", "c", Language::Cpp),
            ("example.cpp", "cpp", Language::Cpp),
            ("example.py", "py", Language::Python),
            ("example.java", "java", Language::Java),
            ("example.js", "js", Language::JavaScript),
        ];
        for (file, ext, expected) in cases {
            assert_eq!(Language::from_extension(ext), Some(*expected), "extension {ext}");
            let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures").join(file);
            let source = std::fs::read(&path).unwrap_or_else(|e| panic!("reading fixture {file}: {e}"));
            let space = expected.parse_metrics(source, &path).unwrap_or_else(|| panic!("no metrics for fixture {file}"));
            assert!(space.metrics.nom.total() >= 1.0, "fixture {file} yielded no functions");
        }
    }

    #[test]
    fn test_strips_rust_test_modules_only_for_rust() {
        assert!(Language::Rust.strips_rust_test_modules());
        for lang in [Language::TypeScript, Language::Tsx, Language::Cpp, Language::Python, Language::Java, Language::JavaScript] {
            assert!(!lang.strips_rust_test_modules(), "{lang:?} must not strip Rust test modules");
        }
    }
}
