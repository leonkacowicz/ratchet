# Migrate cyclomatic complexity to raw tree-sitter (Rust)

## Summary
Compute cyclomatic complexity per function for Rust natively from the raw tree-sitter tree,
matching rca's `cyclomatic`, behind the parity harness. First of the two real complexity
metrics — the node→+1 rules are more involved than the counting metrics but far simpler than
cognitive.

## Acceptance criteria
- [x] Native cyclomatic for Rust computed from the raw tree
      — `native::cyclomatic_of` / `rust_function_cyclomatic`
- [x] Parity with rca's cyclomatic on all Rust fixtures + ratchet's own `src/`
      — `test_function_cyclomatic_parity_over_repo_corpus` (+ a subtree/`&&` snippet test)
- [x] Rust cyclomatic flipped to the native path via the selector in production
      — `FunctionCyclomatic` in `MIGRATED`; recorded via `parity::function_metric_values`.
      Report unchanged, `check`/`compare` green.

## Notes
- **Two subtleties nailed by probing rca:**
  1. `structural.rs` reads `cyclomatic_sum` (a *subtree sum*), not a per-node value. Each space
     carries a base `1`, summed over the function and its nested closures/functions — so a
     function that contains a closure folds in the closure's complexity (probe: `nested` = its
     own 2 + the closure's 2 = 4). Native mirrors this: `1 + nested spaces + decision points`
     over the whole subtree.
  2. rca counts the keyword **tokens** `if`/`for`/`while`/`loop` (enum `If`/`For`/… → `"if"`/…),
     not the expression nodes — so **match guards** (`Some(x) if …`) and `if let` count. Matching
     `if_expression` first under-counted `function_entity_name` by 1 (its match guard); switching
     to the `"if"` token fixed it. Also `match_arm` (both grammar kind-ids stringify to it), the
     `?` operator (`try_expression`), and `&&`/`||`.
- **Refactor:** generalized the per-function dispatch to `function_metric_values(metric, …)`
  (native/rca), now serving both `FunctionArgs` and `FunctionCyclomatic`; `structural.rs` records
  each via its own pass. This scales to the remaining function-level metrics without growth.
- Depends on the harness ([[dual-path-metric-collector-rca-parity-harness]]) and the walk.
