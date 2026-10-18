# Play rest: a Play whose note a nested Function cleared plays nothing and says nothing

Status: resolved

## Problem Statement

ADR 0073 makes a nested Clear leave its parent invalid, diagnosed as
`nested computation at column X, row Y cleared its operand`. A tracker reads
each voice's note from a pattern of Cells through a Read or Track in the Play's
note slot, so a voice resting on an empty step is a Play whose note a nested
Read cleared. In the Learn Orca tracker port, seven of the eight Plays in the
shipped pattern are diagnosed every Tick, although the sound is already right:
an invalid Play sends nothing. Orca's `:` with no note plays nothing silently.

## Decision

Toby chose it on 2026-10-18. ADR 0075 records it.

1. A Clear in the note operand of Raw Play `!>`, Timed Play `!~` or
   Monophonic Play `!%` is a rest: the Turn is taken, sends no Play Command and
   raises no diagnostic.
2. Only the note operand. A cleared channel, velocity or length, and a Clear
   in any operand of any other Function, keep ADR 0073's diagnostic. A rest
   also needs every other operand to decode.
3. Only a Clear. A nested Function that is invalid, such as a Read of a
   half-typed Note, still leaves the Play diagnosed as having returned nothing,
   and a literal unwritten or partly written note slot is invalid input.
4. The Function table declares the rest operand with a `rest: note` clause, so
   the Source tick asks the definition rather than naming Functions.

## Consequences

- A tracker's resting voices are quiet. The nested Read still writes its Clear
  through its own Output Portal.
- A rest does not evaluate the Play, so an out-of-domain channel or velocity
  is diagnosed only on a Turn that plays.
- A Monophonic Play that rests sends nothing, so its channel's voice runs to its
  own Note Off rather than being replaced with silence.
