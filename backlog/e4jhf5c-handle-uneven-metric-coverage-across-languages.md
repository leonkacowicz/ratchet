# Handle uneven metric coverage across languages

## Summary
rca implements cognitive and cyclomatic complexity only for a subset of languages (not
Kotlin) and a real `nargs` only for C/C++. A missing impl returns zero — which a ratchet
would read as "perfectly simple". Represent "not measured" distinctly from a real zero so
an unimplemented metric can never be gamed or falsely satisfied.

## Acceptance criteria
- [ ] Unmeasured metric/language pairs are omitted or explicitly flagged, never recorded as 0 excess
- [ ] A coverage matrix is documented
