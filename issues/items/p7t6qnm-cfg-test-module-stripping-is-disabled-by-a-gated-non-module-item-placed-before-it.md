# cfg(test) module stripping is disabled by a gated non-module item placed before it

## Summary
A `#[cfg(test)]`-gated item that is **not** a module and appears **before** the test module makes
ratchet stop excluding the `#[cfg(test)] mod` block from `file_lines` and `file_functions`. The
file's metrics jump to its raw totals, so a file can go from comfortably under threshold to
hundreds of lines over it because of a single added test-only `use`.

Reported against `ratchet 0.1.1` (upstream report: `github.com/leonkacowicz/ratchet/issues/1`).

Two files with identical bodies — the same 20 top-level functions and the same 40-function
`#[cfg(test)] mod tests` — where the second differs only by three extra lines at the top:

```rust
#[cfg(test)]
use std::fmt::Debug as _;
```

| file | raw lines | `file_lines` counted |
|---|---|---|
| `a.rs` | 64 | **21** — test module excluded ✅ |
| `b.rs` | 67 | **67** — test module counted ❌ |

## Narrowing
Same body in every case; only the placement and kind of the extra gated item change.

| case | counted / raw | correct? |
|---|---|---|
| only `#[cfg(test)] mod tests` (baseline) | 21 / 64 | ✅ |
| `#[cfg(test)] use` **before** the module | 67 / 67 | ❌ |
| `#[cfg(test)] use` **after** the module | 21 / 67 | ✅ |
| `#[cfg(test)] fn` **before** the module | 67 / 67 | ❌ |
| two `#[cfg(test)] mod` blocks | 21 / 69 | ✅ |
| module gated `#[cfg(feature = "x")]` | 64 / 64 | ✅ (not a test module) |

So it is not "more than one gated item" and not "gated `use` specifically" — it is **a gated
non-module item positioned before the test module**. `file_functions` is affected identically
(observed on a real file as excess 9 → 18, exactly all-functions versus functions-minus-tests).

## Root cause
`strip_test_modules` in `src/collectors/structural.rs:156` takes the **first** line equal to
`#[cfg(test)]`, checks whether the next non-blank line starts a `mod`, and returns the source
unchanged when it does not — instead of continuing the scan:

```rust
let Some(cfg_idx) = lines.iter().position(|l| l.trim() == "#[cfg(test)]") else { ... };
```

Everything from the matched attribute to EOF is dropped, which is also why several gated modules
and gated items *after* the module behave correctly: the first match is the module.

## Acceptance criteria
- [ ] A `#[cfg(test)]`-gated non-module item before the test module no longer suppresses
      stripping; the scan continues to the next gated item.
- [ ] Every `#[cfg(test)] mod` block in a file is excluded, independently of what other gated
      items exist or where they sit (not only a single trailing one).
- [ ] `#[cfg(feature = "…")]`-gated modules stay counted.
- [ ] Regression tests cover all six narrowing cases above for both `file_lines` and
      `file_functions`.
- [ ] `quality-report.json` regenerated if ratchet's own metrics shift.

## Notes
- The direction is the safe one — it over-counts, so it fails a build rather than hiding debt.
  But the failure is attributed to whichever file was touched and reads as a genuine regression,
  so the natural response is to restructure code that was never the problem.
- Whether gated **non-module** items should also be excluded as test-only code is a separate
  decision; the inconsistency above is the bug. Deciding that belongs with #swshs5v
  (per-language test/generated-code exclusion hook), which would eventually replace this
  line-based scanner with a configurable, parse-tree-driven exclusion.
- Repro script (writes a throwaway project under the current directory, one file per case, with
  `file_lines` thresholded at 5 so every file lands in `violations` and counted = excess + 5):

```python
#!/usr/bin/env python3
import json, pathlib, shutil

SP = pathlib.Path("ratchet-cfgtest-repro")
shutil.rmtree(SP, ignore_errors=True)
(SP / "src").mkdir(parents=True)

CODE = "\n".join(f"fn a_{i}() {{ let _ = {i}; }}" for i in range(20))
TESTS = ("#[cfg(test)]\nmod tests {\n"
         + "\n".join(f"    fn t_{i}() {{ let _ = {i}; }}" for i in range(40))
         + "\n}\n")

CASES = {
    "a_only_the_module": CODE + "\n\n" + TESTS,
    "b_use_before":      "#[cfg(test)]\nuse std::fmt::Debug as _;\n\n" + CODE + "\n\n" + TESTS,
    "c_use_after":       CODE + "\n\n" + TESTS + "\n#[cfg(test)]\nuse std::fmt::Debug as _;\n",
    "d_fn_before":       "#[cfg(test)]\nfn helper() {}\n\n" + CODE + "\n\n" + TESTS,
    "e_two_modules":     CODE + "\n\n" + TESTS + "\n#[cfg(test)]\nmod more {\n    fn u() {}\n}\n",
    "f_not_test":        CODE + "\n\n" + TESTS.replace("#[cfg(test)]", '#[cfg(feature = "x")]', 1),
}
for name, text in CASES.items():
    (SP / "src" / f"{name}.rs").write_text(text)
    print(f"{name}.rs raw={len(text.splitlines())}")

(SP / "ratchet.json").write_text(json.dumps({"thresholds": {
    "file_lines": 5, "file_functions": 500, "function_args": 10,
    "function_cognitive": 500, "function_cyclomatic": 500,
    "function_lines": 500, "module_files": 500}}, indent=1))
```

```console
$ python3 repro.py && ratchet --root ratchet-cfgtest-repro generate
$ python3 -c "
import json
v = json.load(open('ratchet-cfgtest-repro/quality-report.json'))['violations']['file_lines']
for k in sorted(v): print(f'{k}: counted={v[k] + 5}')
"
src/a_only_the_module.rs: counted=21
src/b_use_before.rs: counted=67
src/c_use_after.rs: counted=21
src/d_fn_before.rs: counted=67
src/e_two_modules.rs: counted=21
src/f_not_test.rs: counted=64
```
