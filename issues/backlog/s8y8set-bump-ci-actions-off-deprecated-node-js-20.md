# Bump CI actions off deprecated Node.js 20

## Summary
The CI workflow (`.github/workflows/ci.yml`) pins `actions/checkout@v4` and
`actions/upload-artifact@v4`, which target the Node.js 20 runtime that GitHub is
deprecating (they are currently force-run on Node.js 24 with a warning annotation). Bump
to versions that target Node.js 24 (or the then-current runtime) to clear the warning
before the forced-migration deadline.

See: https://github.blog/changelog/2025-09-19-deprecation-of-node-20-on-github-actions-runners/

## Acceptance criteria
- [ ] CI runs with no Node.js-runtime deprecation annotations
- [ ] `actions/checkout` and `actions/upload-artifact`/`download-artifact` pinned to a
      supported major version
