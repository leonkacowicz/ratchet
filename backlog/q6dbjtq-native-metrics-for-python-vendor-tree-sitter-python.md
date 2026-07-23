# Native metrics for Python (vendor tree-sitter-python)

## Summary
Bring Python fully onto the native path. Recommended **first** non-Rust language: a clean
grammar, no `self`/closure quirks, indentation handled by the vendored external scanner.

## Acceptance criteria
- [ ] Grammar vendored under `vendor/` + statically linked (`build.rs`); `Language` dispatch
      routes this language's extensions to the native parser
- [ ] Native function-space walk at rca parity (its function/method/closure node kinds and
      rca's naming), verified via the parity harness over the language's fixtures
- [ ] All six metrics at rca parity over the fixtures: `file_lines`, `file_functions`,
      `function_lines`, `function_args`, `function_cyclomatic`, `function_cognitive`
- [ ] Cutover: files of this language no longer parse through rca (`top = None`), report unchanged
## Notes
- Vendor **tree-sitter-python** at the version rca uses (`0.23.6`) so trees match rca's.
- rca space kinds (getter): `function_definition` → Function, `class_definition` → Class,
  `module` → Unit. Metric rules: rca's `PythonCode` `Cognitive`/`Cyclomatic` impls — note
  Python's decision points differ from Rust (e.g. `elif`, `except`, comprehensions).
- Depends on the shared infra (harness, selector, dispatch) already in place.
