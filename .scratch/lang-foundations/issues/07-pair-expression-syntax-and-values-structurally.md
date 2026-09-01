# 07 — Pair Expression syntax and values structurally

**What to build:** Store each successfully parsed, evaluable Expression entry's syntax expectation
and real runtime value as one bounded entry, without forcing incomplete or invalid Source analysis
into placeholder values.

**Blocked by:** None — can start immediately.

**Status:** resolved

**Tags:** release/v1

- [x] Adding an Expression entry records its syntax and Atom atomically.
- [x] Capacity failure cannot leave one half of an entry committed.
- [x] Expression length has one unambiguous source and no maximum-of-parallel-lengths fallback.
- [x] Consumers can inspect syntax and values without reconstructing parallel collections.
- [x] Incomplete and invalid Live Edit analysis uses explicit non-value records rather than
      `Atom::Empty`, `Function::Empty`, or half-populated entries.
- [x] Existing valid, incomplete, invalid, and over-capacity parser outcomes remain covered through
      their strict or permissive interface.
- [x] The paired entry remains an internal Language Map representation linked to same-revision
      Positions and Footprints, not a second mutable Source model or public seam.
- [x] The existing parser benchmark workload is run before and after; the exact command and results
      are recorded and measured forced-inlining choices are retained unless the results justify a
      targeted change.

## Comments

**2026-09-01 — implemented**

`Expression` now owns one bounded array of private structural records. Complete records expose
read-only `(Token, Atom)` entries; incomplete and invalid analysis use explicit non-value variants.
The Language Map exposes runtime atoms only when every record is evaluable, so incomplete Source
never reaches the Interpreter. Existing glyph hints and strict parser errors remain intact.

Benchmark command before and after: `mise run bench`.

- Before: `parse` 108 ns/iter (+/- 2); `parse_invalid` 63 ns/iter (+/- 7).
- After: `parse` 108 ns/iter (+/- 19); `parse_invalid` 64 ns/iter (+/- 41).

The result is unchanged within measurement noise. All existing `#[inline(always)]` choices were
retained.
