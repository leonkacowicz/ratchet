# Go support via native tree-sitter

## Summary

**Re-scoped 2026-07-25: no external tool.** The original plan (shell out to `gocyclo`/`gocognit`
via a "Path B" `Collector`) existed only because rca had no Go grammar. rca is gone — ratchet
now computes every metric itself from the tree-sitter parse tree, driven by per-language
`Rules` data (`src/native/rules/`). A published `tree-sitter-go` grammar exists, so Go should
be a **native language exactly like the other 7** (Rust, C/C++, Java, JS, TS, TSX, Python): it
supplies *data* (node kinds), not a new metric engine or an external dependency.

This also removes the last source of "partial metric coverage" from the roadmap — a native Go
supplies the full `Rules` set, so it measures all five metrics (SLOC, cognitive, cyclomatic,
nargs, function count) like every other language, with no external-tool fallback to degrade.

Follow the pattern established by `mpjw47f` (enable rca-supported languages) and the
"# Adding a language" checklist in `src/native/rules/mod.rs`.

## Acceptance criteria
- [ ] `tree-sitter-go` grammar crate added to `Cargo.toml`.
- [ ] A `NativeLanguage` wiring its `LANGUAGE` in `src/native/lang.rs`, plus an arm in
      `lang::for_language` so `native::supports` recognizes Go.
- [ ] A `GO` `Rules` entry in `src/native/rules/` describing Go's node kinds for every metric —
      function kinds, decision points (cyclomatic), cognitive nesting/flat kinds, nargs
      delimiters. No metric-driving field left empty (an empty set silently zeroes a metric).
- [ ] `.go` dispatched to the Go parser in `src/language.rs`.
- [ ] Fixtures under `tests/fixtures/` exercising Go's constructs (incl. Go-specific forms:
      `select`, type switch, `defer`, goroutines, multiple return values), and the golden
      blessed (`RATCHET_BLESS_GOLDEN=1 cargo test golden`).
- [ ] Per-entity ratchet verified for Go (a Go entity that worsens fails; a split passes).

## Notes

- Native-language pattern and the step-by-step "Adding a language" guide live at the top of
  `src/native/rules/mod.rs`. Grammar wiring: `src/native/lang.rs`; extension dispatch:
  `src/language.rs`; per-language node kinds: `src/native/rules/*.rs`.
- Go quirks to get right in the `Rules`: no ternary (so no `ternary_expression` decision kind);
  `switch` / type-switch / `select` statements; `for` is the only loop kind; short-circuit `&&`
  / `||`; labeled `break`/`continue`. Derive the kinds from the tree-sitter-go grammar and pin
  them with fixtures, the same way the existing languages were.
- The `Collector` seam (`src/collectors/mod.rs`) still exists for a genuine future external
  source, but Go no longer needs it — this is the native `structural` path.
- Cross-language metric comparability is still not required: the ratchet only compares each
  entity against its own past value, so Go's numbers only need to be internally consistent.
- Ripples: this removes the roadmap's only external-tool collector, reinforcing the direction
  discussed on `e4jhf5c` (guarantee complete coverage rather than represent "not measured").
  The dependency on `4mtrdc7` (config loader) is already satisfied; the real prerequisite is
  extension dispatch (`mzm7h9r`, done).
