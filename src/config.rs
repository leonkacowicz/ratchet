use anyhow::{Context, Result};
use serde::Deserialize;
use std::path::{Path, PathBuf};

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
}

impl Default for Config {
    fn default() -> Self {
        Self { sources: vec![PathBuf::from("src")], include: Vec::new(), exclude: Vec::new() }
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
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tempdir() -> tempfile::TempDir {
        tempfile::tempdir().unwrap()
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
