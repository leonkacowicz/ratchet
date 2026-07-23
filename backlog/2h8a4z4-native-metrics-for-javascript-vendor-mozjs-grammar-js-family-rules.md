# Native metrics for JavaScript (vendor mozjs grammar; JS-family rules)

## Summary
Bring JavaScript onto the native path **and establish the shared JS-family metric rules** used
by TypeScript and TSX. rca computes JS/TS/TSX cognitive via one `js_cognitive!` macro, so the
rule logic implemented here should be factored to be reused by the TS/TSX issues.

## Acceptance criteria
- [ ] Grammar vendored under `vendor/` + statically linked (`build.rs`); `Language` dispatch
      routes this language's extensions to the native parser
- [ ] Native function-space walk at rca parity (its function/method/closure node kinds and
      rca's naming), verified via the parity harness over the language's fixtures
- [ ] All six metrics at rca parity over the fixtures: `file_lines`, `file_functions`,
      `function_lines`, `function_args`, `function_cyclomatic`, `function_cognitive`
- [ ] Cutover: files of this language no longer parse through rca (`top = None`), report unchanged
## Notes
- Vendor the **mozjs** grammar rca routes `.js`/`.mjs`/`.cjs`/`.jsx` to (`tree-sitter-mozjs`
  `0.20.3`).
- Shared rules: arrow functions + function expressions as closures; `ternary`, `switch`,
  `for..in`, `catch` as decision points; note rca resets the boolean sequence on
  `ExpressionStatement` for the JS family (unlike Rust) — reproduce that.
- **Factor the JS-family metric rules** so TS/TSX only add their grammars.
- Blocks [[native-metrics-for-typescript-reuse-js-family-rules]] and
  [[native-metrics-for-tsx-reuse-js-family-rules]].
