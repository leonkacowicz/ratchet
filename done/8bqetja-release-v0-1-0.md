# Release v0.1.0

## Summary
First real tagged release. Tracked as a milestone via dependency edges (not containment)
to the must-ship issues: config format+loader, configurable source globs, extension
dispatch, enabling the rca languages, and an example CI workflow. When those land, cut
v0.1.0.

## Acceptance criteria
- [x] All dependency issues done
- [x] Version bumped and CHANGELOG written
- [x] Tagged v0.1.0

## Notes
Dependencies are the release's blockers; see `trck deps` for the current line.

Cut 2026-07-24: crate bumped to 0.1.0, `CHANGELOG.md` added, tag `v0.1.0` pushed. The tag
fires `.github/workflows/release.yml`, which builds prebuilt binaries per platform and
publishes them to the GitHub Release (workflow added in
[[distribution-cargo-install-prebuilt-binaries]]).
