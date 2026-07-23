# ratchet

[![CI](https://github.com/leonkacowicz/ratchet/actions/workflows/ci.yml/badge.svg)](https://github.com/leonkacowicz/ratchet/actions/workflows/ci.yml)

A language-agnostic **code-quality ratchet**. It snapshots structural metrics of a
codebase into a committed `quality-report.json`, and in CI it blocks any change that
makes the code *worse* along those metrics — while always allowing improvement.

The point is not to enforce an absolute quality bar (most real codebases have
pre-existing violations). The point is a **one-way ratchet**: existing debt is
grandfathered, but it may only ever shrink. You can't add a longer function, a more
complex one, or a fatter file than what's already there without paying it down
elsewhere in the same category.

## Metrics

Structural metrics are extracted per file via
[`rust-code-analysis`](https://github.com/mozilla/rust-code-analysis) (tree-sitter based),
plus directory-level aggregation. Each metric has a threshold; the report records only
the **excess over threshold** per entity.

| Category | Entity | Meaning |
|---|---|---|
| `function_lines` | `file::fn` | Source lines per function |
| `function_cognitive` | `file::fn` | Cognitive complexity per function |
| `function_cyclomatic` | `file::fn` | Cyclomatic complexity per function |
| `function_args` | `file::fn` | Argument count per function |
| `file_lines` | `file` | Source lines per file |
| `file_functions` | `file` | Function count per file |
| `module_files` | `dir` | File count per directory |

## The ratchet rules

Comparing a baseline report against the current one, per category:

1. **No existing entity may worsen** — if `file::fn` had excess 12, it may not become 13.
2. **No category total may grow** — the sum of all excesses in a category may not increase.

These two rules together give useful properties for free:

- **Splitting is rewarded.** Breaking an 800-over file into two 300-over files *passes*
  (total dropped 800 → 600).
- **Renaming is neutral.** A rename is treated as remove + add; same size passes.
- **Categories ratchet independently.** You can't offset a worse file against a better
  function.
- **Deletion frees budget.** Removing a violation makes room for a new one of equal or
  smaller size in the same category.

## Usage

```sh
# Snapshot the current codebase (commit the result).
ratchet generate --root .

# CI step 1: fail if the committed report is stale.
ratchet check --root .

# CI step 2: fail if the current report regresses against a baseline ref.
ratchet compare --root . --base origin/main

# Debug: dump the parsed function/space tree for one file.
ratchet dump src/foo.rs
```

`--root` defaults to the current directory. `check` and `compare` read the committed
`quality-report.json`; `compare` reads the baseline via `git show <base>:quality-report.json`
and skips (bootstrap mode) if the baseline has no report yet.

Thresholds may not change in the same commit that adds a violation — a threshold edit
between baseline and HEAD is rejected, forcing it into its own reviewable PR.

## Continuous integration

The [`CI` workflow](.github/workflows/ci.yml) builds, lints and tests the code, then runs
ratchet's quality gate **against ratchet itself** — the gate is the project's own freshly
built binary, not a third-party code-quality service:

1. **Build, lint & test** — `cargo fmt --check`, `cargo clippy -D warnings`,
   `cargo build --release --locked`, `cargo test`. The release binary is passed to the
   next job as an artifact so the (slow) `rust-code-analysis` dependency compiles once.
2. **Code-quality ratchet** — `ratchet check` fails the build if the committed
   `quality-report.json` is stale, and on pull requests `ratchet compare` fails it if any
   metric regresses against the base branch.

It runs on GitHub-hosted runners on every push to `main` and every pull request. To wire
the same gate into another project, drop the two `ratchet` steps into your pipeline after
building the binary (or `cargo install --git https://github.com/leonkacowicz/ratchet`).

## Status

Early standalone extraction. Currently analyzes **Rust** source under `<root>/src/`
(test modules stripped). Multi-language support, a config file, and organizational
metrics are tracked in the issue tracker (`trck`).
