# compare: accept a file path baseline, not just a git ref

## Summary

Today `ratchet compare --base <ref>` only knows how to read the baseline report out of
git: it does `git show <ref>:./quality-report.json` (see `read_report_at_ref` in
`src/main.rs`). There is no way to point `compare` at a plain file on disk.

Let the user pass a filesystem path to a `quality-report.json` to compare against, instead
of (or in addition to) a git ref. This is useful when the baseline isn't in git history at
the expected path — e.g. a report produced by a previous CI job and stashed as a build
artifact, a report downloaded from a release, a hand-crafted baseline, or comparing two
generated reports outside any repo.

## Acceptance criteria
- [ ] `ratchet compare` accepts a baseline that is a filesystem path to a `quality-report.json`.
- [ ] Decide and document the argument surface (e.g. `--base-file <path>` alongside the
      existing `--base <ref>`, or teach `--base` to detect a path vs. a ref). Only one of
      ref/file may be supplied at a time.
- [ ] When given a file path, the baseline is read directly from that file (no git
      invocation), then run through the same `ratchet::check` comparison and the same
      threshold-mismatch guard as the git-ref path.
- [ ] A missing baseline file is a clear error (not the silent bootstrap-skip that a missing
      committed report triggers) — the user explicitly named a file, so a typo shouldn't pass.
- [ ] Happy-path + error tests, mirroring the existing `read_report_at_ref` tests.
- [ ] `--help`, `CLAUDE.md` command list, and README/usage docs updated.

## Notes

- Current baseline read lives in `src/main.rs`: `cmd_compare` → `read_report_at_ref(root, base)`,
  which builds `format!("{base}:./{REPORT_FILE}")` and shells out to git. A file-path mode
  would branch before that and `Report::read`/parse the file directly.
- Interaction with `--root`: the git path just fixed in ww4ye7a resolves the baseline
  relative to `<root>`. A file-path baseline is an absolute/relative path the user gives
  directly, so `--root` should not be joined onto it — worth stating explicitly to avoid
  re-introducing a root-relative surprise.
- Open question for whoever picks this up: single flag with detection, or a separate flag.
  A separate `--base-file` is unambiguous and avoids guessing whether `foo/bar.json` is a
  ref or a path; detection is fewer flags but riskier. Lean toward the explicit flag.
