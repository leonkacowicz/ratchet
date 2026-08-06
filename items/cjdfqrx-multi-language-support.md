# Multi-language support

## Summary
Extend ratchet beyond Rust. `rust-code-analysis` is already multi-language — it ships
tree-sitter grammars and metric impls for Rust, C/C++, Python, Java, JavaScript,
TypeScript/TSX, and (partially) Kotlin. Go is the exception: rca has no Go grammar, so
it needs an external-tool collector. This epic requires extension-based dispatch, the
configurable source globs from the CONFIG epic, and careful handling of the fact that
metric coverage is uneven across languages.

## Acceptance criteria
- [ ] The rca-supported languages can be analyzed
- [ ] Per-language test/generated exclusion works
- [ ] No unimplemented metric/language pair is silently recorded as zero
