# Native function-space walk: identify + name functions to match rca

## Summary
Foundational for all per-function metrics. On the rca side, `structural.rs` relies on
`visit_function_spaces` + `function_entity_name` to (a) find every function/method/closure
in the tree and (b) name it the way rca does (`Foo::bar`, synthesized `{closure_N}`). The
native path needs the same over a raw tree-sitter tree, or per-function metric entities
won't line up with rca's and parity checks become impossible.

Deliver a native walk that yields, for a Rust file, the ordered list of function spaces with
rca-matching names. This is the direct analogue of the rca-organizational-side
[[count-types-per-module-generalize-space-walk]] (#zytfjgw), but for the migration path.

## Acceptance criteria
- [x] Native walk enumerates the same function spaces rca's `visit_function_spaces` does for
      Rust (functions, methods, nested closures), in the same order
      — `native::visit_rust_functions` (`function_item` + `closure_expression`, pre-order DFS)
- [x] Native names match `function_entity_name` output per function, verified against rca on
      all Rust fixtures + ratchet's own `src/` via the parity harness
      — `parity::check_function_walk_parity` + `test_function_walk_parity_over_repo_corpus`
- [x] Exposes a shape the per-function metric migrations can attach values to
      — `visit_rust_functions(tree, source, |name, node|)` hands each function's node to a callback

## Notes
- This gates the four function-level metric migrations (function_lines, cognitive,
  cyclomatic, args) and the function *count* (nom / file_functions). File-level metrics
  (file_lines) do **not** need it — that is why the harness is proven on file_lines first.
- **Naming reality (corrected).** rca does **not** qualify method names — it uses its *default*
  `get_func_space_name`: a node's `name` field text, else `"<anonymous>"`. So methods are bare
  (`new`, not `Structural::new`), the enclosing `impl`/`trait` is a separate space, and the
  `{closure_N}` synthesis in `function_entity_name` is effectively **dead for Rust** (rca always
  supplies `"<anonymous>"`, which is non-empty). Space kinds: `function_item` +
  `closure_expression` → Function; `impl_item`/`trait_item` are containers walked through but not
  emitted. `function_signature_item` (body-less trait method) is **not** a function_item, so it is
  excluded — matching rca. Parity held first-try across the corpus and on trait/module/async/generic
  snippets.
- Implemented off `structural.rs`'s now `pub(crate)` `visit_function_spaces` +
  `function_entity_name` as the rca oracle (single source of truth). The oracle lives in
  `parity.rs`, not `structural.rs`, so it does not add to that file's already-maxed
  `file_functions` count.
- Depends on the scaffold ([[rust-poc-vendor-grammar-static-link-produce-a-raw-tree-sitter-tree]]).
