//! Cyclomatic and cognitive complexity computed over the raw tree-sitter tree,
//! matching rca's `cyclomatic_sum` / `cognitive_sum` (both subtree sums).

use tree_sitter::Node;

use super::{parse_rust, rust_function_name, visit_rust_functions};

/// Per-function cyclomatic complexity for Rust `source`, as `(entity_name, value)`
/// in walk order — matching rca's `cyclomatic_sum` (a subtree sum). Empty when the
/// source fails to parse.
pub fn rust_function_cyclomatic(source: &[u8]) -> Vec<(String, u64)> {
    let Some(tree) = parse_rust(source) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    visit_rust_functions(&tree, source, &mut |name, node| out.push((name.to_string(), cyclomatic_of(&node))));
    out
}

/// Cyclomatic complexity of a Rust function node, matching rca's `cyclomatic_sum`
/// over the function's subtree: a base `1` for the function plus `1` for every
/// nested function/closure space (each carries its own base) and `1` for every
/// decision-point rca counts. rca counts the keyword *tokens* `if`/`for`/`while`/
/// `loop` (so match guards and `if let` count too), each `match_arm`, the `?`
/// operator (`try_expression`), and the `&&`/`||` operators.
fn cyclomatic_of(func: &Node) -> u64 {
    let mut total = 1;
    count_cyclomatic(func, &mut total);
    total
}

fn count_cyclomatic(node: &Node, total: &mut u64) {
    let mut i = 0;
    while i < node.child_count() {
        let child = node.child(i).expect("child within count");
        match child.kind() {
            "function_item" | "closure_expression" => *total += 1,
            "if" | "for" | "while" | "loop" | "match_arm" | "try_expression" | "&&" | "||" => *total += 1,
            _ => {},
        }
        count_cyclomatic(&child, total);
        i += 1;
    }
}

/// Boolean operator tracked for cognitive complexity's boolean-sequence rule.
/// `Not` is the sentinel rca sets on a unary expression (`!`), distinct from
/// `&&`/`||` so the next operator in a sequence counts as a change.
#[derive(Clone, Copy, PartialEq)]
enum BoolOp {
    And,
    Or,
    Not,
}

/// Inherited cognitive context carried top-down: control-structure `nesting`,
/// nested-function `depth`, closure `lambda`, and whether we are inside a function.
#[derive(Clone, Copy)]
struct CogCtx {
    nesting: u64,
    depth: u64,
    lambda: u64,
    in_fn: bool,
}

/// A cognitive-complexity accumulator for one space (rca `Stats`): the space's own
/// `structural` cost, the summed cost of nested spaces, and the current boolean op.
#[derive(Default)]
struct CogSpace {
    structural: u64,
    nested_sum: u64,
    bool_op: Option<BoolOp>,
}

/// Per-function cognitive complexity for Rust `source`, as `(entity_name, value)`
/// in walk order — matching rca's `cognitive_sum` (subtree sum). Empty when the
/// source fails to parse.
pub fn rust_function_cognitive(source: &[u8]) -> Vec<(String, u64)> {
    let Some(tree) = parse_rust(source) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    Cog { source, out: &mut out }.walk(&tree.root_node(), CogCtx { nesting: 0, depth: 0, lambda: 0, in_fn: false }, &mut CogSpace::default());
    out
}

/// Traversal state shared across the whole cognitive walk: the source bytes and
/// the accumulating `(entity, value)` output. Bundled so the recursive methods
/// stay within ratchet's own argument-count limit.
struct Cog<'a> {
    source: &'a [u8],
    out: &'a mut Vec<(String, u64)>,
}

impl Cog<'_> {
    /// Process a function/closure space node: emit its entry (pre-order, matching
    /// the walk) and return its cognitive_sum (own `structural` + nested spaces).
    fn space(&mut self, node: &Node, ctx: CogCtx) -> u64 {
        let idx = self.out.len();
        self.out.push((rust_function_name(node, self.source), 0));
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

    /// Apply `node`'s cognitive rule to `space`, then recurse — nested
    /// function/closure nodes start their own spaces. Mirrors rca's per-node
    /// `Cognitive::compute` for Rust plus its nesting-map inheritance.
    fn walk(&mut self, node: &Node, ctx: CogCtx, space: &mut CogSpace) {
        let child = cog_apply(node, ctx, space);
        let mut i = 0;
        while i < node.child_count() {
            let c = node.child(i).expect("child within count");
            match c.kind() {
                "function_item" => {
                    let depth = if child.in_fn { child.depth + 1 } else { child.depth };
                    space.nested_sum += self.space(&c, CogCtx { nesting: 0, depth, lambda: child.lambda, in_fn: true });
                },
                "closure_expression" => {
                    space.nested_sum += self.space(&c, CogCtx { lambda: child.lambda + 1, ..child });
                },
                _ => self.walk(&c, child, space),
            }
            i += 1;
        }
    }
}

/// Apply one node's cognitive rule to `space` and return the context its children
/// inherit. A control point costs `nesting + depth + lambda + 1` and bumps nesting;
/// an `else` token or a labeled `break`/`continue` costs a flat `1`.
fn cog_apply(node: &Node, ctx: CogCtx, space: &mut CogSpace) -> CogCtx {
    let mut child = ctx;
    let kind = node.kind();
    if is_nesting_increase(node, kind) {
        space.structural += ctx.nesting + ctx.depth + ctx.lambda + 1;
        space.bool_op = None;
        child.nesting = ctx.nesting + 1;
    } else if kind == "else" || (is_break_continue(kind) && is_labeled(node)) {
        // rca counts the `else` *token* (enum `Else`) — in an `else_clause`
        // (plain/`else if`) and a `let ... else` alike — plus labeled loop jumps.
        space.structural += 1;
    } else if kind == "unary_expression" {
        space.bool_op = Some(BoolOp::Not);
    } else if kind == "binary_expression" {
        cog_booleans(node, space);
    }
    child
}

/// Whether `node` adds a nesting-weighted cost: a `for`/`while`/`match`, or an
/// `if` that is not the `if` of an `else if` (those are scored by their `else`).
fn is_nesting_increase(node: &Node, kind: &str) -> bool {
    matches!(kind, "for_expression" | "while_expression" | "match_expression")
        || (kind == "if_expression" && node.parent().is_none_or(|p| p.kind() != "else_clause"))
}

fn is_break_continue(kind: &str) -> bool {
    matches!(kind, "break_expression" | "continue_expression")
}

fn is_labeled(node: &Node) -> bool {
    node.child(1).is_some_and(|c| c.kind() == "label")
}

/// rca's `compute_booleans`: scan a binary expression's direct `&&`/`||` children,
/// evaluating each against the running sequence.
fn cog_booleans(node: &Node, space: &mut CogSpace) {
    let mut i = 0;
    while i < node.child_count() {
        if let Some(op) = boolean_op(&node.child(i).expect("child within count")) {
            eval_boolean(op, space);
        }
        i += 1;
    }
}

/// The boolean operator of a `&&`/`||` token node, if it is one.
fn boolean_op(node: &Node) -> Option<BoolOp> {
    match node.kind() {
        "&&" => Some(BoolOp::And),
        "||" => Some(BoolOp::Or),
        _ => None,
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
