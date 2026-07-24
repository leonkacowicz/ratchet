# Consumer-facing reusable GitHub Action for the ratchet gate

## Summary
Make ratchet drop-in for a third-party repo's CI. Today the only shipped workflow
(`8a7532w` → `.github/workflows/ci.yml`) is *self-hosted*: ratchet builds itself and gates
itself. A consumer repo can't `uses:` any of it — they have to hand-wire binary
acquisition and the check/compare steps. Turn "clone and compile in your CI" into a few
lines: `uses: leonkacowicz/ratchet@v1`.

Ship one (or both) of:
- a **composite action** (`action.yml` at the repo root) that detects the runner OS/arch,
  downloads the matching prebuilt release asset, verifies its `.sha256`, puts `ratchet` on
  `PATH`, and optionally runs `check` / `compare`;
- a **reusable workflow** (`on: workflow_call`) a consumer calls with a couple of inputs
  (base ref, config path).

## Acceptance criteria
- [ ] `action.yml` (composite) or a `workflow_call` workflow published from this repo
- [ ] Platform-detecting download of the right release asset + checksum verification
      (Linux / macOS / Windows runners)
- [ ] Falls back to `cargo install --git --locked` when no matching prebuilt asset exists
- [ ] A worked consumer example in the README (`uses:` snippet + how to reference a version)
- [ ] Documented consumer bootstrap: run `ratchet generate` and commit `quality-report.json`,
      set `fetch-depth: 0` (or fetch the base ref) so `compare` can resolve the baseline,
      wire `--base origin/<base_ref>`

## Notes
- Depends on `8bqetja` (Release v0.1.0): the download path needs an actual tagged release
  with attached binaries — produced by the release workflow added in
  [[distribution-cargo-install-prebuilt-binaries]].
- The platform-detect + download + checksum logic overlaps
  [[standalone-install-script-for-prebuilt-binaries]]; the action can shell out to that
  script rather than duplicating it.
- Grandfathering of a consumer's pre-existing violations is already handled by ratchet's
  bootstrap mode — no extra work needed there.
