//! Native raw-tree-sitter parsing path (rca-free), starting with Rust.
//!
//! Foundation for migrating off `rust-code-analysis`: grammars are vendored and
//! statically linked (see `build.rs` and `vendor/`), and we obtain a raw
//! tree-sitter [`Tree`] to walk ourselves rather than going through rca. Metric
//! computation over this tree lands in later steps of the migration epic.
//!
//! Nothing in production consumes this module yet — the parity harness and metric
//! collectors of the rca→tree-sitter migration wire it in. Until then its items are
//! only exercised by tests, so dead-code is allowed at the module level.
#![allow(dead_code)]

use tree_sitter::{Parser, Tree};
use tree_sitter_language::LanguageFn;

extern "C" {
    /// Entry point of the vendored, statically-linked Rust grammar (see `build.rs`).
    fn tree_sitter_rust() -> *const ();
}

/// The vendored Rust grammar as a tree-sitter [`LanguageFn`].
const RUST_LANGUAGE: LanguageFn = unsafe { LanguageFn::from_raw(tree_sitter_rust) };

/// Parse Rust `source` into a raw tree-sitter syntax tree, or `None` if the
/// grammar could not be loaded or the source produced no tree.
pub fn parse_rust(source: &[u8]) -> Option<Tree> {
    let mut parser = Parser::new();
    parser.set_language(&RUST_LANGUAGE.into()).ok()?;
    parser.parse(source, None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_rust_finds_a_function_item() {
        let tree = parse_rust(b"fn add(a: i32, b: i32) -> i32 { a + b }").expect("Rust source should parse");
        let root = tree.root_node();
        assert_eq!(root.kind(), "source_file");
        let mut cursor = root.walk();
        let has_fn = root.children(&mut cursor).any(|n| n.kind() == "function_item");
        assert!(has_fn, "expected a function_item in the parse tree");
    }
}
