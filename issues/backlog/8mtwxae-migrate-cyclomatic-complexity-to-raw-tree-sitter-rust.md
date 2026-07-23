# Migrate cyclomatic complexity to raw tree-sitter (Rust)

## Summary
Compute cyclomatic complexity per function for Rust natively from the raw tree-sitter tree,
matching rca's `cyclomatic`, behind the parity harness. First of the two real complexity
metrics — the node→+1 rules are more involved than the counting metrics but far simpler than
cognitive.

## Acceptance criteria
- [ ] Native cyclomatic for Rust computed from the raw tree
- [ ] Parity with rca's cyclomatic on all Rust fixtures + ratchet's own `src/`
- [ ] Rust cyclomatic flipped to the native path via the selector; happy-path test

## Notes
- Enumerate the Rust node kinds rca counts as decision points (`if`/`else if`, `match` arms,
  loops, `&&`/`||`, `?`, etc.) from rca's `metrics/cyclomatic.rs` and reproduce exactly.
- Depends on the harness ([[dual-path-metric-collector-rca-parity-harness]]).
