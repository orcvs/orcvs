# 06 — Migrate the spatial write composition tests

**What to build:** The second migration batch. Move the tests whose subject is what writes do to
Cells — Cell-wise composition of overlapping writes, competing writers resolved by Position,
complete-fit refusal at a row or Grid edge, and pending operand encodings decoded at consumption —
onto the constructed schedule from `04`. Their assertions do not change.

**Blocked by:** 04 — Build a destination-carrying schedule a test can construct.

**Status:** resolved

- [x] Every test in this batch drives execution through the constructed schedule.
- [x] Every behaviour the batch asserted before it moved is still asserted after.
- [x] The crate builds and its tests pass; the configured route still exists for the batches that
      have not moved.

## Comments

Split out of the architecture review of the Sequence branch, 2026-09-10.

Carry this batch's share of `10` rather than repeating it: these tests change route here, and the
state record `10` introduces is what they read once they have. Changing them twice is the cost of
sequencing the two apart.

2026-09-10: Resolved by the commit this line lands in, alongside `07` and `10` on one branch. The
three were taken together because splitting them costs work rather than saves it —
`plan_with_destinations` and `execution::stated::plan_with_answers` each span both batches, and
`07`'s "nothing populates the destination map" is only checkable once this one has landed.

Twelve tests moved and twenty-six `configured_source` call sites became `carried_source` ones. Nine
of the twelve are this ticket's stated subject:

- `live_competing_writers_follow_position_and_emissions_follow_configuration` and
  `competing_writers_preserve_an_independent_rejected_destination_diagnostic` — competing writers
  resolved by Position.
- `overlapping_outputs_beside_standalone_source_both_contribute_cells` and
  `outputs_beside_and_over_standalone_source_are_admitted` — Cell-wise composition of overlapping
  writes.
- `live_a_sequence_that_writes_over_its_own_producer_rejects_the_tick` — the same rule where a
  write reaches its own producer.
- `live_pending_note_decodes_only_after_all_writers_settle`,
  `live_spatial_note_is_an_encoding_and_nested_note_stays_typed` and
  `an_absolute_difference_beside_a_vertical_rule_is_read_in_two_cell_units` — pending operand
  encodings decoded at consumption.
- `live_deep_sibling_computations_preserve_operand_order` — the operand order that decoding reads.

Where the line was drawn differently than the ticket text assumed. Three of the twelve state no
destination at all and are on the helper only because it is the one that seeds a Source:
`live_claims_and_glyphs_survive_source_edits_and_publication` (six call sites),
`the_hash_collision_that_broke_the_pre_pass_holds_no_comment`, and
`live_unchanged_nested_syntax_errors_do_not_repeat_as_tick_failures`, which `05` named as
straddling and left. Neither this ticket's subject nor `07`'s claims them; they were taken here
because they sit among the write tests and cost the same edit in either batch.

`plan_with_destinations` is not retired by this batch. Two of its seven tests are this ticket's —
`a_destination_at_the_row_edge_costs_one_expression_its_turn_not_the_tick` and
`competing_writers_publish_but_dependency_cycles_abort_before_output`, which `05` left whole
because its two halves belong to two batches. The route only retires when the last of the seven
moves, so all nine of its call sites were swapped in one later commit on this branch rather than
split across the two batches that own them.

`10`'s share for this batch was carried in that ticket's own commit rather than here, because `10`
is shipped code and the migration is not. One of the twelve reads the record `10` retires —
`live_competing_writers_follow_position_and_emissions_follow_configuration` — so it changes twice
on this branch. That is the cost `10`'s own comment names.
