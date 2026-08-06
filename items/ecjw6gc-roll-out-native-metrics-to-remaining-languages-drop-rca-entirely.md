# Roll out native metrics to remaining languages & drop rca entirely

## Summary
Repeat the proven Rust pattern (vendor grammar + static link → migrate each metric behind
the parity harness → cutover) for the remaining languages — C/C++, Python, Java, JavaScript,
TypeScript, TSX — then remove `rust-code-analysis` from `Cargo.toml` entirely. Completing
this is what makes ratchet's dependency graph crates.io-publishable.

**Decomposed** (the Rust PoC cutover proved the pattern) into one issue per language plus a
final dependency-removal issue. Shared infrastructure already exists — the parity harness,
the `use_native`/`MIGRATED` selector, the `Option<&FuncSpace>` dispatch, and the native
file-level metrics (`rust_file_lines`/`rust_file_functions` generalize by node span/count).
What each language adds: a vendored grammar, a function-space walk (function/closure/method
node kinds + naming), and per-language metric rules where they differ from Rust.

## Children
- **C/C++** ([[native-metrics-for-c-c-vendor-tree-sitter-cpp]]) — one grammar covers both.
- **Python** ([[native-metrics-for-python-vendor-tree-sitter-python]]).
- **Java** ([[native-metrics-for-java-vendor-tree-sitter-java]]).
- **JavaScript** ([[native-metrics-for-javascript-vendor-mozjs-grammar-js-family-rules]]) —
  establishes the shared JS-family metric rules (rca's `js_cognitive!` macro covers JS/TS/TSX).
- **TypeScript** / **TSX** — reuse the JS-family rules, so both depend on JavaScript.
- **Drop rca** ([[drop-the-rust-code-analysis-dependency]]) — depends on all six.

## Acceptance criteria
- [ ] Every language vendored, statically linked, and all metrics migrated with parity verified
- [ ] `rust-code-analysis` removed from `Cargo.toml`; no git dependencies remain
- [ ] crates.io publish unblocked (relates to #m9435x3)

## Notes
- Per-language rules differ: rca has distinct `Cognitive`/`Cyclomatic` impls for Cpp, Python,
  Java, and a shared `js_*` path for JS/TS/TSX. Reproduce each from rca's source and verify via
  the harness, exactly as the Rust metrics were.
- Scanners differ (Python indentation, C++ raw strings, JS regex-vs-divide) — each vendored
  `scanner.{c,cc}` copied verbatim.
- npm/npa ([[public-method-attribute-counts-from-rca-npm-npa-metrics]]) — if adopted before
  this lands, reimplement natively here too (class-based langs).
- Was gated on the Rust cutover ([[remove-rca-from-the-rust-path-all-rust-metrics-native]], done).
