# Answer "which Language Unit covers these Cells?" in one query

Status: resolved

## Problem Statement

Three places asked which Language Unit covers some Cells, each scanning every
unit with its own overlap rule:

- `portal.rs`, `occupancy_of`: Snapshot occupancy for Halt's lock and
  `Portal::occupancy`.
- `portal.rs`, `Portal::language_unit`: the pair a Copy, Read or Track reads.
- `tick/execution/working.rs`, `WorkingSource::contact`: a mover's contact,
  the only one that knew which units a Source effect placed or obscured this
  Tick.

Because `language_unit` ignored placements, a Copy reading a mover that had
stepped one Cell into its Input Portal met the Span the Snapshot parsed that
mover at, half-covered it, and diagnosed partial or invalid input, though
working Source held the moved `>>` whole. The same mover entering the pair
from the next row was copied. Reproduced on main @ e3b867e with
`" >> =>  "`: the grid became `"  >>=>  "` with `=> has partial or invalid
input`.

## Solution

`portal::coverage` answers what some Cells meet among units, each given with
its Cells, through a visibility rule: `Empty`, `Complete(unit)` or `Partial`.

- `occupancy_of` asks it over the Snapshot units with every Cell visible.
- `WorkingSource::coverage` asks it over the units placed this Tick, then the
  Snapshot units through the Cells no Source effect obscured, and answers an
  `Occupant` (`Unit` or `Placed`).
- `WorkingSource::contact` and the Copy read (`Portal::language_unit`, handed
  that coverage) both read it, so a placed unit is one unit to either and a
  Snapshot unit a Source effect moved away is met by neither.

The fix follows ADR 0067 ("the pair Track reads is whatever working Source
holds there at its Turn") and ADR 0060 (Snapshot ownership does not reserve a
computation's original Cells after it moves).

## Decisions

- Toby asked for candidate 4 on 2026-10-16. Claude took the mover read as a
  bug, per the ADRs above, since the vertical case already copies.
- A Copy still requires the covering unit to span exactly its pair.

## Out of scope

- Row-indexing the unit scan. No cost claim is made, so no benchmark is owed.
