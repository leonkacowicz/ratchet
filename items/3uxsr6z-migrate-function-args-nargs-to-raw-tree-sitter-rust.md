# Migrate function-args (nargs) to raw tree-sitter (Rust)

## Summary
Compute per-function argument count for Rust natively from the raw tree-sitter tree,
matching rca's `nargs`, behind the parity harness.

## Acceptance criteria
- [x] Native nargs for Rust computed from the raw tree (count `parameters` children)
      — `native::nargs_of` / `rust_function_nargs`
- [x] Parity with rca's nargs on all Rust fixtures + ratchet's own `src/`
      — `test_function_args_parity_over_repo_corpus` (+ a self/closure snippet test)
- [x] Rust nargs flipped to the native path via the selector in production
      — `FunctionArgs` in `MIGRATED`; `structural.rs` records it via
      `parity::function_args_values`. Report unchanged (native == rca), `check`/`compare` green.

## Notes
- **rca nargs rule reproduced exactly.** Per rca's `compute_args` + `is_non_arg`: count every
  child of the `parameters` field except the delimiters (`(` `)` `,` `|`) and `attribute_item`.
  Consequence: **`self` counts** (`self_parameter` is not excluded), so `fn m(&self, a, b)` is 3;
  a closure `|x|` is 1. `args_for` takes `max(fn_args, closure_args)`, but a node is either a
  function or a closure, so counting its own `parameters` yields the same. Verified: snippet + the
  whole repo corpus matched first-try.
- **First function-level migration** — established the production pattern: the rca function loop
  in `structural.rs` records lines/cognitive/cyclomatic; a separate pass records `function_args`
  via `function_args_values` (native or rca), entity names aligned by the proven walk order.
- **Refactor:** the test-only parity oracle (`compute`/`check_parity`/`diff` + map wrappers) moved
  into `parity.rs`'s `#[cfg(test)]` module so the file stays under the `file_functions` ratchet as
  more metrics land; production keeps only the dispatch surface.
- Depends on the harness ([[dual-path-metric-collector-rca-parity-harness]]) and the walk.
