# Rust PoC: vendor grammar + static-link, produce a raw tree-sitter tree

## Summary
Foundation for the epic. Vendor the Rust grammar into ratchet and statically link it via a
`build.rs`, depending only on the `tree-sitter` runtime crate — no `tree-sitter-rust`
wrapper, no rca. End state: given a `.rs` file's bytes, we can produce a raw tree-sitter
`Tree` and walk its nodes.

This is the easy, low-risk half of the migration — it is exactly what the wrapper crates'
`build.rs` already does, inlined into our tree. Rust only; other languages come later.

## Acceptance criteria
- [x] Rust `parser.c` + `scanner.c` vendored into the repo (with upstream license notice)
      — `vendor/tree-sitter-rust/` (tag v0.23.2, MIT `LICENSE`)
- [x] `build.rs` compiles them via the `cc` crate; runtime bound through the `tree-sitter`
      crate already in the graph (`tree-sitter`/`tree-sitter-language` as direct deps, `cc`
      as a build-dep — no `tree-sitter-rust` wrapper, no rca)
- [x] A helper parses `.rs` bytes → `tree_sitter::Tree`; happy-path test walks the tree and
      finds a `function_item` (`src/native.rs::parse_rust`)
- [x] Grammar revision/source recorded (`vendor/tree-sitter-rust/VENDORED.md`)

## Notes
- Pick a Rust grammar revision whose bundled `parser.c` ABI version is within the chosen
  `tree-sitter` runtime's supported range — this is the coordination rca 0.0.25 got wrong,
  now owned by us.
- No metrics here yet; this issue only establishes "raw tree in hand." Metrics follow behind
  the parity harness (its dependant).
