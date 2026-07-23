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
pub mod complexity;
pub mod lang;
pub mod rules;

use tree_sitter::{Parser, Tree};

use crate::language::Language;
use lang::NativeLanguage;

/// Whether ratchet can measure `lang` natively (it has a vendored grammar and a
/// rule set). Everything else still routes through rca.
pub fn supports(lang: Language) -> bool {
    lang::for_language(lang).is_some()
}

/// Detect → dispatch → parse, once per file. `None` when the language has no
/// native implementation or the source produced no tree.
pub fn analyze(lang: Language, source: &[u8]) -> Option<Analysis<'_>> {
    let implementation = lang::for_language(lang)?;
    let mut parser = Parser::new();
    parser.set_language(&implementation.grammar().into()).ok()?;
    let tree = parser.parse(source, None)?;
    Some(Analysis { implementation, tree, source })
}

/// One parsed file bound to its language implementation. Each accessor delegates
/// to that implementation's glue, which in turn calls the shared building blocks —
/// so the tree is parsed once and no metric code branches on the language.
pub struct Analysis<'a> {
    implementation: &'static dyn NativeLanguage,
    tree: Tree,
    source: &'a [u8],
}

impl Analysis<'_> {
    /// The parsed syntax tree (exposed for tests and debugging).
    pub fn tree(&self) -> &Tree {
        &self.tree
    }

    pub fn file_lines(&self) -> u64 {
        self.implementation.file_lines(&self.tree, self.source)
    }

    pub fn file_functions(&self) -> u64 {
        self.implementation.file_functions(&self.tree, self.source)
    }

    pub fn function_lines(&self) -> Vec<(String, u64)> {
        self.implementation.function_lines(&self.tree, self.source)
    }

    pub fn function_nargs(&self) -> Vec<(String, u64)> {
        self.implementation.function_nargs(&self.tree, self.source)
    }

    pub fn function_cyclomatic(&self) -> Vec<(String, u64)> {
        self.implementation.function_cyclomatic(&self.tree, self.source)
    }

    pub fn function_cognitive(&self) -> Vec<(String, u64)> {
        self.implementation.function_cognitive(&self.tree, self.source)
    }

    pub fn function_entities(&self) -> Vec<String> {
        self.implementation.function_entities(&self.tree, self.source)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_supports_rust_but_not_unvendored_languages() {
        assert!(supports(Language::Rust));
        assert!(!supports(Language::Python));
    }

    #[test]
    fn test_analyze_parses_once_and_answers_metrics() {
        let a = analyze(Language::Rust, b"fn add(a: i32, b: i32) -> i32 { a + b }").expect("Rust is native");
        assert_eq!(a.tree().root_node().kind(), "source_file");
        assert_eq!(a.file_functions(), 1);
        assert_eq!(a.function_nargs(), vec![("add".to_string(), 2)]);
    }

    #[test]
    fn test_analyze_returns_none_for_a_language_without_a_grammar() {
        assert!(analyze(Language::Python, b"def f(): pass").is_none());
    }
}
