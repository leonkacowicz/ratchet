//! Counting metrics over the raw tree-sitter tree: SLOC by node span, function
//! counts, and argument counts, parameterized by the language's [`Rules`].

use tree_sitter::Node;

use super::rules::Rules;
use super::{parse_with_rules, visit_functions};
use crate::language::Language;

/// File-level SLOC, matching rca's `loc.sloc()` on the unit: the root node's
/// `end_row - start_row`. `None` if the language is not native or parsing fails.
pub fn file_lines(lang: Language, source: &[u8]) -> Option<u64> {
    let (tree, _) = parse_with_rules(lang, source)?;
    let root = tree.root_node();
    Some((root.end_position().row - root.start_position().row) as u64)
}

/// File-level function count, matching rca's `nom.total()`: the number of
/// function spaces. `None` if the language is not native or parsing fails.
pub fn file_functions(lang: Language, source: &[u8]) -> Option<u64> {
    let (tree, rules) = parse_with_rules(lang, source)?;
    let mut count = 0u64;
    visit_functions(rules, &tree, source, &mut |_name, _node| count += 1);
    Some(count)
}

/// Per-function SLOC as `(entity_name, value)` in walk order — matching rca's
/// `loc.sloc()` on a function space (the non-unit branch, `end_row - start_row + 1`
/// of the function node). Empty when the language is not native or parsing fails.
pub fn function_lines(lang: Language, source: &[u8]) -> Vec<(String, u64)> {
    let Some((tree, rules)) = parse_with_rules(lang, source) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    visit_functions(rules, &tree, source, &mut |name, node| {
        let lines = (node.end_position().row - node.start_position().row) as u64 + 1;
        out.push((name.to_string(), lines));
    });
    out
}

/// Per-function argument counts as `(entity_name, nargs)` in walk order, matching
/// rca's `nargs`. Empty when the language is not native or parsing fails.
pub fn function_nargs(lang: Language, source: &[u8]) -> Vec<(String, u64)> {
    let Some((tree, rules)) = parse_with_rules(lang, source) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    visit_functions(rules, &tree, source, &mut |name, node| out.push((name.to_string(), nargs_of(&node, rules))));
    out
}

/// Argument count of a function node, mirroring rca's `compute_args` and
/// `is_non_arg`: every child of the `parameters` field except the language's
/// delimiter/attribute kinds. For Rust that means `self` counts as an argument.
fn nargs_of(node: &Node, rules: &Rules) -> u64 {
    let Some(params) = node.child_by_field_name("parameters") else {
        return 0;
    };
    let mut cursor = params.walk();
    params.children(&mut cursor).filter(|c| !rules.non_arg_kinds.contains(&c.kind())).count() as u64
}

/// Ordered function entity names for a file, matching the rca path's function
/// list. Empty when the language is not native or parsing fails.
pub fn function_entities(lang: Language, source: &[u8]) -> Vec<String> {
    let Some((tree, rules)) = parse_with_rules(lang, source) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    visit_functions(rules, &tree, source, &mut |name, _node| out.push(name.to_string()));
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_function_entities_names_functions_and_closures() {
        let src = b"struct S;\nimpl S {\n    fn new() -> S { let f = || 1; S }\n}\nfn top() {}\n";
        assert_eq!(function_entities(Language::Rust, src), vec!["new", "<anonymous>", "top"]);
    }

    #[test]
    fn test_function_lines_spans_the_function_node() {
        let src = b"fn f() {\n    let x = 1;\n}\nfn g() { 1 }\n";
        assert_eq!(function_lines(Language::Rust, src), vec![("f".to_string(), 3), ("g".to_string(), 1)]);
    }

    #[test]
    fn test_nargs_counts_self_params_and_closure_args() {
        // `self` counts; a 2-arg method is 3; the closure `|x|` is 1; no-arg fn is 0.
        let src = b"struct S;\nimpl S { fn m(&self, a: i32, b: u8) { let c = |x: i32| x; } }\nfn n() {}\n";
        assert_eq!(function_nargs(Language::Rust, src), vec![("m".to_string(), 3), ("<anonymous>".to_string(), 1), ("n".to_string(), 0)]);
    }

    #[test]
    fn test_metrics_are_empty_for_a_language_without_a_grammar() {
        assert!(file_lines(Language::Python, b"def f(): pass").is_none());
        assert!(function_nargs(Language::Python, b"def f(): pass").is_empty());
    }
}
