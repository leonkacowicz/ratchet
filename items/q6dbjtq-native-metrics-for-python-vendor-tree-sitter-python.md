# Native metrics for Python (vendor tree-sitter-python)

## Summary
Bring Python fully onto the native path. Recommended **first** non-Rust language: a clean
grammar, no `self`/closure quirks, indentation handled by the vendored external scanner.

## Acceptance criteria
- [x] Grammar sourced as a pinned crates.io dependency (`tree-sitter-python = "=0.23.6"`)
- [x] Native function-space walk at rca parity over the corpus
- [x] All six metrics at rca parity over the corpus
- [x] Cutover: `.py` files no longer parse through rca (`top` is `None`)
## Notes
- **Stretched the rule model most so far** — four new seams were needed, all surfaced by the
  parity corpus rather than by reading rca:
  - `and`/`or` as the logical operators (already rule data from the earlier audit — worked first try);
  - `lambda` carries nesting weight but is **not** a function space, so it never appears in the
    walk, yet rca's `nom` counts it and its arguments accumulate into the enclosing function's
    `closure_nargs`. Hence `file_functions` counts a wider set than the walk emits
    (`counts_toward_nom`) and `nargs` folds in non-space closures;
  - a nested `def` does **not** restart nesting (`fn_resets_nesting: false`), unlike Rust/JS;
  - `except_clause` takes rca's *stateful* increment (`cog_nesting_state_kinds`), reusing
    whatever the last nesting increase left in `stats.nesting`;
  - a `boolean_operator` gains a point per enclosing `lambda` (`cog_extra`).
- **Third rca bug found:** Python's cyclomatic intends to count only a loop-`else`, but
  `has_ancestors` only climbs when the parent matches its *first* predicate — and an `else`'s
  parent is always an `else_clause`, so it never climbs, then matches that same parent against the
  second predicate, which always holds. Every `else` closing a suite therefore counts (a
  conditional expression's does not). Reproduced for parity; a candidate to drop later, like the
  TS/TSX pair.
- **Ratchet policed its own code again:** `rules.rs` crossed the file-lines limit (now a module per
  language) and `complexity.rs` crossed the function-count limit (now `cyclomatic.rs` +
  `cognitive.rs`).
- Corpus: `example_constructs.py` — functions, methods, lambdas, elif/loop-else,
  try/except/finally, with, assert, comprehensions, word booleans, nested defs.
- Vendor **tree-sitter-python** at the version rca uses (`0.23.6`) so trees match rca's.
- rca space kinds (getter): `function_definition` → Function, `class_definition` → Class,
  `module` → Unit. Metric rules: rca's `PythonCode` `Cognitive`/`Cyclomatic` impls — note
  Python's decision points differ from Rust (e.g. `elif`, `except`, comprehensions).
- Depends on the shared infra (harness, selector, dispatch) already in place.
