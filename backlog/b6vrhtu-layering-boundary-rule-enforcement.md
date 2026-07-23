# Layering / boundary rule enforcement

## Summary
Let users declare layering rules (e.g. "module A must not import module B") in config and
enforce them against the dependency graph.

## Acceptance criteria
- [ ] Rules expressible in config
- [ ] Violations block the ratchet
