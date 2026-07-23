# Native metrics for TSX (reuse JS-family rules)

## Summary
Bring TSX onto the native path, reusing the JS-family rules. TSX shares the TypeScript grammar
family with a separate `tsx` parser (JSX enabled).

## Acceptance criteria
- [ ] Grammar vendored under `vendor/` + statically linked (`build.rs`); `Language` dispatch
      routes this language's extensions to the native parser
- [ ] Native function-space walk at rca parity (its function/method/closure node kinds and
      rca's naming), verified via the parity harness over the language's fixtures
- [ ] All six metrics at rca parity over the fixtures: `file_lines`, `file_functions`,
      `function_lines`, `function_args`, `function_cyclomatic`, `function_cognitive`
- [ ] Cutover: files of this language no longer parse through rca (`top = None`), report unchanged
## Notes
- Vendor the **tsx** parser from `tree-sitter-typescript` (rca's `0.23.2`).
- Same JS-family walk + rules as TypeScript; JSX elements add node kinds — verify they do not
  affect metric counts.
- Depends on the JavaScript issue.
