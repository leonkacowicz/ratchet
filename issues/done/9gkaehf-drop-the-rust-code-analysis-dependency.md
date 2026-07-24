# Drop the rust-code-analysis dependency

## Summary
Once every language computes natively, remove `rust-code-analysis` from the project entirely:
delete it from `Cargo.toml`, drop the rca code paths (`Language::parse_metrics`, the rca
branches in `parity`, the `ratchet dump` rca tree dump or reimplement it natively), and remove
the rca-based parity oracle's rca side (or repoint it at a golden corpus).

This is the payoff of the whole epic: **no git dependencies remain**, so ratchet's dependency
graph becomes crates.io-publishable (unblocks #m9435x3).

## Acceptance criteria
- [x] `rust-code-analysis` removed from `Cargo.toml` and `Cargo.lock`
- [x] No remaining rca call sites (metric path, `dump`, tests) — `grep rust_code_analysis src/` is empty
- [x] `cargo build`/`clippy`/`test` green; `ratchet check`/`compare` green
- [x] **No git dependencies in the graph** — `grep 'source = "git' Cargo.lock` returns nothing;
      crates.io publish unblocked (relates to #m9435x3)

## Notes
- **Done.** `Language::parse_metrics`, the rca metric helpers and the rca space walk are gone;
  `structural.rs` collects straight from a native `Analysis` with no fallback; `parity.rs` — whose
  whole purpose was the dual-path dispatch and the oracle — is deleted.
- **Regression net replaced, not dropped:** a golden characterization test measures every fixture
  and compares against a committed `tests/fixtures/golden.json` (the values rca itself verified,
  plus the deliberate corrections). Re-bless with `RATCHET_BLESS_GOLDEN=1`. The per-language tests
  pinning each divergence from rca survive, grouped as documentation of those decisions.
- **`ratchet dump` reimplemented natively:** it prints the function spaces the native path sees
  with all four per-function metrics, for every supported language.
- Deleting the rca plumbing shrank `structural.rs` below the `file_functions` threshold, so the
  report improved (category 5 → 0, now empty).
- The parity harness's rca oracle loses its reference here — either keep a committed golden
  set of expected values, or accept that native is now the sole source of truth.
- `ratchet dump` currently prints rca's FuncSpace tree; reimplement over the native tree or drop it.
- Depends on all six language migrations.
