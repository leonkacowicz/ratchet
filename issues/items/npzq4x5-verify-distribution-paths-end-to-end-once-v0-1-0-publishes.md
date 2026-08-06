# Verify distribution paths end-to-end once v0.1.0 publishes

## Summary
The distribution pieces were built and unit-verified offline, but their *live* paths
couldn't be exercised because the v0.1.0 release is still blocked on GitHub's macOS-Intel
runner queue (no assets published yet). Once the release actually publishes, confirm the
real paths work.

## Acceptance criteria
- [x] `release.yml` produced all four archives + `.sha256` files, and each checksum verifies
      (`sha256sum -c` / `shasum -a 256 -c`) — all 8 assets present, 4/4 checksums OK
- [x] Each prebuilt binary runs (`ratchet --version`) on its platform — Linux verified
      directly; macOS + Windows binaries run via the smoke test (`uses: ./` → `ratchet 0.1.0`
      on macos-14 and windows-latest)
- [x] `install.sh` with the default `latest` path downloads, verifies, and installs — Linux
      verified directly; macOS via the smoke test (the action shells out to install.sh on unix)
- [x] The composite action (`uses: leonkacowicz/ratchet@v0.1.0`) installs and PATHs ratchet
      on Linux, macOS, **and Windows** runners — smoke test green on all three (run 30137339000)
- [ ] The cargo-install fallback path works for a target with no prebuilt asset (e.g.
      linux-aarch64) — the smoke test uses the prebuilt path, so this branch isn't forced;
      tracked as a follow-up

## Verification log (2026-07-25, v0.1.0 published after the release.yml Intel-mac fix)
- All 8 release assets present; downloaded and `sha256sum -c` verified all four archives.
- Linux binary runs: `ratchet 0.1.0`, `x86-64 ELF`, dynamically linked.
- `install.sh` (default `latest`) exercised live against the real release: resolved v0.1.0
  via the API, downloaded, verified the checksum (`OK`), installed, and ran — full success.
- Added `.github/workflows/action-smoke-test.yml` (matrix ubuntu/macos-14/windows → `uses: ./`
  → `ratchet --version`). It immediately caught two macOS bugs that Linux + bash 5.x had hidden:
  (1) `api.github.com` 403 rate-limit resolving `latest` on a shared runner IP — fixed by
  resolving via the github.com `/releases/latest` redirect in both install.sh and the action's
  PowerShell step; (2) an empty-bash-array crash under `set -u` on macOS bash 3.2 in the action's
  cargo fallback — fixed by dropping the array. A follow-up pwsh fix (follow the redirect via HEAD
  rather than blocking it with `-MaximumRedirection 0`) got Windows green. Final: all three OSes
  install and run `ratchet 0.1.0` (run 30137339000).
- Residual: the action's **cargo-install fallback** (no prebuilt asset for the target) isn't
  exercised by the smoke test. Its mechanism (`cargo install --git`) was verified end-to-end in
  [[distribution-cargo-install-prebuilt-binaries]], and the branch is now bash-3.2-safe +
  shellcheck-clean, but it isn't run on a live runner — see the follow-up issue.

## Notes
- Offline verification already done: checksum-format compatibility (workflow ↔ installer),
  archive layout, all platform→target mappings, arg parsing, shellcheck-clean bash. See
  [[standalone-install-script-for-prebuilt-binaries]] and
  [[consumer-facing-reusable-github-action-for-the-ratchet-gate]].
- A cheap way to exercise the action on real runners: add a smoke-test job (matrix over
  ubuntu/macos/windows) that does `uses: ./` then `ratchet --version`.
- Depends on `8bqetja` (Release v0.1.0), whose release build is stuck on macOS-Intel runner
  availability — the true gate here is that build finally completing.
