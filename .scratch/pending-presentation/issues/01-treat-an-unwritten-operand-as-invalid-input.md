# 01 — Treat an unwritten operand as invalid input

Status: resolved
Blocked by: None — can start immediately

**What to build:**

Implement the execution half of [ADR 0069](../../../docs/adr/0069-an-unwritten-operand-is-invalid-input.md). An operand slot whose Cells are all empty is invalid input, handled exactly as a partly written slot is today. Remove the pending state from execution, and give the diagnostics that an unwritten slot causes a pending classification for the console to present.

Today the state lives in two places: `is_pending` in `orcvs/src/source/language_map.rs`, which keeps such an Expression out of the diagnostics and the roots, and `Operands::pending` in `orcvs/src/source/tick/execution/operands.rs`, which gives a Function no Turn and no diagnostic and passes pending to its parent.

## Acceptance criteria

- [x] An Expression with an unwritten slot, in the root or in a nested Function, is diagnosed and treated by the Language Map as a partly written one is. `is_pending` no longer decides diagnostics or roots.
- [x] A slot a Portal write empties during the Tick makes its Function invalid at its Turn, diagnosed as other invalid input is. A nested Function invalid that way fails its parent under ADR 0034's nested-consumer clause. `Operands::pending` no longer exists as an execution outcome.
- [x] A nested Jump or Track that copies empty Cells still evaluates and returns them, and its parent is invalid.
- [x] An invalid Function writes nothing, so Cells it wrote on an earlier Tick keep their characters. A test pins this for a value Function whose operand a Portal empties, with a Play reading its Output Portal through Cells: the next Bang replays the last Note.
- [x] A Bang reaching a Play whose note slot is unwritten emits no Play Command, directly and through a nested Track that copies an empty pair.
- [x] Each diagnostic whose cause is an unwritten slot, in the Function or in a nested Function it waits on, carries a pending classification naming the slot's declared literal type; an Absence Marker failure and a malformed slot do not. The classification is reachable by the console without parsing the message.
- [x] Increment and Interpolation keep their state across a Tick on which an operand is unwritten.
- [x] Tests that assert pending as "no diagnostic" (`an_expression_waiting_on_unwritten_operands_is_pending_and_not_diagnosed`, `a_banged_play_with_an_unwritten_note_is_pending_and_emits_nothing`, the nested Jump tests in `orcvs/src/source/tick/nested_jump/`, and Track's pending tests in `orcvs/src/source/tick/track.rs`) assert the diagnostic and its classification instead.
- [x] Gates for `orcvs` and `console`, and the workspace pre-pull-request gates.

## Comments

Until ticket 02 lands, pending diagnostics show in the console as ordinary diagnostics. Decide before merging whether 01 and 02 ship together.

Resolved in a44d08ec. `unwritten_slot` in `orcvs/src/source/language_map.rs` classifies the Map's diagnostic instead of suppressing it, and `Execution::decode` refuses an unwritten slot (direct, or copied empty by a nested Jump or Track) as pending, carrying the classification from a refused child to its parent. `Diagnostic::pending` exposes the declared literal type. Covering tests: `an_expression_waiting_on_unwritten_operands_is_diagnosed_as_pending` and `a_partly_written_or_edge_cut_operand_diagnoses_as_a_fault` (Language Map); `a_banged_play_with_an_unwritten_note_is_invalid_and_emits_nothing`, `a_banged_play_whose_nested_track_copies_an_empty_pair_is_invalid`, `a_value_function_whose_operand_a_portal_empties_leaves_its_last_note_to_replay`, `a_nested_function_invalid_on_an_unwritten_slot_fails_its_parent_as_pending`, `a_nested_absence_marker_fails_its_parent_as_a_fault`, `an_unwritten_operand_does_not_hide_an_earlier_malformed_one`, `an_unwritten_operand_in_the_source_blocks_its_function_as_the_map_reports` and `increment_and_interpolation_keep_their_state_across_an_unwritten_operand` (`source::tick`); Track's `an_empty_operand_makes_track_invalid_as_pending`, `a_partially_written_operand_diagnoses_as_a_fault` and `a_nested_track_that_reads_empty_cells_makes_its_parent_invalid`; and the converted nested Jump tests in `orcvs/src/source/tick/nested_jump/`. Where a Turn has both a malformed and an unwritten operand, the first operand in signature order that fails to decode names the diagnostic, so a malformed earlier operand is reported as a fault.

The pending classification on `Diagnostic` that this ticket asked for was removed in a follow-up commit on the same branch: ADR 0069 no longer makes pending a classification, and nothing in the console read it.
