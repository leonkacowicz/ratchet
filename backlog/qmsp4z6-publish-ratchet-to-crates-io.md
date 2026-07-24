# Publish ratchet to crates.io

## Summary
Publish ratchet to crates.io so it installs with a plain `cargo install ratchet` (no
`--git`). Deferred out of the distribution work in
[[distribution-cargo-install-prebuilt-binaries]], which shipped `cargo install --git` plus
prebuilt release binaries; crates.io is the more committal remaining path.

## Acceptance criteria
- [ ] `publish = false` removed from `Cargo.toml` (or crate renamed if the name is taken)
- [ ] Vendored tree-sitter-mozjs C sources confirmed to package into the `.crate` tarball
      (check `cargo package --list`) so `build.rs` finds them on a clean crates.io build
- [ ] `license` / `readme` / `repository` / `keywords` set for a decent crates.io page
- [ ] `cargo install ratchet` works from crates.io

## Notes
- Weigh before doing: crates.io publishes are irreversible (versions can only be yanked,
  not deleted), and the name `ratchet` may already be taken — check availability first.
- The registry-dependency grammars are all published; only the vendored Mozjs fork is local
  C source, which packages fine as long as it lands in the tarball.
