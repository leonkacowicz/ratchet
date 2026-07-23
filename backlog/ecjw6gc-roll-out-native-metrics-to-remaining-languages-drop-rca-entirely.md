# Roll out native metrics to remaining languages & drop rca entirely

## Summary
Repeat the proven Rust pattern (vendor grammar + static link → migrate each metric behind
the parity harness → cutover) for the remaining languages — C/C++, Python, Java, JavaScript,
TypeScript, TSX — then remove `rust-code-analysis` from `Cargo.toml` entirely. Completing
this is what makes ratchet's dependency graph crates.io-publishable.

Large and currently coarse: this should be **decomposed into per-language sub-issues** (or
promoted to a sub-epic) once the Rust PoC cutover lands and the real per-language effort is
understood. Left as a single placeholder deliberately until then.

## Acceptance criteria
- [ ] Each remaining language vendored, statically linked, and all its metrics migrated with
      parity verified
- [ ] `rust-code-analysis` removed from `Cargo.toml`; no git dependencies remain
- [ ] Full `quality-report.json` parity across the whole codebase (deliberate diffs documented)
- [ ] Follow-up captured: crates.io publish is now unblocked (relates to #m9435x3)

## Notes
- Per-language scanners differ (Python indentation, C++ raw strings, JS regex-vs-divide, TS
  vs TSX) — each vendored `scanner.{c,cc}` copied verbatim; expect the JS/TS family to be the
  fiddliest.
- npm/npa ([[public-method-attribute-counts-from-rca-npm-npa-metrics]]) — if adopted before
  this lands, they too must be reimplemented natively here (class-based langs only).
- Depends on the Rust cutover proving the pattern
  ([[remove-rca-from-the-rust-path-all-rust-metrics-native]]).
