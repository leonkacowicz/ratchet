//! C / C++ rule set — one grammar covers both, as in rca.

use tree_sitter::Node;

use super::Rules;
use crate::native::analysis::field_text;

/// C and C++.
///
/// The language rca models least like the others:
///
/// * a function's name and its parameters both hang off a `declarator` rather
///   than a `name` field, so this set supplies its own `name_of` and sets
///   `params_via_declarator`;
/// * when no name can be recovered rca yields *no* name at all (not
///   `"<anonymous>"`), which is what makes its `{closure_N}` numbering fire — the
///   only language where it does;
/// * a `lambda_expression` is not a function space (it is in neither `is_func`
///   nor `is_func_space`), so like Python's and Java's it is counted by `nom` and
///   lends its arguments and nesting weight to the enclosing function;
/// * `goto` costs a flat point, which no other language here has.
pub static CPP: Rules = Rules {
    function_kinds: &["function_definition"],
    // rca has no function arm in its C/C++ cognitive rules, so a function neither
    // restarts nesting nor deepens depth.
    fn_kinds: &[],
    lambda_kinds: &["lambda_expression"],
    non_arg_kinds: &["(", ")", ","],
    decision_kinds: &["if", "for", "while", "case", "catch", "conditional_expression", "&&", "||"],
    cog_nesting_kinds: &["if_statement", "for_statement", "while_statement", "do_statement", "switch_statement", "catch_clause"],
    cog_flat_kinds: &["else", "goto_statement"],
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
    else_if_parent: Some("else_clause"),
    fn_resets_lambda: false,
    fn_resets_nesting: false,
    name_of: cpp_function_name,
    params_via_declarator: true,
};

/// Node kinds rca accepts as the identifier inside a `function_declarator`.
const CPP_NAME_KINDS: &[&str] =
    &["type_identifier", "identifier", "field_identifier", "destructor_name", "operator_name", "qualified_identifier", "template_function", "template_method"];

/// rca's C/C++ naming: an `operator_cast` names itself; otherwise descend the
/// `declarator` to the first `function_declarator` and take its leading
/// identifier. Returning `None` when neither is found is deliberate — that is
/// what makes the walk number the function `{closure_N}`.
fn cpp_function_name(node: &Node, source: &[u8]) -> Option<String> {
    if let Some(cast) = first_child_of_kind(node, "operator_cast") {
        return text_of(&cast, source);
    }
    let declarator = node.child_by_field_name("declarator")?;
    let function_declarator = first_descendant_of_kind(&declarator, "function_declarator")?;
    let first = function_declarator.child(0)?;
    if !CPP_NAME_KINDS.contains(&first.kind()) {
        return None;
    }
    text_of(&first, source)
}

fn text_of(node: &Node, source: &[u8]) -> Option<String> {
    std::str::from_utf8(&source[node.byte_range()]).ok().map(str::to_string)
}

fn first_child_of_kind<'a>(node: &Node<'a>, kind: &str) -> Option<Node<'a>> {
    let mut i = 0;
    while i < node.child_count() {
        let child = node.child(i as u32)?;
        if child.kind() == kind {
            return Some(child);
        }
        i += 1;
    }
    None
}

/// The first node of `kind` in `node`'s subtree, `node` itself included.
fn first_descendant_of_kind<'a>(node: &Node<'a>, kind: &str) -> Option<Node<'a>> {
    if node.kind() == kind {
        return Some(*node);
    }
    let mut i = 0;
    while i < node.named_child_count() {
        let child = node.named_child(i as u32)?;
        if let Some(found) = first_descendant_of_kind(&child, kind) {
            return Some(found);
        }
        i += 1;
    }
    None
}

/// Unused here but kept for symmetry with the other rule modules.
#[allow(dead_code)]
fn _unused(node: &Node, source: &[u8]) -> Option<String> {
    field_text(node, "name", source)
}
