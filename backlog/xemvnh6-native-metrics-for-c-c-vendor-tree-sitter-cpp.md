# Native metrics for C/C++ (vendor tree-sitter-cpp)

## Summary
Bring both C and C++ onto the native path — one grammar (`tree-sitter-cpp`) covers both, as in
rca. The hardest grammar (preprocessor, raw strings, function-declarator nesting).

## Acceptance criteria
- [ ] Grammar vendored under `vendor/` + statically linked (`build.rs`); `Language` dispatch
      routes this language's extensions to the native parser
- [ ] Native function-space walk at rca parity (its function/method/closure node kinds and
      rca's naming), verified via the parity harness over the language's fixtures
- [ ] All six metrics at rca parity over the fixtures: `file_lines`, `file_functions`,
      `function_lines`, `function_args`, `function_cyclomatic`, `function_cognitive`
- [ ] Cutover: files of this language no longer parse through rca (`top = None`), report unchanged
## Notes
- Vendor **tree-sitter-cpp** at rca's version (`0.23.4`). Both `.c`/`.h` and `.cpp`/`.hpp`
  route to it (see `Language::Cpp`).
- Metric rules from rca's `CppCode` impls. **Watch the naming/args edge cases**: rca's
  `get_func_space_name` and `nargs` for C/C++ dig through the `declarator` (operator casts,
  qualified ids) — reproduce that, not the generic `name`-field path.
- Function-space kinds: `function_definition` variants → Function; struct/class/namespace spaces.
