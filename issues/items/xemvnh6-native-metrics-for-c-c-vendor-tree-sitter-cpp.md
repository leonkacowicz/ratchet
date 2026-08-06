# Native metrics for C/C++ (vendor tree-sitter-cpp)

## Summary
Bring both C and C++ onto the native path — one grammar (`tree-sitter-cpp`) covers both, as in
rca. The hardest grammar (preprocessor, raw strings, function-declarator nesting).

## Acceptance criteria
- [x] Grammar sourced as a pinned crates.io dependency (`tree-sitter-cpp = "=0.23.4"`); one
      grammar serves both `.c`/`.h` and `.cpp`/`.hpp`, as in rca
- [x] Native function-space walk at rca parity over the corpus
- [x] All six metrics at rca parity over the corpus
- [x] Cutover: C/C++ files no longer parse through rca (`top` is `None`)
## Notes
- **Passed parity on the first run.** Both overrides flagged at the start of the rollout were
  needed and both fitted existing seams: naming reads the `declarator` (descend to the first
  `function_declarator`, take its leading identifier, or an `operator_cast`) through the `name_of`
  function pointer JavaScript introduced, and arguments hang off that same declarator via a new
  `params_via_declarator` flag.
- **Only language where rca returns no name at all** (rather than `"<anonymous>"`), which is what
  makes its `{closure_N}` numbering fire. `name_of` now returns `Option` and the walk synthesizes
  the numbering, matching rca's `function_entity_name`.
- As in Python and Java, a `lambda_expression` is **not** a function space. `goto` costs a flat
  cognitive point, unique to this language.
- **Milestone:** with C/C++ done the dispatch match is exhaustive — the compiler flagged the
  `_ => None` arm as unreachable. No language routes through rca for any metric any more, which
  unblocks [[drop-the-rust-code-analysis-dependency]].
- **The `NativeLanguage` trait collapsed to data.** No language ever overrode its glue: every
  difference, including the two that looked like code, proved expressible as `Rules` data. It is
  now a `grammar + rules` struct and a table — ratchet's own gate forced the simplification by
  flagging `lang.rs` over the function limit.
- Corpus: `example_constructs.cpp` — free functions, methods, constructor/destructor, operator
  overload, templates, lambdas, switch, try/catch, goto, ternary, boolean sequences, nesting.
- Vendor **tree-sitter-cpp** at rca's version (`0.23.4`). Both `.c`/`.h` and `.cpp`/`.hpp`
  route to it (see `Language::Cpp`).
- Metric rules from rca's `CppCode` impls. **Watch the naming/args edge cases**: rca's
  `get_func_space_name` and `nargs` for C/C++ dig through the `declarator` (operator casts,
  qualified ids) — reproduce that, not the generic `name`-field path.
- Function-space kinds: `function_definition` variants → Function; struct/class/namespace spaces.
