# Verify the action's cargo-install fallback on a runner

## Summary
The composite action falls back to `cargo install --git` when no prebuilt asset matches the
runner. The smoke test ([[verify-distribution-paths-end-to-end-once-v0-1-0-publishes]]) always
hits the prebuilt path, so the fallback branch is never exercised on a live runner.

## Acceptance criteria
- [ ] A CI job forces the fallback (e.g. a target/arch with no prebuilt asset, or a version
      input that has no matching release asset) and confirms `ratchet` still installs, is on
      PATH, and runs

## Notes
- Low risk: the fallback's core mechanism (`cargo install --git --locked [--tag]`) was verified
  end-to-end in [[distribution-cargo-install-prebuilt-binaries]], and the branch is now
  bash-3.2-safe (no arrays) and shellcheck-clean. This issue is only about exercising it live.
- Forcing it cleanly is the tricky part — the action has no "force source build" input. Options:
  a linux-aarch64 runner (no prebuilt asset for that target), or add a hidden/test input, or a
  smoke-test variant pointed at a version tag with assets removed.
