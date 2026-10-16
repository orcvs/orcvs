# Derive the Output Portal Reservation once

Status: resolved

## Problem Statement

Paint and the Tick schedule each derive which Cells a value answer reserves:

- `orcvs/src/source/language_map.rs`, `output_portal_reservations`, walks every
  Expression's leading Function and its nested value Functions and resolves an
  `Output::Static` Portal's Reservation. `SourceRevision::output_portal_highlight`
  paints from it.
- `orcvs/src/source/tick.rs`, `computations`, walks the same Functions and asks
  `PortalAccess::resolve` (`portal.rs`), whose write sites `Lookup::new` turns
  into write Claims through `Portal::at(..).reservation()`.

The two pick the same Functions and resolve the same Portals, but nothing makes
them agree. The one difference is deliberate and invisible: the schedule also
reserves a Source Bundle's Spans, and paint does not, while `PortalOutput::Writes`
merges the two.

The tests that guard agreement compare the scheduler with
`LanguageMap::output_portal_cells`, a `#[cfg(test)]` line-for-line copy of the
shipped `output_portal_highlight`. They never check the code paint reads.

## Solution

One allocation-free function in `portal.rs` answers the Output Portal
Reservation a Function at an anchor reserves, root or nested: the Cell pair a
value answer's `Output::Static` Portal names, and none for every other Output,
for a nested Function that answers no value, or where the row edge leaves no
room for a pair. `PortalAccess::resolve`'s Static arm resolves its write site
through it, and the paint highlight calls it directly.

One walk over the scheduling projection's Expressions yields each Function
with its anchor and whether it is nested. `computations` and the paint
highlight both use it, so paint highlights only Functions the schedule
computes, and reads no fact the projection does not compare.

## Decisions (Toby, 2026-10-16)

- The change is strictly behaviour-preserving. The console paints exactly what
  it paints today, so a Source Bundle's Spans stay out of the highlight.
- The shared derivation is one allocation-free function plus a shared walk,
  rather than splitting `PortalAccess` or reading the schedule's `Lookup`.
  Paint's walk keeps its allocation bound
  (`output_portal_reservations_allocate_the_same_blocks_for_every_shape`).
- Paint walks `ScheduleInputs::expressions()`, the borrowed projection.
- `output_portal_cells` is deleted. Its tests read the shipped
  `output_portal_highlight`. No property test is added: agreement holds because
  both callers run the same code.
- CONTEXT.md's Reservation entry defines the Output Portal Reservation.
- Two tickets, delivered on one PR with this spec.

## Testing Decisions

- Ticket 01 changes no test. Every Tick, paint, highlight, render-frame and
  schedule-reuse test passes unchanged.
- Ticket 02 moves the tests that read `output_portal_cells` onto the shipped
  highlight, and replaces the hand-copied scheduler projection loops in
  `tick.rs`'s `output_portal_exclusion` with focused tests on the shared
  function, including the Source Bundle exclusion.

## Out of Scope

- Painting Source Bundle Spans.
- Declaring Portal selection in the Function table.
- Dynamic Output Portals, whose pair is known only at the Turn.
