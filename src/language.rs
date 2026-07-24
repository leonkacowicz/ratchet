/// A source language ratchet can parse and measure.
///
/// The set is intentionally narrow: only languages whose metrics ratchet fully
/// implements are enabled. Widening it is a deliberate step (e.g. Kotlin parses
/// but has no complexity metrics), tracked separately in the multi-language roadmap.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    Rust,
    TypeScript,
    Tsx,
    /// C and C++ (and C-family headers), both handled by `tree-sitter-cpp`.
    Cpp,
    Python,
    Java,
    /// JavaScript (including JSX), via the vendored Mozjs grammar, which handles
    /// `.js`/`.mjs`/`.cjs`/`.jsx`.
    JavaScript,
}

impl Language {
    /// Map a file extension (without the leading dot) to a supported language,
    /// or `None` when ratchet does not analyze that extension.
    ///
    /// Extension routing follows the conventional per-grammar mapping (e.g.
    /// `.js`/`.jsx` → Mozjs, `.tsx` → the TSX grammar, C-family headers → cpp).
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

    /// Whether trailing `#[cfg(test)]` module stripping applies before parsing.
    /// Only Rust uses that convention.
    pub fn strips_rust_test_modules(self) -> bool {
        matches!(self, Self::Rust)
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;

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
            let analysis = crate::native::analyze(*expected, &source).unwrap_or_else(|| panic!("no native analysis for fixture {file}"));
            assert!(analysis.file_functions() >= 1, "fixture {file} yielded no functions");
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
