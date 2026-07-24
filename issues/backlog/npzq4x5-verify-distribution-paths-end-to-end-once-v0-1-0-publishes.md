# Verify distribution paths end-to-end once v0.1.0 publishes

## Summary
The distribution pieces were built and unit-verified offline, but their *live* paths
couldn't be exercised because the v0.1.0 release is still blocked on GitHub's macOS-Intel
runner queue (no assets published yet). Once the release actually publishes, confirm the
real paths work.

## Acceptance criteria
- [ ] `release.yml` produced all four archives + `.sha256` files, and each checksum verifies
      (`sha256sum -c` / `shasum -a 256 -c`)
- [ ] Each prebuilt binary runs (`ratchet --version`) on its platform
- [ ] `install.sh` with the default `latest` path downloads, verifies, and installs on Linux
      and macOS
- [ ] The composite action (`uses: leonkacowicz/ratchet@v0.1.0`) installs and PATHs ratchet
      on Linux, macOS, **and Windows** runners — the Windows/PowerShell branch is currently
      hand-reviewed only
- [ ] The cargo-install fallback path works for a target with no prebuilt asset (e.g.
      linux-aarch64)

## Notes
- Offline verification already done: checksum-format compatibility (workflow ↔ installer),
  archive layout, all platform→target mappings, arg parsing, shellcheck-clean bash. See
  [[standalone-install-script-for-prebuilt-binaries]] and
  [[consumer-facing-reusable-github-action-for-the-ratchet-gate]].
- A cheap way to exercise the action on real runners: add a smoke-test job (matrix over
  ubuntu/macos/windows) that does `uses: ./` then `ratchet --version`.
- Depends on `8bqetja` (Release v0.1.0), whose release build is stuck on macOS-Intel runner
  availability — the true gate here is that build finally completing.
