# Migrate function-count (nom) to raw tree-sitter (Rust)

## Summary
Compute number-of-methods/functions per file for Rust natively from the raw tree-sitter
tree, matching rca's `nom` (the metric ratchet's `file_functions` category is built on),
behind the parity harness.

## Acceptance criteria
- [x] Native nom for Rust computed from the raw tree
      — `parity::native_file_functions` = length of the `visit_rust_functions` walk
- [x] Parity with rca's nom on all Rust fixtures + ratchet's own `src/`
      — `test_file_functions_parity_over_repo_corpus` (+ snippet test)
- [x] Native impl registered behind the selector; happy-path test
      — `Metric::FileFunctions` in the dual-path `compute`. Production **flip deferred to the
      Rust cutover** ([[remove-rca-from-the-rust-path-all-rust-metrics-native]]), per the
      harness design: `MIGRATED` stays empty and `structural.rs` keeps using rca until all
      five metrics are native, then flips in one reviewable step. Safe because native output
      is identical to rca.

## Notes
- rca `nom.total()` counts exactly the Function-kind spaces (`function_item` +
  `closure_expression`) — so native `file_functions` is simply the length of the function
  walk from [[native-function-space-walk-identify-name-functions-to-match-rca]]. Verified: a
  4-function snippet (fn/fn/closure/method) matched rca, and the whole repo corpus matched
  first-try. Body-less trait signatures are excluded on both sides.
- Depends on the harness ([[dual-path-metric-collector-rca-parity-harness]]) and the walk.
