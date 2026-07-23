//! Language-agnostic building blocks.
//!
//! Every function here is pure computation over `(rules, tree, source)` — it never
//! sees a [`Language`](crate::language::Language) and never branches on one. All
//! language variation arrives as data through [`Rules`]. Language-specific glue
//! (see `native::lang`) picks which blocks to call and assembles the results.

use tree_sitter::{Node, Tree};

use super::rules::Rules;

/// The entity name for a function node, matching rca's default
/// `get_func_space_name`: the node's `name` field text, or `"<anonymous>"` when it
/// has none. Note rca does **not** qualify methods by their type.
pub fn function_name(node: &Node, source: &[u8]) -> String {
    match node.child_by_field_name("name") {
        Some(name) => std::str::from_utf8(&source[name.byte_range()]).unwrap_or("<anonymous>").to_string(),
        None => "<anonymous>".to_string(),
    }
}

/// Visit every function space in the tree in pre-order, invoking `f(name, node)`.
/// Order and naming mirror rca's `visit_function_spaces` + `function_entity_name`,
/// so entities line up one-to-one with the rca path.
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

/// File-level SLOC, matching rca's `loc.sloc()` on the unit: the root node's
/// `end_row - start_row`.
pub fn file_lines(tree: &Tree) -> u64 {
    let root = tree.root_node();
    (root.end_position().row - root.start_position().row) as u64
}

/// File-level function count, matching rca's `nom.total()`: the number of
/// function spaces.
pub fn file_functions(rules: &Rules, tree: &Tree, source: &[u8]) -> u64 {
    let mut count = 0;
    visit_functions(rules, tree, source, &mut |_name, _node| count += 1);
    count
}

/// Per-function SLOC as `(entity_name, value)` in walk order — matching rca's
/// `loc.sloc()` on a function space (`end_row - start_row + 1` of the node).
pub fn function_lines(rules: &Rules, tree: &Tree, source: &[u8]) -> Vec<(String, u64)> {
    let mut out = Vec::new();
    visit_functions(rules, tree, source, &mut |name, node| {
        let lines = (node.end_position().row - node.start_position().row) as u64 + 1;
        out.push((name.to_string(), lines));
    });
    out
}

/// Per-function argument counts as `(entity_name, nargs)` in walk order, matching
/// rca's `nargs`.
pub fn function_nargs(rules: &Rules, tree: &Tree, source: &[u8]) -> Vec<(String, u64)> {
    let mut out = Vec::new();
    visit_functions(rules, tree, source, &mut |name, node| out.push((name.to_string(), nargs_of(&node, rules))));
    out
}

/// Argument count of a function node, mirroring rca's `compute_args` and
/// `is_non_arg`: every child of the `parameters` field except the language's
/// delimiter/attribute kinds. For Rust that means `self` counts as an argument.
pub fn nargs_of(node: &Node, rules: &Rules) -> u64 {
    let Some(params) = node.child_by_field_name("parameters") else {
        return 0;
    };
    let mut cursor = params.walk();
    params.children(&mut cursor).filter(|c| !rules.non_arg_kinds.contains(&c.kind())).count() as u64
}

/// Ordered function entity names, matching the rca path's function list.
pub fn function_entities(rules: &Rules, tree: &Tree, source: &[u8]) -> Vec<String> {
    let mut out = Vec::new();
    visit_functions(rules, tree, source, &mut |name, _node| out.push(name.to_string()));
    out
}
