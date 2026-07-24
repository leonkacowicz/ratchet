//! JavaScript / TypeScript / TSX rule set.

use tree_sitter::Node;

use super::Rules;
use crate::native::analysis::{field_text, ANONYMOUS};

/// The JS family — JavaScript/JSX, TypeScript and TSX.
///
/// rca applies the same rules to all three (one `js_cognitive!` macro, identical
/// space kinds, cyclomatic set, naming fallback and non-arg kinds), so they share
/// one rule set here and differ only in which grammar parses them.
///
/// The family differs from Rust in three ways beyond node names: a function
/// declaration resets the lambda weight as well as nesting; every statement
/// resets the boolean sequence; and `function_expression`/`method_definition`/
/// generators are function *spaces* that are neither `fn` nor lambda for
/// cognitive purposes (they inherit their context unchanged).
const JS_BASE: Rules = Rules {
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
    fn_resets_nesting: true,
    cog_nesting_state_kinds: &[],
    cog_extra: None,
    extra_decision: None,
    name_of: js_family_function_name,
};

/// JavaScript/JSX, TypeScript and TSX all share this one rule set.
///
/// rca actually treats them slightly differently, but only through two bugs, which
/// ratchet deliberately does **not** reproduce (parity against them was proved
/// first, then dropped — see the migration epic):
///
/// * rca's TS/TSX naming fallback compares the parent's kind id against the *Mozjs*
///   enum, so it never fires and anonymous functions stay `"<anonymous>"` there
///   while JavaScript names them from their `variable_declarator`/`pair`.
/// * rca's `TsxCode::is_else_if` tests for an `IfStatement` parent, but an
///   `else if`'s parent is always an `else_clause`, so rca never detects an
///   else-if in TSX and charges each a full nesting increment.
///
/// Both are arbitrary inconsistencies within one language family, so ratchet
/// applies JavaScript's (correct) behaviour uniformly.
pub static JS_FAMILY: Rules = JS_BASE;

/// rca's JS-family naming: an anonymous function takes its name from an enclosing
/// `pair` (`foo: function () {}`) or `variable_declarator` (`var f = () => {}`).
fn js_family_function_name(node: &Node, source: &[u8]) -> String {
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
