//! Compiles the vendored tree-sitter grammars (see `vendor/`) into ratchet and
//! statically links them, exposing each grammar's `tree_sitter_<lang>()` entry
//! point to the `native` module. No grammar wrapper crates are involved.

use std::path::Path;

fn main() {
    compile_grammar("tree-sitter-rust", &["parser.c", "scanner.c"]);
}

/// Compile one vendored grammar's C sources into a static archive linked into the
/// binary. `dir` is the grammar's directory under `vendor/`; `files` are its C
/// sources relative to that directory.
fn compile_grammar(dir: &str, files: &[&str]) {
    let root = Path::new("vendor").join(dir);
    let mut build = cc::Build::new();
    build.include(&root);
    // Generated parser tables trip a lot of pedantic C warnings; silence them so
    // ratchet's own build stays clean without touching upstream sources.
    build.warnings(false);
    for file in files {
        let path = root.join(file);
        println!("cargo:rerun-if-changed={}", path.display());
        build.file(path);
    }
    build.compile(dir);
}
