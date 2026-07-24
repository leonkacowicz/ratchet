//! Java rule set.

use super::Rules;
use crate::native::analysis::default_function_name;

/// Java.
///
/// Parity against rca was proved first, then three of its defects were dropped
/// rather than inherited (the same call made for TypeScript and TSX):
///
/// * rca's `is_non_arg` always returns `false`, so *nothing* is excluded when
///   counting arguments — the parentheses and commas of `formal_parameters` count
///   too and a two-argument method scores 5. Ratchet excludes the delimiters, as
///   it does for every other language.
/// * rca's `is_else_if` always returns `false`, so an `else if` is never
///   recognised and takes a full nesting increment. Ratchet detects it: Java has
///   no `else_clause` node, so an `else if` is simply the `if_statement` sitting
///   in the outer statement's `alternative` — hence the `if_statement` marker.
///   (A merely nested `if` has a `block` parent, so it is unaffected.)
/// * rca's cyclomatic counts the `for` *token* so it sees an enhanced
///   `for (X x : xs)`, while its cognitive matches only `for_statement` and misses
///   it. Ratchet counts the enhanced form in both.
///
/// One genuine rca behaviour is kept: its cognitive rules have no method arm, so a
/// method neither restarts nesting nor deepens function depth — hence the empty
/// `fn_kinds`. That is a modelling choice, not a defect.
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
    non_arg_kinds: &["(", ")", ","],
    decision_kinds: &["if", "for", "while", "case", "catch", "ternary_expression", "&&", "||"],
    cog_nesting_kinds: &["if_statement", "for_statement", "enhanced_for_statement", "while_statement", "do_statement", "switch_block", "catch_clause"],
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
    else_if_parent: Some("if_statement"),
    fn_resets_lambda: false,
    fn_resets_nesting: false,
    name_of: default_function_name,
    params_via_declarator: false,
};
