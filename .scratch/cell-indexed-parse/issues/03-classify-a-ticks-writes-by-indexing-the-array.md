# 03: Classify a Tick's writes by indexing the array

**What to build:** The Tick scheduler decides what a two-Cell result written at a Cell means by
reading that Cell's entry and the next one, rather than by searching a sorted collection of operand
slots gathered from every root, and finds two producers targeting one Cell by the same index rather
than by a separate map.

Both entries naming the same root and slot, with the write beginning at that slot's first Cell, is a
whole-slot projection and draws a Data edge. Entries naming different slots are a partial input
projection. Either entry being structural is an unsupported structural write. Entries that are empty
or hold a character belonging to no Expression are a free write.

**Blast radius is not uniform, and this ticket must preserve today's split.** ADR 0032 rejects the
whole Tick for competing writers, cycles, partial input projections and unsupported structural
writes. A failure local to one Expression instead withdraws that producer — it loses its destination
and every other root still plays. Whichever entry a Cell carries, the outcome must match the class
the ADR puts it in, not the class the neighbouring code happens to use.

**Blocked by:** 02

**Status:** wontfix

**Tags:** release/v1

## Pre-delivery audit at `593613c` — 2026-09-08

The implementation statements in this audit describe commit `593613c`, before
live-typed-execution delivery. For the completed transferred work and current
implementation, see [delivery evidence](../../live-typed-execution/evidence.md).

Not implemented: `source/tick.rs` still builds and searches the flat slot collection
in `schedule`, calls `root_layout`, and rejects competing/partial writes. ADR 0034
replaces those blanket rejection rules with ordered Cell-wise composition and supports
bounded replacement; requiring the old graph-error tests unchanged would restore the wrong
contract. The mandatory array algorithm is abandoned. Dependency ordering, cycle rejection,
write admission, cleared Bang handling, observable diagnostics and revision of superseded
regressions transfer to [live-typed-execution](../../live-typed-execution/spec.md), including
the half-typed-root diagnostic left by ticket 01. Existing unrelated activation behavior
must remain covered. This ticket is superseded, not delivered.

The original checklist below is retained as historical scope; this audit records
its disposition at `593613c`. The linked delivery evidence records subsequent delivery.

- [ ] A whole-slot projection draws a Data edge, and the consumer runs after its producer.
- [ ] A result straddling two operand slots is diagnosed as a partial input projection and rejects
      the Tick.
- [ ] A result landing on an Expression's structural spelling is diagnosed as an unsupported
      structural write and rejects the Tick.
- [ ] A result landing on Cells no Expression claims is a free write and is not diagnosed.
- [ ] Two producers targeting one Cell are diagnosed as competing writers and reject the Tick.
- [ ] A producer that lost its destination holds no Cell against a producer that keeps one.
- [ ] A producer rewriting its own Bang display over the previous Tick's is planned with no
      diagnostic, as it is today.
- [ ] Every diagnostic keeps the blast radius ADR 0032 gives it: whole-Tick for the four graph
      errors, withdrawal for a failure local to one Expression.
- [ ] The flat slot collection across all roots, its ordering, its widest-slot bound, its bounded
      search, both of its preconditions, and the map of Cells already targeted this Tick are deleted.
- [ ] Ordering is unchanged: dependency first, Grid position as the tie-break, and a producer below
      its consumer still supplies it in the same Tick.
- [ ] The existing graph-error tests pass untouched.
