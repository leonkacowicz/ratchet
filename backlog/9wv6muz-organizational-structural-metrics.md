# Organizational & structural metrics

## Summary
Add cross-file "logical organization" metrics that aggregate per-file facts by
module/package. `module_files` (files per directory) already proves the pattern — it is
computed outside rca by walking the tree. This epic covers the cheap counting-style
metrics (Tier 1-2 of the design discussion); genuine relationship/graph metrics live in
their own epic.

## Acceptance criteria
- [ ] Functions/lines/types per module
- [ ] Public API surface per module
- [ ] Test-vs-production ratio per module
- [ ] All new categories ratchet like the existing ones (split-tolerant, per-category)
