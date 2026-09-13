# 02 — Reuse admitted write coverage during execution

**What to build:** After a spatial write is admitted, execution uses its validated SpanWrite
coverage to find affected computations. It no longer independently reconstructs that coverage
from a destination and encoding width.

**Blocked by:** None — can start immediately.

**Status:** resolved

- [x] Ordinary value delivery and Source Function delivery both consume coverage from the
      admitted write when determining affected computations.
- [x] The execution relationship lookup no longer accepts a separately supplied destination and
      width that could disagree with the write it is processing.
- [x] Remove destination/width reconstruction and redundant destination bookkeeping where the
      admitted write already supplies the required facts.
- [x] Activation, suppression, and Function Replacement still follow only the actual written
      Cells, not the full Reservation.
- [x] Tests preserve narrower-write non-suppression, row-edge refusals, Source Function behavior,
      and successful nested typed answers whose spatial delivery is refused.
- [x] Reaching an already-executed computation still rejects the Tick effects under the existing
      ordering rule; this work does not change scheduling or re-execute computations.
- [x] Reuse the existing validated write representation without changing general Span validation
      or adding test-only shipped inputs or return values.

## Verification

Run focused Source execution tests for spatial delivery, activation, suppression, and replacement,
plus formatting and scoped Clippy. Full final verification remains with CI. The existing
SpanWrite already holds validated coverage, so this ticket does not depend on ticket 01.

## Completion

Implemented on `portal-geometry`. See [verification](../verification.md) for focused checks and CI deferrals.
