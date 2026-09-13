# 01 — Centralize Portal fit for reads and writes

**What to build:** Feedback reads and spatial write admission obtain their complete row-confined
coverage from the Portal module. Callers no longer unpack a destination and independently
calculate whether the requested Cells fit.

**Blocked by:** None — can start immediately.

**Status:** resolved

- [x] The Portal module owns the shared destination-to-Span calculation; Grid remains responsible
      for checked coordinate arithmetic.
- [x] Feedback reads and write admission both use this calculation, removing their duplicated
      caller-side row-fit logic rather than adding a forwarding module.
- [x] Exact fits succeed. Missing destinations and row-crossing requests retain their existing
      refusals, with no clamping, wrapping, partial write, or changed diagnostics.
- [x] Feedback still borrows working Source without allocating a spelling at Turn. Number
      decoding and empty-Cell initialization remain the binding policy rather than geometry.
- [x] Cell operand faults still precede Portal faults; live editing and earlier same-Tick writes
      remain visible to feedback.
- [x] Portal-interface tests cover exact fit, row-edge refusal, and unavailable destinations;
      existing Source acceptance protects stale tails and rejected spatial writes that retain
      successful nested typed answers.
- [x] No general Span validation redesign or test-only shipped interface is introduced.

## Verification

Run focused Portal and feedback Source tests, formatting, and scoped Clippy. Full final
verification remains with CI. This ticket is independently verifiable without Reservation
migration or the execution-coverage change.

## Completion

Implemented on `portal-geometry`. See [verification](../verification.md) for focused checks and CI deferrals.
