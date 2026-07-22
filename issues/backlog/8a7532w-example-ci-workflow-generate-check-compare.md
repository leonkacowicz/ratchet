# Example CI workflow (generate / check / compare)

## Summary
Ship a reusable CI workflow that runs `ratchet check` (report freshness) and
`ratchet compare --base origin/main` (regression gate) on pull requests. Must-ship for
v0.1.0.

## Acceptance criteria
- [ ] Workflow snippet committed under docs/ or .github/
- [ ] Documented in the README
