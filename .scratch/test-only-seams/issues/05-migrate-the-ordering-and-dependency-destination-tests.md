# 05 — Migrate the ordering and dependency destination tests

**What to build:** The first migration batch. Move the tests whose subject is dependency ordering —
which producer takes its Turn before which consumer, what a cycle rejects, what an inactive owner
suppresses, and what a failed producer leaves standing for its spatial consumers — onto the
constructed schedule from `04`. Their assertions do not change.

Batch boundaries are sized to keep the tree green between tickets rather than drawn from any rule;
an implementer may move the line between `05`, `06` and `07` as long as each lands green on its own.

**Blocked by:** 04 — Build a destination-carrying schedule a test can construct.

**Status:** ready-for-agent

- [ ] Every test in this batch drives execution through the constructed schedule.
- [ ] Every behaviour the batch asserted before it moved is still asserted after.
- [ ] The crate builds and its tests pass; the configured route still exists for the batches that
      have not moved.

## Comments

Split out of the architecture review of the Sequence branch, 2026-09-10.

Carry this batch's share of `10` rather than repeating it: these tests change route here, and the
state record `10` introduces is what they read once they have. Changing them twice is the cost of
sequencing the two apart.
