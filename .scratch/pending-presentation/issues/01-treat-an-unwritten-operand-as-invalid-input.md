# 01 — Treat an unwritten operand as invalid input

Status: ready-for-agent
Blocked by: None — can start immediately

**What to build:**

Implement the execution half of [ADR 0069](../../../docs/adr/0069-an-unwritten-operand-is-invalid-input.md). An operand slot whose Cells are all empty is invalid input, handled exactly as a partly written slot is today. Remove the pending state from execution, and give the diagnostics that an unwritten slot causes a pending classification for the console to present.

Today the state lives in two places: `is_pending` in `orcvs/src/source/language_map.rs`, which keeps such an Expression out of the diagnostics and the roots, and `Operands::pending` in `orcvs/src/source/tick/execution/operands.rs`, which gives a Function no Turn and no diagnostic and passes pending to its parent.

## Acceptance criteria

- [ ] An Expression with an unwritten slot, in the root or in a nested Function, is diagnosed and treated by the Language Map as a partly written one is. `is_pending` no longer decides diagnostics or roots.
- [ ] A slot a Portal write empties during the Tick makes its Function invalid at its Turn, diagnosed as other invalid input is. A nested Function invalid that way fails its parent under ADR 0034's nested-consumer clause. `Operands::pending` no longer exists as an execution outcome.
- [ ] A nested Jump or Track that copies empty Cells still evaluates and returns them, and its parent is invalid.
- [ ] An invalid Function writes nothing, so Cells it wrote on an earlier Tick keep their characters. A test pins this for a value Function whose operand a Portal empties, with a Play reading its Output Portal through Cells: the next Bang replays the last Note.
- [ ] A Bang reaching a Play whose note slot is unwritten emits no Play Command, directly and through a nested Track that copies an empty pair.
- [ ] Each diagnostic whose cause is an unwritten slot, in the Function or in a nested Function it waits on, carries a pending classification naming the slot's declared literal type; an Absence Marker failure and a malformed slot do not. The classification is reachable by the console without parsing the message.
- [ ] Increment and Interpolation keep their state across a Tick on which an operand is unwritten.
- [ ] Tests that assert pending as "no diagnostic" (`an_expression_waiting_on_unwritten_operands_is_pending_and_not_diagnosed`, `a_banged_play_with_an_unwritten_note_is_pending_and_emits_nothing`, the nested Jump tests in `orcvs/src/source/tick/nested_jump/`, and Track's pending tests in `orcvs/src/source/tick/track.rs`) assert the diagnostic and its classification instead.
- [ ] Gates for `orcvs` and `console`, and the workspace pre-pull-request gates.

## Comments

Until ticket 02 lands, pending diagnostics show in the console as ordinary diagnostics. Decide before merging whether 01 and 02 ship together.
