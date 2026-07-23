# Remove rca from the Rust path (all Rust metrics native)

## Summary
Cutover for the proof-of-concept language. Once every Rust metric (SLOC, nargs, nom,
cyclomatic, cognitive) is native and parity-verified, route `.rs` files entirely through the
native raw-tree-sitter path and stop calling rca for Rust. rca remains a dependency for the
other languages until the rollout ([[roll-out-native-metrics-to-remaining-languages-drop-rca-entirely]])
completes.

## Acceptance criteria
- [ ] Rust files no longer invoke rca for any metric
- [ ] Full report on ratchet's own `src/` is unchanged vs the pre-migration baseline
      (or differences are deliberate and documented)
- [ ] Dual-path selector for Rust can be retired (native is the only Rust path)
- [ ] Committed `quality-report.json` regenerated if any values legitimately shifted

## Notes
- This proves the whole pattern end-to-end on one language before committing to the other
  six. A clean cutover here is the go/no-go signal for the rollout.
- Depends on all five Rust metric migrations.
