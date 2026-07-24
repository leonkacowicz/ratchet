//! Per-language node-kind rule sets for the native metric path.
//!
//! The metric *algorithms* are shared across languages (space walk, SLOC by node
//! span, function counting, argument counting, cyclomatic decision points,
//! cognitive nesting + boolean sequences). What differs is which tree-sitter node
//! kinds play each role — plus a couple of structural quirks. Those live here, so
//! adding a language is mostly a matter of supplying its kinds and a grammar.
//!
//! Every set mirrors what `rust-code-analysis` matches for that language, since
//! rca is the parity oracle each migration is verified against.
//!
//! # Adding a language
//!
//! The metric code is language-agnostic — a new language supplies *data*, not
//! algorithms:
//!
//! 1. Vendor its grammar under `vendor/<grammar>/` (`parser.c`, any
//!    `scanner.{c,cc}`, the `tree_sitter/` headers, its `LICENSE`) and add one
//!    `compile_grammar(..)` call in `build.rs`.
//! 2. Declare its `tree_sitter_<lang>()` extern and add one arm to
//!    `native::grammar` — that is all [`super::supports`] needs.
//! 3. Add a `Rules` entry here and map it in [`for_language`].
//! 4. Verify against rca with the parity harness (`parity`'s corpus tests) and
//!    add fixtures under `tests/fixtures/` exercising the language's constructs.
//!
//! Nothing else should need editing. If a language cannot be expressed by these
//! fields, prefer adding a field here over branching inside the algorithms — the
//! one known case that will need it is C/C++, whose names and argument lists hang
//! off a `declarator` rather than a `name` field.

use tree_sitter::Node;

/// The node kinds that drive each metric for one language.
#[derive(Clone, Copy)]
pub struct Rules {
    /// Kinds that form a function space (functions, methods, closures/lambdas).
    /// Drives the walk, the function count, and cyclomatic's per-space base.
    pub function_kinds: &'static [&'static str],
    /// Function-space kinds that reset cognitive nesting and deepen function depth.
    pub fn_kinds: &'static [&'static str],
    /// Function-space kinds weighted as lambdas by cognitive complexity.
    pub lambda_kinds: &'static [&'static str],
    /// Children of a `parameters` node that are *not* arguments (delimiters etc.).
    pub non_arg_kinds: &'static [&'static str],
    /// Cyclomatic decision points (rca counts keyword tokens as well as nodes).
    pub decision_kinds: &'static [&'static str],
    /// Cognitive: control structures costing `nesting + depth + lambda + 1`.
    pub cog_nesting_kinds: &'static [&'static str],
    /// Cognitive: kinds costing a flat `1` (e.g. an `else` token).
    pub cog_flat_kinds: &'static [&'static str],
    /// Cognitive: kinds costing a flat `1` only when labeled (`break`/`continue`).
    pub cog_labeled_kinds: &'static [&'static str],
    /// Cognitive: kinds that reset the boolean sequence (the JS family does this
    /// per statement; Rust does not).
    pub cog_reset_kinds: &'static [&'static str],
    /// Cognitive: kinds whose direct boolean-operator children form a sequence.
    pub cog_binary_kinds: &'static [&'static str],
    /// Cognitive: unary kinds that mark the boolean sequence (rca's `not_operator`).
    pub cog_unary_kinds: &'static [&'static str],
    /// The language's logical-AND operator tokens (`&&`; Python's is `and`).
    pub bool_and_kinds: &'static [&'static str],
    /// The language's logical-OR operator tokens (`||`; Python's is `or`).
    pub bool_or_kinds: &'static [&'static str],
    /// Kinds a labeled `break`/`continue` carries as its label child (Rust
    /// `label`; the JS family uses `statement_identifier`).
    pub label_kinds: &'static [&'static str],
    /// Parent kind marking an `else if`, whose `if` must not add nesting (it is
    /// already scored by the `else`). `None` when the language has no such form.
    pub else_if_parent: Option<&'static str>,
    /// Whether entering a `fn_kinds` space also resets the lambda weight. The JS
    /// family does (rca zeroes `lambda` at a function declaration); Rust does not.
    pub fn_resets_lambda: bool,
    /// Whether entering a `fn_kinds` space restarts control-structure nesting.
    /// Rust and the JS family do; Python does not (rca only deepens its function
    /// depth), so a nested `def` keeps the enclosing nesting weight.
    pub fn_resets_nesting: bool,
    /// Cognitive: kinds charged rca's *stateful* increment — `stats.nesting + 1`,
    /// where `stats.nesting` is whatever the last nesting increase in this space
    /// set it to (Python's `except_clause`; it is 0 when nothing preceded it).
    pub cog_nesting_state_kinds: &'static [&'static str],
    /// Cognitive: extra structural cost a node contributes beyond the standard
    /// rules. Python needs it for its boolean-operator/lambda-ancestor rule.
    pub cog_extra: Option<fn(&Node) -> u64>,
    /// A context-dependent decision point cyclomatic must also count, beyond
    /// [`Self::decision_kinds`]. Python needs it: an `else` counts when it closes
    /// a suite, but not when it belongs to a conditional expression.
    pub extra_decision: Option<fn(&Node) -> bool>,
    /// How a function's entity name is derived. Behaviour-as-data: the shared walk
    /// just calls this, so naming never branches on a language. Most languages use
    /// [`default_function_name`]; the JS family looks through a `pair` /
    /// `variable_declarator` parent, and C/C++ will read its `declarator`.
    pub name_of: fn(&Node, &[u8]) -> String,
}

impl Rules {
    /// Whether `kind` forms a function space — drives the walk, entity naming and
    /// cyclomatic's per-space base.
    pub fn is_function(&self, kind: &str) -> bool {
        self.function_kinds.contains(&kind)
    }

    /// Whether `kind` counts toward the file's function count (rca's `nom`, which
    /// sums functions *and* closures).
    ///
    /// Usually the same set as [`Self::is_function`], because a language's
    /// closures are also function spaces. Python is the exception: its `lambda`
    /// is counted by `nom` but is not a space, so it never appears in the walk.
    pub fn counts_toward_nom(&self, kind: &str) -> bool {
        self.is_function(kind) || self.lambda_kinds.contains(&kind)
    }
}

/// Every `else` that closes a suite counts toward Python's cyclomatic complexity.
///
/// rca *intends* to count only a loop-`else`, but its `has_ancestors(typ, typs)`
/// helper only climbs when the parent matches `typ` (`for`/`while`) — and an
/// `else`'s parent is always an `else_clause`, so it never climbs and then matches
/// that same parent against `typs` (`ElseClause`), which always holds. The guard
/// therefore admits every `else`. A conditional expression's `else` still does not
/// count: its parent is the expression, not an `else_clause`.
mod java;
mod js;
mod python;
mod rust;

pub use java::JAVA;
pub use js::JS_FAMILY;
pub use python::PYTHON;
pub use rust::RUST;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_function_matches_a_language_s_function_spaces() {
        assert!(RUST.is_function("function_item"));
        assert!(RUST.is_function("closure_expression"));
        // A body-less trait signature is not a function space.
        assert!(!RUST.is_function("function_signature_item"));
    }

    #[test]
    fn test_nom_counts_closures_that_are_not_function_spaces() {
        // Rust closures are themselves spaces, so the two sets coincide.
        assert!(RUST.is_function("closure_expression"));
        assert!(RUST.counts_toward_nom("closure_expression"));
        // A Python lambda is counted by `nom` but is not a space, so it never
        // appears in the walk.
        assert!(!PYTHON.is_function("lambda"));
        assert!(PYTHON.counts_toward_nom("lambda"));
    }
}
