# Publish ratchet to crates.io

## Summary
Publish ratchet to crates.io so it installs with a plain `cargo install ratchet` (no
`--git`). Deferred out of the distribution work in
[[distribution-cargo-install-prebuilt-binaries]], which shipped `cargo install --git` plus
prebuilt release binaries; crates.io is the more committal remaining path.

## Acceptance criteria
- [ ] `publish = false` removed from `Cargo.toml` (or crate renamed if the name is taken)
- [ ] `license` / `readme` / `repository` / `keywords` set for a decent crates.io page
- [ ] `cargo install ratchet` works from crates.io

## Notes
- Weigh before doing: crates.io publishes are irreversible (versions can only be yanked,
  not deleted), and the name `ratchet` may already be taken — check availability first.
- Packaging is now straightforward: every grammar is a published registry dependency and the
  vendored C sources / `build.rs` are gone (see
  [[drop-vendored-mozjs-for-published-tree-sitter-javascript]]), so there is nothing local to
  bundle into the `.crate` tarball.
