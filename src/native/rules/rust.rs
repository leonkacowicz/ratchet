//! Rust rule set.

use super::Rules;
use crate::native::analysis::default_function_name;

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
    fn_resets_nesting: true,
    cog_nesting_state_kinds: &[],
    cog_extra: None,
    extra_decision: None,
    name_of: default_function_name,
    params_via_declarator: false,
};
