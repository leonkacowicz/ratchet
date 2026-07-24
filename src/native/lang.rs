//! Language-specific support: the grammar, the rule data, and the glue that
//! assembles the language-agnostic building blocks into metric values.
//!
//! This is the **only** module that knows about [`Language`]. Everything below it
//! ([`analysis`](super::analysis), [`complexity`](super::complexity)) is pure
//! computation over `(rules, tree, source)` and never branches on a language.
//!
//! The default glue suits any language whose metrics follow rca's generic shape;
//! a language that differs overrides just the method it needs (C/C++ will override
//! naming and argument counting, whose values hang off a `declarator`).

use tree_sitter::Tree;
use tree_sitter_language::LanguageFn;

use super::rules::Rules;
use super::rules::{JS_FAMILY, PYTHON, RUST};
use super::{analysis, cognitive, cyclomatic};
use crate::language::Language;

extern "C" {
    /// Entry point of the vendored mozjs grammar (see `build.rs`). Mozilla's JS
    /// fork is not published, so it is the one grammar ratchet vendors.
    fn tree_sitter_mozjs() -> *const ();
}

/// One language's native implementation.
///
/// Required: which grammar parses it and which [`Rules`] describe its node kinds.
/// The metric methods are glue with sensible defaults — override only where a
/// language genuinely differs.
pub trait NativeLanguage: Sync {
    /// The vendored tree-sitter grammar for this language.
    fn grammar(&self) -> LanguageFn;

    /// The node-kind rule set driving the shared algorithms.
    fn rules(&self) -> &'static Rules;

    fn file_lines(&self, tree: &Tree, _source: &[u8]) -> u64 {
        analysis::file_lines(tree)
    }

    fn file_functions(&self, tree: &Tree, source: &[u8]) -> u64 {
        analysis::file_functions(self.rules(), tree, source)
    }

    fn function_lines(&self, tree: &Tree, source: &[u8]) -> Vec<(String, u64)> {
        analysis::function_lines(self.rules(), tree, source)
    }

    fn function_nargs(&self, tree: &Tree, source: &[u8]) -> Vec<(String, u64)> {
        analysis::function_nargs(self.rules(), tree, source)
    }

    fn function_entities(&self, tree: &Tree, source: &[u8]) -> Vec<String> {
        analysis::function_entities(self.rules(), tree, source)
    }

    fn function_cyclomatic(&self, tree: &Tree, source: &[u8]) -> Vec<(String, u64)> {
        cyclomatic::function_cyclomatic(self.rules(), tree, source)
    }

    fn function_cognitive(&self, tree: &Tree, source: &[u8]) -> Vec<(String, u64)> {
        cognitive::function_cognitive(self.rules(), tree, source)
    }
}

/// Rust — the reference implementation, verified byte-for-byte against rca. Every
/// metric uses the default glue.
struct Rust;

impl NativeLanguage for Rust {
    fn grammar(&self) -> LanguageFn {
        tree_sitter_rust::LANGUAGE
    }

    fn rules(&self) -> &'static Rules {
        &RUST
    }
}

static RUST_LANG: Rust = Rust;

/// JavaScript (including JSX), parsed with the vendored mozjs grammar — the fork
/// rca routes `.js`/`.mjs`/`.cjs`/`.jsx` to, and the one grammar with no crate.
struct JavaScript;

impl NativeLanguage for JavaScript {
    fn grammar(&self) -> LanguageFn {
        unsafe { LanguageFn::from_raw(tree_sitter_mozjs) }
    }

    fn rules(&self) -> &'static Rules {
        &JS_FAMILY
    }
}

static JAVASCRIPT_LANG: JavaScript = JavaScript;

/// TypeScript — the JS-family rules with the TypeScript grammar.
struct TypeScript;

impl NativeLanguage for TypeScript {
    fn grammar(&self) -> LanguageFn {
        tree_sitter_typescript::LANGUAGE_TYPESCRIPT
    }

    fn rules(&self) -> &'static Rules {
        &JS_FAMILY
    }
}

static TYPESCRIPT_LANG: TypeScript = TypeScript;

/// TSX — the JS-family rules with the TSX grammar (TypeScript plus JSX).
struct Tsx;

impl NativeLanguage for Tsx {
    fn grammar(&self) -> LanguageFn {
        tree_sitter_typescript::LANGUAGE_TSX
    }

    fn rules(&self) -> &'static Rules {
        &JS_FAMILY
    }
}

static TSX_LANG: Tsx = Tsx;

/// Python.
struct Python;

impl NativeLanguage for Python {
    fn grammar(&self) -> LanguageFn {
        tree_sitter_python::LANGUAGE
    }

    fn rules(&self) -> &'static Rules {
        &PYTHON
    }
}

static PYTHON_LANG: Python = Python;

/// Resolve a detected [`Language`] to its native implementation, or `None` when it
/// has none yet (those languages still route through rca).
///
/// **This is the single dispatch point on `Language` in the native path.**
pub fn for_language(lang: Language) -> Option<&'static dyn NativeLanguage> {
    match lang {
        Language::Rust => Some(&RUST_LANG),
        Language::JavaScript => Some(&JAVASCRIPT_LANG),
        Language::TypeScript => Some(&TYPESCRIPT_LANG),
        Language::Tsx => Some(&TSX_LANG),
        Language::Python => Some(&PYTHON_LANG),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dispatch_resolves_native_languages_and_rejects_the_rest() {
        assert!(for_language(Language::Rust).is_some());
        assert!(for_language(Language::Python).is_some());
        assert!(for_language(Language::Java).is_none());
    }

    #[test]
    fn test_rust_impl_exposes_its_rule_set() {
        let rust = for_language(Language::Rust).expect("Rust is native");
        assert!(rust.rules().is_function("function_item"));
    }
}
