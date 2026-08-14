# Split src/main.rs: 21 functions over the file_functions threshold

## Summary
`src/main.rs` carries 21 functions against a `file_functions` threshold of 20, so it sits in
`quality-report.json` as a grandfathered violation with excess 1. It surfaced when the
`#[cfg(test)]` exemption was dropped (#46dzb3w): the file's `#[cfg(test)] mod golden;`
declaration on line 3 had been truncating the whole file away, so its 21 functions were
counted as 0 and it never appeared in the report.

Nothing about the file got worse — it was simply never measured. But it is the only entity in
its category, and carrying an excess of 1 that nobody intends to fix is the kind of thing the
ratchet is supposed to make you deal with.

## Acceptance criteria
- [ ] `src/main.rs` is at or under 20 functions, or the split that gets it there is
      deliberately rejected and this issue closed `wontfix` with the reason recorded.
- [ ] `file_functions` has no entries in `quality-report.json`.

## Notes
- Natural seam: the verb bodies (`generate` / `check` / `compare` / `dump`) and their report
  I/O helpers are separable from the CLI parsing. A `src/cli.rs` or per-verb module would
  drop the count well under the threshold without inventing structure.
- The tests inside `main.rs` move with whatever they cover.
