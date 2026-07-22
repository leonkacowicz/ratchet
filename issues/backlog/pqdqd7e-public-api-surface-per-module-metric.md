# Public API surface per module metric

## Summary
Count public API items (pub functions/types) per module. Requires reading a visibility
node from the tree, which varies by language. Builds on the generalized space walk.

## Acceptance criteria
- [ ] New `pub_api_per_module` category emitted
- [ ] Visibility detection handled per language
