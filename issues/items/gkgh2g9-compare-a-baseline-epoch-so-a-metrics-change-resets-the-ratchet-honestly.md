# compare: a baseline epoch so a metrics change resets the ratchet honestly

## Summary

`compare --base <rev>` resolves the baseline with `git show <rev>:quality-report.json` — the file as
whatever ratchet wrote it at the time. That is sound only while both sides were measured by the same
ratchet. It stops being sound the day ratchet's own measurement changes: 0.2.0 began counting test
code (#46dzb3w), and every file in a consuming repository read as a large regression against a 0.1.1
baseline, on a change that touched no code at all.

Add an explicit **baseline epoch** to the report. `compare` compares numbers only when head and
baseline carry the same epoch; when they differ it accepts head's numbers as the new baseline
instead of comparing. Bumping the epoch is then the sanctioned way to absorb a measurement change —
and, so it cannot double as a way to launder a regression, **the bump must be the only thing in its
commit**: no measured file may change alongside it. The next commit carries the same epoch, so the
comparison is meaningful again immediately.

## Acceptance criteria

- [ ] `generate` writes a baseline epoch into the report, sourced from `ratchet.json` so it is the
      consumer's to bump; absent means epoch 0.
- [ ] `generate` also stamps the ratchet version that measured the report, so a forgotten epoch bump
      produces a clear "baseline measured by X, running Y" diagnostic instead of a wrong comparison.
- [ ] Equal epochs: `compare` behaves exactly as it does today.
- [ ] Head epoch **newer** than the baseline's: `compare` skips the metric comparison and exits 0 —
      but prints what it forgave (which categories worsened, and the largest single move), so a
      re-baseline that hides a regression is visible in the log rather than invisible.
- [ ] Head epoch **older** than the baseline's: fail with "baseline moved, rebase". Otherwise a
      branch created before the epoch commit silently loses the gate for as long as it is open.
- [ ] An epoch bump that arrives alongside a change to any measured file is refused — the same
      spirit as the existing rule that a threshold edit cannot land with new violations.
- [ ] README / example workflow document the epoch and the one-change-per-commit rule.

## Notes

**The open design question is who enforces "nothing else changed".** Ratchet cannot see it from the
reports alone: the numbers differing *is* the expected state of an epoch bump, so there is no signal
in the data. Detecting it needs `git diff --name-only <base> HEAD` filtered to measured paths —
cheap (no checkout, no second `generate`), but it puts git back into ratchet's execution path, which
it currently does not need. The alternative is to leave the rule to CI or a pre-commit hook and keep
ratchet report-only. Decide before implementing; the acceptance criterion above assumes ratchet
enforces it.

**Why not just detect the version mismatch and re-measure.** Ratchet could check the base tree out
itself and re-measure it with the running binary — correct by construction, no user action at all —
but that means a git dependency plus a temp-tree dance on every comparison. The epoch is a fraction
of the work and also covers re-baselines that have nothing to do with a version bump. The two
compose: the version stamp catches the case nobody foresaw, the epoch is the deliberate answer to
it.

**What this replaces downstream.** `trck` works around the problem today with a script that
`git archive`s the base revision into a temp directory, runs `ratchet generate` over it with the
ratchet from the current job, and then compares via `--base-file` (#dd9gu4d, which is what makes the
workaround possible). It is correct across every future version bump, but it costs `fetch-depth: 0`
on the checkout and a second full `generate` per CI run. An epoch would let both go away.

Related: #swshs5v and #tj4qdt9 are the other threads on test-code counting, which is the change that
surfaced this.
