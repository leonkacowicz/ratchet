# Drop vendored mozjs for published tree-sitter-javascript

## Summary
The `tree-sitter-mozjs` grammar was vendored (`vendor/`, compiled by `build.rs` + `cc`) only
to match `rust-code-analysis`'s JavaScript parse trees. With rca gone
([[relax-grammar-pins-upgrade-tree-sitter-now-that-rca-is-gone]]) ratchet defines its own
metrics, so the published `tree-sitter-javascript` crate can replace the fork.

## Acceptance criteria
- [x] JavaScript wired to `tree-sitter-javascript` instead of the vendored mozjs extern
- [x] `vendor/tree-sitter-mozjs/`, `build.rs`, and the `cc` build-dependency removed
- [x] No metric change (golden + JS/JSX divergence tests pass; `quality-report.json` unchanged)
- [x] Docs refreshed (README grammar table/routing, `language.rs`, "Adding a language" guide)

## Notes
- Verified drop-in: `tree-sitter-javascript` 0.25.0 produces byte-identical metrics for every
  fixture including `example.jsx` (JSX parses in-grammar, same as mozjs). clippy + fmt clean.
- Removes the build's only bespoke piece (~2.5MB generated C, a build script, and a
  build-dependency) and simplifies crates.io publishing — see [[publish-ratchet-to-crates-io]].
- Kept the "rca's Mozjs …" algorithm-derivation comments in `native/rules/js.rs` and
  `golden.rs`: they document node-kind semantics (still valid — the kinds match), like the
  other retained "matching rca's `<fn>`" notes.
