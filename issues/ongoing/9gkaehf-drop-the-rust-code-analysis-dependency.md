# Drop the rust-code-analysis dependency

## Summary
Once every language computes natively, remove `rust-code-analysis` from the project entirely:
delete it from `Cargo.toml`, drop the rca code paths (`Language::parse_metrics`, the rca
branches in `parity`, the `ratchet dump` rca tree dump or reimplement it natively), and remove
the rca-based parity oracle's rca side (or repoint it at a golden corpus).

This is the payoff of the whole epic: **no git dependencies remain**, so ratchet's dependency
graph becomes crates.io-publishable (unblocks #m9435x3).

## Acceptance criteria
- [ ] `rust-code-analysis` removed from `Cargo.toml` and `Cargo.lock`
- [ ] No remaining rca call sites (metric path, `dump`, tests)
- [ ] `cargo build`/`clippy`/`test` green; `ratchet check`/`compare` green
- [ ] No git dependencies in the graph (verify `cargo metadata`); crates.io publish unblocked

## Notes
- The parity harness's rca oracle loses its reference here — either keep a committed golden
  set of expected values, or accept that native is now the sole source of truth.
- `ratchet dump` currently prints rca's FuncSpace tree; reimplement over the native tree or drop it.
- Depends on all six language migrations.
