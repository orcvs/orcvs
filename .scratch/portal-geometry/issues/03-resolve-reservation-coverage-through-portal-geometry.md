# 03 — Resolve Reservation coverage through Portal geometry

**What to build:** Scheduling obtains scalar-pair and remaining-row coverage from Portal-owned
geometry. Reservation policy still chooses how much to reserve, while the Portal module owns
which Cells that choice reaches without leaving the destination's row.

**Blocked by:** 01 — Centralize Portal fit for reads and writes.

**Status:** resolved

- [x] Scalar-pair Reservations use the shared Portal fit calculation and remain refused when
      the complete pair does not fit in the destination row.
- [x] Remaining-row Reservations obtain coverage through the Portal module, including the
      destination Cell through the last Cell of that same row.
- [x] Remove independent row-end and scalar-fit calculations from Reservation callers.
- [x] Reservation policy remains distinct from read and write policy: a Reservation may exceed
      an eventual write, and reserving Cells neither writes nor suppresses them.
- [x] Portal-interface tests cover remaining-row coverage at the first, interior, and last
      destination columns, including a one-column Grid.
- [x] Existing Source tests retain Reservation row confinement and prove a narrower admitted
      write leaves untouched computations standing.
- [x] No scheduling semantics, diagnostics, language behavior, or general Grid/Span validation
      changes are introduced; no test-only shipped interface is added.

## Verification

Run focused Portal geometry and Reservation Source tests, formatting, and scoped Clippy.
Full final verification remains with CI. Reuse ticket 01's geometry owner; ticket 02 is not a
blocker because Reservation geometry does not depend on how execution consumes admitted writes.

## Completion

Implemented on `portal-geometry`. See [verification](../verification.md) for focused checks and CI deferrals.
