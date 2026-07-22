# Move metric thresholds into the config file

## Summary
Move `report::default_thresholds()` values into the config, keeping the current numbers
as defaults. The existing "threshold edits must land in their own PR" guard in the
`compare` command must keep working (it compares baseline vs HEAD thresholds).

## Acceptance criteria
- [ ] Thresholds read from config, defaults unchanged
- [ ] Threshold-change-in-own-PR guard still enforced
