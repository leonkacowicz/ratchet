# Count types per module (generalize space walk)

## Summary
`visit_function_spaces` currently filters to `SpaceKind::Function`. Generalize it to also
visit `Struct`/`Enum`/`Trait`/`Class`/`Impl` spaces so a types-per-module metric can be
computed. The parse tree already distinguishes these kinds.

## Acceptance criteria
- [ ] Space walk generalized without changing existing function metrics
- [ ] New `types_per_module` category emitted
