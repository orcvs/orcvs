# 05 — Add explicit Number and Note conversions

**What to build:** Add the numeric conversion Functions `.v` and `.^` from ADR 0021. Both have a
fixed result type and are idempotent for that type; neither enables implicit conversion in another
Function.

**Blocked by:** 01, 04

**Status:** ready-for-agent

- [ ] `.v Number` returns the same Number and `.v Note` returns its underlying MIDI Number.
- [ ] `.^ Note` returns the same Note and `.^ Number` converts `00`–`7F` to the corresponding Note.
- [ ] `.^` diagnoses Numbers `80`–`FF` and produces no result.
- [ ] Both Functions extend atom-wise across Sequences and return no partial Sequence if one element fails.
- [ ] Both spellings parse and round-trip through `Display` without colliding with another Language Unit.
- [ ] Arithmetic, time, feedback, random, and Play tests prove that no implicit Number/Note coercion remains.

## Comments

The `.` prefix provides the numeric namespace: `v` lowers a Note to its underlying Number and `^`
raises an in-range Number to a Note. Other Function families may reuse either suffix.
