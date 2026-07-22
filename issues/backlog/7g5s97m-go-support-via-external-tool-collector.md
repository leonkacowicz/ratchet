# Go support via external-tool collector

## Summary
rca has no Go grammar. Add a `Collector` that shells out to `gocyclo`/`gocognit` (plus
own line counting) and maps the results into the same `CategoryMap` — the "Path B"
approach. Cross-language metric comparability is not required because the ratchet only
compares each entity against its own past value.

## Acceptance criteria
- [ ] Go files produce metrics via the external collector
- [ ] Graceful degradation when the tools are not installed
- [ ] Per-entity ratchet verified for Go
