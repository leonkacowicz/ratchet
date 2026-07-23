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

use crate::language::Language;

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
    /// Cognitive: kinds whose direct `&&`/`||` children form a boolean sequence.
    pub cog_binary_kinds: &'static [&'static str],
    /// Cognitive: unary kinds that mark the boolean sequence (rca's `not_operator`).
    pub cog_unary_kinds: &'static [&'static str],
    /// Parent kind marking an `else if`, whose `if` must not add nesting (it is
    /// already scored by the `else`). `None` when the language has no such form.
    pub else_if_parent: Option<&'static str>,
}

impl Rules {
    /// Whether `kind` forms a function space.
    pub fn is_function(&self, kind: &str) -> bool {
        self.function_kinds.contains(&kind)
    }
}

/// Rust — the reference implementation, verified byte-for-byte against rca.
static RUST: Rules = Rules {
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
    else_if_parent: Some("else_clause"),
};

/// The rule set for `lang`, or `None` when it has no native implementation yet
/// (those languages still route through rca).
pub fn for_language(lang: Language) -> Option<&'static Rules> {
    match lang {
        Language::Rust => Some(&RUST),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rust_has_rules_and_other_languages_do_not_yet() {
        assert!(for_language(Language::Rust).is_some());
        assert!(for_language(Language::Python).is_none());
    }

    #[test]
    fn test_is_function_matches_rust_function_spaces() {
        let rust = for_language(Language::Rust).expect("Rust rules");
        assert!(rust.is_function("function_item"));
        assert!(rust.is_function("closure_expression"));
        assert!(!rust.is_function("function_signature_item"));
    }
}
