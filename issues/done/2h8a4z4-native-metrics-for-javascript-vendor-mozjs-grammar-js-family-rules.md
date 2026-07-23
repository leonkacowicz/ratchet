# Native metrics for JavaScript (vendor mozjs grammar; JS-family rules)

## Summary
Bring JavaScript onto the native path **and establish the shared JS-family metric rules** used
by TypeScript and TSX. rca computes JS/TS/TSX cognitive via one `js_cognitive!` macro, so the
rule logic implemented here should be factored to be reused by the TS/TSX issues.

## Acceptance criteria
- [x] Grammar vendored + statically linked — `vendor/tree-sitter-mozjs/` (MIT), taken from the
      rca repo at the pinned rev so trees match rca's exactly; one `compile_grammar` call
- [x] Native function-space walk at rca parity — verified over the JS corpus (`.js` + `.jsx`)
- [x] All six metrics at rca parity over the corpus
- [x] Cutover: `.js`/`.mjs`/`.cjs`/`.jsx` no longer parse through rca (`Language::JavaScript`
      resolves to a native implementation, so `top` is `None`)
## Notes
- **Purely additive**, as the generalized layer intended: one vendored grammar, one
  `compile_grammar` call, one extern, one `Rules` entry, one dispatch arm. No shared algorithm
  changed; `analysis`/`complexity`/`rules`/`parity` still contain no `Language` references.
- **JS-family rules captured as data**: function spaces = function/generator declarations and
  expressions, method definitions, arrow functions; only a *function declaration* resets
  cognitive nesting **and** the lambda weight (`fn_resets_lambda`); arrow functions add lambda
  weight; every `expression_statement` resets the boolean sequence; ternary/switch/catch are
  nesting kinds; cyclomatic counts `case` and `catch` but **not** `do`.
- **New seam discovered:** rca names an anonymous JS function after its enclosing `pair` key or
  `variable_declarator` (`const arrow = (x) => …` is `"arrow"`, not `"<anonymous>"`). Naming is
  now behaviour-as-data — a `name_of` fn pointer on `Rules` that the shared walk calls — so the
  agnostic layer still never branches. C/C++ will use the same seam for its `declarator` naming.
- **Corpus:** `tests/fixtures/example_constructs.js` was written for this (every function-space
  kind, control structure, boolean sequence and nesting case) plus `example.jsx`. The original
  one-function `example.js` would not have caught the naming divergence — richer fixtures are
  worth writing up front for the remaining languages.
- Vendor the **mozjs** grammar rca routes `.js`/`.mjs`/`.cjs`/`.jsx` to (`tree-sitter-mozjs`
  `0.20.3`).
- Shared rules: arrow functions + function expressions as closures; `ternary`, `switch`,
  `for..in`, `catch` as decision points; note rca resets the boolean sequence on
  `ExpressionStatement` for the JS family (unlike Rust) — reproduce that.
- **Factor the JS-family metric rules** so TS/TSX only add their grammars.
- Blocks [[native-metrics-for-typescript-reuse-js-family-rules]] and
  [[native-metrics-for-tsx-reuse-js-family-rules]].
