# 07 — Migrate the activation and replacement destination tests

**What to build:** The final migration batch. Move the remaining destination-carrying tests —
Bang activation and its ordering edges, suppression of a nested computation by an inactive parent,
Function replacement at an original anchor, and the diagnostics each of those produces — onto the
constructed schedule from `04`. Their assertions do not change.

When this ticket lands nothing populates the destination map.

**Blocked by:** 04 — Build a destination-carrying schedule a test can construct.

**Status:** resolved

- [x] Every remaining destination-carrying test drives execution through the constructed schedule.
- [x] No test in the crate populates the destination map after this ticket, except the three whose
      subject is the configured route itself — see the note below.
- [x] Every behaviour the batch asserted before it moved is still asserted after.
- [x] The crate builds and its tests pass.

## Comments

Split out of the architecture review of the Sequence branch, 2026-09-10.

Carry this batch's share of `10` rather than repeating it: these tests change route here, and the
state record `10` introduces is what they read once they have. Changing them twice is the cost of
sequencing the two apart.

2026-09-10: Resolved by the commit this line lands in, on one branch with `06` and `10`.

Eight tests moved and nine `configured_source` call sites became `carried_source` ones:

- `fixed_bang_destinations_respect_alignment_and_operand_contact`,
  `a_bang_half_inside_an_operand_does_not_activate_an_aligned_terminal`,
  `a_bang_touching_a_nested_function_operand_does_not_activate_a_neighbour` and
  `a_bang_cardinally_aligned_with_a_nested_function_anchor_activates_nothing` — Bang activation
  and its alignment and operand-contact edges.
- `live_bang_in_half_typed_terminal_claim_diagnoses_without_activation` — the diagnostic that
  activation edge produces.
- `live_deep_and_top_level_replacement_suppress_descendant_portals` and
  `a_value_replaces_nested_computation_and_preserves_next_tick_source` — suppression of a nested
  computation's Portal by what is written over it.
- `nested_computation_returns_and_projects_once` — a nested answer projected once.

Two more routes were carried in a later commit on the same branch, because neither belongs to one
batch. `plan_with_destinations` had nine call sites across seven tests, five of them this ticket's
— `a_bang_activates_its_aligned_neighbours_and_no_further_root`,
`a_pulse_activates_an_aligned_root_only_on_the_ticks_it_bangs` (two sites),
`fixed_upward_portals_schedule_note_and_bang_before_midi`,
`two_current_bang_results_still_execute_midi_once` and
`one_failed_bang_candidate_does_not_suppress_another_fresh_bang` — and two `06`'s. All nine now
call `plan_carrying` and the route is deleted; each fixture states its destination as the
one-element `Vec` a carried schedule takes per anchor.

`execution::stated::plan_with_answers` took a `Configuration` and now takes the same carried
destinations, calling `carry` over an empty one. That parameter was how the `stated_source`,
`replaced_source` and `sequence_source` fixtures reached the destination map: twenty-eight call
sites across twenty-two tests, migrated by the one swap. `04` left this route deliberately and `05`
named it as belonging to whichever ticket takes the sequence tests, or to `08`; it is taken here
because splitting it would have half-migrated a helper every batch shares.

Where the criterion is drawn more narrowly than its text. Three tests still state a Configuration,
not two: `a_carried_destination_schedules_the_tick_a_configured_one_would_have` and
`a_refused_portal_is_diagnosed_before_the_row_edge_layout_it_shares_a_tick_with`, which drive both
routes on purpose, and `carrying_destinations_onto_a_configured_refusal_is_refused`, whose whole
subject is a Configuration still holding destinations when `carry` runs. All three test the
configured route rather than use it, and all three go when `08` deletes it.

What `08` still owns: `Configuration`, `plan_configured`, `schedule`, `Source::execute_configured`,
the `configured_source` and `configured_tick` helpers, the `destinations` helper that folds a
Configuration, and the three tests above.

Three of the eight tests here read the record `10` retires —
`live_deep_and_top_level_replacement_suppress_descendant_portals`,
`a_value_replaces_nested_computation_and_preserves_next_tick_source` and
`nested_computation_returns_and_projects_once` — so they change twice on this branch, as `10`'s own
comment says they must.
