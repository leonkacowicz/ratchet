# Dispatch parser by file extension

## Summary
Replace the hard-coded `RustParser::new` in `collectors/structural.rs` with dispatch
that selects the rca parser by file extension. Foundational for every other language.

## Acceptance criteria
- [ ] File extension selects the correct rca parser
- [ ] Unknown extensions are skipped with a warning
- [ ] Rust output is byte-for-byte unchanged
