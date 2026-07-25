# Bump softprops/action-gh-release off deprecated Node.js 20

## Summary
The release workflow (`.github/workflows/release.yml`) pins
`softprops/action-gh-release@v2`, which still targets the Node.js 20 runtime GitHub is
deprecating — it is currently force-run on Node.js 24 with a warning annotation on the
"Publish GitHub Release" job. Bump it to a version (or pinned SHA) that targets Node.js 24
to clear the warning before the forced-migration deadline.

This is the follow-up sibling to [[bump-ci-actions-off-deprecated-node-js-20]], which
cleared `ci.yml`'s actions but did not touch the release workflow.

See: https://github.blog/changelog/2025-09-19-deprecation-of-node-20-on-github-actions-runners/

## Acceptance criteria
- [ ] The Release run's "Publish GitHub Release" job produces no Node.js-runtime
      deprecation annotation
- [ ] `softprops/action-gh-release` pinned to a version that targets a supported runtime

## Notes
Observed on release run 30135776271 (v0.1.0): the only remaining Node.js-20 annotation is
`softprops/action-gh-release@v2`. It was the last action still on the old runtime after
the `ci.yml` bumps. Check whether a newer `@v2` patch tag has already migrated before
jumping majors, since the annotation targets the resolved runtime, not the tag.
