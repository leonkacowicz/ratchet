# Migrate function-args (nargs) to raw tree-sitter (Rust)

## Summary
Compute per-function argument count for Rust natively from the raw tree-sitter tree,
matching rca's `nargs`, behind the parity harness.

## Acceptance criteria
- [ ] Native nargs for Rust computed from the raw tree (count `parameters` children)
- [ ] Parity with rca's nargs on all Rust fixtures + ratchet's own `src/`
- [ ] Rust nargs flipped to the native path via the selector; happy-path test

## Notes
- Watch Rust-specific cases: `self`/`&self`/`&mut self` receivers, and whether rca counts
  them; closures vs `fn` items. Reproduce rca's choice, don't guess.
- Depends on the harness ([[dual-path-metric-collector-rca-parity-harness]]).
