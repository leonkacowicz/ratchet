//! Cyclomatic and cognitive complexity over the raw tree-sitter tree, matching
//! rca's `cyclomatic_sum` / `cognitive_sum` (both subtree sums). The algorithms
//! are language-agnostic; the node kinds they match come from the language's
//! [`Rules`].

use tree_sitter::{Node, Tree};

use super::analysis::{function_name, visit_functions};
use super::rules::Rules;

/// Per-function cyclomatic complexity as `(entity_name, value)` in walk order —
/// matching rca's `cyclomatic_sum`.
pub fn function_cyclomatic(rules: &Rules, tree: &Tree, source: &[u8]) -> Vec<(String, u64)> {
    let mut out = Vec::new();
    visit_functions(rules, tree, source, &mut |name, node| out.push((name.to_string(), cyclomatic_of(&node, rules))));
    out
}

/// Cyclomatic complexity of a function node, matching rca's `cyclomatic_sum` over
/// its subtree: a base `1` for the function, plus `1` for every nested function
/// space (each carries its own base) and `1` for every decision point.
fn cyclomatic_of(func: &Node, rules: &Rules) -> u64 {
    let mut total = 1;
    count_cyclomatic(func, rules, &mut total);
    total
}

fn count_cyclomatic(node: &Node, rules: &Rules, total: &mut u64) {
    let mut i = 0;
    while i < node.child_count() {
        let child = node.child(i).expect("child within count");
        let kind = child.kind();
        if rules.is_function(kind) || rules.decision_kinds.contains(&kind) {
            *total += 1;
        }
        count_cyclomatic(&child, rules, total);
        i += 1;
    }
}

/// Boolean operator tracked for cognitive complexity's boolean-sequence rule.
/// `Not` is the sentinel rca sets on a unary expression, distinct from `&&`/`||`
/// so the next operator in a sequence counts as a change.
#[derive(Clone, Copy, PartialEq)]
enum BoolOp {
    And,
    Or,
    Not,
}

/// Inherited cognitive context carried top-down: control-structure `nesting`,
/// nested-function `depth`, lambda `lambda`, and whether we are inside a function.
#[derive(Clone, Copy)]
struct CogCtx {
    nesting: u64,
    depth: u64,
    lambda: u64,
    in_fn: bool,
}

/// A cognitive accumulator for one space (rca `Stats`): the space's own
/// `structural` cost, the summed cost of nested spaces, and the current boolean op.
#[derive(Default)]
struct CogSpace {
    structural: u64,
    nested_sum: u64,
    bool_op: Option<BoolOp>,
}

/// Per-function cognitive complexity as `(entity_name, value)` in walk order —
/// matching rca's `cognitive_sum` (subtree sum).
pub fn function_cognitive(rules: &Rules, tree: &Tree, source: &[u8]) -> Vec<(String, u64)> {
    let mut out = Vec::new();
    let ctx = CogCtx { nesting: 0, depth: 0, lambda: 0, in_fn: false };
    Cog { source, rules, out: &mut out }.walk(&tree.root_node(), ctx, &mut CogSpace::default());
    out
}

/// Traversal state shared across the cognitive walk: source bytes, the language's
/// rules, and the accumulating output. Bundled so the recursive methods stay
/// within ratchet's own argument-count limit.
struct Cog<'a> {
    source: &'a [u8],
    rules: &'a Rules,
    out: &'a mut Vec<(String, u64)>,
}

impl Cog<'_> {
    /// Process a function space node: emit its entry (pre-order, matching the
    /// walk) and return its cognitive_sum (own `structural` + nested spaces).
    fn space(&mut self, node: &Node, ctx: CogCtx) -> u64 {
        let idx = self.out.len();
        self.out.push((function_name(node, self.source), 0));
        let mut space = CogSpace::default();
        let mut i = 0;
        while i < node.child_count() {
            self.walk(&node.child(i).expect("child within count"), ctx, &mut space);
            i += 1;
        }
        let total = space.structural + space.nested_sum;
        self.out[idx].1 = total;
        total
    }

    /// Apply `node`'s cognitive rule to `space`, then recurse — nested function
    /// spaces start their own spaces. Mirrors rca's per-node `Cognitive::compute`
    /// plus its nesting-map inheritance.
    fn walk(&mut self, node: &Node, ctx: CogCtx, space: &mut CogSpace) {
        let child = cog_apply(node, ctx, space, self.rules);
        let mut i = 0;
        while i < node.child_count() {
            let c = node.child(i).expect("child within count");
            let kind = c.kind();
            if self.rules.fn_kinds.contains(&kind) {
                let depth = if child.in_fn { child.depth + 1 } else { child.depth };
                space.nested_sum += self.space(&c, CogCtx { nesting: 0, depth, lambda: child.lambda, in_fn: true });
            } else if self.rules.lambda_kinds.contains(&kind) {
                space.nested_sum += self.space(&c, CogCtx { lambda: child.lambda + 1, ..child });
            } else {
                self.walk(&c, child, space);
            }
            i += 1;
        }
    }
}

/// Apply one node's cognitive rule to `space` and return the context its children
/// inherit. A control point costs `nesting + depth + lambda + 1` and bumps nesting;
/// a flat kind (an `else` token) or a labeled `break`/`continue` costs `1`.
fn cog_apply(node: &Node, ctx: CogCtx, space: &mut CogSpace, rules: &Rules) -> CogCtx {
    let mut child = ctx;
    let kind = node.kind();
    if is_nesting_increase(node, kind, rules) {
        space.structural += ctx.nesting + ctx.depth + ctx.lambda + 1;
        space.bool_op = None;
        child.nesting = ctx.nesting + 1;
    } else if rules.cog_flat_kinds.contains(&kind) || (rules.cog_labeled_kinds.contains(&kind) && is_labeled(node, rules)) {
        space.structural += 1;
    } else if rules.cog_reset_kinds.contains(&kind) {
        space.bool_op = None;
    } else if rules.cog_unary_kinds.contains(&kind) {
        space.bool_op = Some(BoolOp::Not);
    } else if rules.cog_binary_kinds.contains(&kind) {
        cog_booleans(node, space, rules);
    }
    child
}

/// Whether `node` adds a nesting-weighted cost: one of the language's control
/// structures, excluding the `if` of an `else if` (already scored by its `else`).
fn is_nesting_increase(node: &Node, kind: &str, rules: &Rules) -> bool {
    rules.cog_nesting_kinds.contains(&kind) && !is_else_if(node, rules)
}

/// Whether `node` is the `if` of an `else if`, per the language's parent marker.
fn is_else_if(node: &Node, rules: &Rules) -> bool {
    let Some(parent_kind) = rules.else_if_parent else {
        return false;
    };
    node.parent().is_some_and(|p| p.kind() == parent_kind)
}

/// Whether a `break`/`continue` carries a label, per the language's label kinds.
fn is_labeled(node: &Node, rules: &Rules) -> bool {
    node.child(1).is_some_and(|c| rules.label_kinds.contains(&c.kind()))
}

/// rca's `compute_booleans`: scan a binary expression's direct `&&`/`||` children,
/// evaluating each against the running sequence.
fn cog_booleans(node: &Node, space: &mut CogSpace, rules: &Rules) {
    let mut i = 0;
    while i < node.child_count() {
        if let Some(op) = boolean_op(&node.child(i).expect("child within count"), rules) {
            eval_boolean(op, space);
        }
        i += 1;
    }
}

/// The logical operator a token node represents, per the language's operator
/// kinds, if it is one. (C-family uses `&&`/`||`; Python uses `and`/`or`.)
fn boolean_op(node: &Node, rules: &Rules) -> Option<BoolOp> {
    let kind = node.kind();
    if rules.bool_and_kinds.contains(&kind) {
        Some(BoolOp::And)
    } else if rules.bool_or_kinds.contains(&kind) {
        Some(BoolOp::Or)
    } else {
        None
    }
}

/// rca sets `boolean_op` only on the first operator of a sequence; a later,
/// differing operator adds `1` but does not become the new reference.
fn eval_boolean(op: BoolOp, space: &mut CogSpace) {
    match space.bool_op {
        None => {
            space.bool_op = Some(op);
            space.structural += 1;
        },
        Some(prev) => {
            if prev != op {
                space.structural += 1;
            }
        },
    }
}

#[cfg(test)]
mod tests {
    use super::super::rules::RUST;
    use super::*;
    use crate::language::Language;
    use crate::native::analyze;

    /// Parse Rust source and run `f` against the pure building block.
    fn on_rust<T>(src: &[u8], f: impl Fn(&Rules, &Tree, &[u8]) -> T) -> T {
        let a = analyze(Language::Rust, src).expect("Rust is native");
        f(&RUST, a.tree(), src)
    }

    #[test]
    fn test_cyclomatic_sums_subtree_including_nested_closures() {
        let src = b"fn simple() {}\nfn one_if(x: bool) { if x {} }\nfn two(a: bool, b: bool) { if a && b {} }\nfn nested() { let c = || { if true {} }; if false {} }\n";
        assert_eq!(
            on_rust(src, function_cyclomatic),
            vec![("simple".to_string(), 1), ("one_if".to_string(), 2), ("two".to_string(), 3), ("nested".to_string(), 4), ("<anonymous>".to_string(), 2)]
        );
    }

    #[test]
    fn test_cognitive_weights_nesting_and_booleans() {
        // outer if: +1; `&&`: +1; inner if at nesting 1: +2. total 4.
        let src = b"fn f(a: bool, b: bool) {\n    if a && b {\n        if a { }\n    }\n}\n";
        assert_eq!(on_rust(src, function_cognitive), vec![("f".to_string(), 4)]);
    }
}
