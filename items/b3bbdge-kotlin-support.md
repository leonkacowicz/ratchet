# Kotlin support

## Summary
Enable Kotlin. It parses via rca and yields line/function/type counts, but has no
cognitive or cyclomatic impl. Depends on the uneven-coverage handling so the missing
complexity metrics are marked unmeasured rather than zero.

## Acceptance criteria
- [ ] Kotlin files yield line/function/type metrics
- [ ] Complexity metrics reported as unmeasured, not zero
