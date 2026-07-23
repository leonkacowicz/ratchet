use anyhow::{bail, Context, Result};
use serde::Deserialize;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::report::default_thresholds;

/// Name of the config file discovered at the project root.
pub const CONFIG_FILE: &str = "ratchet.json";

/// User-facing configuration, deserialized from `ratchet.json`.
///
/// Every field defaults, so an absent or partial file still yields a usable
/// config. The defaults reproduce the pre-config behaviour: scan `src` for
/// every supported language with no glob filtering.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Config {
    /// Source roots to scan, relative to the project root.
    pub sources: Vec<PathBuf>,
    /// Include globs matched against root-relative paths. Empty means include
    /// every supported file.
    pub include: Vec<String>,
    /// Exclude globs matched against root-relative paths. Empty means exclude
    /// nothing.
    pub exclude: Vec<String>,
    /// Per-category metric threshold overrides. Categories omitted here keep
    /// their built-in default (see [`default_thresholds`]); an unknown category
    /// name is rejected by [`Config::effective_thresholds`].
    pub thresholds: BTreeMap<String, u64>,
}

impl Default for Config {
    fn default() -> Self {
        Self { sources: vec![PathBuf::from("src")], include: Vec::new(), exclude: Vec::new(), thresholds: BTreeMap::new() }
    }
}

impl Config {
    /// Load configuration for `root`.
    ///
    /// When `explicit` (from `--config`) is given, that file is required to
    /// exist. Otherwise `<root>/ratchet.json` is used if present, falling back
    /// to defaults when it is not.
    pub fn load(root: &Path, explicit: Option<&Path>) -> Result<Self> {
        match explicit {
            Some(path) => Self::read(path),
            None => {
                let path = root.join(CONFIG_FILE);
                if path.exists() {
                    Self::read(&path)
                } else {
                    Ok(Self::default())
                }
            },
        }
    }

    fn read(path: &Path) -> Result<Self> {
        let text = std::fs::read_to_string(path).with_context(|| format!("reading config {}", path.display()))?;
        serde_json::from_str(&text).with_context(|| format!("parsing config {}", path.display()))
    }

    /// The effective metric thresholds: the built-in defaults with any
    /// per-category overrides from the config applied on top. Errors if an
    /// override names a category that is not a known metric.
    pub fn effective_thresholds(&self) -> Result<BTreeMap<String, u64>> {
        let mut thresholds = default_thresholds();
        for (category, &value) in &self.thresholds {
            if !thresholds.contains_key(category) {
                bail!("unknown metric category in thresholds: {category}");
            }
            thresholds.insert(category.clone(), value);
        }
        Ok(thresholds)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::report::default_thresholds;

    fn tempdir() -> tempfile::TempDir {
        tempfile::tempdir().unwrap()
    }

    #[test]
    fn test_effective_thresholds_defaults_to_builtin() {
        let config = Config::default();
        assert_eq!(config.effective_thresholds().unwrap(), default_thresholds());
    }

    #[test]
    fn test_effective_thresholds_merges_partial_override() {
        let config = Config { thresholds: [("file_lines".to_string(), 400)].into_iter().collect(), ..Config::default() };
        let effective = config.effective_thresholds().unwrap();
        // Overridden category takes the new value...
        assert_eq!(effective.get("file_lines"), Some(&400));
        // ...while every unspecified category keeps its built-in default.
        assert_eq!(effective.get("function_lines"), default_thresholds().get("function_lines"));
        assert_eq!(effective.len(), default_thresholds().len());
    }

    #[test]
    fn test_effective_thresholds_rejects_unknown_category() {
        let config = Config { thresholds: [("not_a_metric".to_string(), 10)].into_iter().collect(), ..Config::default() };
        assert!(config.effective_thresholds().is_err());
    }

    #[test]
    fn test_load_reads_partial_thresholds() {
        let dir = tempdir();
        std::fs::write(dir.path().join(CONFIG_FILE), r#"{"thresholds":{"file_lines":400}}"#).unwrap();
        let config = Config::load(dir.path(), None).unwrap();
        let effective = config.effective_thresholds().unwrap();
        assert_eq!(effective.get("file_lines"), Some(&400));
        assert_eq!(effective.get("function_lines"), Some(&50));
    }

    #[test]
    fn test_load_returns_defaults_when_no_file() {
        let dir = tempdir();
        let config = Config::load(dir.path(), None).unwrap();
        assert_eq!(config, Config::default());
        assert_eq!(config.sources, vec![PathBuf::from("src")]);
    }

    #[test]
    fn test_load_reads_ratchet_json_from_root() {
        let dir = tempdir();
        std::fs::write(dir.path().join(CONFIG_FILE), r#"{"sources":["lib","app"],"exclude":["**/*.d.ts"]}"#).unwrap();
        let config = Config::load(dir.path(), None).unwrap();
        assert_eq!(config.sources, vec![PathBuf::from("lib"), PathBuf::from("app")]);
        assert_eq!(config.exclude, vec!["**/*.d.ts".to_string()]);
        // Unspecified fields fall back to defaults.
        assert!(config.include.is_empty());
    }

    #[test]
    fn test_load_explicit_path_is_required_to_exist() {
        let dir = tempdir();
        let missing = dir.path().join("nope.json");
        assert!(Config::load(dir.path(), Some(&missing)).is_err());
    }

    #[test]
    fn test_unknown_field_is_rejected() {
        let dir = tempdir();
        std::fs::write(dir.path().join(CONFIG_FILE), r#"{"soruces":["src"]}"#).unwrap();
        assert!(Config::load(dir.path(), None).is_err());
    }

    #[test]
    fn test_empty_object_yields_defaults() {
        let dir = tempdir();
        std::fs::write(dir.path().join(CONFIG_FILE), "{}").unwrap();
        assert_eq!(Config::load(dir.path(), None).unwrap(), Config::default());
    }
}
