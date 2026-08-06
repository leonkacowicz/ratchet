# Migrate cognitive complexity to raw tree-sitter (Rust)

## Summary
Compute cognitive complexity per function for Rust natively from the raw tree-sitter tree,
matching rca's `cognitive`, behind the parity harness. Deliberately **last** — it is the
hardest metric (nesting-dependent increments, boolean-sequence rules, recursion handling)
and the most language-specific, so it benefits from everything the earlier migrations shake
out.

## Acceptance criteria
- [x] Native cognitive for Rust computed from the raw tree, including nesting increments and
      boolean-operator-sequence rules — `native::complexity` (`rust_function_cognitive` + `Cog`)
- [x] Parity with rca's cognitive on all Rust fixtures + ratchet's own `src/`
      — `test_function_cognitive_parity_over_repo_corpus` + `..._on_tricky_constructs`
- [x] Rust cognitive flipped to the native path via the selector in production
      — `FunctionCognitive` in `MIGRATED`; report unchanged, `check`/`compare` green

## Notes
- **Exact parity achieved** — no redefinition needed. rca's `cognitive_sum` (a subtree sum)
  reproduced: nesting-weighted increment `nesting + depth + lambda + 1` per control point
  (`if` non-else-if, `for`, `while`, `match`), flat `+1` per `else` token and labeled
  break/continue, `lambda`/`depth` weighting for closures/nested-fns.
- **Two subtleties found by probing rca** (both caught by the corpus/tricky tests):
  1. rca scores the **`else` token** (enum `Else`), not the `else_clause` node — so `let ... else`
     counts, which an `else_clause`-only match missed.
  2. A boolean sequence is compared against its **first** operator: `eval_based_on_prev` sets
     `boolean_op` only on the `None` case and never advances it, so `a && b || c` interacts with
     following statements differently than naive per-operator tracking.
- **Dogfooding pressure.** The metric code itself must pass ratchet's thresholds. Split
  `native.rs` into `native/{mod,complexity}.rs` (file limits) and factored the walk into small
  helpers (a `Cog` struct + `cog_apply`/`boolean_op`/`eval_boolean`) to stay under the
  cyclomatic/cognitive/args limits — a nice proof the tool keeps its own implementation honest.
- Retiring the rca visit loop dropped `structural.rs` 6→5 functions (improvement; baseline
  regenerated). rca still parses for non-Rust and as the Rust fallback — full removal is the
  cutover ([[remove-rca-from-the-rust-path-all-rust-metrics-native]]).
- Depends on the harness ([[dual-path-metric-collector-rca-parity-harness]]) and the walk.
