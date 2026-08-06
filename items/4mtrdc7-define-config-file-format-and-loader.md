# Define config file format and loader

## Summary
Pick a config format (`ratchet.toml` or `.ratchet.json`) and implement discovery +
loading into a typed `Config` struct that the rest of the tool reads from. Discover by
walking up from `--root`; allow a `--config PATH` override.

## Acceptance criteria
- [ ] Format chosen and documented
- [ ] Loader with defaults when the file is absent
- [ ] `--config` override flag
- [ ] Round-trip / defaulting tests

## Notes
Blocks the other CONFIG children — they all read from this struct.
