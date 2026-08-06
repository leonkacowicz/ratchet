# Module relationship & dependency metrics

## Summary
Relationship metrics that need a cross-file dependency graph (Tier 3 of the design
discussion): coupling, cycles, layering. tree-sitter yields the raw import/use tokens per
file, but resolving those names to modules and building the graph is a real analysis pass
that rca does not provide. Long-horizon, low priority.

## Acceptance criteria
- [ ] A module import/dependency graph is extracted
- [ ] Coupling, cycle, and layering checks ratchet like other categories
