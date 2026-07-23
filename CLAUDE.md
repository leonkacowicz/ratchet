# ratchet — CLAUDE.md

Developer guide for Claude Code working on this repo. Instructions here override default
behavior.

---

## What ratchet is

A language-agnostic **code-quality ratchet**. It snapshots structural metrics of a
codebase into a committed `quality-report.json`, and in CI it blocks any change that makes
the code worse along those metrics while always allowing improvement. The point is a
one-way ratchet: pre-existing debt is grandfathered but may only ever shrink.

It was extracted from the `xtask` quality gate of **sqltgen** (a separate, public project);
this repo is the standalone, generalized home for that tool. It is **public** at
`github.com/leonkacowicz/ratchet`, so treat everything here as world-readable — including
this file and the `issues/` tracker.

---

## Architecture

```
source files → collectors/ (metrics) → report (excess vs threshold) → ratchet (baseline compare)
```

| Path | Responsibility |
|---|---|
| `src/main.rs` | CLI entry point and the `generate` / `check` / `compare` / `dump` verbs |
| `src/config.rs` | `Config` — the optional `ratchet.json` (source roots + include/exclude globs) |
| `src/sources.rs` | `Sources` — compiles the config into glob sets and discovers `(path, Language)` pairs |
| `src/language.rs` | `Language` — extension→parser dispatch (Rust / C/C++ / Python / Java / JavaScript / TypeScript / TSX) |
| `src/collectors/mod.rs` | `Collector` trait — extracts a `category → entity → excess` map from one source |
| `src/collectors/structural.rs` | The only collector today: per-file metrics via `rust-code-analysis` (tree-sitter based) plus directory-level aggregation |
| `src/report.rs` | `Report` type, thresholds, per-category totals, deterministic JSON |
| `src/ratchet.rs` | The two ratchet rules comparing a baseline report to the current one |

**Metric tiers.** tree-sitter parses; `rust-code-analysis` computes the metrics
(SLOC / cognitive / cyclomatic / nargs / function counts) into a `FuncSpace` tree; this
crate turns those numbers into policy (thresholds, report format, ratchet). Only the last
tier is opinionated and ours.

**The ratchet rules** (per category, in `ratchet.rs`): (1) no existing entity may worsen;
(2) no category total may grow. Together these make splitting a fat file/function *pass*
(total drops), renames neutral, and categories ratchet independently.

**Current scope.** Rust, C/C++, Python, Java, JavaScript, TypeScript, and TSX — the
languages `rust-code-analysis` measures with full metric coverage. Files are dispatched to
a parser by extension (`src/language.rs`, mirroring rca's own extension routing); source
roots and include/exclude globs come from an optional `ratchet.json` (`src/config.rs` →
`src/sources.rs`), defaulting to scanning `src`. `#[cfg(test)]` stripping applies to Rust
only. Runnable per-language examples live in `tests/fixtures/`. Further languages (Kotlin,
Go via an external tool), thresholds-in-config, and organizational metrics are the
roadmap — see issue tracking below.

---

## Commands

```sh
ratchet generate --root .              # write quality-report.json (commit the result)
ratchet check --root .                 # CI: fail if the committed report is stale
ratchet compare --root . --base origin/main   # CI: fail on regression vs a baseline ref
ratchet dump src/foo.rs                # debug: dump the parsed FuncSpace tree
```

`--root` defaults to the current directory. ratchet dogfoods itself: the committed
`quality-report.json` is its own baseline, so run `ratchet generate` and commit the result
after any change that shifts the metrics.

---

## Code style

- Run `cargo fmt` after every change (`rustfmt.toml` at the repo root; wide layout).
- Keep `cargo clippy -- -D warnings` clean.
- Every `pub` item gets a `///` doc comment.
- Every function gets at least a happy-path test; tests live in a `#[cfg(test)]` module at
  the bottom of each file (`test_<subject>_<scenario>` naming).
- Function-size guidance: <=50 lines fine, 51-75 consider splitting, >100 always split.
- Adapter/core spirit carries over: the `Collector` trait is the seam for new metric
  sources — add a collector rather than branching inside `structural.rs`.

---

## Issue tracking — trck

All work is tracked with **trck**, a single-file in-repo issue tracker. State lives under
`issues/`: one markdown file per issue plus a generated `index.jsonl` and `SUMMARY.md`.
There is **no** GitHub issues usage — trck is the source of truth.

- Invoke the **trck skill** at the start of any task in this repo; use `trck --help` /
  `trck <verb> -h` for specifics.
- Common verbs: `trck list` (roadmap tree), `trck ready` / `trck next` (what to pick up),
  `trck show ID`, `trck new`, `trck start ID`, `trck done ID`, `trck set`, `trck dep`,
  `trck check`.
- **Only ever hand-edit an issue's body prose** (Summary / Acceptance criteria / Notes).
  `index.jsonl` and `SUMMARY.md` are generated — never hand-edit them; change metadata
  (status, priority, kind, parent, deps) through the verbs.
- **Never delete an issue file** — close it with `trck done ID --resolution …`.
- Keep the tracker honest: `start` what you begin, `done` what you finish, and capture any
  new "this needs doing" as an issue immediately.
- A **pre-commit hook** runs `trck check` whenever `issues/` is staged and aborts the
  commit if the tracker is inconsistent. (It fires via the global `core.hooksPath`
  delegator, so `trck check` must pass before an issues/ commit succeeds.)

### Roadmap shape

Epics (see `trck list`): **Configuration file support** (foundation), **Multi-language
support** (extension dispatch → enable the rca languages → uneven-coverage handling →
Kotlin → Go via an external-tool collector), **Organizational & structural metrics**
(functions/lines/types per module, pub-API surface, test ratio), **Module relationship &
dependency metrics** (import graph → coupling / cycles / layering; long-horizon),
**Packaging, CI & developer experience**, and a **Release v0.1.0** milestone gated by
dependency edges on the must-ship issues.

---

## Commits

- Scope git operations to this repo.
- Never put AI attribution in commit messages (no `Co-Authored-By: Claude`, no "Generated
  with" lines).
- Keep tracker changes (moved issue file + `index.jsonl` + `SUMMARY.md`) as their own
  commit, separate from code changes, where reasonable.
