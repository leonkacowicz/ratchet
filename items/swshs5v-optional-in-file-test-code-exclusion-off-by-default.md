# Optional in-file test-code exclusion, off by default

## Summary
ratchet measures test code like production code (#46dzb3w), and that stays the default. But
it is a position, not a law, and a consumer who disagrees currently has only one lever:
`exclude` globs. That works for test *files* and not at all for test code that lives inside a
production file — Rust `#[cfg(test)]` modules, Python doctests, Go `func TestX` in
`*_test.go` compiled alongside the package. For those, the choice today is all-or-nothing per
file.

So offer the exclusion properly, as an opt-in a consumer configures, instead of the hidden
per-language rule that used to be hardcoded:

```json
{ "exclude_test_code": true }
```

Off by default. When on, each language's rules decide what counts as test code and it is
dropped before metrics are computed.

Deliberately scoped to **test** code. Generated and vendored output is a separate concern
that `exclude` globs already handle well, and the two do not want the same mechanism: one is
"this directory is machine-written", the other is "this region of a hand-written file is a
test".

## Acceptance criteria
- [ ] A config flag turns in-file test-code exclusion on; default off, so nothing changes for
      an existing `ratchet.json`.
- [ ] Rust excludes `#[cfg(test)]` modules — *every* such module in the file, at any position,
      not only a trailing one, and a `#[cfg(test)] mod NAME;` declaration excludes nothing but
      itself (the bug the old stripper had; see #p7t6qnm).
- [ ] Exclusion is expressed in the per-language rules (`src/native/rules/`), not as a
      pre-parse text transform — it should work off the parse tree.
- [ ] Languages with no rule for it are unaffected rather than silently half-covered, and
      `generate` makes clear which languages the flag actually did something for.
- [ ] The flag's value is recorded in `quality-report.json` and, like a threshold change,
      `compare` rejects a report whose value differs from the baseline's — flipping it moves
      every metric at once and must land in its own reviewable PR.
- [ ] README documents it under Test code, stating plainly that the default is the
      recommended setting and why.

## Notes
- Low priority on purpose: this exists so the default is a *choice* rather than something
  imposed, and to serve consumers adopting ratchet on a codebase whose tests would otherwise
  bury the signal on day one. Nothing here needs doing for ratchet's own use.
- The old implementation is the anti-pattern to avoid: `strip_test_modules` was a line-based
  text truncation applied before parsing, which is why a single misplaced attribute could
  erase an entire file. Whatever replaces it works on the tree.
- A gentler middle option, if this proves fiddly: per-category threshold overrides scoped to a
  glob, so tests can be held to *looser* limits rather than no limits. Keeps the ratchet
  engaged on test code instead of blinding it.
