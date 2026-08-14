//! Reading quality reports back in: the committed report at a project root, a
//! baseline named by path, and a baseline resolved out of a git ref.
//!
//! The three differ in how a *missing* report is treated, which is the whole
//! reason they are separate functions — see each one's docs. Writing reports out
//! belongs to [`crate::report`]; this module only reads.

use anyhow::{Context, Result};
use std::path::Path;
use std::process::Command;

use crate::report::Report;

/// File name of the committed report, at the root of the analyzed project.
pub const REPORT_FILE: &str = "quality-report.json";

/// Read the committed `quality-report.json` at the project root.
pub fn read_committed(root: &Path) -> Result<Report> {
    let path = root.join(REPORT_FILE);
    Report::read_from(&path).with_context(|| format!("reading {}", path.display()))
}

/// Read a baseline report directly from a filesystem path. The path is used as given (not
/// joined onto `--root`), and a missing or unparseable file is an error — the user named it
/// explicitly, so a typo must surface rather than silently bootstrap-skip the gate.
pub fn read_report_from_file(path: &Path) -> Result<Report> {
    Report::read_from(path).with_context(|| format!("reading baseline report {}", path.display()))
}

/// Read the baseline report out of a git ref, or `None` when the ref has no report
/// yet — that is bootstrap mode, not an error, so the very first PR into a repo can
/// pass before a baseline exists.
pub fn read_report_at_ref(root: &Path, base: &str) -> Result<Option<Report>> {
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

    /// The committed report is read from `<root>/quality-report.json`, so `--root` picks the
    /// component in a per-component layout.
    #[test]
    fn test_read_committed_reads_the_report_under_root() {
        let dir = TempDir::new().unwrap();
        let report = Report::new(default_thresholds(), CategoryMap::new());
        report.write_to(&dir.path().join(REPORT_FILE)).unwrap();

        assert_eq!(read_committed(dir.path()).unwrap(), report);
    }
}
