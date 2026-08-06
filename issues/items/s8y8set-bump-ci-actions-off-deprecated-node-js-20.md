# Bump CI actions off deprecated Node.js 20

## Summary
The CI workflow (`.github/workflows/ci.yml`) pins `actions/checkout@v4` and
`actions/upload-artifact@v4`, which target the Node.js 20 runtime that GitHub is
deprecating (they are currently force-run on Node.js 24 with a warning annotation). Bump
to versions that target Node.js 24 (or the then-current runtime) to clear the warning
before the forced-migration deadline.

See: https://github.blog/changelog/2025-09-19-deprecation-of-node-20-on-github-actions-runners/

## Acceptance criteria
- [x] CI runs with no Node.js-runtime deprecation annotations
- [x] `actions/checkout` and `actions/upload-artifact`/`download-artifact` pinned to a
      supported major version

## Notes
Bumped `actions/checkout@v4→v5`, `actions/upload-artifact@v4→v7`, and
`actions/download-artifact@v4→v8` — the first majors on the Node.js 24 runtime.
`upload-artifact@v5`/`download-artifact@v5` are still Node.js 20, so those two skipped
ahead. `Swatinem/rust-cache@v2` was already Node 24 and `dtolnay/rust-toolchain` is a
composite action, so neither needed touching. Verified: run 30017461034 succeeded with 0
annotations on both jobs.
