# 06 — Migrate the spatial write composition tests

**What to build:** The second migration batch. Move the tests whose subject is what writes do to
Cells — Cell-wise composition of overlapping writes, competing writers resolved by Position,
complete-fit refusal at a row or Grid edge, and pending operand encodings decoded at consumption —
onto the constructed schedule from `04`. Their assertions do not change.

**Blocked by:** 04 — Build a destination-carrying schedule a test can construct.

**Status:** ready-for-agent

- [ ] Every test in this batch drives execution through the constructed schedule.
- [ ] Every behaviour the batch asserted before it moved is still asserted after.
- [ ] The crate builds and its tests pass; the configured route still exists for the batches that
      have not moved.

## Comments

Split out of the architecture review of the Sequence branch, 2026-09-10.
