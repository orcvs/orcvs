# 02 — Use the scheduling projection for lock occupancy

**What to build:** Halt classifies its target and schedule reuse sees the same
Language Unit facts, so a lock rule cannot silently acquire an input omitted
from equality. All schedule construction consumes the shared borrowed projection.
See the accompanying spec for the confirmed design.

**Blocked by:** 01 — Use one borrowed projection for Expression scheduling

**Status:** ready-for-agent

- [ ] The projection includes Language Unit anchor, Span and kind, including
  Comments and units outside scheduled Expressions.
- [ ] Equality and lock construction consume these shared facts, preserving
  the current full Language Unit equality and every existing invalidation rule.
- [ ] Schedule construction, including lock occupancy classification, has no
  unrestricted Language Map access; execution retains the live access it needs.
- [ ] The remaining independently maintained scheduling comparison is removed;
  Expression and Language Unit facts have one interpretation at the schedule seam.
- [ ] Borrowing introduces no stored duplicate Parser structure, projection
  allocation or projection memoization. Row sharing and lazy construction remain.
- [ ] Halt behaviour, Tick Plans, diagnostics, activation and Turn order remain
  unchanged under ADRs 0032, 0034 and 0068.
- [ ] Existing Halt, schedule-reuse and edit-property tests pass; focused
  regressions cover occupancy or invalidation gaps identified during the change.
- [ ] Required crate and dependant gates pass with PROPTEST_CASES=32, and required
  workspace gates pass before an implementation PR. The complete diff is reviewed
  and completion evidence recorded.
