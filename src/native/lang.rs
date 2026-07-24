//! Language-specific support: which grammar parses a language and which rule set
//! describes it.
//!
//! This is the **only** module that knows about [`Language`]. Everything below it
//! ([`analysis`](super::analysis), [`cognitive`](super::cognitive),
//! [`cyclomatic`](super::cyclomatic)) is pure computation over
//! `(rules, tree, source)` and never branches on a language.
//!
//! Every per-language difference so far has been expressible as data in
//! [`Rules`] — including the two that looked like they would need code, C/C++'s
//! `declarator` naming and the JS family's `variable_declarator` fallback, which
//! are function pointers on the rule set. So a language is exactly a grammar plus
//! a rule set, and this module is a table. Should one ever need genuinely custom
//! glue, add a function-pointer field to `Rules` the way `name_of` did.

use tree_sitter_language::LanguageFn;

use super::rules::{Rules, CPP, JAVA, JS_FAMILY, PYTHON, RUST};
use crate::language::Language;

extern "C" {
    /// Entry point of the vendored mozjs grammar (see `build.rs`). Mozilla's JS
    /// fork is not published, so it is the one grammar ratchet vendors.
    fn tree_sitter_mozjs() -> *const ();
}

/// One language's native implementation: the grammar that parses it and the node
/// kinds that drive its metrics.
pub struct NativeLanguage {
    pub grammar: LanguageFn,
    pub rules: &'static Rules,
}

static RUST_LANG: NativeLanguage = NativeLanguage { grammar: tree_sitter_rust::LANGUAGE, rules: &RUST };
static CPP_LANG: NativeLanguage = NativeLanguage { grammar: tree_sitter_cpp::LANGUAGE, rules: &CPP };
static PYTHON_LANG: NativeLanguage = NativeLanguage { grammar: tree_sitter_python::LANGUAGE, rules: &PYTHON };
static JAVA_LANG: NativeLanguage = NativeLanguage { grammar: tree_sitter_java::LANGUAGE, rules: &JAVA };
static TYPESCRIPT_LANG: NativeLanguage = NativeLanguage { grammar: tree_sitter_typescript::LANGUAGE_TYPESCRIPT, rules: &JS_FAMILY };
static TSX_LANG: NativeLanguage = NativeLanguage { grammar: tree_sitter_typescript::LANGUAGE_TSX, rules: &JS_FAMILY };
/// JavaScript/JSX uses the vendored mozjs fork, which handles `.js`, `.mjs`,
/// `.cjs` and `.jsx`.
static JAVASCRIPT_LANG: NativeLanguage = NativeLanguage { grammar: unsafe { LanguageFn::from_raw(tree_sitter_mozjs) }, rules: &JS_FAMILY };

/// Resolve a detected [`Language`] to its native implementation.
///
/// **This is the single dispatch point on `Language` in the native path.**
///
/// Every language ratchet supports now has one, so the match is exhaustive and
/// this never returns `None` — the `Option` is kept as the seam for a future
/// language that parses but isn't yet measured, which callers (`structural.rs`)
/// handle by skipping the file.
pub fn for_language(lang: Language) -> Option<&'static NativeLanguage> {
    Some(match lang {
        Language::Rust => &RUST_LANG,
        Language::Cpp => &CPP_LANG,
        Language::Python => &PYTHON_LANG,
        Language::Java => &JAVA_LANG,
        Language::JavaScript => &JAVASCRIPT_LANG,
        Language::TypeScript => &TYPESCRIPT_LANG,
        Language::Tsx => &TSX_LANG,
    })
}

/// Every language ratchet measures — all of them native.
#[cfg(test)]
pub(crate) const ALL_LANGUAGES: [Language; 7] =
    [Language::Rust, Language::Cpp, Language::Python, Language::Java, Language::JavaScript, Language::TypeScript, Language::Tsx];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_every_supported_language_resolves_to_a_native_implementation() {
        for lang in ALL_LANGUAGES {
            assert!(for_language(lang).is_some(), "{lang:?} has no native implementation");
        }
    }

    #[test]
    fn test_rust_resolves_to_its_own_rule_set() {
        let rust = for_language(Language::Rust).expect("Rust is native");
        assert!(rust.rules.is_function("function_item"));
    }
}
