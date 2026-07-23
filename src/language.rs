use rust_code_analysis::{metrics, FuncSpace, ParserTrait, RustParser, TsxParser, TypescriptParser};
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
}

impl Language {
    /// Map a file extension (without the leading dot) to a supported language,
    /// or `None` when ratchet does not analyze that extension.
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext {
            "rs" => Some(Self::Rust),
            "ts" => Some(Self::TypeScript),
            "tsx" => Some(Self::Tsx),
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
    fn test_from_extension_rejects_unknown() {
        assert_eq!(Language::from_extension("py"), None);
        assert_eq!(Language::from_extension("md"), None);
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
    fn test_strips_rust_test_modules_only_for_rust() {
        assert!(Language::Rust.strips_rust_test_modules());
        assert!(!Language::TypeScript.strips_rust_test_modules());
        assert!(!Language::Tsx.strips_rust_test_modules());
    }
}
