use anyhow::{Context, Result};
use rust_code_analysis::{FuncSpace, SpaceKind};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::collectors::Collector;
use crate::language::Language;
use crate::parity::{file_level_metrics, function_metric_values, Metric};
use crate::report::CategoryMap;
use crate::sources::Sources;

const CATEGORY_FUNCTION_LINES: &str = "function_lines";
const CATEGORY_FUNCTION_COGNITIVE: &str = "function_cognitive";
const CATEGORY_FUNCTION_CYCLOMATIC: &str = "function_cyclomatic";
const CATEGORY_FUNCTION_ARGS: &str = "function_args";
const CATEGORY_FILE_FUNCTIONS: &str = "file_functions";
const CATEGORY_FILE_LINES: &str = "file_lines";
const CATEGORY_MODULE_FILES: &str = "module_files";

/// Structural metrics collector backed by `rust-code-analysis`.
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
        let source = if unit.lang.strips_rust_test_modules() { strip_test_modules(&raw) } else { raw };
        let source_bytes = source.into_bytes();
        let Some(top) = unit.lang.parse_metrics(source_bytes.clone(), unit.path) else {
            return Ok(());
        };

        let rel = &unit.rel;
        // Migrated metrics compute natively over the raw tree-sitter tree for
        // Rust (identical values to rca, verified by the parity harness); every
        // other metric/language still routes through rca. The dispatch lives in
        // `parity` so this (already `file_functions`-maxed) file stays flat.
        let (file_lines, file_functions) = file_level_metrics(unit.lang, &source_bytes, &top);
        self.record(violations, CATEGORY_FILE_LINES, rel.to_string(), file_lines);
        self.record(violations, CATEGORY_FILE_FUNCTIONS, rel.to_string(), file_functions);

        let mut closure_counter: u32 = 0;
        visit_function_spaces(&top, &mut |space| {
            let entity_name = function_entity_name(space, &mut closure_counter);
            let entity = format!("{rel}::{entity_name}");
            self.record(violations, CATEGORY_FUNCTION_LINES, entity.clone(), sloc_for(space));
            self.record(violations, CATEGORY_FUNCTION_COGNITIVE, entity, cognitive_for(space));
        });
        // Migrated function-level metrics dispatch to the native path (Rust) or
        // rca otherwise; entity names line up with the walk above.
        for (name, value) in function_metric_values(Metric::FunctionCyclomatic, unit.lang, &source_bytes, &top) {
            self.record(violations, CATEGORY_FUNCTION_CYCLOMATIC, format!("{rel}::{name}"), value);
        }
        for (name, value) in function_metric_values(Metric::FunctionArgs, unit.lang, &source_bytes, &top) {
            self.record(violations, CATEGORY_FUNCTION_ARGS, format!("{rel}::{name}"), value);
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

pub(crate) fn sloc_for(space: &FuncSpace) -> u64 {
    space.metrics.loc.sloc().round() as u64
}

fn cognitive_for(space: &FuncSpace) -> u64 {
    space.metrics.cognitive.cognitive_sum().round() as u64
}

pub(crate) fn cyclomatic_for(space: &FuncSpace) -> u64 {
    space.metrics.cyclomatic.cyclomatic_sum().round() as u64
}

pub(crate) fn args_for(space: &FuncSpace) -> u64 {
    let fn_args = space.metrics.nargs.fn_args();
    let closure_args = space.metrics.nargs.closure_args();
    fn_args.max(closure_args).round() as u64
}

pub(crate) fn function_count_for(space: &FuncSpace) -> u64 {
    space.metrics.nom.total().round() as u64
}

/// Print the FuncSpace tree for a single file, dispatching on its extension.
/// Used by the `ratchet dump` debug command to inspect what
/// rust-code-analysis emits.
pub fn dump_tree(path: &Path) -> Result<()> {
    let Some(lang) = path.extension().and_then(|e| e.to_str()).and_then(Language::from_extension) else {
        println!("(unsupported extension)");
        return Ok(());
    };
    let bytes = std::fs::read(path).with_context(|| format!("reading {}", path.display()))?;
    let Some(top) = lang.parse_metrics(bytes, path) else {
        println!("(no metrics)");
        return Ok(());
    };
    fn rec(s: &FuncSpace, depth: usize) {
        println!("{:>3}-{:>3} {:width$}{:?} {:?}", s.start_line, s.end_line, "", s.kind, s.name, width = depth * 2);
        for c in &s.spaces {
            rec(c, depth + 1);
        }
    }
    rec(&top, 0);
    Ok(())
}

/// Truncate a Rust source file at the first trailing `#[cfg(test)] mod
/// NAME { … }` block and return only the production-code prefix.
///
/// Targets the convention of one test module at the bottom of each `src/*.rs`
/// file. Detects the first line that is exactly `#[cfg(test)]` followed (after
/// possible blanks) by a `[pub ]mod` line, and drops everything from the
/// attribute to EOF. Files without that pattern are returned unchanged.
fn strip_test_modules(source: &str) -> String {
    let lines: Vec<&str> = source.lines().collect();
    let Some(cfg_idx) = lines.iter().position(|l| l.trim() == "#[cfg(test)]") else {
        return source.to_string();
    };
    let next_non_blank = lines.iter().skip(cfg_idx + 1).find(|l| !l.trim().is_empty());
    let Some(next) = next_non_blank else {
        return source.to_string();
    };
    let trimmed = next.trim();
    if !trimmed.starts_with("mod ") && !trimmed.starts_with("pub mod ") {
        return source.to_string();
    }
    let mut result = lines[..cfg_idx].join("\n");
    result.push('\n');
    result
}

/// Visit every `Function`-kind space in the tree, including nested ones
/// (closures and methods).
pub(crate) fn visit_function_spaces(top: &FuncSpace, f: &mut impl FnMut(&FuncSpace)) {
    fn recurse(space: &FuncSpace, f: &mut impl FnMut(&FuncSpace)) {
        for child in &space.spaces {
            if child.kind == SpaceKind::Function {
                f(child);
            }
            recurse(child, f);
        }
    }
    recurse(top, f);
}

/// Return a stable per-file entity name for a function space.
///
/// Named functions and methods reuse the name produced by
/// `rust-code-analysis` (e.g. `Foo::bar`). Anonymous closures get a
/// sequential `{closure_NN}` synthesized in source order.
pub(crate) fn function_entity_name(space: &FuncSpace, closure_counter: &mut u32) -> String {
    match &space.name {
        Some(name) if !name.is_empty() => name.clone(),
        _ => {
            let id = *closure_counter;
            *closure_counter += 1;
            format!("{{closure_{id}}}")
        },
    }
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

    #[test]
    fn test_function_entity_name_synthesizes_closure_names() {
        let mut counter = 0;
        let mut empty_name_space = blank_space();
        empty_name_space.name = None;
        assert_eq!(function_entity_name(&empty_name_space, &mut counter), "{closure_0}");
        assert_eq!(function_entity_name(&empty_name_space, &mut counter), "{closure_1}");
    }

    fn blank_space() -> FuncSpace {
        FuncSpace { name: Some("foo".into()), start_line: 1, end_line: 1, kind: SpaceKind::Function, spaces: Vec::new(), metrics: Default::default() }
    }

    #[test]
    fn test_strip_test_modules_removes_trailing_cfg_test_block() {
        let src = r#"pub fn real() -> i32 { 1 }

#[cfg(test)]
mod tests {
    #[test]
    fn does_a_thing() {
        assert_eq!(real(), 1);
    }
}
"#;
        let stripped = strip_test_modules(src);
        assert!(stripped.contains("pub fn real()"));
        assert!(!stripped.contains("does_a_thing"));
        assert!(!stripped.contains("assert_eq"));
        // Stripped output must not include the cfg(test) attribute or any
        // line that followed it.
        assert!(!stripped.contains("#[cfg(test)]"));
    }

    #[test]
    fn test_strip_test_modules_passes_through_files_without_test_block() {
        let src = "pub fn x() {}\n";
        assert_eq!(strip_test_modules(src), src);
    }

    #[test]
    fn test_strip_test_modules_ignores_cfg_test_not_followed_by_mod() {
        let src = "#[cfg(test)]\nfn lone_test_fn() {}\n";
        assert_eq!(strip_test_modules(src), src);
    }
}
