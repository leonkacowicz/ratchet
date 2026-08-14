# Measure test code like production: drop the cfg(test) exemption, add tests to sources

## Summary
Test code is held to the same structural discipline as production code. Tests are where mess is
normally tolerated, and that is exactly backwards: a test has to be readable to explain what is
being tested. If a test file needs too many cases to stay organized, the cases should be grouped;
if grouping still isn't enough, that is a signal the code under test does too much and should be
split. The ratchet should say so rather than look away.

That reverses the previous implicit policy, which was never decided so much as inherited: Rust
inline `#[cfg(test)] mod` blocks were stripped before parsing (`strip_test_modules`), so they were
invisible to every metric, while every other language's tests were measured or not purely by
accident of which directories the source roots happened to cover.

Two changes implement it:
1. Delete `strip_test_modules` and `Language::strips_rust_test_modules` — no source transformation
   before parsing; a file is measured as written.
2. Add `tests` to this repo's `sources`, excluding `tests/fixtures/**` (see Notes).

## Acceptance criteria
- [ ] `strip_test_modules` and `Language::strips_rust_test_modules` are gone; `collect_for_file`
      parses the file as read from disk.
- [ ] A regression test asserts a Rust file with a `#[cfg(test)] mod` block is measured at its
      full size and function count.
- [ ] `ratchet.json` exists at the repo root with `tests` among `sources` and
      `tests/fixtures/**` excluded.
- [ ] README and CLAUDE.md no longer describe test-module stripping.
- [ ] `cargo fmt`, `cargo clippy -- -D warnings`, `cargo test` all clean.
- [ ] `quality-report.json` regenerated and committed; `ratchet check` passes.

## Notes
- **Measured impact of the removal on this repo** (patched binary vs. release binary over a copy of
  `src`, all other inputs identical): two new grandfathered entries, `file_functions` excess 1 for
  `src/main.rs` and `src/collectors/structural.rs`. Nothing else moves — no `file_lines` violation,
  and no test function trips `function_lines`, `function_cognitive`, `function_cyclomatic`, or
  `function_args`. The exemption was buying almost nothing.
- **The exemption was actively hiding production code.** `src/main.rs:3` is
  `#[cfg(test)] mod golden;` — a module *declaration*, not an inline test module — and the stripper
  truncates from the first `#[cfg(test)]` line to EOF, so the whole CLI was measured as 2 lines and
  0 functions instead of 295 and 21. Deleting the stripper fixes that as a side effect.
- **`tests/fixtures/**` stays excluded.** Those files are test *data*, not test code: deliberately
  complex per-language examples whose purpose is to exercise the metric engine. Measuring them
  would grandfather permanent violations and make any fixture edit trip the ratchet. They are also
  the golden test's input, so their shape is pinned by `tests/fixtures/golden.json`.
- After this lands, `tests/` contributes no files here (it holds only fixtures). The root entry is
  intent-declaring: it takes effect the moment a real integration test appears.
- Supersedes #p7t6qnm — that bug is in `strip_test_modules`, which this deletes.
- Leaves open whether the **built-in default** `sources` (`src/config.rs:35`, currently `["src"]`)
  should also gain `tests`. That is a behavior change for every consumer and deserves its own
  decision; tracked separately.
- Also leaves open what becomes of #swshs5v (per-language test/generated-code exclusion hook) —
  tests are out of its remit now, but excluding *generated* code is still a live need.
