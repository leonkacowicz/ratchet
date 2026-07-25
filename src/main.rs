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
use std::process::Command;

use crate::config::Config;
use crate::report::Report;
use crate::sources::Sources;

const REPORT_FILE: &str = "quality-report.json";

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

/// Read the committed `quality-report.json` at the project root.
fn read_committed(root: &Path) -> Result<Report> {
    let path = root.join(REPORT_FILE);
    Report::read_from(&path).with_context(|| format!("reading {}", path.display()))
}

/// Read a baseline report directly from a filesystem path. The path is used as given (not
/// joined onto `--root`), and a missing or unparseable file is an error — the user named it
/// explicitly, so a typo must surface rather than silently bootstrap-skip the gate.
fn read_report_from_file(path: &Path) -> Result<Report> {
    Report::read_from(path).with_context(|| format!("reading baseline report {}", path.display()))
}

fn read_report_at_ref(root: &Path, base: &str) -> Result<Option<Report>> {
    // The `./` prefix makes git resolve the pathspec relative to `current_dir(root)`
    // rather than the repository top-level, so `compare --root <subdir>` reads the
    // baseline from `<ref>:<subdir>/quality-report.json` (matching where `check` and
    // `generate` read/write it) instead of a root-level report that may not exist.
    let spec = format!("{base}:./{REPORT_FILE}");
    let out = Command::new("git").args(["show", &spec]).current_dir(root).output().context("running git show")?;
    if !out.status.success() {
        return Ok(None);
    }
    let report: Report = serde_json::from_slice(&out.stdout).with_context(|| format!("parsing report from `git show {spec}`"))?;
    Ok(Some(report))
}

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
    use crate::report::{default_thresholds, CategoryMap};
    use tempfile::TempDir;

    /// Run a git command in `dir`, with a self-contained identity and hooks skipped
    /// so the test is hermetic.
    fn git(dir: &Path, args: &[&str]) {
        let ok = Command::new("git")
            .args(args)
            .current_dir(dir)
            .env("GIT_AUTHOR_NAME", "t")
            .env("GIT_AUTHOR_EMAIL", "t@example.com")
            .env("GIT_COMMITTER_NAME", "t")
            .env("GIT_COMMITTER_EMAIL", "t@example.com")
            .status()
            .expect("run git")
            .success();
        assert!(ok, "git {args:?} failed");
    }

    /// `compare --root <subdir>` must read the baseline from `<ref>:<subdir>/quality-report.json`,
    /// not the repo-root path — otherwise a per-component gate (a report per subdirectory, none at
    /// the repo root) never finds a baseline and silently bootstrap-skips (bug ww4ye7a).
    #[test]
    fn test_read_report_at_ref_respects_root_subdir() {
        let repo = TempDir::new().unwrap();
        let root = repo.path();
        let sub = root.join("component");
        std::fs::create_dir_all(&sub).unwrap();

        // A committed per-component baseline under the subdir, and NO report at the repo root.
        let baseline = Report::new(default_thresholds(), CategoryMap::new());
        baseline.write_to(&sub.join(REPORT_FILE)).unwrap();

        git(root, &["init", "-q"]);
        git(root, &["add", "-A"]);
        git(root, &["commit", "-q", "--no-verify", "-m", "baseline"]);

        let got = read_report_at_ref(&sub, "HEAD").unwrap();
        assert_eq!(got, Some(baseline), "baseline must be read from <root>/quality-report.json at the ref");
    }

    /// The common case — report at the repo root, `--root` = repo root — still works after the
    /// `./` pathspec change.
    #[test]
    fn test_read_report_at_ref_reads_root_level_report() {
        let repo = TempDir::new().unwrap();
        let root = repo.path();

        let baseline = Report::new(default_thresholds(), CategoryMap::new());
        baseline.write_to(&root.join(REPORT_FILE)).unwrap();

        git(root, &["init", "-q"]);
        git(root, &["add", "-A"]);
        git(root, &["commit", "-q", "--no-verify", "-m", "baseline"]);

        assert_eq!(read_report_at_ref(root, "HEAD").unwrap(), Some(baseline));
    }

    /// No committed report at the ref → `None` (bootstrap mode), not an error.
    #[test]
    fn test_read_report_at_ref_missing_is_none() {
        let repo = TempDir::new().unwrap();
        let root = repo.path();
        std::fs::write(root.join("marker.txt"), "x").unwrap();

        git(root, &["init", "-q"]);
        git(root, &["add", "-A"]);
        git(root, &["commit", "-q", "--no-verify", "-m", "no report"]);

        assert_eq!(read_report_at_ref(root, "HEAD").unwrap(), None);
    }

    /// A file-path baseline is read directly from disk, with no git involved.
    #[test]
    fn test_read_report_from_file_reads_the_named_file() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("baseline.json");
        let baseline = Report::new(default_thresholds(), CategoryMap::new());
        baseline.write_to(&path).unwrap();

        assert_eq!(read_report_from_file(&path).unwrap(), baseline);
    }

    /// Unlike the git-ref path (missing → bootstrap `None`), a missing file baseline is a
    /// hard error: the user named the file explicitly, so a typo must not silently skip the gate.
    #[test]
    fn test_read_report_from_file_missing_is_error() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("does-not-exist.json");

        assert!(read_report_from_file(&path).is_err());
    }
}
