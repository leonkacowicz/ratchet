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

use tree_sitter::{Node, Parser, Tree};
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

/// Tree-sitter node kinds ratchet treats as Rust function spaces — the same set
/// `rust-code-analysis` maps to `SpaceKind::Function` (its getter routes
/// `function_item` and `closure_expression` there).
fn is_rust_function(node: &Node) -> bool {
    matches!(node.kind(), "function_item" | "closure_expression")
}

/// The entity name for a Rust function node, matching rca's default
/// `get_func_space_name`: the node's `name` field text, or `"<anonymous>"` when
/// it has none (closures). Note rca does **not** qualify methods by their type.
fn rust_function_name(node: &Node, source: &[u8]) -> String {
    match node.child_by_field_name("name") {
        Some(name) => std::str::from_utf8(&source[name.byte_range()]).unwrap_or("<anonymous>").to_string(),
        None => "<anonymous>".to_string(),
    }
}

/// Visit every Rust function space (`function_item` + `closure_expression`) in
/// the tree in pre-order, invoking `f(name, node)`. Order and naming mirror
/// rca's `visit_function_spaces` + `function_entity_name`, so entities line up
/// one-to-one with the rca path. The node handle lets metric collectors compute
/// per-function values as they migrate.
pub fn visit_rust_functions(tree: &Tree, source: &[u8], f: &mut impl FnMut(&str, Node)) {
    fn recurse(node: Node, source: &[u8], f: &mut impl FnMut(&str, Node)) {
        let mut i = 0;
        while i < node.named_child_count() {
            let child = node.named_child(i).expect("named_child within count");
            if is_rust_function(&child) {
                let name = rust_function_name(&child, source);
                f(&name, child);
            }
            recurse(child, source, f);
            i += 1;
        }
    }
    recurse(tree.root_node(), source, f);
}

/// File-level SLOC for Rust `source`, matching rca's `loc.sloc()` on the unit:
/// the root node's `end_row - start_row`. `None` if the source fails to parse.
pub fn rust_file_lines(source: &[u8]) -> Option<u64> {
    let tree = parse_rust(source)?;
    let root = tree.root_node();
    Some((root.end_position().row - root.start_position().row) as u64)
}

/// File-level function count for Rust `source`, matching rca's `nom.total()`:
/// the number of function spaces (`function_item` + `closure_expression`).
/// `None` if the source fails to parse.
pub fn rust_file_functions(source: &[u8]) -> Option<u64> {
    let tree = parse_rust(source)?;
    let mut count = 0u64;
    visit_rust_functions(&tree, source, &mut |_name, _node| count += 1);
    Some(count)
}

/// Per-function argument counts for Rust `source`, as `(entity_name, nargs)` in
/// walk order — matching rca's `nargs` (`fn_args`/`closure_args`). Empty when the
/// source fails to parse.
pub fn rust_function_nargs(source: &[u8]) -> Vec<(String, u64)> {
    let Some(tree) = parse_rust(source) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    visit_rust_functions(&tree, source, &mut |name, node| out.push((name.to_string(), nargs_of(&node))));
    out
}

/// Argument count of a Rust function/closure node, mirroring rca's `compute_args`
/// and `is_non_arg`: every child of the `parameters` field except the delimiters
/// (parens, comma, pipe) and `attribute_item`. Note that `self` (a
/// `self_parameter`) counts as an argument, matching rca.
fn nargs_of(node: &Node) -> u64 {
    let Some(params) = node.child_by_field_name("parameters") else {
        return 0;
    };
    let mut cursor = params.walk();
    params.children(&mut cursor).filter(|c| !matches!(c.kind(), "(" | ")" | "," | "|" | "attribute_item")).count() as u64
}

/// Ordered function entity names for a Rust `source` file, matching the rca
/// path's function list. Empty when the source fails to parse.
pub fn rust_function_entities(source: &[u8]) -> Vec<String> {
    let Some(tree) = parse_rust(source) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    visit_rust_functions(&tree, source, &mut |name, _node| out.push(name.to_string()));
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rust_function_entities_names_functions_and_closures() {
        let src = b"struct S;\nimpl S {\n    fn new() -> S { let f = || 1; S }\n}\nfn top() {}\n";
        assert_eq!(rust_function_entities(src), vec!["new", "<anonymous>", "top"]);
    }

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
