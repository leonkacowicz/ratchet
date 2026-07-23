# Generalize the native layer to be language-parameterized

## Summary
Shared prerequisite for every non-Rust migration. The native layer is currently Rust-only by
construction: `parse_rust`, `visit_rust_functions`, `rust_function_*`, and hardcoded node kinds
(`function_item`, `if_expression`, `&&` …). Adding a second language requires parameterizing it
by `Language` so each grammar plugs in its own rule set.

Extract the parts that are genuinely shared (the algorithms) from the parts that are per-language
(the node kinds and a few structural quirks), so each language issue only supplies its rules and
its vendored grammar.

## Acceptance criteria
- [ ] `build.rs` compiles N vendored grammars; a `Language → LanguageFn` dispatch replaces the
      single `tree_sitter_rust()` extern
- [ ] `native::parse(lang, source)` replaces `parse_rust`; the walk and the six metric functions
      take a `Language`
- [ ] Per-language rules expressed as data/impl (function-space kinds, decision-point kinds,
      cognitive nesting/flat/reset kinds, non-arg kinds) rather than inline literals
- [ ] Rust behaviour is byte-identical afterwards — the existing parity + corpus tests still pass
      and `quality-report.json` is unchanged
- [ ] `use_native` stops hardcoding `lang == Language::Rust` and instead asks whether the language
      has a native rule set

## Notes
- **Shared vs per-language** (from reading rca): the *algorithms* are shared — space walk,
  file/function SLOC (node row spans), function count, nargs (`parameters` children minus
  non-arg kinds), cyclomatic (base 1 + nested spaces + decision points), cognitive
  (nesting-weighted increments + boolean sequence). What differs is the **node-kind sets** and a
  few quirks: rca's Cpp overrides naming/args via `declarator`; the JS family resets the boolean
  sequence on `expression_statement` and adds ternaries; Rust's `else if` is detected via
  `parent == else_clause`.
- Keep the rca side untouched — it stays the parity oracle for each new language.
- Blocks all per-language migrations under [[roll-out-native-metrics-to-remaining-languages-drop-rca-entirely]].
