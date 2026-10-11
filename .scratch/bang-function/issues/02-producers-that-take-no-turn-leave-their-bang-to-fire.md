# 02 — Producers that take no Turn leave their Bang to fire

Status: ready-for-agent
Blocked by: 01

**What to build:**

Pin D1 and D4 from ticket 07. A clear is a producer's ordinary Absence write in its Turn; a producer that takes no Turn, or is invalid, writes nothing, and its last `**` fires once as a Bang Function. No new behaviour beyond ticket 01: this ticket owes the tests. Orca agrees: a locked `D` does not run, and its `*` stands until it erases itself.

## Acceptance criteria

- [ ] For Equality, Delay and Euclidean: banging on Tick T, then on Tick T+1 Halt-locked (E4), invalid by a blanked operand (E5), failing evaluation, or inactive by nesting in a root that takes no Turn — the `**` fires once on T+1 and the aligned root plays twice in total.
- [ ] Valid and answering Absence on T+1: the clear suppresses the `**`; the aligned root plays once in total.
- [ ] Overwritten before its Turn on T+1: the `**` fires unless the overwrite covers it.
- [ ] A Write whose destination is refused (out of Grid) suppresses nothing.
- [ ] Cycle-stopped on T+1: the `**` stands, is diagnosed as waiting on the cycle, and fires once on the first Tick the cycle is gone.
- [ ] A dynamic writer behind another in the writers-first chain: assert the actual order and whether its old `**` fires before it rewrites it.
- [ ] No Turn emits a clear through the shared suppression flag.
