# Per-language test/generated-code exclusion hook

## Summary
`strip_test_modules` only understands Rust's `#[cfg(test)] mod`. Generalize code
exclusion into a per-language mechanism (exclude globs plus optional in-file markers)
so test/generated code can be dropped for any target language. This is the one
inherently per-language piece of the tool.

## Acceptance criteria
- [ ] Current Rust behaviour preserved
- [ ] Exclusions configurable per language
- [ ] Documented
