//! Language-agnostic building blocks.
//!
//! Every function here is pure computation over `(rules, tree, source)` — it never
//! sees a [`Language`](crate::language::Language) and never branches on one. All
//! language variation arrives as data through [`Rules`]. Language-specific glue
//! (see `native::lang`) picks which blocks to call and assembles the results.

use tree_sitter::{Node, Tree};

use super::rules::Rules;

/// The name ratchet gives a function when the language has no better idea.
pub const ANONYMOUS: &str = "<anonymous>";

/// The text of `node`'s `field` child, if it has one.
pub fn field_text(node: &Node, field: &str, source: &[u8]) -> Option<String> {
    let child = node.child_by_field_name(field)?;
    std::str::from_utf8(&source[child.byte_range()]).ok().map(str::to_string)
}

/// rca's *default* `get_func_space_name`: the node's `name` field text, or
/// `"<anonymous>"`. Languages whose naming differs supply their own function via
/// [`Rules::name_of`] — rca does **not** qualify methods by their type.
pub fn default_function_name(node: &Node, source: &[u8]) -> String {
    field_text(node, "name", source).unwrap_or_else(|| ANONYMOUS.to_string())
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
                let name = (rules.name_of)(&child, source);
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

/// File-level function count, matching rca's `nom.total()` — functions *plus*
/// closures. That is a wider set than the function spaces the walk emits: a
/// Python `lambda` counts here but is not a space.
pub fn file_functions(rules: &Rules, tree: &Tree, _source: &[u8]) -> u64 {
    fn recurse(rules: &Rules, node: Node, count: &mut u64) {
        let mut i = 0;
        while i < node.named_child_count() {
            let child = node.named_child(i).expect("named_child within count");
            if rules.counts_toward_nom(child.kind()) {
                *count += 1;
            }
            recurse(rules, child, count);
            i += 1;
        }
    }
    let mut count = 0;
    recurse(rules, tree.root_node(), &mut count);
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

/// Argument count for a function space, mirroring rca's `max(fn_args, closure_args)`.
///
/// rca accumulates arguments into the *enclosing space*, so a closure that is not
/// itself a space contributes to the function containing it. That only bites in
/// Python, whose `lambda` is not a space: `def f(v)` containing two one-argument
/// lambdas scores `max(1, 2) = 2`. Where closures are spaces (Rust, the JS
/// family) the closure sum is empty and this reduces to the node's own parameters.
pub fn nargs_of(node: &Node, rules: &Rules) -> u64 {
    let own = own_params(node, rules);
    let mut closures = 0;
    sum_closure_params(node, rules, &mut closures);
    own.max(closures)
}

/// Parameters declared directly by `node`, excluding the language's delimiters.
fn own_params(node: &Node, rules: &Rules) -> u64 {
    let Some(params) = node.child_by_field_name("parameters") else {
        return 0;
    };
    let mut cursor = params.walk();
    params.children(&mut cursor).filter(|c| !rules.non_arg_kinds.contains(&c.kind())).count() as u64
}

/// Sum the parameters of closure nodes that belong to this space — that is,
/// lambda-kind descendants, not descending into nested function spaces.
fn sum_closure_params(node: &Node, rules: &Rules, total: &mut u64) {
    let mut i = 0;
    while i < node.named_child_count() {
        let child = node.named_child(i).expect("named_child within count");
        let kind = child.kind();
        if rules.is_function(kind) {
            // A nested space accounts for its own closures.
        } else {
            if rules.lambda_kinds.contains(&kind) {
                *total += own_params(&child, rules);
            }
            sum_closure_params(&child, rules, total);
        }
        i += 1;
    }
}

/// Ordered function entity names, matching the rca path's function list.
pub fn function_entities(rules: &Rules, tree: &Tree, source: &[u8]) -> Vec<String> {
    let mut out = Vec::new();
    visit_functions(rules, tree, source, &mut |name, _node| out.push(name.to_string()));
    out
}
