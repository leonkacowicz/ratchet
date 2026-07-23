use anyhow::{Context, Result};
use globset::{Glob, GlobSet, GlobSetBuilder};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use crate::config::Config;
use crate::language::Language;

/// A compiled view of the source-selection config: the roots to walk plus
/// include/exclude glob sets ready to match against root-relative paths.
pub struct Sources {
    roots: Vec<PathBuf>,
    include: GlobSet,
    exclude: GlobSet,
    include_all: bool,
}

impl Sources {
    /// Compile a [`Config`] into matchable glob sets, failing on an invalid
    /// glob pattern.
    pub fn from_config(config: &Config) -> Result<Self> {
        Ok(Self {
            roots: config.sources.clone(),
            include: build_globs(&config.include)?,
            exclude: build_globs(&config.exclude)?,
            include_all: config.include.is_empty(),
        })
    }

    /// Discover every analyzable file under the configured roots, paired with
    /// its language. Results are sorted by path for deterministic reports.
    /// Non-existent roots are skipped.
    pub fn collect(&self, root: &Path) -> Vec<(PathBuf, Language)> {
        let mut found: Vec<(PathBuf, Language)> = Vec::new();
        for source_root in &self.roots {
            self.collect_root(&root.join(source_root), root, &mut found);
        }
        found.sort_by(|a, b| a.0.cmp(&b.0));
        found
    }

    /// Walk a single (absolute) source root, appending every analyzable file.
    /// A non-existent root is a no-op.
    fn collect_root(&self, abs_root: &Path, root: &Path, found: &mut Vec<(PathBuf, Language)>) {
        if !abs_root.exists() {
            return;
        }
        for entry in WalkDir::new(abs_root).into_iter().filter_map(Result::ok) {
            if !entry.file_type().is_file() {
                continue;
            }
            let path = entry.into_path();
            if let Some(lang) = self.language_for(&path, root) {
                found.push((path, lang));
            }
        }
    }

    /// Resolve the language for `path` when its extension is supported and it
    /// passes the include/exclude globs. Globs match the path relative to the
    /// project root.
    fn language_for(&self, path: &Path, root: &Path) -> Option<Language> {
        let lang = path.extension().and_then(|e| e.to_str()).and_then(Language::from_extension)?;
        let rel = path.strip_prefix(root).unwrap_or(path);
        if self.exclude.is_match(rel) {
            return None;
        }
        if !self.include_all && !self.include.is_match(rel) {
            return None;
        }
        Some(lang)
    }
}

/// Build a [`GlobSet`] from raw patterns. An empty input yields a set that
/// matches nothing.
fn build_globs(patterns: &[String]) -> Result<GlobSet> {
    let mut builder = GlobSetBuilder::new();
    for pattern in patterns {
        builder.add(Glob::new(pattern).with_context(|| format!("invalid glob: {pattern}"))?);
    }
    builder.build().context("building glob set")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tempdir() -> tempfile::TempDir {
        tempfile::tempdir().unwrap()
    }

    fn write(root: &Path, rel: &str) {
        let path = root.join(rel);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, "").unwrap();
    }

    fn names(found: &[(PathBuf, Language)], root: &Path) -> Vec<String> {
        found.iter().map(|(p, _)| p.strip_prefix(root).unwrap().to_string_lossy().into_owned()).collect()
    }

    #[test]
    fn test_collect_finds_supported_files_and_skips_others() {
        let dir = tempdir();
        write(dir.path(), "src/a.rs");
        write(dir.path(), "src/b.ts");
        write(dir.path(), "src/c.tsx");
        write(dir.path(), "src/README.md");
        let sources = Sources::from_config(&Config::default()).unwrap();
        let found = sources.collect(dir.path());
        assert_eq!(names(&found, dir.path()), vec!["src/a.rs", "src/b.ts", "src/c.tsx"]);
    }

    #[test]
    fn test_collect_pairs_files_with_language() {
        let dir = tempdir();
        write(dir.path(), "src/a.rs");
        write(dir.path(), "src/b.tsx");
        let sources = Sources::from_config(&Config::default()).unwrap();
        let found = sources.collect(dir.path());
        assert_eq!(found[0].1, Language::Rust);
        assert_eq!(found[1].1, Language::Tsx);
    }

    #[test]
    fn test_exclude_glob_drops_matching_files() {
        let dir = tempdir();
        write(dir.path(), "src/keep.ts");
        write(dir.path(), "src/types.d.ts");
        let config = Config { exclude: vec!["**/*.d.ts".into()], ..Config::default() };
        let sources = Sources::from_config(&config).unwrap();
        assert_eq!(names(&sources.collect(dir.path()), dir.path()), vec!["src/keep.ts"]);
    }

    #[test]
    fn test_include_glob_restricts_to_matches() {
        let dir = tempdir();
        write(dir.path(), "src/a.rs");
        write(dir.path(), "src/b.ts");
        let config = Config { include: vec!["**/*.ts".into()], ..Config::default() };
        let sources = Sources::from_config(&config).unwrap();
        assert_eq!(names(&sources.collect(dir.path()), dir.path()), vec!["src/b.ts"]);
    }

    #[test]
    fn test_multiple_roots_are_scanned() {
        let dir = tempdir();
        write(dir.path(), "lib/a.rs");
        write(dir.path(), "app/b.ts");
        let config = Config { sources: vec!["lib".into(), "app".into()], ..Config::default() };
        let sources = Sources::from_config(&config).unwrap();
        assert_eq!(names(&sources.collect(dir.path()), dir.path()), vec!["app/b.ts", "lib/a.rs"]);
    }

    #[test]
    fn test_missing_root_is_skipped() {
        let dir = tempdir();
        let sources = Sources::from_config(&Config::default()).unwrap();
        assert!(sources.collect(dir.path()).is_empty());
    }

    #[test]
    fn test_invalid_glob_is_an_error() {
        let config = Config { exclude: vec!["[".into()], ..Config::default() };
        assert!(Sources::from_config(&config).is_err());
    }
}
