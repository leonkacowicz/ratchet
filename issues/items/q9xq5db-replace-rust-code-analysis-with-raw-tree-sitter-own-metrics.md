# Replace rust-code-analysis with raw tree-sitter + own metrics

## Summary
Remove the dependency on `rust-code-analysis` (rca) and compute ratchet's metrics ourselves
directly over raw tree-sitter parse trees, using vendored, statically-linked grammars and
the `tree-sitter` runtime crate only.

**Why.** rca is pulled in as a pinned **git** dependency because the published crate
(0.0.25) does not build — a tree-sitter major-version split in its dependency graph
(verified: `E0308`, `tree_sitter 0.20.9` vs `0.26.11`). A git dependency cannot be
published to crates.io, so ratchet can never be `cargo install ratchet`-able while rca is a
git dep. Owning the grammar-linking coordination ourselves (one `tree-sitter` runtime,
grammars we compile) structurally eliminates the version conflict and makes ratchet
publishable. It also puts the metric definitions under our control (see
[[public-method-attribute-counts-from-rca-npm-npa-metrics]]).

**Division of labour we are taking on.** tree-sitter runtime = bytes → syntax tree
(language-agnostic). Vendored grammar (`parser.c` + hand-written `scanner.{c,cc}`) = the
per-language parse table. rca's job — walking that tree into metrics — is the part we
reimplement here. The linking half is easy (it's the wrapper crates' `build.rs`, inlined);
the metric half is the real work and carries a **numeric-parity risk** against the existing
committed `quality-report.json` baseline, which rca produced.

## Roadmap (this epic's shape)
1. **Rust as proof-of-concept** — stand up the vendor+link plumbing for Rust only, so we can
   parse a `.rs` file to a raw tree-sitter tree without rca.
2. **A parity harness** — a dual-path collector that computes a metric via either raw
   tree-sitter or rca and diffs them on fixtures, so each metric can be migrated and
   *proven* to match before switching over.
3. **Migrate metrics one at a time** (Rust) — SLOC, nargs, nom, cyclomatic, cognitive —
   each behind the parity harness. Ordered easiest-first to shake out the harness; cognitive
   last (hardest, most language-specific rules).
4. **Rust cutover** — once all Rust metrics are native, route `.rs` through the native path
   and drop rca for Rust (rca stays for the other languages meanwhile).
5. **Roll out to remaining languages & drop rca entirely** — repeat the pattern for
   C/C++, Python, Java, JS, TS, TSX, then remove the rca dependency. This is the point at
   which crates.io publishing becomes possible.

## Acceptance criteria
- [ ] `rust-code-analysis` removed from `Cargo.toml`
- [ ] All existing metric categories still produced, matching the pre-migration baseline
      (parity demonstrated per metric/language, deviations deliberate and documented)
- [ ] No git dependencies remain (crates.io-publishable dependency graph)

## Notes
- Alternative considered: **fork rca and fix its grammar pins**, publishing the fork. Cheaper
  (keeps rca's metric math, no parity risk) but you maintain/publish an rca fork and inherit
  its metric definitions rather than owning them. This epic chooses the own-the-metrics route
  deliberately; if the metric-reimplementation cost proves too high mid-flight, the fork route
  is the fallback.
- `scanner.{c,cc}` is **hand-written** upstream (indentation, raw strings, regex-vs-divide) —
  it is copied verbatim, not generated. `parser.c` is generated but shipped pre-generated in
  each grammar repo; vendor it rather than regenerating (regeneration needs the tree-sitter
  CLI + Node).
- Each vendored grammar carries its own license (mostly MIT) — preserve notices.
- Enables, but is not required by, crates.io distribution — see #m9435x3.
