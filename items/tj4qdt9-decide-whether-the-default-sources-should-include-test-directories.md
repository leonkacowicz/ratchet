# Decide whether the default sources should include test directories

## Summary
`Config::default()` sets `sources` to `["src"]` (`src/config.rs:35`). Since #46dzb3w, ratchet
measures test code like production code — but only the test code that happens to live under a
configured root. That makes the default quietly inconsistent across languages: Rust and Python
inline tests are gated because they sit inside `src`, while out-of-tree tests (`tests/`,
`src/test/java`, `spec/`, `__tests__/`) escape entirely unless the consumer lists them.

So a repo adopting ratchet gets its tests gated or not based on its language's layout
convention, which is not a policy anybody chose.

## Acceptance criteria
- [ ] A decision is recorded here: keep `["src"]`, widen the default, or drop the default and
      require `sources` explicitly.
- [ ] If the default changes, README's Configuration section and the adoption checklist say so,
      and the change is called out as consumer-visible in `CHANGELOG.md`.

## Notes
- Widening the default (e.g. `["src", "tests"]`) matches the policy but silently enlarges the
  baseline of every existing consumer on upgrade — their next `generate` grows, and until they
  regenerate, `compare` is measuring different file sets on the two sides.
- Requiring `sources` explicitly is the most honest option and the most annoying one; it turns
  a silent mis-scope into a startup error. Relates to the "if it's near-empty, your config is
  probably wrong" warning already in the README adoption checklist.
- A middle option: keep the default narrow, but have `generate` report which roots it scanned
  and how many files each contributed, so an unintended empty root is visible rather than
  inferred.
