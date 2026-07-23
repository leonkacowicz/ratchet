# Native metrics for TSX (reuse JS-family rules)

## Summary
Bring TSX onto the native path, reusing the JS-family rules. TSX shares the TypeScript grammar
family with a separate `tsx` parser (JSX enabled).

## Acceptance criteria
- [x] Grammar sourced as a pinned crates.io dependency (`tree-sitter-typescript = "=0.23.2"`,
      rca's exact version) rather than vendored — only unpublished forks are vendored
- [x] Native function-space walk at rca parity over the corpus
- [x] All six metrics at rca parity over the corpus
- [x] Cutover: files of this language no longer parse through rca (`top` is `None`)
## Notes
- Ships in the same crate as TypeScript (`LANGUAGE_TSX`), so no extra dependency.
- **Two rca quirks reproduced:** the naming fallback never fires (ids 153/236 are
  `DASHQMARKCOLON`/`ClassHeritage` in the TSX grammar), and — unique to TSX —
  `TsxCode::is_else_if` tests whether the parent is an `IfStatement` while an `else if`'s parent
  is always an `else_clause`, so rca **never detects an else-if in TSX** and charges each a full
  nesting increment. Encoded as `else_if_parent: Some("if_statement")`, which by construction
  never matches. Caught by `example.tsx` (cognitive 4 vs 2).
- Corpus: `example_constructs.tsx` (JS-family constructs plus JSX and TS types together).
- Vendor the **tsx** parser from `tree-sitter-typescript` (rca's `0.23.2`).
- Same JS-family walk + rules as TypeScript; JSX elements add node kinds — verify they do not
  affect metric counts.
- Depends on the JavaScript issue.
