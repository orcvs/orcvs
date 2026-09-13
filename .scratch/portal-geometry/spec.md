# Give Portal geometry one owner

**Status:** resolved

## Goal

Concentrate destination-to-Span geometry in the existing Portal module so reads, writes, and
Reservations no longer reconstruct row-fit calculations independently. Execution consumes the
coverage already validated by an admitted write.

## Agreed design

- Portal owns destination resolution and complete, row-confined coverage. Grid retains its
  checked coordinate arithmetic.
- Reads borrow working Source Cells; writes admit complete encodings; Reservations choose the
  width to reserve. These policies remain distinct behind the shared geometry calculation.
- Execution reuses validated SpanWrite coverage when finding affected computations rather than
  rebuilding it from a destination and width.
- A Reservation may extend beyond the actual write. Only written Cells participate in execution's
  activation, suppression, and replacement decisions.
- Preserve existing language behavior and diagnostic wording and precedence, including cell
  operand faults before Portal faults.
- The existing Portal interface is the geometry test seam. Existing Source execution tests remain
  the behavior test seam; no test-only inputs or return values are added to shipped code.

## Tickets and dependencies

1. Centralize Portal fit for reads and writes — no blockers.
2. Reuse admitted write coverage during execution — no blockers.
3. Resolve Reservation coverage through Portal geometry — blocked by 01.

Tickets 01 and 02 can begin immediately. They may touch shared implementation, but neither
requires the other's completed behavior. Ticket 03 reuses the geometry ownership established
by 01. Each ticket must leave its path independently verifiable.

## Behavior to preserve

- Exact-fit reads and writes succeed; a missing destination or a span crossing the row edge is
  refused without wrapping, clamping, or admitting a partial write.
- Empty feedback Cells initialize as Number `00`; invalid content still diagnoses at binding.
- Feedback reads working Source at Turn, including earlier same-Tick writes.
- Ordinary writes leave stale tails untouched.
- A successful nested typed answer survives refusal of its spatial write.
- Scalar-pair and remaining-row Reservations retain their existing meanings.
- A write narrower than its Reservation does not suppress or replace untouched computations.

## Constraints

Respect ADR 0009's Portal ownership, ADR 0034's typed nested results and spatial delivery,
ADR 0036's Reservation semantics, and ADR 0012's feedback binding rules. Do not reparse Source
between Turns or change scheduling policy.

General Grid or Span validation redesign, Parser and Diagnostic construction changes, new Portal
geometry, general Source Read, language changes, and performance claims are outside this effort.
Do not modify or close the completed feedback or Portal-input parent work.

## Verification

Run focused Portal and Source behavior tests for the path changed by each ticket, together with
formatting and scoped Clippy. Use 32 property cases locally where applicable. Exact fit, row
refusal, missing destination, and remaining-row coverage belong at the Portal seam; stale tails,
diagnostic precedence, earlier-Turn reads, and narrower-write behavior remain Source acceptance.
Full final workspace and platform verification, full-case property runs, and benchmark comparisons
are deferred to CI at the user's request.

## Completion

Implemented on `portal-geometry`. See [verification](verification.md) for focused checks and CI deferrals.
