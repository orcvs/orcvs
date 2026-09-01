# 07 — Pair Expression syntax and values structurally

**What to build:** Store each Expression's syntax expectation and parsed Atom as one bounded entry,
so their lengths and ordering cannot diverge during parsing, inspection, or extraction.

**Blocked by:** 04 — Simplify Parser borrowing and Atom handoff.

**Status:** ready-for-agent

- [ ] Adding an Expression entry records its syntax and Atom atomically.
- [ ] Capacity failure cannot leave one half of an entry committed.
- [ ] Expression length has one unambiguous source and no maximum-of-parallel-lengths fallback.
- [ ] Consumers can inspect syntax and values without reconstructing parallel collections.
- [ ] Existing valid, incomplete, invalid, and over-capacity parser outcomes remain covered.
- [ ] The existing parser benchmark workload is run before and after; the exact command and results
      are recorded and measured forced-inlining choices are retained unless the results justify a
      targeted change.
