# Distribution: cargo install & prebuilt binaries

## Summary
Make ratchet installable via `cargo install` and, as a stretch, prebuilt per-platform
binaries so CI doesn't have to compile it.

## Acceptance criteria
- [x] Install path documented
- [x] (stretch) Release binaries produced

## Notes
- `cargo install --git https://github.com/leonkacowicz/ratchet --locked` verified end to
  end from a clean checkout (vendored tree-sitter-mozjs grammar builds through the git
  install path). `publish = false` only blocks crates.io publishing, not `cargo install`.
- Added `version` to the clap command so `ratchet --version` works on distributed binaries.
- `.github/workflows/release.yml`: on a `v*` tag, builds a binary per platform on a native
  runner (linux-gnu x86_64, macOS x86_64 + aarch64, windows-msvc x86_64 — native runners so
  the `cc`-built grammar never cross-compiles), packages each with a `.sha256`, and attaches
  them all to the tag's GitHub Release.
- README gained an Installation section covering both `cargo install` and prebuilt binaries.
- crates.io publishing was deliberately left out of scope (more committal; irreversible
  publishes, name-squatting). Could be a future issue if wanted.
