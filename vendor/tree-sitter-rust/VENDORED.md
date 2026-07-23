# Vendored: tree-sitter-rust

Generated grammar C sources for the Rust language, statically linked into ratchet
via the top-level `build.rs`. This lets ratchet obtain a raw tree-sitter parse tree
for `.rs` files without depending on the `tree-sitter-rust` wrapper crate or on
`rust-code-analysis`.

## Source

- Upstream: https://github.com/tree-sitter/tree-sitter-rust
- Tag: **v0.23.2**
- Commit: `cad8a206f2e4194676b9699f26f6560d07130d3f`
- License: MIT (see `LICENSE`)

## Why this exact version

Pinned to **v0.23.2** deliberately: this is the same grammar version
`rust-code-analysis` (our current metric source) parses Rust with. Matching it means
the native raw-tree-sitter path produces the *same* syntax tree rca does, so upcoming
metric-parity checks isolate our metric computation rather than tree differences.
ABI-compatible with the `tree-sitter = 0.25` runtime already in the dependency graph.

## Files (copied verbatim from upstream `src/`)

- `parser.c` — generated LR parser tables (do not edit).
- `scanner.c` — hand-written external scanner (raw strings, block comments, etc.).
- `tree_sitter/parser.h`, `tree_sitter/alloc.h`, `tree_sitter/array.h` — headers the
  generated C includes.

## Updating

Re-download the same files at the desired tag and update the tag/commit above. Do not
hand-edit any file here — regenerate from upstream.
