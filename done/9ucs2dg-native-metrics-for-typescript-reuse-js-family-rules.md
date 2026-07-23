# Native metrics for TypeScript (reuse JS-family rules)

## Summary
Bring TypeScript onto the native path, reusing the JS-family metric rules established by the
JavaScript issue. Mostly grammar-vendoring + verification.

## Acceptance criteria
- [x] Grammar sourced as a pinned crates.io dependency (`tree-sitter-typescript = "=0.23.2"`,
      rca's exact version) rather than vendored — only unpublished forks are vendored
- [x] Native function-space walk at rca parity over the corpus
- [x] All six metrics at rca parity over the corpus
- [x] Cutover: files of this language no longer parse through rca (`top` is `None`)
## Notes
- **Rules fully shared with JavaScript** — rca applies one `js_cognitive!` macro and identical
  space/cyclomatic/non-arg sets to JS, TS and TSX, so `TS_FAMILY` derives from the JS base.
- **One rca bug found, reproduced, then dropped:** TS's naming fallback compares the parent's
  kind id against the *Mozjs* enum; ids 153/236 are `Keyof`/`AssignmentExpression` in TypeScript,
  neither of which has the `name`/`key` field it then reads — so the fallback never fires and
  anonymous functions stay `"<anonymous>"` (unlike JS). Parity was proved against that first;
  ratchet then **stopped reproducing it**, so TS names anonymous functions from their
  `variable_declarator`/`pair` like JavaScript. Pinned by a golden test; file-level metrics keep
  rca parity.
- Corpus: `example_constructs.ts` (every function-space kind, control structure, boolean
  sequence, nesting case, plus TS-only types/generics/enums/interfaces).
- Vendor **tree-sitter-typescript** (typescript parser) at rca's version (`0.23.2`).
- Reuse the JS-family walk + metric rules from
  [[native-metrics-for-javascript-vendor-mozjs-grammar-js-family-rules]]; TS adds type nodes but
  the decision-point/space kinds are the js-family set. Verify parity — TS's grammar has extra
  node kinds that must not perturb the counts.
- Depends on the JavaScript issue.
