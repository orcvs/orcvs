# 05 — Add the Range Functions

**What to build:** Implement Number Range `:-` and Note Range `:#` as two monomorphic Functions
with distinct signatures, per ADR 0007 and ADR 0023. Each fixes its own operand and result type:
`:-` takes two Numbers and returns an inclusive unit-step Sequence of Numbers; `:#` takes two Notes
and returns an inclusive chromatic Sequence of Notes. Neither selects its behaviour from its
operands, and neither converts implicitly.

**Blocked by:** None — 06, 07, and grid-boundedness/01 are resolved.

**Status:** resolved

**Tags:** release/v1

- [x] `:-` returns an inclusive unit-step Number Sequence; `:#` returns an inclusive chromatic Note
      Sequence that preserves Note identity in every member.
- [x] Bound order selects ascending or descending output, and equal bounds return a singleton, for
      both Functions.
- [x] Mixed-type bounds diagnose and produce no Sequence. Neither Function converts a bound to its
      own type, and neither falls back to the other Range's behaviour.
- [x] A Bang is an invalid bound for both Functions and diagnoses. So do Function and Sequence
      bounds.
- [x] `:#` rejects a Number bound above `7F` through the Note domain rather than through Range.
- [x] Both Functions parse and round-trip through their canonical two-Cell spellings, and `:#` is
      recognised as a complete two-Cell Function. ADR 0035 spells the Comment introducer `||`, so
      no Comment rule reaches into `:#`.
- [x] Range output takes the ordinary complete-fit Portal rule and never writes a partial Sequence.
- [x] ADR 0036's reservation derivation is exercised end to end from a declaration: a Range row
      reserves the Cells from its destination through the end of that row, a pervasive parent
      widens over a Range child, a parent that is not pervasive does not, and the Turns inside each
      reservation are ordered after the computation that reserved them. Until this lands the
      derivation pass has no test at all — every `Reserved::Row` in the crate is stated by a
      fixture, so the pass can only ever answer `Reserved::Pair` from what Functions declare.

## Comments

Split out of `issues/03` during the `release/v1` issue alignment on 2026-09-04. Issue 03 owns the
four structural Sequence Functions `:<`, `:&`, `:?` and `:=`, which ADR 0023 keeps generic because
they do not reinterpret their Atoms. Range is the opposite case, so it does not belong there: ADR
0023 gives `:-` and `:#` distinct names precisely so that each has one fixed operand and result
signature and a mistyped bound diagnoses instead of silently selecting the other behaviour.

Both Range Functions are owned here, together. Do not open a second issue for `:#` — one Function
per issue would give the two halves of one decision two owners and two chances to disagree about
what a mistyped bound does.

The reservation acceptance line is owed by `test-only-seams/02`, which deleted
`live_a_nested_sequence_answer_widens_the_reservation_of_the_root_above_it`. That test asserted the
widening against a Sequence answer stated through a `cfg(test)` seam, so with the seam gone it
asserted a width the fixture had already stated.

The loss is wider than that one test. With nothing stating a Sequence answer through the scheduler,
and no Function declaring one, every input to the derivation pass says `Reserved::Pair`: the pass
runs on every Tick and cannot answer anything else, and the fixtures that need a row-wide
reservation state it directly. Declaring a Sequence answer here is what gives that pass its first
real input, which is why this line owes the derivation rather than only the widening.

These Range fixtures do not inherit the stated-reservation seam. A declared `:-` row derives its
own width, so a test written here spells the Function in Source and states nothing — the seam of
`test-only-seams/02` exists precisely because that is not possible yet.

The mixed-bounds acceptance line is the one carried over from issue 03 rather than dropped.
`orcvs-language-migration/04` settled that the same two Source characters take exactly one type
from the containing Expression and its Function signature; the diagnostic is what that decision
costs, and without this line nothing in the tracker requires it.

`CONTEXT.md:99-101` and ADR 0007 are the wording of record for both Functions.

### Verification, 2026-09-14

Verified all eight items against `main` (`87fc793`), per item, by reading the code that decides the
behaviour and then confirming a test drives it. Landed in `fcb1a40` with review follow-up in
`651f9d6`. Comment respelling (`07`) and Sequence projection (`06`) are already resolved, so the
blockers listed above no longer apply.

- **Inclusive Number / chromatic Note.** `inclusive_number_range` / `inclusive_note_range` in
  `lang/src/functions/sequence.rs`. Covered by `number_range_is_inclusive_and_respects_bound_order`
  (`00`–`03`) and `note_range_preserves_note_identity_and_respects_bound_order` (`C4`–`D4` as Notes
  60–62). Source projection `live_a_declared_note_range_result_reaches_its_destination_cells`
  writes `C4c4D4`, which is the identity claim: sharp `c4` is a Note, not a Number.
- **Bound order and singleton.** Same two tests: descending Number `03`–`00`, equal `05` → `[05]`;
  descending Notes 62–60; Source `:#C4C4` is a singleton.
- **Mixed-type bounds.** `range_functions_diagnose_mixed_type_and_invalid_bounds` and
  `number_range_interpret_diagnoses_a_note_operand_bound` (`:-.^3C03` → `TypeError::Number("C4")`).
  Signatures are `[Number, Number]` and `[Note, Note]`; there is no conversion arm.
- **Bang, Function, Sequence bounds.** Bang in the mixed-type test; Function and Sequence in
  `range_functions_diagnose_function_and_sequence_bounds`; Source `:-**0003` and `:#**C4`.
- **Note domain above `7F`.** `interpret(":#8080")` is `TypeError::Note("80")`, not a Range error.
  `Note::try_from` is the domain; Range never sees the byte.
- **Parse, round-trip, `:#` vs Comment.** `every_sequence_function_parses_and_round_trips_its_spelling`
  covers `:-0003` and `:#C4C5`. `the_hash_collision_that_broke_the_pre_pass_holds_no_comment`
  (`orcvs/src/source/tick.rs`) pins `:#` as a Function at Cells 3–4 with a trailing `#` that is
  not a Comment; ADR 0035's introducer is `||`.
- **Complete-fit Portal.** `live_a_declared_number_range_that_leaves_its_row_writes_no_cell_of_it`
  and `live_a_declared_note_range_that_leaves_its_row_writes_no_cell_of_it`: diagnostics, zero
  writes, Source unchanged. Happy path:
  `live_a_declared_number_range_result_reaches_its_destination_cells` (`:-0003` → `00010203`).
- **ADR 0036 derivation from a declaration.** `a_declared_number_range_row_derives_its_own_reservation`
  and the Note sibling derive `Reserved::Row` from Source with no stated width.
  `a_pervasive_parent_widens_over_a_declared_number_range_child` (`.+:-0003`);
  `a_non_pervasive_parent_does_not_widen_over_a_declared_number_range_child` (`:?00:-0003` stays
  `Pair`). Ordering: `live_a_declared_number_range_reservation_orders_computations_it_covers` and
  the Note sibling — the Range in row 1 takes Turn 0, the covered `.+` in row 0 takes Turn 1.
