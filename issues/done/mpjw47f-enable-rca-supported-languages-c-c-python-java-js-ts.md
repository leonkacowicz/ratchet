# Enable rca-supported languages (C/C++, Python, Java, JS, TS)

## Summary
Turn on the languages rca already supports with full metric coverage: C/C++, Python,
Java, JavaScript, TypeScript, and TSX. Depends on extension dispatch and configurable
source globs.

## Acceptance criteria
- [x] Each language parses and yields metrics
- [x] Example fixtures per language
- [x] README language matrix updated

## Notes
Implemented in `src/language.rs` alone — the dispatch seam was the only thing that needed
touching. Added `Cpp`, `Python`, `Java`, and `JavaScript` variants (TypeScript/TSX were
already on), routing each to the grammar rca itself uses: `CppParser` for the C/C++
header/impl extensions, `PythonParser`, `JavaParser`, and `MozjsParser` for JavaScript
(`.js`/`.mjs`/`.cjs`/`.jsx`, JSX included). Extension routing mirrors rca's own table.

Fixtures: one runnable example per language under `tests/fixtures/`, exercised by
`test_example_fixtures_parse_and_yield_metrics` (each dispatches to the expected language
and yields ≥1 measured function). CLI smoke-tested via `ratchet dump` on the Python and
Java fixtures. README language matrix + CLAUDE.md scope note updated. No downstream code
changed; the self-report was unaffected (new code stays under all thresholds).
