# Verify the tag-derived action version on a runner after the next release

## Summary

`#vatzr6d` made the action's `version` input default to the action's own release tag, derived
from `GITHUB_ACTION_REF`. That derivation is verified two ways short of the real thing: a case
table covering release tags, branches, SHAs and an unset ref, and a smoke-test job that pins
`version:` explicitly on all three runner OSes and asserts the installed binary matches.

Neither exercises the actual default path, and neither can. `uses: ./` — the only form the smoke
test can use pre-release — deliberately carries no release tag, so it takes the `latest` fallback;
and `uses: leonkacowicz/ratchet@<tag>` runs the `action.yml` *from that tag*, which for every tag
published so far still defaults to `latest`. The derivation only becomes reachable once a release
ships carrying it.

So this is a one-time check to run after the first release that includes the fix, not a gap in the
implementation. It mirrors `#ec8ph6b`, which parks the same kind of can-only-be-seen-on-a-runner
confirmation for the cargo-install fallback.

## Acceptance criteria

- [ ] After the first release carrying the fix, a workflow using `leonkacowicz/ratchet@<that tag>`
      with no `version:` input installs exactly that version — confirmed from the job log, not by
      reading `action.yml`
- [ ] The same holds on the Windows runner, which resolves the version through a separate pwsh
      block that has never been executed locally (no pwsh on the dev machine)
- [ ] A workflow on an *older* tag still installs `latest`, since its `action.yml` predates the
      fix — confirm this is so and that the README's account of pinning does not overclaim for
      tags published before the change
- [ ] Once two releases carry the fix, pin the older of them and confirm it installs the older
      binary — the end-to-end case `#vatzr6d` is really about

## Notes

If the check fails, the likely suspects are the tag-shape pattern (`v[0-9]*.[0-9]*.[0-9]*` in
bash, `^v\d+\.\d+\.\d+` in pwsh) against whatever `GITHUB_ACTION_REF` actually contains, and
whether that variable is populated at all inside a composite action's steps.

Related: `#vatzr6d` (the fix), `#ec8ph6b` (the sibling runner check), `#gkgh2g9` (the epoch that
makes a deliberate upgrade honest once one happens).

## Status

**v0.2.1 (2026-09-13) is the first release carrying the fix** — that is the baseline this check
needs, and it is now published.

It is still not decidable today. `latest` currently *is* v0.2.1, so `uses: ...@v0.2.1` installs
0.2.1 whether the version was derived from the tag or floated, and the pre-fix `action.yml` on
every older tag would land on the same binary. The two behaviours only diverge once a second
release exists and the older of the two can be pinned — criterion four. So: do this after v0.2.2.

What *is* confirmed on real runners, from the smoke test on the release commit
(run 34758580331, all six jobs green): the pwsh path executes correctly on windows-latest — it had
never run locally, there is no pwsh on the dev machine — the explicit `version:` input installs the
pinned older release on ubuntu/macOS/Windows alike, and the floating fallback under `uses: ./`
emits the expected notice on all three, naming the version it resolved to.
