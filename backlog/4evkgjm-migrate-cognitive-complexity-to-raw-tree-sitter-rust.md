# Migrate cognitive complexity to raw tree-sitter (Rust)

## Summary
Compute cognitive complexity per function for Rust natively from the raw tree-sitter tree,
matching rca's `cognitive`, behind the parity harness. Deliberately **last** — it is the
hardest metric (nesting-dependent increments, boolean-sequence rules, recursion handling)
and the most language-specific, so it benefits from everything the earlier migrations shake
out.

## Acceptance criteria
- [ ] Native cognitive for Rust computed from the raw tree, including nesting increments and
      boolean-operator-sequence rules
- [ ] Parity with rca's cognitive on all Rust fixtures + ratchet's own `src/`
- [ ] Rust cognitive flipped to the native path via the selector; happy-path test

## Notes
- rca's `metrics/cognitive.rs` is the reference (~57 KB, largest metric module). Expect the
  bulk of this epic's parity-debugging effort here.
- If exact parity proves impractical, a *deliberate, documented* redefinition is an option —
  but that reprices the baseline and must be an explicit decision, not drift.
- Depends on the harness ([[dual-path-metric-collector-rca-parity-harness]]).
