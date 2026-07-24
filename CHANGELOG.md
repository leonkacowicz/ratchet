# Changelog

All notable changes to this project are documented here. The format is based on
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project adheres to
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-07-24

First tagged release: a language-agnostic, one-way code-quality ratchet that snapshots
structural metrics into a committed `quality-report.json` and, in CI, blocks any change
that worsens them while always allowing improvement.

### Added

- **Ratchet gate** — two rules compared against a baseline report, per category: no
  existing entity may worsen, and no category total may grow. Splitting a fat file or
  function passes, renames are neutral, and categories ratchet independently.
- **Structural metrics** across seven languages via tree-sitter grammars: Rust, C/C++,
  Python, Java, JavaScript, TypeScript, and TSX. Per-function lines, cognitive and
  cyclomatic complexity, and argument count; per-file lines and function count; per-directory
  file count.
- **CLI verbs** — `generate` (write the report), `check` (fail on a stale report),
  `compare --base <ref>` (fail on regression vs a baseline git ref), and `dump` (inspect the
  parsed function/space tree). `--version` reports the build version.
- **Configuration** — an optional `ratchet.json` with source roots, include/exclude globs,
  and per-category threshold overrides. Thresholds are recorded in the report, so a
  threshold change must land in its own PR.
- **Self-hosted CI workflow** (`.github/workflows/ci.yml`) that builds, lints, and tests the
  code, then runs ratchet's gate against ratchet itself.
- **Distribution** — installable via `cargo install --git … --locked`, and a tag-triggered
  release workflow (`.github/workflows/release.yml`) that publishes prebuilt binaries for
  Linux (x86-64), macOS (x86-64 and Apple silicon), and Windows (x86-64), each with a
  SHA-256 checksum.

[0.1.0]: https://github.com/leonkacowicz/ratchet/releases/tag/v0.1.0
