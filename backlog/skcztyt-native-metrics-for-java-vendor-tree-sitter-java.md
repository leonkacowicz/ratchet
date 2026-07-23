# Native metrics for Java (vendor tree-sitter-java)

## Summary
Bring Java onto the native path. Class-and-method structure; a good second target after Python.

## Acceptance criteria
- [ ] Grammar vendored under `vendor/` + statically linked (`build.rs`); `Language` dispatch
      routes this language's extensions to the native parser
- [ ] Native function-space walk at rca parity (its function/method/closure node kinds and
      rca's naming), verified via the parity harness over the language's fixtures
- [ ] All six metrics at rca parity over the fixtures: `file_lines`, `file_functions`,
      `function_lines`, `function_args`, `function_cyclomatic`, `function_cognitive`
- [ ] Cutover: files of this language no longer parse through rca (`top = None`), report unchanged
## Notes
- Vendor **tree-sitter-java** at rca's version (`0.23.5`).
- rca space kinds: methods/constructors → Function, class/interface → Class/Interface. Metric
  rules from rca's `JavaCode` impls (switch/case, ternary, `catch`, `&&`/`||`, etc.).
- Candidate for npm/npa ([[public-method-attribute-counts-from-rca-npm-npa-metrics]]) since Java
  is class-based.
