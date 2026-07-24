# Relax grammar pins & upgrade tree-sitter now that rca is gone

## Summary
The grammar deps were pinned to the exact versions `rust-code-analysis` used so parse trees
(and therefore metrics) matched rca. rca has since been dropped, so ratchet's metrics are
now self-defined — the exact grammar version is no longer load-bearing. Relax the pins to
the latest compatible releases; if metrics shift, the golden test catches it and it gets
re-blessed deliberately.

## Acceptance criteria
- [x] Exact `=0.23.x` pins relaxed; grammars + tree-sitter core on the latest compatible
- [x] Builds, clippy-clean, all tests pass (golden re-blessed if metrics moved)
- [x] Stale "rca is the current engine" references corrected in docs and comments

## Notes
- Upgrades: tree-sitter core 0.25 → 0.26.11, tree-sitter-rust 0.23.2 → 0.23.3,
  tree-sitter-python 0.23.6 → 0.25.0 (cpp/java/typescript were already latest). JS is the
  vendored mozjs fork, unaffected.
- tree-sitter 0.26 API change: `Node::child`/`named_child` now take `u32` (the `*_count`
  methods still return `usize`); cast `i as u32` at the 9 child-access sites.
- **No metric shift** — the golden fixture test and `quality-report.json` are both unchanged
  across the upgrade, so nothing needed re-blessing. Confirms the metrics are stable and
  genuinely rca-independent.
- Doc/comment cleanup (commit): corrected README, CLAUDE.md, `structural.rs`, `language.rs`,
  `native/mod.rs`, `native/lang.rs`, `native/metrics.rs`. Deliberately kept the
  "matching rca's `<fn>`" semantics notes in `native/rules` + metric modules and
  `golden.rs`'s past-tense history — those document each metric's exact definition/derivation
  (rca was the historical oracle) and are still encoded by the golden + divergence tests.
