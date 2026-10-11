# 01 — A standalone `**` is a Bang Function

Status: ready-for-agent
Blocked by: 07, placement-semantics/03

**What to build:**

A standalone `**` in Source is the Bang Function: a Function-table row with no operands, Intrinsic activation and can-emit-Bang, named by the parser's root-only `**` arm and absent from `from_spelling`. It reserves its own Span; its Turn activates its aligned roots and vacates its Span, as the start-of-Tick fire does today. Remove the provenance record, the start-of-Tick Bang clear and contact activation together: contact plus a Bang Function would fire a contacted root twice. Lift the missed-Bang diagnostic's Selected-only condition. Add no ordering edge. See `spec.md` and ticket 07 (D1–D17, E1–E9). Build test-first.

## Acceptance criteria

- [ ] E1–E9 from ticket 07 pass as Source Tick tests, written failing first.
- [ ] A `**` typed, pasted or loaded above Raw Play, with no writer covering it, plays on the next Tick and is cleared; the Tick after plays nothing.
- [ ] Equality that bangs plays its aligned root in the same Tick and leaves `**`; going quiet, its clear suppresses that `**` and nothing plays. Banging every Tick, it plays once per Tick.
- [ ] A Copy chain carries a Bang down a column in one Tick and plays once; when the source goes quiet nothing plays.
- [ ] `***` beside a written `**` plays nothing and the typed `*` survives.
- [ ] A `**` written into an operand activates nothing; freeing it fires exactly once, with zero or more quiet Ticks before the edit.
- [ ] Deleting a writer, or moving a Write or Push destination, in the Tick after it banged makes the old `**` fire once more; old and new destinations asserted separately.
- [ ] A Source saved and read back at the same absolute Tick plans the same Tick as the original; the saved-display reload test is flipped.
- [ ] A typed `**` under any intrinsically active writer's Output Portal (`.+0102`, a quiet `.=0102`) is suppressed; the typed-over-display, block-edit-over-display and `a_planned_tick_fires_a_typed_bang_once_and_never_the_display_a_tick_wrote` tests are flipped.
- [ ] A partial overwrite suppresses the whole `**`; the stray `*` is not a Bang.
- [ ] `*v` above a typed `**`: the `**` activates `*v` and its other aligned roots, and `*v` emits `vv` into the vacated pair in the same Tick.
- [ ] A typed `**` with roots north, south, east and west activates all four.
- [ ] A Self-Banging Function blocked by a root, a Comment or the Grid edge writes `**` and activates nothing that Tick; the `**` plays its aligned roots on the next Tick. The mover contact tests and `tick.rs:1703-1723` are flipped.
- [ ] A Directional Bang Function whose mover is blocked on its first step alternates `vv` / `**` (E9); the Function reference asset and its settled-area test are updated.
- [ ] A `**` in an Increment's own Output Portal is diagnosed as a same-Tick cycle; a Copy carrying a Function spelling onto a standing `**` is a refused Function replacement.
- [ ] No Bang-specific state survives a commit; Tick planning takes no Bang argument; test helpers plan exactly as production does.
- [ ] ADR 0072 is accepted and the superseded ADRs' status lines point to it. CONTEXT.md defines Bang, Bang Function and Missed Bang, retires Bang display, and amends Source Snapshot, Producer, Self-Banging Function and Halt Function.
