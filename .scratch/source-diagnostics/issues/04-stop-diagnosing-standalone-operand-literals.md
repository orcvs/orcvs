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
- [ ] Update the other tests that pin these diagnostics, including `stray_value` and its callers in `orcvs/src/source/tick/nested_copy/vertical_and_map.rs`.
- [ ] In `console`, rewrite the doc comment of `every_example_expression_parses_without_a_diagnostic_outside_a_result_row` in `console/src/function_reference.rs`, and narrow its excuse list to the result Cells that still draw a diagnostic.
- [ ] Cell-tracker 06's test pins that the pairs east of Track draw no Language Map diagnostic: in that test if 06 has shipped, otherwise as a note on 06.
- [ ] Schedule identity is unaffected: no new scheduling input, and the schedule-inputs projection still reuses across literal-only edits.
- [ ] Paint is unchanged: these Cells still paint as ordinary Source.
