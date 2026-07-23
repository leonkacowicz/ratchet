# Migrate function-count (nom) to raw tree-sitter (Rust)

## Summary
Compute number-of-methods/functions per file for Rust natively from the raw tree-sitter
tree, matching rca's `nom` (the metric ratchet's `file_functions` category is built on),
behind the parity harness.

## Acceptance criteria
- [ ] Native nom for Rust computed from the raw tree
- [ ] Parity with rca's nom on all Rust fixtures + ratchet's own `src/`
- [ ] Rust nom flipped to the native path via the selector; happy-path test

## Notes
- Decide what counts as a "function" the way rca does: `fn` items, associated methods,
  closures, trait method signatures vs bodies. The space walk in
  `src/collectors/structural.rs` shows what ratchet currently treats as a function space.
- Depends on the harness ([[dual-path-metric-collector-rca-parity-harness]]).
