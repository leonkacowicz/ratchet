# Native metrics for TypeScript (reuse JS-family rules)

## Summary
Bring TypeScript onto the native path, reusing the JS-family metric rules established by the
JavaScript issue. Mostly grammar-vendoring + verification.

## Acceptance criteria
- [ ] Grammar vendored under `vendor/` + statically linked (`build.rs`); `Language` dispatch
      routes this language's extensions to the native parser
- [ ] Native function-space walk at rca parity (its function/method/closure node kinds and
      rca's naming), verified via the parity harness over the language's fixtures
- [ ] All six metrics at rca parity over the fixtures: `file_lines`, `file_functions`,
      `function_lines`, `function_args`, `function_cyclomatic`, `function_cognitive`
- [ ] Cutover: files of this language no longer parse through rca (`top = None`), report unchanged
## Notes
- Vendor **tree-sitter-typescript** (typescript parser) at rca's version (`0.23.2`).
- Reuse the JS-family walk + metric rules from
  [[native-metrics-for-javascript-vendor-mozjs-grammar-js-family-rules]]; TS adds type nodes but
  the decision-point/space kinds are the js-family set. Verify parity — TS's grammar has extra
  node kinds that must not perturb the counts.
- Depends on the JavaScript issue.
