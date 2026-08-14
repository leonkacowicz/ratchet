use anyhow::{Context, Result};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::collectors::Collector;
use crate::language::Language;
use crate::native;
use crate::report::CategoryMap;
use crate::sources::Sources;

const CATEGORY_FUNCTION_LINES: &str = "function_lines";
const CATEGORY_FUNCTION_COGNITIVE: &str = "function_cognitive";
const CATEGORY_FUNCTION_CYCLOMATIC: &str = "function_cyclomatic";
const CATEGORY_FUNCTION_ARGS: &str = "function_args";
const CATEGORY_FILE_FUNCTIONS: &str = "file_functions";
const CATEGORY_FILE_LINES: &str = "file_lines";
const CATEGORY_MODULE_FILES: &str = "module_files";

/// Structural metrics collector backed by the native tree-sitter path (`src/native/`).
///
/// Parses every source file the [`Sources`] selector discovers, dispatching to
/// the right grammar per file, and emits per-function and per-file excess
/// values. `module_files` is computed locally by walking directories.
pub struct Structural {
    thresholds: BTreeMap<String, u64>,
}

impl Structural {
    pub fn new(thresholds: BTreeMap<String, u64>) -> Self {
        Self { thresholds }
    }

    fn threshold(&self, category: &str) -> u64 {
        *self.thresholds.get(category).unwrap_or(&u64::MAX)
    }

    fn record(&self, vio: &mut CategoryMap, category: &str, entity: String, value: u64) {
        let threshold = self.threshold(category);
        if value > threshold {
            vio.entry(category.to_string()).or_default().insert(entity, value - threshold);
        }
    }
}

impl Collector for Structural {
    fn name(&self) -> &str {
        "structural"
    }

    fn collect(&self, root: &Path, sources: &Sources) -> Result<CategoryMap> {
        let mut violations: CategoryMap = BTreeMap::new();
        let files = sources.collect(root);

        for (file, lang) in &files {
            let unit = SourceFile { path: file, lang: *lang, rel: relative_path(file, root) };
            self.collect_for_file(&unit, &mut violations)?;
        }

        let paths: Vec<PathBuf> = files.into_iter().map(|(path, _)| path).collect();
        self.collect_module_files(&paths, root, &mut violations);

        Ok(violations)
    }
}

/// One discovered source file: its path, resolved language, and root-relative
/// display path used as the metric entity prefix.
struct SourceFile<'a> {
    path: &'a Path,
    lang: Language,
    rel: String,
}

impl Structural {
    fn collect_for_file(&self, unit: &SourceFile, violations: &mut CategoryMap) -> Result<()> {
        let raw = std::fs::read_to_string(unit.path).with_context(|| format!("reading {}", unit.path.display()))?;
        // Measured exactly as written: no source is transformed away before parsing.
        // Test code is held to the same thresholds as production code.
        let source_bytes = raw.into_bytes();
        // Detect -> dispatch -> parse, once. Every language ratchet measures has a
        // native implementation, so a file that fails to parse is simply skipped.
        let Some(analysis) = native::analyze(unit.lang, &source_bytes) else {
            return Ok(());
        };

        let rel = &unit.rel;
        let (file_lines, file_functions) = (analysis.file_lines(), analysis.file_functions());
        self.record(violations, CATEGORY_FILE_LINES, rel.to_string(), file_lines);
        self.record(violations, CATEGORY_FILE_FUNCTIONS, rel.to_string(), file_functions);

        // Each function-level metric, keyed by the same walk order so entity names
        // line up across categories.
        let function_metrics = [
            (CATEGORY_FUNCTION_LINES, analysis.function_lines()),
            (CATEGORY_FUNCTION_COGNITIVE, analysis.function_cognitive()),
            (CATEGORY_FUNCTION_CYCLOMATIC, analysis.function_cyclomatic()),
            (CATEGORY_FUNCTION_ARGS, analysis.function_nargs()),
        ];
        for (category, values) in function_metrics {
            for (name, value) in values {
                self.record(violations, category, format!("{rel}::{name}"), value);
            }
        }

        Ok(())
    }

    fn collect_module_files(&self, files: &[PathBuf], root: &Path, violations: &mut CategoryMap) {
        let mut counts: BTreeMap<PathBuf, u64> = BTreeMap::new();
        for file in files {
            if let Some(parent) = file.parent() {
                *counts.entry(parent.to_path_buf()).or_insert(0) += 1;
            }
        }
        for (dir, count) in counts {
            let rel = relative_path(&dir, root);
            self.record(violations, CATEGORY_MODULE_FILES, rel, count);
        }
    }
}

fn relative_path(path: &Path, root: &Path) -> String {
    path.strip_prefix(root).unwrap_or(path).to_string_lossy().into_owned()
}

/// Print the function spaces ratchet finds in a single file, with their metrics.
/// Used by the `ratchet dump` debug command to inspect what the native path sees.
pub fn dump_tree(path: &Path) -> Result<()> {
    let Some(lang) = path.extension().and_then(|e| e.to_str()).and_then(Language::from_extension) else {
        println!("(unsupported extension)");
        return Ok(());
    };
    let bytes = std::fs::read(path).with_context(|| format!("reading {}", path.display()))?;
    let Some(analysis) = native::analyze(lang, &bytes) else {
        println!("(could not parse)");
        return Ok(());
    };
    println!("{lang:?}  file_lines={}  file_functions={}", analysis.file_lines(), analysis.file_functions());
    let lines = analysis.function_lines();
    let args = analysis.function_nargs();
    let cyclomatic = analysis.function_cyclomatic();
    let cognitive = analysis.function_cognitive();
    for i in 0..lines.len() {
        let (name, sloc) = &lines[i];
        println!("  {name:<32} lines={sloc:<4} args={:<3} cyclomatic={:<3} cognitive={}", args[i].1, cyclomatic[i].1, cognitive[i].1);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn structural() -> Structural {
        Structural::new(crate::report::default_thresholds())
    }

    #[test]
    fn test_record_skips_values_at_or_below_threshold() {
        let s = structural();
        let mut vio = BTreeMap::new();
        s.record(&mut vio, CATEGORY_FUNCTION_LINES, "x.rs::foo".into(), 50);
        s.record(&mut vio, CATEGORY_FUNCTION_LINES, "x.rs::bar".into(), 51);
        let entries = vio.get(CATEGORY_FUNCTION_LINES).unwrap();
        assert!(!entries.contains_key("x.rs::foo"));
        assert_eq!(entries.get("x.rs::bar"), Some(&1));
    }

    /// Test code counts. A `#[cfg(test)]` module is measured like any other
    /// code, and a `#[cfg(test)] mod NAME;` declaration does not truncate the
    /// file that declares it.
    #[test]
    fn test_collect_for_file_measures_cfg_test_modules() {
        let src = r#"#[cfg(test)]
mod declared;

pub fn real() -> i32 { 1 }

#[cfg(test)]
mod tests {
    #[test]
    fn does_a_thing() {
        assert_eq!(super::real(), 1);
    }
}
"#;
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("x.rs");
        std::fs::write(&path, src).unwrap();

        // Thresholds of zero so every metric lands in the map as excess = value.
        let thresholds = [(CATEGORY_FILE_LINES.to_string(), 0), (CATEGORY_FILE_FUNCTIONS.to_string(), 0)].into_iter().collect();
        let mut violations = CategoryMap::new();
        let unit = SourceFile { path: &path, lang: Language::Rust, rel: "x.rs".to_string() };
        Structural::new(thresholds).collect_for_file(&unit, &mut violations).unwrap();

        // The whole file, not just the production prefix. Truncating at the
        // first `#[cfg(test)]` would have yielded 0 lines and 0 functions.
        assert_eq!(violations[CATEGORY_FILE_LINES].get("x.rs"), Some(&12));
        assert_eq!(violations[CATEGORY_FILE_FUNCTIONS].get("x.rs"), Some(&2));
    }
}
