//! Native raw-tree-sitter metric path (rca-free).
//!
//! The flow mirrors how a file is actually processed:
//!
//! ```text
//! Language detected (sources.rs)
//!        │
//!        ▼
//! lang::for_language(lang)      ← the only branch on Language
//!        │
//!        ▼
//! impl NativeLanguage           ← language-specific glue
//!        │  calls
//!        ▼
//! analysis::* / complexity::*   ← pure (rules, tree, source); never see a Language
//!        │
//!        ▼
//! glue assembles the results
//! ```
//!
//! [`analyze`] does the dispatch and the parse **once per file**; the resulting
//! [`Analysis`] then answers each metric without re-parsing.
#![allow(dead_code)]

pub mod analysis;
pub mod cognitive;
pub mod cyclomatic;
pub mod lang;
pub mod rules;

use tree_sitter::{Parser, Tree};

use crate::language::Language;
use lang::NativeLanguage;

use rules::Rules;

/// Whether ratchet can measure `lang` natively — it has a grammar and a rule set.
/// Every currently-supported language does (see [`lang::for_language`]).
pub fn supports(lang: Language) -> bool {
    lang::for_language(lang).is_some()
}

/// Detect → dispatch → parse, once per file. `None` when the language has no
/// native implementation or the source produced no tree.
pub fn analyze(lang: Language, source: &[u8]) -> Option<Analysis<'_>> {
    let implementation = lang::for_language(lang)?;
    let mut parser = Parser::new();
    parser.set_language(&implementation.grammar.into()).ok()?;
    let tree = parser.parse(source, None)?;
    Some(Analysis { implementation, tree, source })
}

/// One parsed file bound to its language implementation. Each accessor delegates
/// to that implementation's glue, which in turn calls the shared building blocks —
/// so the tree is parsed once and no metric code branches on the language.
pub struct Analysis<'a> {
    implementation: &'static NativeLanguage,
    tree: Tree,
    source: &'a [u8],
}

impl Analysis<'_> {
    /// The node-kind rules for this file's language.
    fn rules(&self) -> &'static Rules {
        self.implementation.rules
    }

    /// The parsed syntax tree (exposed for tests and debugging).
    pub fn tree(&self) -> &Tree {
        &self.tree
    }

    pub fn file_lines(&self) -> u64 {
        analysis::file_lines(&self.tree)
    }

    pub fn file_functions(&self) -> u64 {
        analysis::file_functions(self.rules(), &self.tree, self.source)
    }

    pub fn function_lines(&self) -> Vec<(String, u64)> {
        analysis::function_lines(self.rules(), &self.tree, self.source)
    }

    pub fn function_nargs(&self) -> Vec<(String, u64)> {
        analysis::function_nargs(self.rules(), &self.tree, self.source)
    }

    pub fn function_cyclomatic(&self) -> Vec<(String, u64)> {
        cyclomatic::function_cyclomatic(self.rules(), &self.tree, self.source)
    }

    pub fn function_cognitive(&self) -> Vec<(String, u64)> {
        cognitive::function_cognitive(self.rules(), &self.tree, self.source)
    }

    pub fn function_entities(&self) -> Vec<String> {
        analysis::function_entities(self.rules(), &self.tree, self.source)
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;

    #[test]
    fn test_every_language_is_natively_supported() {
        for lang in lang::ALL_LANGUAGES {
            assert!(supports(lang), "{lang:?} is not natively supported");
        }
    }

    #[test]
    fn test_analyze_parses_once_and_answers_metrics() {
        let a = analyze(Language::Rust, b"fn add(a: i32, b: i32) -> i32 { a + b }").expect("Rust is native");
        assert_eq!(a.tree().root_node().kind(), "source_file");
        assert_eq!(a.file_functions(), 1);
        assert_eq!(a.function_nargs(), vec![("add".to_string(), 2)]);
    }

    /// Guard against the failure that motivated this whole migration: two
    /// semver-incompatible tree-sitter runtimes in one graph give two distinct
    /// `Language` types and a compile error (rca 0.0.25's `E0308`).
    ///
    /// Grammar crates depend only on `tree-sitter-language`, never the runtime, so
    /// this should hold — but a future grammar bump could split either crate, and
    /// the failure is confusing enough to be worth catching in CI.
    #[test]
    fn test_exactly_one_tree_sitter_runtime_in_the_lockfile() {
        let lock = std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.lock")).expect("Cargo.lock");
        for crate_name in ["tree-sitter", "tree-sitter-language"] {
            let needle = format!("name = \"{crate_name}\"\n");
            let count = lock.matches(&needle).count();
            assert_eq!(count, 1, "expected exactly one `{crate_name}` in Cargo.lock, found {count}");
        }
    }

    #[test]
    fn test_analyze_succeeds_for_every_language() {
        // One trivial source per language, each of which must parse natively.
        let samples: [(Language, &[u8]); 7] = [
            (Language::Rust, b"fn f() {}"),
            (Language::Cpp, b"int f() { return 0; }"),
            (Language::Python, b"def f():\n    pass\n"),
            (Language::Java, b"class A { void f() {} }"),
            (Language::JavaScript, b"function f() {}"),
            (Language::TypeScript, b"function f(): void {}"),
            (Language::Tsx, b"const f = () => <b>x</b>;"),
        ];
        for (lang, src) in samples {
            let analysis = analyze(lang, src).unwrap_or_else(|| panic!("{lang:?} should parse natively"));
            assert_eq!(analysis.file_functions(), 1, "{lang:?}");
        }
    }
}
