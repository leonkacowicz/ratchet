# Configuration file support

## Summary
Lift ratchet's hard-coded assumptions into a config file so it can target arbitrary
projects and (later) languages. Today these are baked in: the `src/` source dir and
`.rs` filter (`collectors/structural.rs`), the metric thresholds (`report::default_thresholds`),
the Rust-only parser, and the `#[cfg(test)]` test-module stripping. This epic is the
foundation the multi-language work depends on.

## Acceptance criteria
- [ ] A config file is discovered from the project root and loaded into a typed config
- [ ] Thresholds, source roots/globs, and exclusions are all sourced from it
- [ ] Sensible defaults apply when no config is present (current behaviour preserved)
- [ ] Format and options are documented in the README

## Notes
Children decompose the format/loader from the individual settings it feeds.
