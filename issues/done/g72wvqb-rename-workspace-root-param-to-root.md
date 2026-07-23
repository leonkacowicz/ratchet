# Rename workspace_root param to root

## Summary
Rename the `workspace_root` parameter on the `Collector` trait and in
`collectors/structural.rs` to `root` — it is the project root, not necessarily a Cargo
workspace. Small cleanup carried over from the extraction.

## Acceptance criteria
- [ ] Parameter renamed across the trait and impls
- [ ] Tests still pass
