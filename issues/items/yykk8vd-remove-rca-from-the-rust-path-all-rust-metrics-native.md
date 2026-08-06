# Remove rca from the Rust path (all Rust metrics native)

## Summary
Cutover for the proof-of-concept language. Once every Rust metric (SLOC, nargs, nom,
cyclomatic, cognitive) is native and parity-verified, route `.rs` files entirely through the
native raw-tree-sitter path and stop calling rca for Rust. rca remains a dependency for the
other languages until the rollout ([[roll-out-native-metrics-to-remaining-languages-drop-rca-entirely]])
completes.

## Acceptance criteria
- [x] Rust files no longer invoke rca for any metric — `structural.rs` sets `top = None` for
      `Language::Rust`; `parse_metrics` runs only in the non-Rust match arm
- [x] Full report on ratchet's own `src/` is unchanged vs the pre-migration baseline
      — `check`/`compare` green, report byte-identical (native == rca)
- [x] Dual-path selector for Rust retired in effect — `use_native` resolves to native for every
      Rust metric, so no rca path is taken; `top: Option<&FuncSpace>` is `None` for Rust
- [x] `quality-report.json` needed no change (values identical)

## Notes
- Proves the whole pattern end-to-end on one language: Rust is fully off rca in the metric
  path. rca still backs the other six languages (and the `ratchet dump` debug command), so the
  crate dependency remains until the rollout
  ([[roll-out-native-metrics-to-remaining-languages-drop-rca-entirely]]).
- Side benefit: Rust files skip the (slow) rca parse entirely — only tree-sitter runs.
- Depends on all Rust metric migrations.
