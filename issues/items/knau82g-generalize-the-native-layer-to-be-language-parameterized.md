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
- [x] `build.rs` compiles N vendored grammars (reusable `compile_grammar` helper); a
      `Language → LanguageFn` dispatch (`native::grammar`) replaces the bare extern
- [x] `native::parse(lang, source)` replaces `parse_rust`; the walk and all six metric functions
      take a `Language` (`native::{file_lines, file_functions, function_lines, function_nargs,
      function_cyclomatic, function_cognitive}`)
- [x] Per-language rules expressed as data — `native::rules::Rules` (function/fn/lambda kinds,
      non-arg kinds, decision kinds, cognitive nesting/flat/labeled/reset/binary/unary kinds,
      `else_if_parent`) with a `for_language` lookup
- [x] Rust behaviour byte-identical — all parity + corpus tests pass, `quality-report.json`
      unchanged, `check`/`compare` green
- [x] `use_native` now asks `native::supports(lang)` (vendored grammar + rule set) instead of
      hardcoding Rust

## Notes
- **Delivered layout:** `native/mod.rs` (grammar dispatch, parse, naming, walk),
  `native/rules.rs` (per-language kind sets), `native/metrics.rs` (SLOC/counts/nargs),
  `native/complexity.rs` (cyclomatic/cognitive). Split four ways so each file stays under
  ratchet's own `file_lines`/`file_functions` thresholds as languages are added.
- Adding a language is now: vendor its grammar + one `compile_grammar` call, one `grammar()`
  match arm, and one `Rules` entry — plus verifying parity via the existing harness.
- **Shared vs per-language** (from reading rca): the *algorithms* are shared — space walk,
  file/function SLOC (node row spans), function count, nargs (`parameters` children minus
  non-arg kinds), cyclomatic (base 1 + nested spaces + decision points), cognitive
  (nesting-weighted increments + boolean sequence). What differs is the **node-kind sets** and a
  few quirks: rca's Cpp overrides naming/args via `declarator`; the JS family resets the boolean
  sequence on `expression_statement` and adds ternaries; Rust's `else if` is detected via
  `parent == else_clause`.
- Keep the rca side untouched — it stays the parity oracle for each new language.
- Blocks all per-language migrations under [[roll-out-native-metrics-to-remaining-languages-drop-rca-entirely]].
