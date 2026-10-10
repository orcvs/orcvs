# 02 — Read a pair in a direction

Status: resolved
Blocked by: 01 — Let each Function resolve its dynamic Input Portal; 07 — Accept the Read, Write and Copy families; 08 — Rename Jump to Copy

**What to build:**

The directional Reads `&^ &v &< &> n` (ADR 0070), replacing the two-axis Source Read `@< column row` this ticket first built. Each is a value Function with one Number operand `n` whose dynamic Input Portal is the pair `n` Portals from its `n` operand in its arrow's direction, as ADR 0070 settles. It reads that pair exactly as a Copy does and answers it through its Output Portal one row south or, nested, as its Return. A performer can lay Notes out in a column below a voice and read any row of it.

Adapt the code built for `@<` rather than starting over: `Function::SourceRead`, `PairSelection::ColumnRowOffset`, `read_offset`, and the tests in `orcvs/src/source/tick/read.rs` and `schedule_reuse.rs` already resolve, read and order a pair at a relative offset. The selection rule becomes one per direction (or one rule carrying a direction), and `@<` leaves the Function table, freed for the west Write (ticket 05). The `&` arrows are free once ticket 08 has moved the Jumps to `=`.

## Acceptance criteria

- [x] `&^`, `&v`, `&<` and `&>` are in the Function table with one Number operand `n`; it may be a literal, nested, or written by a Portal, and an unwritten one makes the Read invalid (ADR 0069).
- [x] Distances count Portals from the `n` operand with no special cases: one pair east or west, one row north or south, in the operand's columns. `&>00C4` answers `00`, `&>01C4` answers `C4` and `&>02C4D4` answers `D4`.
- [x] A nested `n` counts from the slot it occupies: a test pins that `&>.+0101C4` answers the second `01` (its `n` is `02`) and `&>.+0102C4` answers `C4`, and that Track still counts its list from the end of its last operand, nested Functions included.
- [x] Tests pin the intended consequences: every `&x 00` answers its own operand; `&v 01` reads the pair below the operand, beside the Output Portal; and `&< 01` reads the Function's own spelling and writes a copy of that Function.
- [x] The `PairSelection` rule counts pairs, not Cells, east and west; the Cell-counting `column` built for `@<` does not survive.
- [x] Reads follow Copy's reading rules: empty Cells are copied (a nested Read of them leaves its parent invalid), `**` relays a Bang, a Function spelling answers that Function, and a partial pair, a Comment or Cells straddling two Language Units diagnose.
- [x] A pair outside the Grid in any direction, or cut short by the row edge, diagnoses and writes nothing.
- [x] Ordering follows ADR 0032 completed at the Turn, as `@<` did: tests drive writers before and after each Read in Grid order, same-Tick writes to `n`, and a writer that waits on the Read as a diagnosed cycle under ADRs 0065 and 0068.
- [x] A Source holding each Read gives the same result from a reused schedule as from a fresh one.
- [x] `@< column row` no longer parses as a Read, and no test or shipped Source spells it.
- [x] The Function reference names the directional Reads. ADR 0019's table and the glossary already do, from ADR 0070's acceptance.

## Comments

Resolved earlier as the two-axis relative Read, before ADR 0070 was proposed; that build was adapted into the directional Reads rather than landed. `@<` was `Function::SourceRead`, a value Function with Number operands `column` and `row` whose Input Portal was `Dynamic(PairSelection::ColumnRowOffset)`: `column` Cells (not pairs) east of the end of its last operand and `row` rows south of its anchor. It read with Jump's rules through `jump` and was ordered at its Turn by the same wait Track uses. An offset too far east to represent diagnosed as partial or invalid input, as a pair outside the Grid does. Because the selection rule is part of the declared Input Portal, a Function Replacement between `@t` and `@<` was refused as a change to the Source write the Function declares.

Its tests in `orcvs/src/source/tick/read.rs` cover literal, nested and Portal-written operands; zero, row-edge and out-of-Grid offsets; writers before it in Grid order, east of the pair and below it; competing writers; a writer that waits on the Read; a Read of Cells a stopped cycle writes; same-Tick writes to `column` and `row`; empty, `**`, Function-spelling, partial, Comment and straddling pairs; nested Return and invalid parent; and a Timed Play fed by a nested Read. They are the starting point for the directional Reads.

Reopened: ADR 0070 proposes separate Read, Write and Copy families, with directional and absolute Reads in place of the two-axis relative Read.
