# 10 — Read a pair at a Position

Status: resolved
Blocked by: 02 — Read a pair in a direction; 07 — Accept the Read, Write and Copy families

**What to build:**

The absolute Read `&$ column row` (ADR 0070), the follow-up ADR 0049 left under the `&` prefix. It is a value Function with Number operands `column` and `row` whose dynamic Input Portal is the pair at that Position: column then row, counted in Cells from `00 00` at the top-left of the Grid. It reads that pair exactly as a Copy does and answers it through its Output Portal one row south or, nested, as its Return. It builds on the directional Reads' selection and ordering (ticket 02); unlike them, its address does not depend on its anchor.

## Acceptance criteria

- [x] `&$` is in the Function table with Number operands `column` and `row`; each may be a literal, nested, or written by a Portal, and an unwritten one makes the Read invalid (ADR 0069).
- [x] `&$ 00 00` reads the two Cells at the top-left of the Grid, wherever the Read stands; moving the Read does not change what it reads.
- [x] A pair starting at column `FF` is cut short by the row edge and diagnoses, writing nothing.
- [x] Reads follow Copy's reading rules, as ticket 02 tests them.
- [x] Ordering follows ADR 0032 completed at the Turn: tests drive writers of the addressed pair before and after the Read in Grid order, including writers north and west of it, same-Tick writes to `column` and `row`, and a writer that waits on the Read as a diagnosed cycle.
- [x] A Source holding `&$` gives the same result from a reused schedule as from a fresh one.
- [x] The Function reference names `&$`. ADR 0019's table and the glossary already do, from ADR 0070's acceptance.
