# Expand documentation and usage examples

## Summary
Grow the README into proper docs: a metrics reference, config reference, and worked
examples covering multiple languages and CI.

## Acceptance criteria
- [ ] Docs cover config, languages, and CI
- [ ] At least one worked example
- [ ] Consumer bootstrap story documented: adopting the gate in a new repo — run
      `ratchet generate` and commit `quality-report.json`, set `fetch-depth: 0` (or fetch
      the base ref) so `compare` resolves the baseline, wire `--base origin/<base_ref>`,
      and note that pre-existing violations are grandfathered (bootstrap mode)

## Notes
- The `uses:`-style consumer CI snippet is tracked separately in
  [[consumer-facing-reusable-github-action-for-the-ratchet-gate]]; this issue is the prose
  setup story around it.
