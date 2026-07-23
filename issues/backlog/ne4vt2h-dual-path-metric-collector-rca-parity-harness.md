# Dual-path metric collector + rca parity harness

## Summary
The mechanism that lets metrics migrate **one at a time** safely. Introduce a seam where a
given metric can be computed via either the native raw-tree-sitter path or the existing rca
path, plus a test harness that runs both over the fixtures and asserts they agree.

Until a metric is migrated it keeps flowing through rca; once its native implementation
matches rca on the fixtures, the switch flips for that metric. The harness is what turns
"reimplement rca" from a risky big-bang into an incremental, verifiable sequence.

## Acceptance criteria
- [ ] A per-metric selector (native vs rca) so metrics can be flipped individually
- [ ] Parity test: for each migrated metric, native and rca outputs match on every
      `tests/fixtures/` Rust sample (and ratchet's own `src/`)
- [ ] Clear, logged reporting of any divergence (entity + both values) to drive debugging
- [ ] Harness is Rust-first but structured so other languages plug in later

## Notes
- Parity is measured against the *pinned* rca revision — that is the definition of "correct"
  for the migration, since the committed `quality-report.json` baseline was produced by it.
- Deliberate divergences (where we decide rca is wrong or we want different semantics) are
  allowed but must be recorded on the specific metric's issue, not silently accepted.
- This is the seam the [[replace-rust-code-analysis-with-raw-tree-sitter-own-metrics]] epic
  hangs every metric migration off of.
