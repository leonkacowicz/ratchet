# Standalone install script for prebuilt binaries

## Summary
A one-liner installer (`install.sh`) that detects the host OS/arch, downloads the matching
prebuilt release asset, verifies its `.sha256`, and drops `ratchet` on `PATH`. Serves
consumers outside GitHub Actions (local machines, GitLab CI, etc.) and gives the
[[consumer-facing-reusable-github-action-for-the-ratchet-gate]] a single implementation to
shell out to instead of re-encoding the download logic.

Today the README only has a manual `curl | tar` snippet a user runs by hand — nothing a
`run:` step or `curl -fsSL … | sh` can invoke generically.

## Acceptance criteria
- [x] `install.sh` detects OS/arch and picks the correct release asset
- [x] Downloads and verifies the asset's `.sha256` before installing
- [x] Installs to a sensible dir (`$PREFIX`/`~/.local/bin`, overridable)
- [x] Documented in the README as `curl -fsSL … | sh`

## Notes
- Depends on `8bqetja` (Release v0.1.0): needs an actual release with attached binaries to
  download — produced by the release workflow from
  [[distribution-cargo-install-prebuilt-binaries]].
- Windows is out of scope for a POSIX `install.sh`; the reusable action covers Windows
  runners directly.
- Implemented as `install.sh` at the repo root (POSIX sh; `sh -n` and `dash -n` clean).
  Config via flags or env: `--version`/`RATCHET_VERSION`, `--bin-dir`/`RATCHET_BIN_DIR`
  (or `PREFIX`), `--dry-run`, `-h`. curl-or-wget download; sha256sum-or-shasum verify.
- Verified offline: platform→target mapping for all branches (Linux/macOS x86_64+arm,
  unsupported-arch and unsupported-OS errors), arg parsing, and — critically — that the
  release workflow's `shasum -a 256` `.sha256` verifies with the installer's `sha256sum -c`
  and that a corrupted archive is rejected. The live network fetch is untested until the
  v0.1.0 release actually publishes (currently blocked on a stuck macOS-Intel runner).
