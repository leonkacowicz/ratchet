mod baseline;
mod collectors;
mod config;
#[cfg(test)]
mod golden;
mod language;
mod native;
mod ratchet;
mod report;
mod sources;

use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};
use std::path::{Path, PathBuf};

use crate::baseline::{read_committed, read_report_at_ref, read_report_from_file, REPORT_FILE};
use crate::config::Config;
use crate::report::Report;
use crate::sources::Sources;

#[derive(Parser)]
#[command(name = "ratchet", version, about = "Snapshot structural code metrics and block quality regressions in CI.")]
struct Cli {
    /// Project root to analyze: the directory containing the source tree and
    /// (for `check`/`compare`) the committed `quality-report.json`.
    #[arg(long, default_value = ".", global = true)]
    root: PathBuf,

    /// Path to a config file. Defaults to `<root>/ratchet.json` if present.
    #[arg(long, global = true)]
    config: Option<PathBuf>,

    #[command(subcommand)]
    command: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Generate quality-report.json from the current codebase.
    Generate,
    /// Verify that the committed quality-report.json matches the current codebase.
    Check,
    /// Compare the committed quality-report.json against a baseline git ref or file.
    Compare {
        /// Git ref to compare against (e.g. origin/main).
        #[arg(long, default_value = "origin/main")]
        base: String,
        /// Path to a baseline quality-report.json to compare against, read directly from
        /// disk instead of from a git ref. Mutually exclusive with `--base`; unlike a git
        /// ref, a missing file is an error rather than a bootstrap skip.
        #[arg(long, conflicts_with = "base")]
        base_file: Option<PathBuf>,
    },
    /// Debug: dump the function spaces and metrics ratchet sees in one file.
    Dump { path: PathBuf },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let root = resolve_root(&cli.root)?;
    run(cli.command, &root, cli.config.as_deref())
}

/// Load the config, compile its source selector, and generate a report using
/// the config's effective thresholds.
fn build_report(root: &Path, config: Option<&Path>) -> Result<Report> {
    let cfg = Config::load(root, config)?;
    let sources = Sources::from_config(&cfg)?;
    report::generate(root, &sources, cfg.effective_thresholds()?)
}

/// Resolve the `--root` argument to an absolute path, failing early if it does
/// not exist so later file operations report the real cause.
fn resolve_root(root: &Path) -> Result<PathBuf> {
    std::fs::canonicalize(root).with_context(|| format!("resolving project root {}", root.display()))
}

fn run(cmd: Cmd, root: &Path, config: Option<&Path>) -> Result<()> {
    match cmd {
        Cmd::Generate => cmd_generate(root, config),
        Cmd::Check => cmd_check(root, config),
        Cmd::Compare { base, base_file } => cmd_compare(root, &base, base_file.as_deref()),
        Cmd::Dump { path } => collectors::structural::dump_tree(&path),
    }
}

/// Generate the report and write it to `<root>/quality-report.json`.
fn cmd_generate(root: &Path, config: Option<&Path>) -> Result<()> {
    let report = build_report(root, config)?;
    report.write_to(&root.join(REPORT_FILE))?;
    println!("wrote {}", root.join(REPORT_FILE).display());
    Ok(())
}

/// Fail if the committed report no longer matches the current codebase.
fn cmd_check(root: &Path, config: Option<&Path>) -> Result<()> {
    let actual = build_report(root, config)?;
    let committed = read_committed(root)?;
    if actual != committed {
        bail!(
            "{} is out of date. Run `ratchet generate` and commit the result.\n\n\
             diff (committed → regenerated):\n{}",
            REPORT_FILE,
            pretty_diff(&committed.to_pretty_string(), &actual.to_pretty_string()),
        );
    }
    println!("ok: {} matches the codebase", REPORT_FILE);
    Ok(())
}

/// Fail if the committed report regresses against the baseline. The baseline is read from a
/// file when `base_file` is given, otherwise from the git ref `base`.
fn cmd_compare(root: &Path, base: &str, base_file: Option<&Path>) -> Result<()> {
    let current = read_committed(root)?;
    // A file baseline is used verbatim (never joined onto `--root`) and a missing file is a
    // hard error; only the git-ref path bootstrap-skips when no baseline exists.
    let baseline = match base_file {
        Some(path) => Some(read_report_from_file(path)?),
        None => read_report_at_ref(root, base)?,
    };
    let Some(baseline) = baseline else {
        eprintln!("warning: no {REPORT_FILE} at {base} — bootstrap mode, ratchet skipped");
        return Ok(());
    };
    if baseline.thresholds != current.thresholds {
        let source = match base_file {
            Some(path) => path.display().to_string(),
            None => base.to_string(),
        };
        bail!(
            "thresholds differ between {source} and HEAD; threshold edits must \
             land in their own PR. Revert the threshold change or split the PR."
        );
    }
    let errors = ratchet::check(&baseline, &current);
    if !errors.is_empty() {
        bail!("ratchet violations:\n{}", ratchet::format_errors(&errors));
    }
    println!("ok: ratchet check passed");
    Ok(())
}

/// Render a unified-ish diff of two pretty-printed reports, capped at 60 changed
/// lines so a wholesale regeneration doesn't bury the terminal.
fn pretty_diff(a: &str, b: &str) -> String {
    use std::fmt::Write;
    let mut out = String::new();
    let a_lines: Vec<&str> = a.lines().collect();
    let b_lines: Vec<&str> = b.lines().collect();
    let mut shown = 0;
    let max = a_lines.len().max(b_lines.len());
    for i in 0..max {
        let a_line = a_lines.get(i).copied();
        let b_line = b_lines.get(i).copied();
        if a_line == b_line {
            continue;
        }
        if let Some(a) = a_line {
            let _ = writeln!(out, "  - {a}");
            shown += 1;
        }
        if let Some(b) = b_line {
            let _ = writeln!(out, "  + {b}");
            shown += 1;
        }
        if shown >= 60 {
            let _ = writeln!(out, "  ...");
            break;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Only differing lines are shown, each as a `-`/`+` pair.
    #[test]
    fn test_pretty_diff_shows_only_changed_lines() {
        let diff = pretty_diff("same\nold\ntail\n", "same\nnew\ntail\n");
        assert_eq!(diff, "  - old\n  + new\n");
    }

    /// Identical inputs produce no output at all, so a caller can treat an empty
    /// string as "no difference".
    #[test]
    fn test_pretty_diff_is_empty_for_identical_input() {
        assert!(pretty_diff("a\nb\n", "a\nb\n").is_empty());
    }

    /// A wholesale rewrite is truncated rather than dumped in full.
    #[test]
    fn test_pretty_diff_caps_long_output() {
        let a: String = (0..100).map(|i| format!("a{i}\n")).collect();
        let b: String = (0..100).map(|i| format!("b{i}\n")).collect();
        let diff = pretty_diff(&a, &b);
        assert!(diff.ends_with("  ...\n"), "long diffs must be truncated: {diff}");
        assert_eq!(diff.lines().count(), 61, "60 shown lines plus the ellipsis");
    }
}
