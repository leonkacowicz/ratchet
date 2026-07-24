//! Python rule set.

use tree_sitter::Node;

use super::Rules;
use crate::native::analysis::default_function_name;

/// Python.
///
/// Differs structurally from the C-family languages: its logical operators are
/// the words `and`/`or`, a `lambda` carries nesting weight without being a
/// function space (rca only treats `function_definition` as one), a nested `def`
/// does not restart nesting, and `elif`/`else`/`finally`/`except` each have their
/// own cognitive treatment.
pub static PYTHON: Rules = Rules {
    function_kinds: &["function_definition"],
    fn_kinds: &["function_definition"],
    lambda_kinds: &["lambda"],
    non_arg_kinds: &["(", ")", ","],
    // rca counts the keyword tokens, including `with`/`assert` and the word
    // operators; the loop-`else` case is handled by `extra_decision`.
    decision_kinds: &["if", "elif", "for", "while", "except", "with", "assert", "and", "or"],
    cog_nesting_kinds: &["if_statement", "for_statement", "while_statement", "conditional_expression"],
    cog_flat_kinds: &["elif_clause", "else_clause", "finally_clause"],
    cog_labeled_kinds: &[],
    cog_reset_kinds: &["expression_list", "expression_statement", "tuple"],
    cog_binary_kinds: &["boolean_operator"],
    cog_unary_kinds: &["not_operator"],
    bool_and_kinds: &["and"],
    bool_or_kinds: &["or"],
    label_kinds: &[],
    // Python has no `else if`; it has a distinct `elif_clause`.
    else_if_parent: None,
    fn_resets_lambda: false,
    fn_resets_nesting: false,
    cog_nesting_state_kinds: &["except_clause"],
    cog_extra: Some(python_boolean_lambda_depth),
    extra_decision: Some(python_else),
    name_of: default_function_name,
    params_via_declarator: false,
};

/// rca charges a Python `boolean_operator` an extra point per enclosing `lambda`.
///
/// Its rule: when the operator has no `boolean_operator` ancestor (stopping at a
/// `lambda`), add the number of `lambda` ancestors, stopping at an
/// `expression_list`/`if`/`for`/`while`. So `lambda v: v > 0 and v < 10` scores the
/// boolean sequence *plus* one for sitting inside a lambda.
fn python_boolean_lambda_depth(node: &Node) -> u64 {
    if node.kind() != "boolean_operator" {
        return 0;
    }
    if count_ancestors(node, "boolean_operator", &["lambda"]) > 0 {
        return 0;
    }
    count_ancestors(node, "lambda", &["expression_list", "if_statement", "for_statement", "while_statement"])
}

/// Count ancestors of `node` whose kind is `check`, walking up until an ancestor
/// matches one of `stop` (mirrors rca's `count_specific_ancestors`).
fn count_ancestors(node: &Node, check: &str, stop: &[&str]) -> u64 {
    let mut count = 0;
    let mut current = *node;
    while let Some(parent) = current.parent() {
        if stop.contains(&parent.kind()) {
            break;
        }
        if parent.kind() == check {
            count += 1;
        }
        current = parent;
    }
    count
}

fn python_else(node: &Node) -> bool {
    node.kind() == "else" && node.parent().is_some_and(|p| p.kind() == "else_clause")
}
