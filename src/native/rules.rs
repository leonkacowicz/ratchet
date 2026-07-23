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

use super::analysis::{default_function_name, field_text, ANONYMOUS};

/// The node kinds that drive each metric for one language.
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
    /// How a function's entity name is derived. Behaviour-as-data: the shared walk
    /// just calls this, so naming never branches on a language. Most languages use
    /// [`default_function_name`]; the JS family looks through a `pair` /
    /// `variable_declarator` parent, and C/C++ will read its `declarator`.
    pub name_of: fn(&Node, &[u8]) -> String,
}

impl Rules {
    /// Whether `kind` forms a function space.
    pub fn is_function(&self, kind: &str) -> bool {
        self.function_kinds.contains(&kind)
    }
}

/// Rust — the reference implementation, verified byte-for-byte against rca.
pub static RUST: Rules = Rules {
    function_kinds: &["function_item", "closure_expression"],
    fn_kinds: &["function_item"],
    lambda_kinds: &["closure_expression"],
    non_arg_kinds: &["(", ")", ",", "|", "attribute_item"],
    // rca counts the keyword *tokens*, so match guards and `if let` count too.
    decision_kinds: &["if", "for", "while", "loop", "match_arm", "try_expression", "&&", "||"],
    cog_nesting_kinds: &["if_expression", "for_expression", "while_expression", "match_expression"],
    // The `else` token covers `else`, `else if`, and `let ... else` alike.
    cog_flat_kinds: &["else"],
    cog_labeled_kinds: &["break_expression", "continue_expression"],
    cog_reset_kinds: &[],
    cog_binary_kinds: &["binary_expression"],
    cog_unary_kinds: &["unary_expression"],
    bool_and_kinds: &["&&"],
    bool_or_kinds: &["||"],
    label_kinds: &["label"],
    else_if_parent: Some("else_clause"),
    fn_resets_lambda: false,
    name_of: default_function_name,
};

/// JavaScript (including JSX) via the vendored mozjs grammar — the same grammar
/// and rules rca applies to `.js`/`.mjs`/`.cjs`/`.jsx`.
///
/// Note the JS family differs from Rust in three ways beyond node names: a
/// function declaration resets the lambda weight as well as nesting; every
/// statement resets the boolean sequence; and `function_expression`/
/// `method_definition`/generators are function *spaces* that are neither `fn` nor
/// lambda for cognitive purposes (they inherit their context unchanged).
pub static JAVASCRIPT: Rules = Rules {
    function_kinds: &[
        "function_declaration",
        "generator_function_declaration",
        "function_expression",
        "generator_function",
        "method_definition",
        "arrow_function",
    ],
    fn_kinds: &["function_declaration"],
    lambda_kinds: &["arrow_function"],
    non_arg_kinds: &["(", ")", ","],
    // rca's Mozjs cyclomatic: if/for/while keywords, switch `case`, `catch`,
    // the ternary, and the boolean operators. Note `do` is not counted.
    decision_kinds: &["if", "for", "while", "case", "catch", "ternary_expression", "&&", "||"],
    cog_nesting_kinds: &[
        "if_statement",
        "for_statement",
        "for_in_statement",
        "while_statement",
        "do_statement",
        "switch_statement",
        "catch_clause",
        "ternary_expression",
    ],
    cog_flat_kinds: &["else"],
    // rca's JS cognitive has no labeled break/continue rule.
    cog_labeled_kinds: &[],
    cog_reset_kinds: &["expression_statement"],
    cog_binary_kinds: &["binary_expression"],
    cog_unary_kinds: &["unary_expression"],
    bool_and_kinds: &["&&"],
    bool_or_kinds: &["||"],
    label_kinds: &["statement_identifier"],
    else_if_parent: Some("else_clause"),
    fn_resets_lambda: true,
    name_of: javascript_function_name,
};

/// rca's JS naming: an anonymous function takes its name from an enclosing
/// `pair` (`foo: function () {}`) or `variable_declarator` (`var f = () => {}`).
fn javascript_function_name(node: &Node, source: &[u8]) -> String {
    if let Some(name) = field_text(node, "name", source) {
        return name;
    }
    let borrowed = node.parent().and_then(|parent| {
        let field = match parent.kind() {
            "pair" => "key",
            "variable_declarator" => "name",
            _ => return None,
        };
        field_text(&parent, field, source)
    });
    borrowed.unwrap_or_else(|| ANONYMOUS.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_function_matches_rust_function_spaces() {
        assert!(RUST.is_function("function_item"));
        assert!(RUST.is_function("closure_expression"));
        assert!(!RUST.is_function("function_signature_item"));
    }
}
