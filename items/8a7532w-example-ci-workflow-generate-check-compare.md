# Example CI workflow (generate / check / compare)

## Summary
Ship a reusable CI workflow that runs `ratchet check` (report freshness) and
`ratchet compare --base origin/main` (regression gate) on pull requests. Must-ship for
v0.1.0.

## Acceptance criteria
- [x] Workflow snippet committed under docs/ or .github/
- [x] Documented in the README

## Notes
Shipped as `.github/workflows/ci.yml` (a live workflow, not just a snippet): a `build`
job (fmt / clippy / build / test) that uploads the release binary, feeding a `quality`
job that runs `ratchet check` (freshness) always and `ratchet compare` (regression gate)
on pull requests. Self-hosted in the sense that the gate is ratchet's own binary, on
GitHub-hosted runners. README gained a "Continuous integration" section and a CI badge.
