# Migrate SLOC to raw tree-sitter (Rust)

## Summary
First metric through the harness — chosen first because it is the simplest, so it validates
the parity harness itself as much as the metric. Compute source-lines-of-code per
function/file for Rust natively from the raw tree-sitter tree, matching rca's `loc`/`sloc`.

## Acceptance criteria
- [ ] Native SLOC for Rust computed from the raw tree
- [ ] Parity with rca's sloc on all Rust fixtures + ratchet's own `src/`
- [ ] Rust SLOC flipped to the native path via the selector; happy-path test

## Notes
- **Scope split.** File-level SLOC (`file_lines`) is delivered as the *proving client* of the
  harness ([[dual-path-metric-collector-rca-parity-harness]]) since it needs no function
  identification. What remains for this issue is **function-level SLOC** (`function_lines`),
  which needs the native function-space walk
  ([[native-function-space-walk-identify-name-functions-to-match-rca]]) — hence the added
  dependency on it.
- Confirm rca's exact SLOC definition (physical vs logical, blank/comment handling) and
  reproduce it — this is what the parity check pins down.
