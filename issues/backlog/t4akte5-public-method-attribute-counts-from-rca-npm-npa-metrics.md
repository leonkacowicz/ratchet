# Public method/attribute counts from rca npm/npa metrics

## Summary
rust-code-analysis already computes two public-surface metrics that ratchet currently
discards: `npm` (number of public methods) and `npa` (number of public attributes). They
sit on the same `CodeMetrics` struct ratchet already reads for `nom`/`cognitive`/etc.
(`space.metrics.npm`, `space.metrics.npa`), so wiring them into the report is the same
shape of change as any existing category — no new parsing, no per-language visibility
detection, no space-walk changes.

Emit them as new ratchet categories (working names `public_methods` and
`public_attributes`, likely at file and/or directory grain) so a codebase's public surface
ratchets like everything else: it may shrink freely but only grow deliberately.

## Acceptance criteria
- [ ] `public_methods` category emitted from `metrics.npm`
- [ ] `public_attributes` category emitted from `metrics.npa`
- [ ] Default thresholds chosen and documented (README threshold table + config docs)
- [ ] Per-language availability documented (see Notes — npm/npa are class-only)
- [ ] Happy-path tests per collector, plus a fixture asserting non-zero counts on a
      class-based language (Java/C++)

## Notes
- **Availability caveat (important).** In rca, `npm`/`npa` carry an `is_disabled` guard and
  are only computed for class-based languages — effectively **Java and C/C++**. For Rust,
  Python, JS/TS they are disabled and contribute nothing. So this metric is meaningful only
  on the OO subset of ratchet's languages; the report/thresholds must treat "disabled" as
  "no entities emitted", not "zero excess". Verify exact per-language behaviour against the
  pinned rca revision before finalizing.
- **Relationship to [[public-api-surface-per-module-metric]] (#pqdqd7e).** That issue frames
  public-API surface as requiring per-language visibility-node parsing on top of the
  generalized space walk ([[count-types-per-module-generalize-space-walk]], #zytfjgw). For
  the *method/attribute* slice of public surface, rca's npm/npa make that unnecessary — the
  visibility work is already done upstream. This issue delivers that slice cheaply and
  independently (no dependency on #zytfjgw). #pqdqd7e can then narrow to what npm/npa do
  *not* cover (e.g. public **types**), or be re-scoped in light of this. Left unlinked by a
  hard dependency on purpose — they overlap but neither blocks the other.
- Discovered while investigating #m9435x3 (distribution): rca exposes 13 metrics; ratchet
  uses 5. npm/npa were flagged as the unused pair most aligned with the roadmap's
  Organizational-metrics epic.
