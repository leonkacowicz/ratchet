# compare reads baseline from repo root, ignoring --root (per-component always bootstrap-skips)

## Summary
`ratchet compare --root <subdir>` reads the *current* committed report from
`<subdir>/quality-report.json` (correct), but reads the *baseline* report from the repo-root
`quality-report.json` in the base ref — ignoring `--root`. In any monorepo / per-component setup
(one `quality-report.json` per component subdirectory, no report at the repo root), `compare`
therefore never finds a baseline and always prints "bootstrap mode, ratchet skipped". The
regression gate silently has no teeth.

Root cause in `src/main.rs::read_report_at_ref`:

```rust
let spec = format!("{base}:{REPORT_FILE}");            // "HEAD:quality-report.json"
let out = Command::new("git").args(["show", &spec]).current_dir(root).output()...;
```

`git show <ref>:<path>` resolves `<path>` **relative to the repository top-level**, not to the
process cwd, unless the path is prefixed with `./`. So `current_dir(root)` has no effect here and
git looks for a root-level `quality-report.json`. `read_committed` (disk read) uses `root.join(...)`
and is correct; only the git-ref read is wrong.

## Acceptance criteria
- [x] `ratchet compare --root <subdir> --base <ref>` reads the baseline from
      `<ref>:<subdir>/quality-report.json` (respects `--root`), matching where `check`/`generate`
      read/write the report
- [x] A committed per-component baseline is found (no false "bootstrap mode") and regressions are
      detected
- [x] Regression test: a repo with `sub/quality-report.json` (and none at root) where `compare
      --root sub` against a base ref both (a) finds the baseline and (b) fails on a seeded regression

## Resolution
Fixed in `read_report_at_ref` by prefixing the pathspec with `./` — `git show <ref>:./quality-report.json`
with `current_dir(root)` resolves to `<root>/quality-report.json` (as suggested). TDD: added a
`#[cfg(test)]` module to `src/main.rs` — a failing test for the subdir case first, then the fix,
plus regression tests for the root-level and missing-report (bootstrap → `None`) branches.
Verified end-to-end with the issue's minimal repro: `compare --root sub` now finds the baseline
(`ok: ratchet check passed`) and fails on a seeded regression (`function_args total excess grew:
0 → 1`). Shipped in v0.1.1.

## Notes
- **ratchet version:** `ratchet 0.1.0`
- **Suggested fix:** prefix the pathspec with `./` so git resolves it relative to `current_dir(root)`:
  ```rust
  let spec = format!("{base}:./{REPORT_FILE}");
  ```
  (`git show HEAD:./quality-report.json` with `current_dir(root)` → `<root>/quality-report.json`.)
  Alternatively, join the root's repo-relative path into the spec explicitly.
- **Minimal repro:**
  ```sh
  mkdir -p repo/sub && cd repo && git init -q
  # put a ratchet.json + sources under sub/, then:
  ratchet generate --root sub && git add -A && git commit -qm baseline
  ratchet compare --root sub --base HEAD
  # actual:  "warning: no quality-report.json at HEAD — bootstrap mode, ratchet skipped"
  # expected: "ok: ratchet check passed"  (baseline read from HEAD:sub/quality-report.json)
  ```
- **Impact:** blocks per-component adoption in a monorepo — `check` (freshness) works per component,
  but `compare` (the actual regression gate) is inert until this is fixed. Found while wiring a
  per-component ratchet gate into a monorepo CI. Relates to the source-roots work (#9h98w7h) and the
  example CI workflow (#8a7532w), which only exercised a single root-level report.
