# 04 — Stop diagnosing standalone Operand Literals

Status: ready-for-agent
Blocked by: 01

**What to build:**

Implement ticket 01's ADR in `Parser::analyze` and the Language Map's row walk: a two-Cell Unit that spells a Number or a Note where an Expression could start produces no diagnostic and is consumed as one Unit.

## Acceptance criteria

- [ ] A row of Number and Note pairs, such as the pairs east of a Track, produces no Language Map diagnostic.
- [ ] Spellings that name no Function and are not a Number or a Note (`Z `, `id`, a lone `|`) are still diagnosed as today.
- [ ] A Note or Number in a declared operand slot of the wrong type is still diagnosed (`.+c401`).
- [ ] Cells a Function wrote on an earlier Tick that spell a Number or a Note are not diagnosed on the next Tick; update the test at `orcvs/src/source/model.rs` that pins "a lone Number reads as an unknown Function on the next Tick".
- [ ] Schedule identity is unaffected: no new scheduling input, and the schedule-inputs projection still reuses across literal-only edits.
- [ ] Paint is unchanged: these Cells still paint as ordinary Source.
