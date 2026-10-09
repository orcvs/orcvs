# 03 — An unwritten operand leaves its Function pending

Status: resolved
Blocked by: 02

**What to build:**

Implement ADR 0066 through the Language Map and root and nested execution. An
inline operand slot with no Cell written leaves its Function pending: it does
not evaluate, writes nothing and raises no diagnostic. A pending nested Function
leaves its parent pending. ADR 0066 supersedes ADR 0062, so Orcvs has no Blank
Answer.

## Acceptance criteria

- [x] A Function whose only unbound operands are unwritten slots inside the row reports no diagnostic and is not a root, whether the slot is in the root or in a nested Function.
- [x] A slot with some Cells written, or one the row edge cuts short, still diagnoses.
- [x] A root whose operand an earlier Turn empties during the Tick is pending for that Turn, and nothing diagnoses it.
- [x] A Bang reaching a Play whose note slot is unwritten plays nothing and diagnoses nothing.
- [x] A Jump copies empty aligned input to its destination and, nested, returns the empty Cells, so its parent is pending rather than diagnosed.
- [x] ADR 0066 records the decision, and the ADRs that deferred to ADR 0062 point at it.

## Comments

The criteria are covered by `an_expression_waiting_on_unwritten_operands_is_pending_and_not_diagnosed` and `a_partly_written_or_edge_cut_operand_still_diagnoses` in `orcvs/src/source/language_map.rs`, `a_banged_play_with_an_unwritten_note_is_pending_and_emits_nothing` in `orcvs/src/source/tick.rs`, and the nested Jump tests in `orcvs/src/source/tick/nested_jump/` whose Jump empties an operand.

2026-10-09: ADR 0069 superseded ADR 0066, so the pending state this ticket delivered no longer exists: an unwritten operand is invalid input and is diagnosed, a nested invalid Function fails its parent, and a Banged Play whose note slot is unwritten is invalid and emits nothing. That was implemented by `.scratch/pending-presentation/issues/01-treat-an-unwritten-operand-as-invalid-input.md`, which converted the tests named above. The criteria above record what this ticket delivered under ADR 0066.
