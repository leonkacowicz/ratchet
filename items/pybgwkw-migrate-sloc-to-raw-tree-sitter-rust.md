# Migrate SLOC to raw tree-sitter (Rust)

## Summary
First metric through the harness — chosen first because it is the simplest, so it validates
the parity harness itself as much as the metric. Compute source-lines-of-code per
function/file for Rust natively from the raw tree-sitter tree, matching rca's `loc`/`sloc`.

## Acceptance criteria
- [x] Native SLOC for Rust computed from the raw tree (file *and* function level)
      — file: `native::rust_file_lines`; function: `native::rust_function_lines`
- [x] Parity with rca's sloc on all Rust fixtures + ratchet's own `src/`
      — `test_file_lines_parity_over_repo_corpus` + `test_function_lines_parity_over_repo_corpus`
- [x] Rust SLOC flipped to the native path via the selector in production
      — `FileLines` + `FunctionLines` in `MIGRATED`. Report unchanged, `check`/`compare` green.

## Notes
- **Scope split.** File-level SLOC (`file_lines`) shipped earlier as the harness's proving
  client ([[dual-path-metric-collector-rca-parity-harness]]). This issue delivered
  **function-level SLOC** (`function_lines`) on top of the native walk
  ([[native-function-space-walk-identify-name-functions-to-match-rca]]).
- **rca SLOC rule reproduced.** `loc.sloc()` = the node's row span: for the file *unit*
  `end_row - start_row`; for a function space the non-unit branch `end_row - start_row + 1`.
  Both are the raw physical span of the node (nested closures included, since they're within
  the function node) — no blank/comment subtraction. Verified: a snippet plus the whole repo
  corpus matched first-try, both levels.
