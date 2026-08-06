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
- [x] `action.yml` (composite) or a `workflow_call` workflow published from this repo
- [x] Platform-detecting download of the right release asset + checksum verification
      (Linux / macOS / Windows runners)
- [x] Falls back to `cargo install --git --locked` when no matching prebuilt asset exists
- [x] A worked consumer example in the README (`uses:` snippet + how to reference a version)
- [x] Documented consumer bootstrap: run `ratchet generate` and commit `quality-report.json`,
      set `fetch-depth: 0` (or fetch the base ref) so `compare` can resolve the baseline,
      wire `--base origin/<base_ref>`

## Notes
- Depends on `8bqetja` (Release v0.1.0): the download path needs an actual tagged release
  with attached binaries — produced by the release workflow added in
  [[distribution-cargo-install-prebuilt-binaries]].
- Grandfathering of a consumer's pre-existing violations is already handled by ratchet's
  bootstrap mode — no extra work needed there.

## Resolution
Composite "Install ratchet" action at `action.yml` → `uses: leonkacowicz/ratchet@v0.1.0`
(or `@main`). Input `version` (default `latest`), output `version`. Design:
- **Linux/macOS** reuse the repo's `install.sh` (from
  [[standalone-install-script-for-prebuilt-binaries]]) for the checksum-verified download,
  as the note intended.
- **Windows** is handled natively in a PowerShell step (asset is a `.zip`; `install.sh` is
  POSIX-only) — download, `Get-FileHash` verify, `Expand-Archive`, add to `GITHUB_PATH`.
- Both branches fall back to `cargo install --git --locked [--tag <ver>]` (repo derived from
  `GITHUB_ACTION_REPOSITORY`) when no prebuilt asset matches the runner (e.g. linux-aarch64).
- README gained an "Adding the gate to your own repo" section: worked `uses:` workflow +
  the one-time `ratchet generate` / commit-report bootstrap.

Verification: `action.yml` valid YAML; the bash step is shellcheck-clean and its `set -e` /
empty-array-under-`set -u` / `--tag` splitting were checked empirically on bash 5.x (GH
runners match). The PowerShell branch is hand-reviewed only — no `pwsh` locally, and it can
only be exercised on a real Windows runner once the v0.1.0 release publishes (still blocked
on GitHub's macOS-Intel runner queue).
