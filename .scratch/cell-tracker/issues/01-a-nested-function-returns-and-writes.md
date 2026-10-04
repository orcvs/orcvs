# 01 — A nested Function returns its encoding and still writes south

Status: needs-triage

## Work

Implement [ADR 0061](../../../docs/adr/0061-a-nested-function-returns-to-the-slot-it-occupies.md).
A nested Function delivers its answer to the inline input portal it occupies as
its two-Cell encoding, decoded by that portal's declared literal type, and also
writes through its own Output Portal as a root does. A Function that cannot
answer one two-Cell encoding is refused by the Parser, not during a Tick.

## Acceptance

- [ ] `.+.+010102` writes `04` at the outer anchor's Output Portal and `02` at
      the inner one's; the row below reads `0402`.
- [ ] A Note returned into a Number operand decodes as the Number its two
      characters spell, matching a Portal write of the same characters.
- [ ] A nested Increment and a nested Interpolation keep their state across
      Ticks through their Output Portal.
- [ ] A nested Function that answers a Bang writes `**` south and activates the
      aligned roots ADR 0006 names.
- [ ] A nested effect Function is marked by the Language Map from the Source
      alone; `NestedEffectFunction` no longer arises during a Tick.
- [ ] Reservations cover a nested computation's Output Portal; the dependency
      schedule orders consumers of that write.
- [ ] The three `tick.rs` tests that assert a nested Function leaves its south
      Cells alone are rewritten to assert the write.
- [ ] `CONTEXT.md` and the Function reference agree with the behaviour.

Touches the Parser, Language Map, portal resolution, Reservation and execution
paths in `orcvs/src/source/`.
