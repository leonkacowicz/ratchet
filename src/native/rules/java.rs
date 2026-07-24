//! Java rule set.

use super::Rules;
use crate::native::analysis::default_function_name;

/// Java.
///
/// Three rca behaviours shape this set, all of them quirks rather than choices:
///
/// * `is_non_arg` always returns `false`, so *nothing* is excluded when counting
///   arguments — the parentheses and commas of `formal_parameters` count too, and
///   a two-argument method scores 5. Reproduced with an empty `non_arg_kinds`.
/// * `is_else_if` always returns `false`, so an `else if` is never recognised and
///   takes a full nesting increment (as in TSX). Reproduced with no
///   `else_if_parent` marker.
/// * Java's cognitive rules have no method arm at all, so a method neither
///   restarts nesting nor deepens function depth — hence the empty `fn_kinds`.
///
/// A `lambda_expression` is *not* a function space: rca creates spaces from
/// `is_func || is_func_space`, and a Java lambda is in neither (despite
/// `get_space_kind` labelling it `Function`). Like a Python `lambda` it is counted
/// by `nom`, contributes its arguments to the enclosing method, and carries
/// nesting weight — but never appears in the walk.
///
/// Note also the asymmetry rca has between the metrics: cyclomatic counts the
/// `for` *token*, so it sees an enhanced `for (X x : xs)`, while cognitive matches
/// only `for_statement` and misses it.
pub static JAVA: Rules = Rules {
    function_kinds: &["method_declaration", "constructor_declaration"],
    fn_kinds: &[],
    lambda_kinds: &["lambda_expression"],
    non_arg_kinds: &[],
    decision_kinds: &["if", "for", "while", "case", "catch", "ternary_expression", "&&", "||"],
    cog_nesting_kinds: &["if_statement", "for_statement", "while_statement", "do_statement", "switch_block", "catch_clause"],
    cog_flat_kinds: &["else"],
    cog_labeled_kinds: &[],
    cog_reset_kinds: &[],
    cog_binary_kinds: &["binary_expression"],
    cog_unary_kinds: &["unary_expression"],
    bool_and_kinds: &["&&"],
    bool_or_kinds: &["||"],
    label_kinds: &[],
    cog_nesting_state_kinds: &[],
    cog_extra: None,
    extra_decision: None,
    else_if_parent: None,
    fn_resets_lambda: false,
    fn_resets_nesting: false,
    name_of: default_function_name,
    params_via_declarator: false,
};
