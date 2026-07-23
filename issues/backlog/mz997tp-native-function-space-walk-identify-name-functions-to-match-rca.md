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
- [ ] Native walk enumerates the same function spaces rca's `visit_function_spaces` does for
      Rust (functions, methods, nested closures), in the same order
- [ ] Native names match `function_entity_name` output per function (including `Foo::bar`
      qualification and `{closure_N}` synthesis), verified against rca on all Rust fixtures +
      ratchet's own `src/` via the parity harness
- [ ] Exposes a shape the per-function metric migrations can attach values to

## Notes
- This gates the four function-level metric migrations (function_lines, cognitive,
  cyclomatic, args) and the function *count* (nom / file_functions). File-level metrics
  (file_lines) do **not** need it — that is why the harness is proven on file_lines first.
- The hardest fidelity detail is name qualification: rca names methods `Type::method` using
  the enclosing impl/trait. Reproduce rca's exact scheme from its space construction.
- Depends on the scaffold ([[rust-poc-vendor-grammar-static-link-produce-a-raw-tree-sitter-tree]]).
