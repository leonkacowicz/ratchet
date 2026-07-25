# Handle uneven metric coverage across languages

## Resolution (2026-07-25): wontfix — superseded by a full-coverage policy

Closed without doing the "not measured ≠ real 0" work. With rca gone, every language is a
native tree-sitter language supplying a full `Rules` set, so every metric is measured for every
supported language — there is no partial-coverage source left to accommodate. The last one,
Go, was re-scoped from an external-tool collector to a native language (`7g5s97m`). The chosen
direction is therefore to *guarantee* complete coverage (no metric-driving `Rules` field left
empty) rather than to *represent* absence in the report. If a future language ever genuinely
can't measure a metric, reopen this — the analysis below still applies.

## Summary

**Re-scoped 2026-07-25: the original rca framing is obsolete.** rca is gone; every metric is
now computed by the shared, language-agnostic algorithms in `src/native/`, driven by
per-language `Rules` *data* (`src/native/rules/`). An audit of the current rule sets (Rust,
C/C++, Java, the JS family = JS/TS/TSX, Python) shows every one supplies real, non-empty
values for every metric-driving field — function kinds (counts), `decision_kinds`
(cyclomatic), `cog_nesting_kinds`/`cog_flat_kinds` (cognitive), `non_arg_kinds` +
`params_via_declarator` (nargs). So the original claim ("cognitive/cyclomatic for a subset
only, real `nargs` only for C/C++") no longer holds: **coverage is uniform and complete
across all 7 currently-supported languages, and there is no unmeasured language×metric pair
today.**

The *risk* the issue guards against is still real, but it is now **forward-looking** rather
than a present defect. It re-enters the moment a language arrives that does **not** go through
the native path with a full `Rules` set:

- **Kotlin** (`b3bbdge`, which this issue blocks) — a new native language whose rules may not
  cover every metric on day one.
- **Go via an external-tool collector** (`7g5s97m`) — a *different* `Collector` that emits
  whatever the external tool produces, which may be a strict subset of the five metrics.

Today an unsupported language already fails safe at the metric layer: `parse_with_rules`
returns `None` and the metric functions return `None`/empty (see the
`test_metrics_are_empty_for_a_language_without_a_grammar` test). The open question is what the
**report + ratchet layer** does when a collector legitimately *omits a category for a file it
does handle* — must guarantee that a missing measurement is never silently read as `0 excess`
(which the ratchet would treat as "perfectly simple" and let pass).

So the job is: (1) lock in and document that coverage is currently complete, and (2) build the
"not measured ≠ real 0" guarantee at the `Collector` → `Report` boundary **before** a
partial-coverage collector or language lands, so it can never be gamed or falsely satisfied.

## Acceptance criteria
- [ ] The current per-language × per-metric coverage matrix is documented (all-green today for
      the 7 native languages), somewhere kept honest as languages/collectors are added.
- [ ] Define how a metric that a collector does *not* measure for a given file/language is
      represented in the report — omitted or explicitly flagged — **distinctly from a real
      measured 0**, so the ratchet never treats "unmeasured" as passing.
- [ ] The ratchet comparison rules honour that representation: an unmeasured category can never
      register `0 excess` and thus silently satisfy the gate.
- [ ] A test enforces the guarantee (a collector reporting partial coverage does not read as a
      clean 0), so it survives the addition of Kotlin / the Go external-tool collector.

## Notes

- Metric engine: `src/native/` — `analysis.rs` (per-function walk, honours
  `params_via_declarator` at line ~130), `metrics.rs` (file-level SLOC / function count /
  nargs), `rules/*.rs` (per-language node-kind data). Report/ratchet layer: `src/report.rs`,
  `src/ratchet.rs`; the `Collector` seam is `src/collectors/mod.rs`.
- Aside (separate concern, not this issue): `metrics.rs::nargs_of` reads
  `child_by_field_name("parameters")` directly and ignores `params_via_declarator`, whereas
  `analysis.rs` honours it. Worth confirming which path the structural collector actually uses
  and whether the `metrics.rs` copy is dead/legacy; file its own issue if it's a live bug.
- This issue **blocks Kotlin (`b3bbdge`)** and is a sibling of the **Go external-tool
  collector (`7g5s97m`)** — those are the concrete consumers of the guarantee built here, and
  the natural forcing functions for the "not measured" representation.
