# Native metrics for Java (vendor tree-sitter-java)

## Summary
Bring Java onto the native path. Class-and-method structure; a good second target after Python.

## Acceptance criteria
- [x] Grammar sourced as a pinned crates.io dependency (`tree-sitter-java = "=0.23.5"`)
- [x] Native function-space walk at rca parity over the corpus
- [x] All six metrics at rca parity over the corpus
- [x] Cutover: `.java` files no longer parse through rca (`top` is `None`)
## Notes
- **Passed parity on the first run** once one structural point was right: a `lambda_expression`
  is *not* a function space. rca creates spaces from `is_func || is_func_space` and a Java lambda
  is in neither — despite `get_space_kind` labelling it `Function`. Like a Python `lambda` it is
  counted by `nom`, contributes arguments and nesting weight to the enclosing method, and never
  appears in the walk. The seams Python introduced covered Java with no new machinery — the first
  language to add nothing to the model.
- **Three rca quirks carried as data:** `is_non_arg` always `false` (so parens and commas count —
  a two-argument method scores 5); `is_else_if` always `false` (so `else if` takes a full nesting
  increment, as in TSX); and no method arm in the cognitive rules (so a method neither restarts
  nesting nor deepens depth).
- **Asymmetry inside rca worth knowing:** cyclomatic counts the `for` *token*, so it sees an
  enhanced `for (X x : xs)`; cognitive matches only `for_statement` and misses it.
- Candidate for npm/npa ([[public-method-attribute-counts-from-rca-npm-npa-metrics]]) since Java
  is class-based.
- Corpus: `example_constructs.java` — methods, constructors, lambdas, enhanced for, switch,
  try/catch/finally, ternary, boolean sequences, nesting.
- Vendor **tree-sitter-java** at rca's version (`0.23.5`).
- rca space kinds: methods/constructors → Function, class/interface → Class/Interface. Metric
  rules from rca's `JavaCode` impls (switch/case, ternary, `catch`, `&&`/`||`, etc.).
- Candidate for npm/npa ([[public-method-attribute-counts-from-rca-npm-npa-metrics]]) since Java
  is class-based.
