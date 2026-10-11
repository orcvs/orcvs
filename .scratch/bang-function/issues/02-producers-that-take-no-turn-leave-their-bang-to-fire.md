# 02 — Producers that take no Turn leave their Bang to fire

Status: resolved
Blocked by: 01

**What to build:**

Pin D1 and D4 from ticket 07. A clear is a producer's ordinary Absence write in its Turn; a producer that takes no Turn, or is invalid, writes nothing, and its last `**` fires once as a Bang Function. No new behaviour beyond ticket 01: this ticket owes the tests. Orca agrees: a locked `D` does not run, and its `*` stands until it erases itself.

## Acceptance criteria

- [x] For Equality, Delay and Euclidean: banging on Tick T, then on Tick T+1 Halt-locked (E4), invalid by a blanked operand (E5), failing evaluation, or inactive by nesting in a root that takes no Turn — the `**` fires once on T+1 and the aligned root plays twice in total.
- [x] Valid and answering Absence on T+1: the clear suppresses the `**`; the aligned root plays once in total.
- [x] Overwritten before its Turn on T+1: the `**` fires unless the overwrite covers it.
- [x] A Write whose destination is refused (out of Grid) suppresses nothing.
- [x] Cycle-stopped on T+1: the `**` stands, is diagnosed as waiting on the cycle, and fires once on the first Tick the cycle is gone.
- [x] A dynamic writer behind another in the writers-first chain: assert the actual order and whether its old `**` fires before it rewrites it.
- [x] No Turn emits a clear through the shared suppression flag.

## Comments

2026-10-11 — Pinned in `orcvs/src/source/tick/bang_function.rs`; no code change beyond ticket 01. Every row of D4 executed as ticket 07 traced it.

- Equality, Delay and Euclidean each fire their last `**` once when Halt-locked, blanked, or nested in a locked Addition; Delay and Euclidean also when a zero operand fails evaluation. Equality answers every pair of values, so it has no evaluation failure: `.=01C4` is unequal and clears.
- An overwrite of the producer's anchor suppresses it, and the suppressed producer writes no clear, so its `**` fires. A writer retyped in place over the producer covers the `**` and suppresses it.
- A Write moved outside the Grid is refused and its old `**` fires.
- A producer waiting on a cycle leaves its `**` standing, diagnosed as waiting, every Tick; once the cycle is gone, the producer is invalid in that fixture and its `**` fires once.
- Writers-first: a Write whose old `**` feeds the Write before it in the chain comes after that `**`, which fires first; the reading Write sees the Cells cleared. Reversed, the rewriting Write goes first and suppresses its old `**`.

