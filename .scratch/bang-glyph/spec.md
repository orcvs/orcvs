# A Bang travels as its glyph

Status: resolved

## Problem Statement

Orcvs holds itself to Orca's performative capabilities (ADR 0019), but three Bang behaviours depart from Orca without a recorded reason:

- A Copy, Read, Track, Write or Push carrying `**` relays it: it activates a root at its destination without overwriting it, writes `**` only into empty Cells, and diagnoses at occupied data. Orca's `J`, `X`, `O`, `T` and `P` write `*` over whatever stands at the destination, and that `*` bangs its neighbours.
- A `**` typed into Source does nothing. Orca's typed `*` bangs its neighbours once.
- Equality, Delay and Euclidean write nothing on a Tick they do not bang. Orca's bang ports write `.`, clearing their output Cell.

## Solution

[ADR 0071](../../docs/adr/0071-a-bang-travels-as-its-glyph.md) decides all three:

- A Function that passes a pair through writes `**` over its complete destination, and that write is its Bang output: the roots aligned with the destination activate during the Tick, in dependency order.
- A standalone `**` that the previous Tick did not write as display fires once at the start of the Tick and is cleared. Display the previous Tick wrote is cleared without firing.
- Equality, Delay and Euclidean, root or nested, clear their Output Portal on a Turn that answers the Absence Marker.

Scheduling, same-Tick delivery regardless of Position, Bang display cleanup, Halt, chain heads and empty-input clearing are unchanged.

## Testing Decisions

Each ticket flips the tests that pin the departed behaviour, rather than deleting them, so the new rule is pinned at the same sites. Tests use the Source Tick helpers in `orcvs/src/source/tick/observed.rs`. `PROPTEST_CASES=32` locally.

## Tickets

- 01 — Carry a Bang through a pass-through as its glyph
- 02 — Clear the output of a Bang producer that does not bang
- 03 — Fire a typed Bang once
