# The action's version input defaults to latest, so pinning the tag pins nothing

## Summary

`action.yml` declares `version` with `default: "latest"`, and nothing in the composite steps
narrows it. A consumer who writes

```yaml
- uses: leonkacowicz/ratchet@v0.1.1
```

has pinned the *installer* and not the *tool*: the tag selects which `action.yml` and `install.sh`
run, while the binary those scripts fetch is whatever the newest release happens to be at the
moment the job runs. The two versions are independent, and only one of them is visible in the
workflow.

Nothing surfaces the gap. `action.yml` is byte-identical between v0.1.1 and v0.2.0, so a consumer
reading `@v0.1.1` and reasonably expecting 0.1.1 behaviour gets 0.2.0 with no warning, no diff in
their repository, and no step name that mentions a version. The installed version *is* written to
the step's `version` output, but a consumer who never wires that output up never sees it.

This is the delivery vector for exactly the breakage `#gkgh2g9` is about. 0.2.0 changed what is
measured — test code is no longer stripped — and under the default that change reaches a consumer's
CI on their next unrelated push. `#gkgh2g9` makes the resulting comparison honest once it happens;
this issue is about it not happening unbidden. The two are complementary, and neither subsumes the
other: an epoch cannot help a consumer who never chose to upgrade.

The same default also makes the gate non-reproducible in the ordinary sense. Re-running a green job
from three months ago can install a different binary than it did the first time, so a historical
run cannot be trusted to mean what it said.

## Proposed direction

Default `version` to **the action's own ref** rather than `latest`, so `@v0.2.0` installs ratchet
0.2.0 and the single version in the workflow is the truth. `GITHUB_ACTION_REF` carries it. Fall
back to `latest` when that ref is not a release tag — a branch, a SHA, or `uses: ./` — since there
is nothing coherent to derive in those cases.

That makes the common usage correct by default and leaves `version:` as the explicit escape hatch
for anyone who genuinely wants to float or to mix versions.

Worth deciding as part of it: whether a floating `latest` should stay silent. A one-line notice
naming the resolved version would make the choice visible in the log without failing anything.

## Acceptance criteria

- [ ] `version` defaults to the action's own release tag when `GITHUB_ACTION_REF` names one
- [ ] Falls back to `latest` for a branch, a SHA, or a local `uses: ./` checkout
- [ ] An explicit `version:` input still wins over both
- [ ] Same behaviour on the Windows path, which resolves the version separately
- [ ] A floating resolution — explicit `latest`, or the fallback — says so in the log, naming the
      version it resolved to
- [ ] README documents that the tag and the tool are two versions, and what pinning each one buys
- [ ] Verified on a runner, not only by reading: a workflow pinned to an older tag installs that
      older binary

## Notes

Found while bumping a consuming repository's workflows. The tag there had read `@v0.1.1` for weeks
while CI was in fact running 0.2.0 — the behaviour was correct, but for no reason the workflow
could account for, and the same mechanism would have silently changed what the gate measured.

Related: `#gkgh2g9` (baseline epoch), `#ec8ph6b` (the cargo-install fallback, the other path through
the same steps).
