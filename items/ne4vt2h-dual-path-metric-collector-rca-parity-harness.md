# Dual-path metric collector + rca parity harness

## Summary
The mechanism that lets metrics migrate **one at a time** safely. Introduce a seam where a
given metric can be computed via either the native raw-tree-sitter path or the existing rca
path, plus a test harness that runs both over the fixtures and asserts they agree.

Until a metric is migrated it keeps flowing through rca; once its native implementation
matches rca on the fixtures, the switch flips for that metric. The harness is what turns
"reimplement rca" from a risky big-bang into an incremental, verifiable sequence.

## Acceptance criteria
- [x] A per-metric selector (native vs rca) so metrics can be flipped individually
      — `parity::Metric` / `Backend` / `MIGRATED` / `Metric::backend()`
- [x] Parity test: for each migrated metric, native and rca outputs match on every
      `tests/fixtures/` Rust sample (and ratchet's own `src/`)
      — `test_file_lines_parity_over_repo_corpus` (proven for `file_lines`)
- [x] Clear, logged reporting of any divergence (entity + both values) to drive debugging
      — `parity::diff` / `check_parity`
- [x] Harness is Rust-first but structured so other languages plug in later
      — `compute(metric, backend, source, path)` keyed by entity

## Notes
- Parity is measured against the *pinned* rca revision — that is the definition of "correct"
  for the migration, since the committed `quality-report.json` baseline was produced by it.
- Deliberate divergences (where we decide rca is wrong or we want different semantics) are
  allowed but must be recorded on the specific metric's issue, not silently accepted.
- This is the seam the [[replace-rust-code-analysis-with-raw-tree-sitter-own-metrics]] epic
  hangs every metric migration off of.
- **Delivered in `src/parity.rs`.** The harness is proven end-to-end on **`file_lines`**
  (file-level SLOC): native = `root.end_row - start_row`, which equals rca's `loc.sloc()` on
  the unit by construction (same grammar + runtime), verified over the whole repo corpus.
- **Production flip deferred.** `MIGRATED` is empty, so the real report still uses rca for
  everything; `structural.rs` does not consult the selector yet. `file_lines` is parity-proven
  and is the first metric ready to flip — the flip (add to `MIGRATED` + wire `structural.rs`)
  belongs to the SLOC step ([[migrate-sloc-to-raw-tree-sitter-rust]]) / the Rust cutover.
  Because native and rca `file_lines` are identical, that flip will leave the report unchanged.
