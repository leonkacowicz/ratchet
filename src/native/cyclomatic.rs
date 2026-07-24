//! Cyclomatic complexity over the raw tree-sitter tree, matching rca's
//! `cyclomatic_sum` (a subtree sum). Language-agnostic: the decision points come
//! from the language's [`Rules`].

use tree_sitter::{Node, Tree};

use super::analysis::visit_functions;
use super::rules::Rules;

/// Per-function cyclomatic complexity as `(entity_name, value)` in walk order —
/// matching rca's `cyclomatic_sum`.
pub fn function_cyclomatic(rules: &Rules, tree: &Tree, source: &[u8]) -> Vec<(String, u64)> {
    let mut out = Vec::new();
    visit_functions(rules, tree, source, &mut |name, node| out.push((name.to_string(), cyclomatic_of(&node, rules))));
    out
}

/// Cyclomatic complexity of a function node, matching rca's `cyclomatic_sum` over
/// its subtree: a base `1` for the function, plus `1` for every nested function
/// space (each carries its own base) and `1` for every decision point.
fn cyclomatic_of(func: &Node, rules: &Rules) -> u64 {
    let mut total = 1;
    count_cyclomatic(func, rules, &mut total);
    total
}

fn count_cyclomatic(node: &Node, rules: &Rules, total: &mut u64) {
    let mut i = 0;
    while i < node.child_count() {
        let child = node.child(i as u32).expect("child within count");
        let kind = child.kind();
        if rules.is_function(kind) || rules.decision_kinds.contains(&kind) || rules.extra_decision.is_some_and(|f| f(&child)) {
            *total += 1;
        }
        count_cyclomatic(&child, rules, total);
        i += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::super::rules::RUST;
    use super::*;
    use crate::language::Language;
    use crate::native::analyze;

    /// Parse Rust source and run `f` against the pure building block.
    fn on_rust<T>(src: &[u8], f: impl Fn(&Rules, &Tree, &[u8]) -> T) -> T {
        let a = analyze(Language::Rust, src).expect("Rust is native");
        f(&RUST, a.tree(), src)
    }

    #[test]
    fn test_cyclomatic_sums_subtree_including_nested_closures() {
        let src = b"fn simple() {}\nfn one_if(x: bool) { if x {} }\nfn two(a: bool, b: bool) { if a && b {} }\nfn nested() { let c = || { if true {} }; if false {} }\n";
        assert_eq!(
            on_rust(src, function_cyclomatic),
            vec![("simple".to_string(), 1), ("one_if".to_string(), 2), ("two".to_string(), 3), ("nested".to_string(), 4), ("<anonymous>".to_string(), 2)]
        );
    }
}
