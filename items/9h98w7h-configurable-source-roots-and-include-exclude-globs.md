# Configurable source roots and include/exclude globs

## Summary
Replace the hard-coded `SOURCE_DIR = "src"` and the `.rs` extension filter in
`collectors/structural.rs` with configurable source roots plus include/exclude globs.
Needed before multiple languages can be scanned in one run.

## Acceptance criteria
- [ ] Multiple source roots supported
- [ ] Include/exclude globs honoured
- [ ] Default remains `src/**` when unconfigured
