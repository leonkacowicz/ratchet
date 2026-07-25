# Verify distribution paths end-to-end once v0.1.0 publishes

## Summary
The distribution pieces were built and unit-verified offline, but their *live* paths
couldn't be exercised because the v0.1.0 release is still blocked on GitHub's macOS-Intel
runner queue (no assets published yet). Once the release actually publishes, confirm the
real paths work.

## Acceptance criteria
- [x] `release.yml` produced all four archives + `.sha256` files, and each checksum verifies
      (`sha256sum -c` / `shasum -a 256 -c`) — all 8 assets present, 4/4 checksums OK
- [~] Each prebuilt binary runs (`ratchet --version`) on its platform — Linux verified
      (`ratchet 0.1.0`, valid x86-64 ELF); macOS/Windows checksum-verified but not executed
      (no host here)
- [~] `install.sh` with the default `latest` path downloads, verifies, and installs — Linux
      verified end-to-end live (API resolve → download → checksum OK → install → runs); macOS
      not executed here
- [ ] The composite action (`uses: leonkacowicz/ratchet@v0.1.0`) installs and PATHs ratchet
      on Linux, macOS, **and Windows** runners — needs a CI runner; the Windows/PowerShell
      branch is still hand-reviewed only
- [ ] The cargo-install fallback path works for a target with no prebuilt asset (e.g.
      linux-aarch64) — needs a runner

## Verification log (2026-07-25, v0.1.0 published after the release.yml Intel-mac fix)
- All 8 release assets present; downloaded and `sha256sum -c` verified all four archives.
- Linux binary runs: `ratchet 0.1.0`, `x86-64 ELF`, dynamically linked.
- `install.sh` (default `latest`) exercised live against the real release: resolved v0.1.0
  via the API, downloaded, verified the checksum (`OK`), installed, and ran — full success.
- Remaining (macOS/Windows execution, the action on real runners, cargo fallback) can only be
  exercised on CI runners — see the smoke-test-job note below.

## Notes
- Offline verification already done: checksum-format compatibility (workflow ↔ installer),
  archive layout, all platform→target mappings, arg parsing, shellcheck-clean bash. See
  [[standalone-install-script-for-prebuilt-binaries]] and
  [[consumer-facing-reusable-github-action-for-the-ratchet-gate]].
- A cheap way to exercise the action on real runners: add a smoke-test job (matrix over
  ubuntu/macos/windows) that does `uses: ./` then `ratchet --version`.
- Depends on `8bqetja` (Release v0.1.0), whose release build is stuck on macOS-Intel runner
  availability — the true gate here is that build finally completing.
