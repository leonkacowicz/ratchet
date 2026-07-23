# Functions-per-module metric

## Summary
Group the per-file function counts already collected (rca `nom`) by directory to get
functions-per-module. Structurally identical to `collect_module_files`, just summing a
different value. The cheapest organizational metric.

## Acceptance criteria
- [ ] New `functions_per_module` category emitted
- [ ] Ratchets like `module_files`
