# 04 — Decide how a dynamic Output Portal is ordered

Status: resolved
Type: grilling

**What to build:**

A recorded decision, as an amendment to ADR 0067. A Function that writes Cells its operands select learns its destination only at its Turn, after readers of those Cells may already have taken theirs. Decide which Turns see such a write, whether readers are held back until every dynamic writer has settled its destination, and when a reader and the writer form a diagnosed cycle.

The decision covers every Function ADR 0070 places in the Write family: the directional Writes `@^ @v @< @> n value`, the absolute Write `@$ column row value` and Push `@t index count value`. It also covers the write of the absolute Copy `=$`, which reads through a dynamic Input Portal and writes through a dynamic Output Portal in one Turn. The directional Copies keep static Portals and are not affected.

## Acceptance criteria

- [x] The amendment states which Turns see a dynamically addressed write, for static readers, dynamic readers and nested readers.
- [x] It states when a reader and a dynamic writer form a cycle, consistent with ADRs 0065 and 0068, including a `=$` whose source and destination overlap.
- [x] It states whether a Write answers a value through an Output Portal one row south, or only performs its Source write.
- [x] It states what happens to schedule reuse for a Source holding a dynamic writer.
- [x] It names the rejected options and why.
- [x] The domain glossary's Output Portal entry names static and dynamic Output Portals.

## Resolution

The user confirmed the decision recorded in ADR 0067's amendment on dynamic Output Portals. A dynamic writer takes its Turn as early as its inputs allow: every computation that does not feed it, whether a static, dynamic or nested reader, comes after it and sees its write in the same Tick, and a dependency the writer finds at its Turn goes first. A write onto Cells a completed Turn already read is next-Tick feedback; nothing is reordered or re-taken. Independent dynamic writers are ordered by Grid position, and a static writer that does not feed a dynamic one comes after it. Cycles form only through real dependencies, and an overlapping `=$` is one Turn that reads and then writes. A Write answers `value` through its dynamic Output Portal and nowhere else, and, nested, returns it. Schedule reuse is unchanged. ADR 0034's ordering-bug clause is amended for dynamic writes. Tickets 05, 06 and 11 carry the consequences in their acceptance criteria; ticket 12 is unaffected.
