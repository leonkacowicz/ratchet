# Move metric thresholds into the config file

## Summary
Move `report::default_thresholds()` values into the config, keeping the current numbers
as defaults. The existing "threshold edits must land in their own PR" guard in the
`compare` command must keep working (it compares baseline vs HEAD thresholds).

## Acceptance criteria
- [x] Thresholds read from config, defaults unchanged
- [x] Threshold-change-in-own-PR guard still enforced

## Notes
`Config` gained an optional `thresholds` map (JSON key `thresholds`) of per-category
overrides, resolved by `Config::effective_thresholds()` — it merges the overrides onto
`report::default_thresholds()` (so omitted categories keep their default) and rejects an
unknown category name. `report::generate` now takes the resolved thresholds as an argument
instead of hard-coding `default_thresholds()`; `main::build_report` wires config → sources
→ thresholds → generate for both `generate` and `check`.

The compare guard (`cmd_compare`, `main.rs`) is untouched: each report embeds its effective
thresholds, so a threshold edit changes the committed report and `compare` still bails when
baseline and HEAD thresholds differ. Verified end-to-end: a `ratchet.json` override changes
the recorded threshold and the resulting violations, and an unknown category errors out.
Documented in the README Configuration section and CLAUDE.md.
