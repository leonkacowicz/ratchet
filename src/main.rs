mod collectors;
mod config;
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
#[command(name = "ratchet", about = "Snapshot structural code metrics and block quality regressions in CI.")]
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
    /// Compare the committed quality-report.json against a baseline git ref.
    Compare {
        /// Git ref to compare against (e.g. origin/main).
        #[arg(long, default_value = "origin/main")]
        base: String,
    },
    /// Debug: dump the FuncSpace tree for one file.
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
        Cmd::Compare { base } => cmd_compare(root, &base),
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

/// Fail if the committed report regresses against the baseline git ref.
fn cmd_compare(root: &Path, base: &str) -> Result<()> {
    let current = read_committed(root)?;
    let Some(baseline) = read_report_at_ref(root, base)? else {
        eprintln!("warning: no {REPORT_FILE} at {base} — bootstrap mode, ratchet skipped");
        return Ok(());
    };
    if baseline.thresholds != current.thresholds {
        bail!(
            "thresholds differ between {base} and HEAD; threshold edits must \
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

fn read_report_at_ref(root: &Path, base: &str) -> Result<Option<Report>> {
    let spec = format!("{base}:{REPORT_FILE}");
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
