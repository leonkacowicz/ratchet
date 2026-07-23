//! Native raw-tree-sitter metric path (rca-free), parameterized by language.
//!
//! Grammars are vendored and statically linked (see `build.rs` and `vendor/`), and
//! metrics are computed by walking the raw tree-sitter tree rather than going
//! through `rust-code-analysis`. The shared algorithms live in [`metrics`] and
//! [`complexity`]; the per-language node kinds they consult live in [`rules`].
//!
//! A language is native once it has both a vendored grammar and a rule set — see
//! [`supports`]. Everything else still routes through rca.
#![allow(dead_code)]

pub mod complexity;
pub mod metrics;
pub mod rules;

pub use complexity::{function_cognitive, function_cyclomatic};
pub use metrics::{file_functions, file_lines, function_lines, function_nargs};

use tree_sitter::{Node, Parser, Tree};
use tree_sitter_language::LanguageFn;

use crate::language::Language;
use rules::Rules;

extern "C" {
    /// Entry point of the vendored, statically-linked Rust grammar (see `build.rs`).
    fn tree_sitter_rust() -> *const ();
}

/// The vendored grammar for `lang`, or `None` when it is not vendored yet.
fn grammar(lang: Language) -> Option<LanguageFn> {
    match lang {
        Language::Rust => Some(unsafe { LanguageFn::from_raw(tree_sitter_rust) }),
        _ => None,
    }
}

/// Whether ratchet can measure `lang` natively: it needs both a vendored grammar
/// and a rule set. Languages without both still route through rca.
pub fn supports(lang: Language) -> bool {
    grammar(lang).is_some() && rules::for_language(lang).is_some()
}

/// Parse `source` for `lang` into a raw tree-sitter syntax tree, or `None` if the
/// language is not native or the source produced no tree.
pub fn parse(lang: Language, source: &[u8]) -> Option<Tree> {
    let mut parser = Parser::new();
    parser.set_language(&grammar(lang)?.into()).ok()?;
    parser.parse(source, None)
}

/// Parse `source` and return both the tree and `lang`'s rule set, or `None` when
/// the language is not native. The common entry point for the metric functions.
pub(crate) fn parse_with_rules(lang: Language, source: &[u8]) -> Option<(Tree, &'static Rules)> {
    let rules = rules::for_language(lang)?;
    Some((parse(lang, source)?, rules))
}

/// The entity name for a function node, matching rca's default
/// `get_func_space_name`: the node's `name` field text, or `"<anonymous>"` when it
/// has none. Note rca does **not** qualify methods by their type.
pub(crate) fn function_name(node: &Node, source: &[u8]) -> String {
    match node.child_by_field_name("name") {
        Some(name) => std::str::from_utf8(&source[name.byte_range()]).unwrap_or("<anonymous>").to_string(),
        None => "<anonymous>".to_string(),
    }
}

/// Visit every function space in the tree in pre-order, invoking `f(name, node)`.
/// Order and naming mirror rca's `visit_function_spaces` + `function_entity_name`,
/// so entities line up one-to-one with the rca path. The node handle lets metric
/// collectors compute per-function values.
pub fn visit_functions(rules: &Rules, tree: &Tree, source: &[u8], f: &mut impl FnMut(&str, Node)) {
    fn recurse(rules: &Rules, node: Node, source: &[u8], f: &mut impl FnMut(&str, Node)) {
        let mut i = 0;
        while i < node.named_child_count() {
            let child = node.named_child(i).expect("named_child within count");
            if rules.is_function(child.kind()) {
                let name = function_name(&child, source);
                f(&name, child);
            }
            recurse(rules, child, source, f);
            i += 1;
        }
    }
    recurse(rules, tree.root_node(), source, f);
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
    fn test_parse_finds_a_function_item() {
        let tree = parse(Language::Rust, b"fn add(a: i32, b: i32) -> i32 { a + b }").expect("Rust source should parse");
        let root = tree.root_node();
        assert_eq!(root.kind(), "source_file");
        let mut cursor = root.walk();
        assert!(root.children(&mut cursor).any(|n| n.kind() == "function_item"));
    }

    #[test]
    fn test_parse_returns_none_for_a_language_without_a_grammar() {
        assert!(parse(Language::Python, b"def f(): pass").is_none());
    }
}
