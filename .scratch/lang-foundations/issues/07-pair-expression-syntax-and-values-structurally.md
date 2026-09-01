# 07 — Pair Expression syntax and values structurally

**What to build:** Store each successfully parsed, evaluable Expression entry's syntax expectation
and real runtime value as one bounded entry, without forcing incomplete or invalid Source analysis
into placeholder values.

**Blocked by:** None — can start immediately.

**Status:** ready-for-agent

**Tags:** release/v1

- [ ] Adding an Expression entry records its syntax and Atom atomically.
- [ ] Capacity failure cannot leave one half of an entry committed.
- [ ] Expression length has one unambiguous source and no maximum-of-parallel-lengths fallback.
- [ ] Consumers can inspect syntax and values without reconstructing parallel collections.
- [ ] Incomplete and invalid Live Edit analysis uses explicit non-value records rather than
      `Atom::Empty`, `Function::Empty`, or half-populated entries.
- [ ] Existing valid, incomplete, invalid, and over-capacity parser outcomes remain covered through
      their strict or permissive interface.
- [ ] The paired entry remains an internal Language Map representation linked to same-revision
      Positions and Footprints, not a second mutable Source model or public seam.
- [ ] The existing parser benchmark workload is run before and after; the exact command and results
      are recorded and measured forced-inlining choices are retained unless the results justify a
      targeted change.
