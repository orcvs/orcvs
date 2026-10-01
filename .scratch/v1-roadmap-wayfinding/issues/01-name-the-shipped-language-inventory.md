# Name the shipped language inventory

Type: grilling

Blocked by: None — can start immediately.

Status: resolved

## Question

Exactly which accepted Orcvs values, Function families, spatial behaviors, Tick behaviors, and
terminal outputs constitute the First Release Candidate, which are already satisfied by current
implementation, and which remain explicit deferrals? Produce one authoritative inventory that can
cross-check `CONTEXT.md`, ADR 0019, implementation tickets, and release evidence without treating
every term in the evolving glossary as automatically shipped.

## Answer

**Current state refreshed 2026-10-01 against `98bd5d0d`** (`v1-roadmap-wayfinding/08`). Earlier
refreshes were 2026-09-09 against `f131e5e` and the original column against `bbe882e`
(2026-09-01). The FRC contract column and the family contract sentences are unchanged. Every
Function named below is a row of `define_functions!` (`lang/src/atom.rs`). Each member's Evidence
names the tests that prove it as `file` `module::test`, so `v1-release/03`'s inventory-to-evidence
mapping can start from these tables. Evidence names positive behaviour and the principal refusals;
it is the starting point for that mapping, not a claim that every applicable proof the definition
of done asks for is already present.

The First Release Candidate ships the complete accepted numeric, Sequence, Tick, spatial, and MIDI
slice below. Release membership follows this inventory, not mere appearance in `CONTEXT.md` or ADR
0019. “Satisfied” means the current implementation substantially provides the release behavior;
“partial” and “missing” remain implementation work even where syntax or a legacy approximation
already exists.

One member is still short of the release: the Self-Banging Functions' collision rule, owned by
`spatial-tick-planning/07`. See [Members still short of the release](#members-still-short-of-the-release).

### Values and Source forms

| Inventory member | FRC contract | Current state | Evidence |
| --- | --- | --- | --- |
| Number | Contextual two-Cell uppercase hexadecimal `00`–`FF`; wrapping general arithmetic | Satisfied: exactly the uppercase two-Cell spellings parse, each Number has one canonical encoding, a literal without a Note spelling stays a Number, and general arithmetic wraps over every byte pair | `lang/src/lib.rs` `test::every_number_has_one_canonical_two_cell_source_encoding`, `test::non_canonical_number_source_encodings_diagnose`, `test::str_to_num_accepts_exactly_the_uppercase_two_cell_hexadecimal_spellings`; `lang/src/parser.rs` `test::conversion_operands_without_a_note_spelling_stay_numbers`; `lang/src/interpreter.rs` `test::general_arithmetic_wraps_for_every_pair_of_bytes` |
| Note | Contextual canonical pitch spelling `C/`–`G9`, carrying MIDI `00`–`7F` | Satisfied: conversion is algorithmic and total over `00`–`7F`, and every Note parses in context | `lang/src/lib.rs` `test::midi_note_to_number_accepts_exactly_the_pitch_spellings_inside_the_midi_range`, `test::every_midi_note_round_trips_through_its_two_cell_source_encoding`, `test::note_source_encoding_covers_the_documented_boundaries`, `test::values_above_the_midi_range_have_no_note_source_encoding`; `lang/src/parser.rs` `test::every_note_source_encoding_parses_in_context` |
| Bang `**` | One-Tick Atom with deterministic activation and expiry | Satisfied: activation at all four cardinal anchors, producer-before-consumer ordering, once-only execution, one-Tick display then expiry, inert manual Bangs, and parser-owned validity ship through the ADR 0032/0034 scheduler. The east and west anchors are driven (west by the Directional Bang Functions, `spatial-tick-planning/06`), and direct delivery ships through Self-Banging root contact and Jump relay. The former owner `spatial-tick-planning/02` is resolved | `orcvs/src/source/tick.rs` `test::a_bang_activates_its_aligned_neighbours_and_no_further_root`, `test::fixed_bang_destinations_respect_alignment_and_operand_contact`, `test::fixed_upward_portals_schedule_note_and_bang_before_midi`, `test::two_current_bang_results_still_execute_midi_once`, `test::an_active_directional_bang_function_emits_its_self_banging_function`, `test::a_relayed_bang_activates_a_root`; `orcvs/src/source/model.rs` `test::a_generated_pulse_does_not_replay_on_the_next_tick`, `test::manually_entered_bang_is_display_only_and_never_activates_midi`, `test::a_rejected_bang_operand_neither_activates_nor_erases`, `test::a_result_beside_a_root_preserves_the_root_and_activates_it_each_tick`; `orcvs/src/source/language_map.rs` `tests::invalid_operand_bang_spellings_are_not_parsed_bang_values` |
| Self-Banging Functions `^^`, `vv`, `<<`, `>>` | Root-only Source Functions with intrinsic Bang activation, one-Cell-per-Tick Portal writes, and collision-to-Bang behavior | Satisfied except the collision rule's statement. All four are root-only, intrinsically active Functions; each moves one Cell per Tick as one atomic bundle, stops at row and Grid edges as `**`, activates a completely contacted root, and diagnoses partial contact. The collision rule that ships was kept on 2026-09-24, but ADR 0006 does not yet state it and the odd-gap, three-mover and emission-against-mover cases have no regression test (`spatial-tick-planning/07`) | `orcvs/src/source/tick.rs` `test::a_self_banging_function_advances_its_whole_span_by_one_cell`, `test::a_self_banging_function_moves_once_per_tick_and_bangs_where_it_stops`, `test::a_move_that_leaves_the_grid_replaces_its_own_span_with_bang`, `test::an_eastward_move_stops_at_the_row_edge_rather_than_wrapping_into_the_next_row`, `test::a_westward_move_stops_at_the_row_edge_rather_than_wrapping_into_the_previous_row`, `test::two_moves_that_want_the_same_cells_each_bang_in_their_own_span`, `test::two_movers_reserving_each_other_each_bang_rather_than_costing_the_tick`, `test::a_move_blocked_by_one_complete_language_unit_bangs_without_diagnosing`, `test::a_move_that_lands_across_two_language_units_diagnoses_its_misalignment`, `test::complete_aligned_root_contact_activates_the_root_it_blocked_against`, `test::a_horizontally_aligned_root_is_contacted_two_columns_away`, `test::a_self_banging_function_is_not_scheduled_inside_another_expression`; `lang/src/parser.rs` `test::bang_and_self_banging_functions_parse_as_complete_language_units`; `lang/src/sequence.rs` `test::a_source_writing_function_is_rejected_as_a_member_and_through_promotion` |
| Sequence | Flat ordered, non-nesting Atom value; compatible Atomic Functions extend pervasively | Satisfied: the value is flat by construction, admits Bang members, and broadcasts pervasively. Source spells one through `:-` and `:#`, and a Sequence result reaches Source as one complete-fit write, or writes no Cell where it does not fit. The former owner `sequence-values/06` is resolved | `lang/src/sequence.rs` `test::sequence_preserves_atom_order_and_type`, `test::nesting_is_impossible_because_no_atom_variant_carries_a_sequence`, `test::bang_survives_construction_promotion_and_encoding`; `lang/src/stack.rs` `test::an_atom_operand_repeats_across_every_element_of_a_sequence_operand`, `test::two_non_scalar_operands_of_different_lengths_diagnose_and_build_no_sequence`; `orcvs/src/source/tick.rs` `test::live_a_sequence_result_reaches_its_destination_cells`, `test::live_a_sequence_result_that_leaves_its_row_writes_no_cell_of_it`, `test::live_a_sequence_result_at_a_grid_edge_writes_no_cell_of_it`; `orcvs/src/source/model.rs` `test::cells_generated_by_a_sequence_result_are_read_as_ordinary_source` |
| Comment | `\|\|` through row end; lone `\|` is incomplete or invalid | Satisfied: the Parser owns the Comment as one row-length Language Unit that is not a value (`sequence-values/07`, ADR 0035) | `lang/src/parser.rs` `test::a_comment_claims_the_rest_of_the_source_and_records_no_atom`, `test::a_lone_vertical_rule_is_not_a_comment`, `test::a_comment_introducer_inside_an_operand_claim_is_a_refused_operand`; `orcvs/src/source/language_map.rs` `tests::a_comment_forms_one_row_length_language_unit_that_is_not_a_value`; `orcvs/tests/language_map.rs` `a_comment_claims_the_row_after_the_expression_that_precedes_it` |

Function and Character implementation variants and `Empty` sentinels are not automatically shipped
language values. Bangs may inhabit Sequences: structural operations preserve, select, or replace
them, while only explicitly compatible Atomic Functions may evaluate them. Bangs remain invalid
Number or Note Range bounds.

### Numeric family

The FRC ships the complete accepted family: Addition `.+`, ordered Subtraction `.-`, Absolute
Difference `.|`, Multiplication `.x`, Division `./`, Modulo `.%`, Minimum `.<`, Maximum `.>`,
Equality `.=` and explicit Note-to-Number `.v` / Number-to-Note `.^` conversion.

All eleven are satisfied. The binary laws run exhaustively over every byte pair.

| Member | Current state | Evidence |
| --- | --- | --- |
| `.+`, `.-`, `.x`, `./` | Satisfied: wrapping byte arithmetic in signature order; a zero divisor diagnoses and commits nothing | `lang/src/interpreter.rs` `test::general_arithmetic_wraps_for_every_pair_of_bytes`, `test::division_is_the_asymmetry_that_diagnoses_every_zero_divisor`, `test::the_numeric_family_rejects_every_non_number_operand`; `lang/src/functions/math.rs` `test::the_non_commutative_functions_read_left_and_right_in_signature_order`; `orcvs/src/source/model.rs` `test::a_zero_divisor_diagnoses_and_commits_nothing` |
| `.\|` | Satisfied | `lang/src/interpreter.rs` `test::absolute_difference_is_symmetric_and_never_underflows`; `orcvs/src/source/tick.rs` `test::an_absolute_difference_beside_a_vertical_rule_is_read_in_two_cell_units` |
| `.%` | Satisfied | `lang/src/interpreter.rs` `test::modulo_returns_the_unsigned_remainder_for_every_non_zero_divisor`, `test::modulo_by_zero_diagnoses_distinctly_from_division_by_zero` |
| `.<`, `.>` | Satisfied | `lang/src/interpreter.rs` `test::minimum_and_maximum_select_one_of_their_operands_for_every_pair_of_bytes` |
| `.=` | Satisfied: answers one Bang only when every broadcast pair is equal | `lang/src/interpreter.rs` `test::equality_answers_a_bang_only_for_equal_numbers`; `lang/src/functions/math.rs` `test::equality_answers_one_bang_only_when_every_broadcast_pair_is_equal`; `orcvs/src/source/model.rs` `test::an_equal_comparison_commits_a_bang_and_an_unequal_one_commits_nothing` |
| `.v`, `.^` | Satisfied: total over the Note domain, refusing Numbers above `7F`, idempotent, and pervasive over Sequences | `lang/src/functions/numeric_conversion.rs` `test::conversion_to_note_is_the_identity_over_every_note_and_refuses_every_number_above_the_range`, `test::a_conversion_extends_atom_wise_and_preserves_order`, `test::evaluation_time_idempotence_survives_broadcasting`; `lang/src/interpreter.rs` `test::explicit_numeric_conversions_have_fixed_result_types`; `orcvs/src/source/model.rs` `test::conversions_are_idempotent_through_nested_source_expressions` |

### Sequence family

The FRC ships Number Range `:-`, Note Range `:#`, Reverse `:<`, Concatenate `:&`, Select `:?`, and
Replace `:=` with the accepted flat-Sequence behavior. Replace is a pure value operation; the
general Source Write that may follow it remains deferred. Complete-fit atomic Sequence result
delivery through Portals is release infrastructure, not a shipped Portal value.

All six are satisfied. Both prerequisites the 2026-09-09 column named are met: the executor plans a
Sequence result as one complete-fit write, and signatures name generic Atom and Sequence operands
(`AtomOrSequence`, `Atom`, `Sequence` in `define_functions!`).

| Member | Current state | Evidence |
| --- | --- | --- |
| `:-`, `:#` | Satisfied: inclusive, respecting bound order, Note identity preserved; mixed, Bang, Function and Sequence bounds diagnose | `lang/src/functions/sequence.rs` `test::number_range_is_inclusive_and_respects_bound_order`, `test::note_range_preserves_note_identity_and_respects_bound_order`, `test::number_range_spans_the_full_byte_without_wrapping`, `test::range_functions_diagnose_mixed_type_and_invalid_bounds`, `test::range_functions_diagnose_function_and_sequence_bounds`; `orcvs/src/source/tick.rs` `test::live_a_declared_number_range_result_reaches_its_destination_cells`, `test::live_a_declared_note_range_result_reaches_its_destination_cells` |
| `:<` | Satisfied | `lang/src/functions/sequence.rs` `test::reverse_preserves_atom_type_and_encoding`, `test::reverse_leaves_singleton_and_empty_sequences_unchanged`; `orcvs/src/source/tick.rs` `test::live_a_declared_reverse_result_reaches_its_destination_cells` |
| `:&` | Satisfied | `lang/src/functions/sequence.rs` `test::concatenate_promotes_atoms_stays_flat_and_treats_empty_as_identity`; `orcvs/src/source/tick.rs` `test::live_a_declared_concatenate_result_reaches_its_destination_cells` |
| `:?` | Satisfied: wrapping index, may answer a Bang member that activates an aligned root | `lang/src/functions/sequence.rs` `test::select_uses_a_wrapping_number_index_and_preserves_the_chosen_atom`, `test::select_diagnoses_empty_sequences_and_non_number_indices`, `test::select_declares_that_it_can_emit_bang_and_returns_a_bang_member`; `orcvs/src/source/tick.rs` `test::a_select_bang_activates_an_aligned_terminal_root` |
| `:=` | Satisfied | `lang/src/functions/sequence.rs` `test::replace_returns_a_new_same_length_sequence_and_allows_a_different_replacement_type`, `test::replace_diagnoses_empty_sequences_non_number_indices_and_sequence_replacements`; `orcvs/src/source/tick.rs` `test::live_a_declared_replace_result_reaches_its_destination_cells` |

Bang preservation across the structural operations: `lang/src/functions/sequence.rs`
`test::structural_operations_preserve_bang_type_and_encoding`.

### Tick and feedback family

The FRC ships Clock `~.`, Delay `~*`, visible Increment `~+`, deterministic Random `~?`, Euclidean
rhythm `~%`, and visible Interpolation `~>`, including the accepted Tick-zero, feedback, and
determinism boundaries.

All six are satisfied (`tick-functions/01`–`04`, all resolved). Each reads the absolute Tick and
its own anchor as explicit inputs; feedback is visible in its Portal Cells rather than hidden state.

| Member | Current state | Evidence |
| --- | --- | --- |
| `~.` | Satisfied | `lang/src/functions/tick.rs` `test::clock_counts_one_step_per_rate_ticks_and_wraps_at_its_modulus`, `test::clock_counts_a_cycle_of_more_ticks_than_a_byte_holds`, `test::clock_diagnoses_a_zero_rate_before_a_zero_modulus`, `test::a_clock_broadcasts_one_step_per_element` |
| `~*` | Satisfied: Bangs once per cycle from Tick zero | `lang/src/functions/tick.rs` `test::delay_bangs_once_per_cycle_beginning_at_the_first_tick`, `test::delay_counts_a_cycle_product_wider_than_a_byte`, `test::a_pulse_refuses_a_sequence_at_either_operand_position`; `orcvs/src/source/tick.rs` `test::a_pulse_activates_an_aligned_root_only_on_the_ticks_it_bangs` |
| `~%` | Satisfied | `lang/src/functions/tick.rs` `test::euclidean_places_its_hits_where_the_adr_formula_does`, `test::euclidean_answers_no_hits_and_a_full_cycle_from_the_same_formula`, `test::euclidean_refuses_a_cycle_of_no_steps_before_it_counts_its_hits` |
| `~+`, `~>` | Satisfied: an empty Portal reads as zero; the previous value is read from working Source | `lang/src/functions/tick.rs` `test::increment_advances_by_step_and_wraps_at_its_modulus`, `test::interpolation_moves_toward_target_without_overshoot`, `test::interpolation_holds_when_its_rate_is_zero`, `test::a_feedback_function_diagnoses_an_occupied_previous`; `orcvs/src/source/tick.rs` `test::an_empty_portal_initialises_increment_and_interpolation_as_zero`, `test::increment_and_interpolation_advance_across_two_ticks`, `test::increment_reads_a_live_edited_previous_from_working_source`, `test::a_previous_that_is_not_a_number_diagnoses_and_writes_nothing` |
| `~?` | Satisfied: seeded by the ADR's ChaCha8 derivation from seed, Tick, anchor and element index | `lang/src/functions/tick.rs` `test::random_golden_vectors_pin_the_adr_seed_chacha8_word_and_range_mapping`, `test::random_normalizes_reversed_bounds_and_returns_an_equal_bound`, `test::a_random_broadcasts_one_draw_per_element`; `orcvs/src/source/tick.rs` `test::moving_a_random_changes_its_stream_and_identical_inputs_reproduce_it`, `test::a_nested_random_uses_its_own_anchor_not_the_expression_root` |
| Tick-zero boundary | Satisfied: a run counts from Tick zero, and Tick Functions answer about the absolute Tick they are planned at | `lang/src/tick.rs` `test::a_playback_run_counts_from_tick_zero_by_ones`; `orcvs/src/playback.rs` `tests::playback_begins_at_the_first_tick_and_advances_one_per_executed_tick`; `orcvs/src/source/tick.rs` `test::the_tick_functions_answer_about_the_absolute_tick_they_are_planned_at`, `test::a_tick_function_with_no_cycle_diagnoses_and_writes_nothing` |

### Spatial and Tick-planning behavior

The FRC ships Directional Bang Functions `*^`, `*v`, `*<`, `*>`; Halt `*!`; and directional Jump
Functions `&^`, `&v`, `&<`, `&>`. It includes Bang routing and expiry, Activation movement and
collision, Source-order root turns, later-root same-Tick activation, Halt locking, directional Jump
chains over complete aligned Language Units, atomic writes, deterministic effect ordering, and
diagnostics. Jump does not transport a Sequence or partial Language Unit. ADR 0032 replaced
Source-order root turns with dependency order, where Position only breaks ties.

All are satisfied (`spatial-tick-planning/01`–`06` and `08`, all resolved) except the collision
rule's statement, which `spatial-tick-planning/07` owns. Bang routing and expiry are the Bang row
above; Activation movement is the Self-Banging row.

| Member | Current state | Evidence |
| --- | --- | --- |
| `*^`, `*v`, `*<`, `*>` | Satisfied: an active one emits its Self-Banging Function two Cells outside its Span, which first moves on the following Tick; an inert one emits nothing; a blocked emission diagnoses and writes no Cell | `orcvs/src/source/tick.rs` `test::an_active_directional_bang_function_emits_its_self_banging_function`, `test::an_inert_directional_bang_function_emits_nothing`, `test::a_refused_emission_diagnoses_and_writes_no_cell`, `test::relayed_bangs_reach_emission_refusals_at_the_right_and_top_edges`, `test::an_emitted_self_banging_function_first_moves_on_the_following_tick` |
| `*!` | Satisfied: an active Halt locks the complete root one row south before it runs, once per Tick; a suppressed Halt does not lock | `orcvs/src/source/tick.rs` `test::an_active_halt_locks_the_complete_root_one_row_south`, `test::an_inert_halt_does_not_lock_and_the_south_root_runs`, `test::a_suppressed_halt_does_not_lock_its_own_target`, `test::halt_is_interpreted_once_per_tick`, `test::an_occupied_non_root_halt_target_diagnoses`, `test::an_active_halt_withholds_a_terminal_root`, `test::multiple_halts_and_activations_follow_dependency_order`; `lang/src/interpreter.rs` `test::halt_locks_at_its_output_portal` |
| `&^`, `&v`, `&<`, `&>` | Satisfied: each relays one complete aligned Language Unit, chains compose, a relayed Bang activates a root, and a Sequence or partial unit is not transported | `orcvs/src/source/tick.rs` `test::each_jump_direction_relays_one_aligned_language_unit`, `test::consecutive_jumps_compose_through_overlapping_portals`, `test::a_jump_does_not_transport_a_sequence`, `test::a_jump_does_not_transport_an_incomplete_language_unit`, `test::a_partial_jump_input_diagnoses_and_writes_nothing`, `test::an_out_of_grid_jump_destination_diagnoses_and_writes_nothing`, `test::a_relayed_bang_activates_a_root`, `test::a_jump_below_its_consumer_reaches_it_the_same_tick`, `test::a_jump_that_closes_a_same_tick_cycle_rejects_the_tick`; `lang/src/functions/jump.rs` `test::a_portal_that_is_not_one_two_cell_unit_diagnoses` |
| Dependency-ordered turns, later-root same-Tick activation | Satisfied (ADR 0032) | `orcvs/src/source/tick.rs` `test::the_schedule_orders_a_move_ahead_of_the_computation_it_would_land_on`, `test::fixed_upward_portals_schedule_note_and_bang_before_midi`, `test::live_cycles_reject_independent_effects_and_self_dependency`; `orcvs/src/source/model.rs` `test::a_later_calculation_reads_an_operand_written_this_tick` |
| Atomic writes, conflicts, effect ordering | Satisfied: a write that leaves the Grid emits no partial write, later effects win each Cell independently, and commands and diagnostics keep producer order. Mover collisions: see `spatial-tick-planning/07` | `orcvs/src/source/tick.rs` `test::test_a_write_whose_destination_leaves_the_grid_emits_no_partial_write`, `test::test_later_effects_win_cell_conflicts_independently`, `test::test_play_commands_and_diagnostics_keep_producer_and_emission_order`, `property::a_tick_plan_gives_each_cell_to_the_last_admitted_write_covering_it`; `orcvs/src/source/model.rs` `test::test_writes_play_commands_and_diagnostics_follow_one_producer_order` |

### MIDI terminal-output family

The FRC ships all five accepted MIDI forms:

- Raw Play `!>` with channel `00`–`0F`, velocity `00`–`7F`, and typed Note.
- Timed Play `!~` with explicit length, zero rules, and Note Off at Tick `T + length`.
- Monophonic Play `!%` with one Playback-Engine-owned voice per MIDI channel.
- Control Change `!c` with direct controller/value data bytes.
- Pitch Bend `!b` with direct LSB then MSB data bytes.

All five are satisfied, and each extends pervasively over Sequence operands
(`midi-output-family/01`–`06`, all resolved). Bang expiry ships with the Bang row. The safety
action sends All Notes Off (CC 123), Reset All Controllers (CC 121) and a centred bend on every
channel, so the residual the 2026-09-09 column gave `midi-output-family/06` is closed. Both targets
have a backend: native through the platform MIDI service, and the browser through Web MIDI (#185,
ADR 0059), which requests access once, answers discovery and connect synchronously from the access
it holds, and opens a port asynchronously before installing it.

| Member | Current state | Evidence |
| --- | --- | --- |
| `!>` | Satisfied: typed channel, velocity and Note; inactive roots send nothing | `lang/src/functions/mod.rs` `test::play_carries_each_operand_into_the_role_its_signature_names`, `test::play_rejects_channels_outside_the_midi_range`, `test::play_rejects_velocities_outside_the_midi_data_byte_range`, `test::play_rejects_implicit_number_note_conversions`; `orcvs/src/source/model.rs` `test::test_root_play_function_emits_one_play_command_without_a_cell_write`, `test::test_play_preserves_zero_velocity_as_an_explicit_command`; `orcvs/src/midi.rs` `tests::submits_commands_as_ordered_note_on_messages` |
| `!~` | Satisfied: Note Off at `T + length`; velocity zero and length zero follow the zero rules | `lang/src/functions/mod.rs` `test::timed_play_takes_the_same_midi_domains_as_raw_play_and_a_whole_byte_of_length`; `orcvs/src/playback/schedule.rs` `tests::a_timed_play_starts_in_tick_plan_order_and_stops_at_the_tick_its_length_names`, `tests::a_timed_play_with_velocity_zero_stops_the_note_and_schedules_nothing`, `tests::a_timed_play_with_no_length_emits_nothing_and_leaves_the_note_it_finds_standing`, `tests::a_repeated_timed_play_stops_the_instance_it_replaces_and_retires_its_expiry`; `orcvs/src/source/model.rs` `test::test_root_timed_play_function_emits_one_command_carrying_its_whole_lifetime` |
| `!%` | Satisfied: one voice per channel, owned by the Playback Engine | `orcvs/src/playback/schedule.rs` `tests::a_monophonic_play_stops_whatever_note_its_channel_was_sounding`, `tests::a_mono_voice_is_owned_per_channel_and_channels_do_not_steal_from_one_another`, `tests::a_stale_mono_expiry_cannot_stop_the_voice_claimed_after_it`, `tests::timed_and_mono_own_separately_and_neither_owns_a_raw_note`; `orcvs/src/source/model.rs` `test::test_root_monophonic_play_function_emits_one_command_of_its_own_kind` |
| `!c`, `!b` | Satisfied: data bytes reach the wire unaltered, LSB before MSB | `lang/src/functions/mod.rs` `test::every_data_byte_reaches_a_control_change_or_pitch_bend_command_unaltered`, `test::control_change_and_pitch_bend_reject_a_note_in_every_operand_position`; `orcvs/src/midi.rs` `tests::submits_control_change_and_pitch_bend_as_their_wire_bytes`; `orcvs/src/playback/schedule.rs` `tests::control_change_and_pitch_bend_are_delivered_unresolved_and_in_tick_plan_order`; `orcvs/src/source/model.rs` `test::test_root_control_change_and_pitch_bend_emit_the_command_their_operands_name` |
| Sequence operands | Satisfied | `lang/src/stack.rs` `test::a_sequence_at_any_operand_position_answers_one_command_per_element_in_order`, `test::a_control_change_and_a_bend_widen_at_every_data_byte_position`; `orcvs/src/playback/schedule.rs` `tests::a_chord_of_timed_plays_stops_each_element_at_its_own_length` |
| Safety action and device lifecycle | Satisfied: CC 123, CC 121 and a centred bend on every channel at stop, delivery failure and destination change; silencing clears the note schedule | `orcvs/src/midi.rs` `tests::the_safety_action_clears_notes_controllers_and_bend_on_every_channel`, `tests::delivery_failure_attempts_the_safety_action_and_reselection_reconnects`, `tests::a_destination_change_sends_the_safety_action_to_the_destination_it_leaves`; `orcvs/src/playback.rs` `tests::every_lifecycle_action_that_silences_output_clears_the_note_schedule` |
| Browser Web MIDI backend | Satisfied in automated tests (#185, ADR 0059): pending, granted and unavailable access; exact bytes to the opened output; failed and abandoned opens. Manual browser checks remain open in `midi-port-ownership/06`, `09` and `10` | `console/src/web_midi.rs` `tests::a_pending_request_answers_that_access_is_awaited`, `tests::granted_access_lists_the_connected_outputs`, `tests::a_browser_without_midi_access_offers_an_empty_list_rather_than_an_error`, `tests::the_frame_after_the_browser_answers_catches_up_without_a_scan`, `tests::playback_delivers_exact_bytes_to_the_output_the_console_opened`, `tests::an_output_that_fails_to_open_leaves_the_playing_destination`, `tests::an_abandoned_open_releases_its_output`; `console/tests/wasm.rs` `web_midi_answers_discovery_and_connect_without_waiting`, `midi_output::the_browser_output_control_is_enabled_and_offers_scan` |

### Members still short of the release

- **Self-Banging collision rule** — `spatial-tick-planning/07` (`Status: ready-for-agent`,
  `release/v1`). The behaviour ships and the rule was kept on 2026-09-24. ADR 0006 must state it,
  and tests must cover the odd gap (`">> <<"`), three or more converging movers, and a Directional
  Bang emission contesting a mover.

Every other member's Current state is satisfied. Release proof that is not a member's state stays
with its own owners: native physical MIDI evidence with `v1-release/04`, the browser MIDI manual
checks with `midi-port-ownership/06`, `09` and `10` (each `ready-for-human`, `release/v1`), and the
candidate-wide evidence run with `v1-release/03`.

### Explicit deferrals and omissions

- General Source Read `@<` and Source Write `@>` addressing, Generator composition, and visible
  Konkat-style reads. Directional Jump is the only shipped Address subset.
- UDP `!u`, OSC `!o`, and their text/message values and transports.
- Application Command `!$` and its command value encoding.
- Cross-version Source, persistence-format, and Rust-interface compatibility while Orcvs remains
  pre-release.
- Improvement-only maintenance that is not required for correctness, safe implementation, or
  measured release evidence.
- Hidden-variable behavior is omitted rather than deferred; Identity Test is retired.

The Language Map, compiler-checked Function definitions, typed operand extraction, canonical Source
generation, Tick planning, and target evidence are prerequisites for proving this inventory, but
they are infrastructure or proof work rather than additional shipped language members.
