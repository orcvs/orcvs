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

**Current state refreshed 2026-10-13 against `cf01e3c`.** Earlier refreshes were 2026-10-01
against `98bd5d0d` (`v1-roadmap-wayfinding/08`), 2026-09-09 against `f131e5e`, and the original
column against `bbe882e` (2026-09-01). This refresh follows the accepted ADRs that changed members
since 2026-10-01: ADR 0063 retired the Sequence value and its Functions, ADR 0069 made an unwritten
operand invalid input, ADR 0070 split the Read, Write and Copy families (the directional Jumps are
now the directional Copies `=^ =v =< =>`, and `&` spells a Read), and ADR 0072 made a standalone
`**` the Bang Function. The FRC contract column changes only where one of those ADRs changed the
member. Every Function named below is a row of `define_functions!` (`lang/src/atom.rs:727-781`,
48 spelled rows and the named `**`). Each member's Evidence names the tests that prove it as `file`
`module::test`, so `v1-release/03`'s inventory-to-evidence mapping can start from these tables.
Evidence names positive behaviour and the principal refusals; it is the starting point for that
mapping, not a claim that every applicable proof the definition of done asks for is already present.

The First Release Candidate ships the complete accepted numeric, Tick, spatial, Read/Write/Copy and
MIDI slice below. Release membership follows this inventory, not mere appearance in `CONTEXT.md` or
ADR 0019. “Satisfied” means the current implementation substantially provides the release behavior;
“partial” and “missing” remain implementation work even where syntax or a legacy approximation
already exists.

Members whose Current state is still short of the release are listed under
[Members still short of the release](#members-still-short-of-the-release).

### Values and Source forms

| Inventory member | FRC contract | Current state | Evidence |
| --- | --- | --- | --- |
| Number | Contextual two-Cell uppercase hexadecimal `00`–`FF`; wrapping general arithmetic | Satisfied: exactly the uppercase two-Cell spellings parse, each Number has one canonical encoding, a literal without a Note spelling stays a Number, and general arithmetic wraps over every byte pair | `lang/src/lib.rs` `test::every_number_has_one_canonical_two_cell_source_encoding`, `test::non_canonical_number_source_encodings_diagnose`, `test::str_to_num_accepts_exactly_the_uppercase_two_cell_hexadecimal_spellings`; `lang/src/parser.rs` `test::conversion_operands_without_a_note_spelling_stay_numbers`; `lang/src/interpreter.rs` `test::general_arithmetic_wraps_for_every_pair_of_bytes` |
| Note | Contextual canonical pitch spelling `C/`–`G9`, carrying MIDI `00`–`7F` | Satisfied: conversion is algorithmic and total over `00`–`7F`, and every Note parses in context | `lang/src/lib.rs` `test::midi_note_to_number_accepts_exactly_the_pitch_spellings_inside_the_midi_range`, `test::every_midi_note_round_trips_through_its_two_cell_source_encoding`, `test::note_source_encoding_covers_the_documented_boundaries`, `test::values_above_the_midi_range_have_no_note_source_encoding`; `lang/src/parser.rs` `test::every_note_source_encoding_parses_in_context` |
| Bang `**` | One-Tick Atom with deterministic activation and expiry; a standalone `**` is the Bang Function (ADR 0072) | Satisfied: a Function that answers Bang writes `**` and activates the roots aligned with it in its own Turn, at all four cardinal anchors, producer before consumer, once per Tick. The `**` it leaves is Source and a Bang Function on the next Tick: root-only and intrinsically active, its Turn activates its aligned roots and clears its two Cells, whether it was typed, written on the previous Tick or read from a Source File. An overwrite before its Turn suppresses it, a Halt locks it, and nothing about Bangs is remembered between Ticks. A blocked mover's `**` fires on the next Tick; root contact activates nothing (ADR 0072). In an untyped operand `**` is the Bang value; in a typed operand it is invalid syntax and activates nothing | `orcvs/src/source/tick.rs` `test::a_bang_activates_its_aligned_neighbours_and_no_further_root`, `test::fixed_bang_destinations_respect_alignment_and_operand_contact`, `test::fixed_upward_portals_schedule_note_and_bang_before_midi`, `test::two_current_bang_results_still_execute_midi_once`, `test::a_copied_bang_activates_a_neighbour_of_empty_source`; `orcvs/src/source/tick/bang_function.rs` `bang_function::a_typed_bang_activates_the_roots_on_all_four_sides`, `bang_function::a_typed_bang_plays_once_on_the_next_tick_and_is_cleared`, `bang_function::a_writer_over_a_typed_bang_suppresses_it`, `bang_function::a_partial_overwrite_suppresses_the_whole_bang_and_the_stray_star_is_no_bang`, `bang_function::identical_snapshots_at_the_same_tick_plan_the_same_tick`; `orcvs/src/source/model.rs` `test::a_generated_pulse_does_not_replay_on_the_next_tick`, `test::a_typed_bang_activates_the_root_above_it_once_and_is_cleared`, `test::a_bang_read_from_a_source_file_fires_once_on_the_first_tick`, `test::a_rejected_bang_operand_neither_activates_nor_erases`, `test::a_result_beside_a_root_preserves_the_root_and_activates_it_each_tick`; `orcvs/src/source/language_map.rs` `tests::invalid_operand_bang_spellings_are_not_parsed_bang_values`; `lang/src/parser.rs` `test::a_bang_in_a_typed_operand_is_still_invalid_syntax` |
| Self-Banging Functions `^^`, `vv`, `<<`, `>>` | Root-only Source Functions with intrinsic Bang activation, one-Cell-per-Tick Portal writes, and collision-to-Bang behavior | Satisfied. All four are root-only, intrinsically active Functions; each moves one Cell per Tick as one atomic bundle and stops at row and Grid edges as `**`, which fires as a Bang Function on the next Tick (ADR 0072 removed contact activation). Partial contact diagnoses. ADR 0006 states the kept collision rule, and the odd-gap, converging-mover and emission-against-mover cases are pinned (`spatial-tick-planning/07`, resolved 2026-10-02) | `orcvs/src/source/tick.rs` `test::a_self_banging_function_advances_its_whole_span_by_one_cell`, `test::a_self_banging_function_moves_once_per_tick_and_bangs_where_it_stops`, `test::a_move_that_leaves_the_grid_replaces_its_own_span_with_bang`, `test::an_eastward_move_stops_at_the_row_edge_rather_than_wrapping_into_the_next_row`, `test::a_westward_move_stops_at_the_row_edge_rather_than_wrapping_into_the_previous_row`, `test::two_moves_that_want_the_same_cells_each_bang_in_their_own_span`, `test::two_movers_reserving_each_other_each_bang_rather_than_costing_the_tick`, `test::an_odd_gap_leaves_the_earlier_mover_the_cell_and_the_later_one_bangs`, `test::converging_movers_resolve_pairwise_in_source_order`, `test::an_emission_and_a_mover_wanting_one_cell_follow_source_order`, `test::a_move_blocked_by_one_complete_language_unit_bangs_without_diagnosing`, `test::a_move_that_lands_across_two_language_units_diagnoses_its_misalignment`, `test::complete_aligned_root_contact_activates_nothing_until_the_bang_it_leaves_fires`, `test::a_horizontally_blocked_move_plays_the_root_two_columns_away_on_the_next_tick`, `test::a_self_banging_function_is_not_scheduled_inside_another_expression`; `lang/src/parser.rs` `test::bang_and_self_banging_functions_parse_as_complete_language_units` |
| Comment | `\|\|` through row end; lone `\|` is incomplete or invalid | Satisfied: the Parser owns the Comment as one row-length Language Unit that is not a value (`sequence-values/07`, ADR 0035) | `lang/src/parser.rs` `test::a_comment_claims_the_rest_of_the_source_and_records_no_atom`, `test::a_lone_vertical_rule_is_not_a_comment`, `test::a_comment_introducer_inside_an_operand_claim_is_a_refused_operand`; `orcvs/src/source/language_map.rs` `tests::a_comment_forms_one_row_length_language_unit_that_is_not_a_value`; `orcvs/tests/language_map.rs` `a_comment_claims_the_row_after_the_expression_that_precedes_it` |

Function implementation variants and the `Empty` Atom are not shipped language values. `Empty` is
the Absence Marker: an answer with no Source encoding that no Function takes as an operand (ADR
0069). Orcvs has no Sequence value (ADR 0063): a pattern is written into Source Cells, and a chord is
several Terminal Output roots one Bang activates. An operand slot whose Cells are all empty is
invalid input, handled as a partly written slot is (ADR 0069).

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
| `.=` | Satisfied: answers Bang for equal Numbers, writing `**`, and the Absence Marker for unequal ones, clearing its Output Portal's two Cells, root or nested | `lang/src/interpreter.rs` `test::equality_answers_a_bang_only_for_equal_numbers`; `lang/src/functions/math.rs` `test::equality_answers_a_bang_or_the_absence_marker`; `orcvs/src/source/tick.rs` `test::an_unequal_root_equality_clears_the_pair_south_and_an_equal_one_bangs_there`; `orcvs/src/source/model.rs` `test::an_equal_comparison_commits_a_bang_and_an_unequal_one_clears_its_pair` |
| `.v`, `.^` | Satisfied: monomorphic. `.v` reads a Note and `.^` a Number `00`–`7F`, refusing `80`–`FF`; each refuses a value of the type it answers, so conversions compose only in opposite directions (`.v.^3C`, `.^.vC4`) and `.v.vC4` diagnoses | `lang/src/functions/numeric_conversion.rs` `test::conversion_to_note_refuses_every_number_above_the_range`, `test::each_conversion_refuses_a_value_of_the_type_it_answers`, `test::a_conversion_over_one_atom_answers_one_atom`, `test::nested_conversions_compose_and_read_their_return_by_the_receiving_operand`; `lang/src/interpreter.rs` `test::explicit_numeric_conversions_have_fixed_result_types`; `orcvs/src/source/model.rs` `test::nested_conversions_compose_only_where_the_return_spells_the_receiving_type` |

### Read, Write and Copy families

The FRC ships the directional Copies `=^`, `=v`, `=<`, `=>` (the renamed directional Jumps) and
the absolute Copy `=$`; the directional Reads `&^`, `&v`, `&<`, `&>`, the absolute Read `&$` and
Track `&t`; and the directional Writes `@^`, `@v`, `@<`, `@>`, the absolute Write `@$` and Push `@t`
(ADR 0067, ADR 0070).
The family prefix names what a Function does to Source: `&` reads, `@` writes and `=` copies.
A Read waits on every writer of the pair it selects that has not taken its Turn; a Write and the
absolute Copy take their Turn before every computation that does not feed them; a pass-through
carries `**` as a Bang. A Read nested in a Write's `value` is Orca's Generator, and Orca's Read,
Query and Konkat lookups are Reads of visible Source.

All six rows are satisfied.

| Member | Current state | Evidence |
| --- | --- | --- |
| `=^`, `=v`, `=<`, `=>` | Satisfied: each copies one complete aligned two-Cell Language Unit from the Portal opposite its output (east and west two columns, north and south one row); chains compose; empty input clears the destination and, nested, makes its parent invalid; partial input and an out-of-Grid destination diagnose and write nothing; a copied `**` overwrites its destination and activates the roots aligned with it; a partial Language Unit is not transported | `orcvs/src/source/tick.rs` `test::each_copy_direction_relays_one_aligned_language_unit`, `test::consecutive_copies_compose_through_overlapping_portals`, `test::an_empty_copy_input_clears_the_destination`, `test::a_partial_copy_input_diagnoses_and_writes_nothing`, `test::a_copy_does_not_transport_an_incomplete_language_unit`, `test::an_out_of_grid_copy_destination_diagnoses_and_writes_nothing`, `test::a_copied_bang_overwrites_a_roots_anchor_and_activates_the_roots_aligned_with_it`, `test::a_copy_below_its_consumer_reaches_it_the_same_tick`, `test::a_copy_that_closes_a_same_tick_cycle_is_diagnosed`; `lang/src/functions/copy.rs` `test::a_portal_that_is_not_one_two_cell_unit_diagnoses`, `test::bang_input_answers_bang` |
| `=$` | Satisfied: reads the pair at its source Position and writes it at its destination Position in one Turn, so an overlapping source and destination form no cycle; a source or destination outside the Grid or cut short diagnoses and writes nothing; nested, it returns the pair | `orcvs/src/source/tick/absolute_copy.rs` `absolute_copy::an_absolute_copy_writes_the_source_pair_at_the_destination`, `absolute_copy::an_absolute_copy_writes_what_a_write_of_a_read_writes`, `absolute_copy::an_empty_source_clears_the_destination`, `absolute_copy::a_source_or_destination_outside_the_grid_or_cut_short_diagnoses_and_writes_nothing`, `absolute_copy::a_source_that_is_the_destination_is_rewritten_unchanged`, `absolute_copy::readers_of_the_destination_see_the_write_the_same_tick`, `absolute_copy::a_nested_absolute_copy_returns_the_pair_it_copies`; `lang/src/functions/copy.rs` `test::an_absolute_copy_answers_what_a_directional_copy_answers_for_the_same_cells` |
| `&^`, `&v`, `&<`, `&>` | Satisfied: each reads the pair `n` Portals from its `n` operand in its arrow's direction, `00` being the operand itself; a nested `n` counts from the slot it occupies; it waits on the selected pair's writers, writes what it reads one row south and, nested, returns it; an unwritten operand makes it invalid; a pair outside the Grid or cut short diagnoses | `orcvs/src/source/tick/read.rs` `read::a_distance_counts_portals_from_the_n_operand`, `read::every_read_at_distance_zero_answers_its_own_operand`, `read::north_and_south_count_rows_in_the_operand_columns`, `read::a_nested_n_counts_from_the_slot_it_occupies`, `read::a_read_answers_what_a_copy_answers_for_the_cells_it_reads`, `read::an_empty_or_partially_written_operand_makes_the_read_invalid`, `read::a_pair_cut_short_by_the_row_edge_diagnoses_and_writes_nothing`, `read::a_pair_outside_the_grid_in_any_direction_diagnoses_and_writes_nothing`, `read::a_writer_below_the_selected_pair_takes_its_turn_first`, `read::a_writer_that_waits_on_the_read_forms_a_cycle_and_the_rest_of_the_tick_runs`, `read::a_nested_read_returns_its_pair_and_writes_its_own_output_portal`; `lang/src/functions/read.rs` `test::a_read_counts_the_distance_its_n_operand_states`; `lang/src/parser.rs` `test::the_ampersand_arrows_parse_as_reads_of_one_number` |
| `&$` | Satisfied: reads the pair at a column and row; it never waits on its own write | `orcvs/src/source/tick/read.rs` `read::an_absolute_read_of_00_00_reads_the_top_left_wherever_it_stands`, `read::an_absolute_read_counts_columns_in_cells`, `read::an_absolute_read_outside_the_grid_diagnoses_and_writes_nothing`, `read::an_absolute_read_of_its_own_cells_reads_them_without_waiting_on_itself`, `read::a_nested_absolute_read_returns_its_pair`; `lang/src/functions/read.rs` `test::an_absolute_read_addresses_column_then_row` |
| `&t` | Satisfied: reads the pair `index % count` pairs east of its last operand and claims only its operands, so the pairs are ordinary Source; a `count` of `00` diagnoses at its Turn and so does a pair past the row edge; an empty pair clears its Output Portal and, nested, makes its parent invalid | `orcvs/src/source/tick/track.rs` `track::track_writes_the_pair_its_operands_select_south`, `track::a_zero_count_diagnoses_at_the_turn`, `track::a_pair_past_the_row_edge_diagnoses_as_a_copy_input_outside_the_grid_does`, `track::a_clock_driven_track_holds_each_pair_for_the_rate_and_wraps_at_the_count`, `track::a_writer_below_the_selected_pair_takes_its_turn_first`, `track::a_writer_that_waits_on_track_forms_a_cycle_and_the_rest_of_the_tick_runs`, `track::all_empty_pairs_clear_the_output_portal_without_a_diagnostic`, `track::a_nested_track_returns_its_pair_and_writes_its_own_output_portal`, `track::a_timed_play_plays_the_note_a_nested_track_supplies`; `orcvs/src/source/tick.rs` `test::a_banged_play_whose_nested_track_copies_an_empty_pair_is_invalid`; `lang/src/functions/track.rs` `test::track_selects_index_modulo_count`, `test::a_zero_count_selects_no_pair` |
| `@^`, `@v`, `@<`, `@>`, `@$` | Satisfied: each writes its untyped `value` unchanged into the pair the Read with the same arrow, or the absolute Read, selects; readers it does not feed see the write the same Tick; a `**` value is written as a Bang and activates the roots aligned with it; a destination outside the Grid or cut short diagnoses and writes nothing; nested, it returns `value` | `orcvs/src/source/tick/write.rs` `write::a_write_answers_its_value_at_its_pair_and_writes_nothing_south`, `write::a_value_is_written_as_it_is_spelled`, `write::a_write_addresses_the_pair_the_read_with_the_same_arrow_reads`, `write::a_bang_in_value_is_written_as_a_bang_and_activates_the_root_below_it`, `write::a_destination_outside_the_grid_or_cut_short_diagnoses_and_writes_nothing`, `write::a_static_reader_north_of_the_write_sees_it_the_same_tick`, `write::a_reader_that_feeds_the_write_sees_it_on_the_next_tick`, `write::a_nested_write_returns_its_value_and_writes_its_pair`, `write::an_empty_or_partially_written_operand_makes_the_write_invalid`, `write::a_read_nested_in_a_write_copies_one_pair_to_another_place`; `lang/src/functions/write.rs` `test::a_write_answers_the_value_it_carries_unchanged`, `test::a_write_refuses_an_absence_marker_as_its_value`; `lang/src/parser.rs` `test::a_writes_value_slot_carries_any_language_unit_a_literal_spells` |
| `@t` | Satisfied: writes `value` into the pair `index % count` pairs east of its default Output Portal, a lane of `count` pairs on the row below it that is ordinary Source; a zero `count` diagnoses at its Turn | `orcvs/src/source/tick/push.rs` `push::push_writes_its_value_into_the_selected_pair_of_the_lane_below_it`, `push::a_bang_pushed_overwrites_its_whole_pair_and_activates_the_roots_aligned_with_it`, `push::a_zero_count_diagnoses_at_the_turn`, `push::a_pair_past_the_row_edge_or_a_lane_below_the_grid_diagnoses_and_writes_nothing`, `push::an_empty_or_partially_written_operand_makes_push_invalid`, `push::a_nested_push_returns_its_value_to_its_parent`, `push::a_track_whose_list_is_the_lane_reads_what_push_writes_in_the_same_tick` |

### Tick and feedback family

The FRC ships Clock `~.`, Delay `~*`, visible Increment `~+`, deterministic Random `~?`, Euclidean
rhythm `~%`, and visible Interpolation `~>`, including the accepted Tick-zero, feedback, and
determinism boundaries.

All six are satisfied (`tick-functions/01`–`04`, all resolved). Each reads the absolute Tick and
its own anchor as explicit inputs; feedback is visible in its Portal Cells rather than hidden state.

| Member | Current state | Evidence |
| --- | --- | --- |
| `~.` | Satisfied | `lang/src/functions/tick.rs` `test::clock_counts_one_step_per_rate_ticks_and_wraps_at_its_modulus`, `test::clock_counts_a_cycle_of_more_ticks_than_a_byte_holds`, `test::clock_counts_a_tick_far_beyond_the_byte_the_step_is_answered_as`, `test::clock_diagnoses_a_zero_rate_before_a_zero_modulus` |
| `~*` | Satisfied: Bangs once per cycle from Tick zero and clears its Output Portal on every other Tick, root or nested | `lang/src/functions/tick.rs` `test::delay_bangs_once_per_cycle_beginning_at_the_first_tick`, `test::delay_with_modulus_one_bangs_once_per_rate_ticks_rather_than_every_tick`, `test::delay_counts_a_cycle_product_wider_than_a_byte`; `orcvs/src/source/tick.rs` `test::a_pulse_activates_an_aligned_root_only_on_the_ticks_it_bangs`, `test::delay_and_euclidean_clear_their_output_portal_on_every_tick_they_do_not_bang` |
| `~%` | Satisfied: clears its Output Portal on every Tick it does not Bang | `lang/src/functions/tick.rs` `test::euclidean_places_its_hits_where_the_adr_formula_does`, `test::euclidean_answers_no_hits_and_a_full_cycle_from_the_same_formula`, `test::euclidean_refuses_a_cycle_of_no_steps_before_it_counts_its_hits`; `orcvs/src/source/tick.rs` `test::delay_and_euclidean_clear_their_output_portal_on_every_tick_they_do_not_bang` |
| `~+`, `~>` | Satisfied: an empty Portal reads as zero; the previous value is read from working Source; state survives an unwritten operand because it is the Cells they wrote | `lang/src/functions/tick.rs` `test::increment_advances_by_step_and_wraps_at_its_modulus`, `test::interpolation_moves_toward_target_without_overshoot`, `test::interpolation_holds_when_its_rate_is_zero`, `test::a_feedback_function_diagnoses_an_occupied_previous`; `orcvs/src/source/tick.rs` `test::an_empty_portal_initialises_increment_and_interpolation_as_zero`, `test::increment_and_interpolation_advance_across_two_ticks`, `test::increment_reads_a_live_edited_previous_from_working_source`, `test::increment_and_interpolation_keep_their_state_across_an_unwritten_operand`, `test::a_previous_that_is_not_a_number_diagnoses_and_writes_nothing` |
| `~?` | Satisfied: seeded by the ADR's ChaCha8 derivation from seed, Tick and anchor | `lang/src/functions/tick.rs` `test::random_golden_vectors_pin_the_adr_seed_chacha8_word_and_range_mapping`, `test::random_normalizes_reversed_bounds_and_returns_an_equal_bound`, `test::two_randoms_at_different_anchors_differ_and_moving_one_changes_its_stream`; `orcvs/src/source/tick.rs` `test::moving_a_random_changes_its_stream_and_identical_inputs_reproduce_it`, `test::a_nested_random_uses_its_own_anchor_not_the_expression_root`, `test::two_randoms_in_one_expression_write_different_numbers` |
| Tick-zero boundary | Satisfied: a run counts from Tick zero, and Tick Functions answer about the absolute Tick they are planned at | `lang/src/tick.rs` `test::a_playback_run_counts_from_tick_zero_by_ones`; `orcvs/src/playback.rs` `tests::playback_begins_at_the_first_tick_and_advances_one_per_executed_tick`; `orcvs/src/source/tick.rs` `test::the_tick_functions_answer_about_the_absolute_tick_they_are_planned_at`, `test::a_tick_function_with_no_cycle_diagnoses_and_writes_nothing` |

### Spatial and Tick-planning behavior

The FRC ships Directional Bang Functions `*^`, `*v`, `*<`, `*>`; Halt `*!`; and the Bang Function
`**`. It includes Bang routing and expiry, Activation movement and collision, dependency-ordered root
Turns with Position breaking ties (ADR 0032), later-root same-Tick activation, Halt locking, Copy
chains over complete aligned Language Units (see the Read, Write and Copy families), atomic writes,
deterministic effect ordering, and diagnostics. A cycle stops only the Expressions it reaches
(ADR 0068).

All are satisfied (`spatial-tick-planning/01`–`08`, all resolved). Bang routing and expiry are the
Bang row above; Activation movement and collision are the Self-Banging row.

| Member | Current state | Evidence |
| --- | --- | --- |
| `*^`, `*v`, `*<`, `*>` | Satisfied: an active one emits its Self-Banging Function two Cells outside its Span, which first moves on the following Tick; an inert one emits nothing; a blocked emission diagnoses and writes no Cell | `orcvs/src/source/tick.rs` `test::an_active_directional_bang_function_emits_its_self_banging_function`, `test::an_inert_directional_bang_function_emits_nothing`, `test::a_refused_emission_diagnoses_and_writes_no_cell`, `test::copied_bangs_reach_emission_refusals_at_the_right_and_top_edges`, `test::an_emitted_self_banging_function_first_moves_on_the_following_tick` |
| `*!` | Satisfied: an active Halt locks the complete root one row south before it runs, once per Tick, including a Bang Function; a suppressed Halt does not lock | `orcvs/src/source/tick.rs` `test::an_active_halt_locks_the_complete_root_one_row_south`, `test::an_inert_halt_does_not_lock_and_the_south_root_runs`, `test::a_suppressed_halt_does_not_lock_its_own_target`, `test::halt_is_interpreted_once_per_tick`, `test::an_occupied_non_root_halt_target_diagnoses`, `test::an_active_halt_withholds_a_terminal_root`, `test::an_active_halt_locks_a_bang_function`, `test::multiple_halts_and_activations_follow_dependency_order`; `lang/src/interpreter.rs` `test::halt_locks_at_its_output_portal` |
| Dependency-ordered turns, later-root same-Tick activation | Satisfied (ADR 0032; a cycle stops only what it reaches, ADR 0068) | `orcvs/src/source/tick.rs` `test::the_schedule_orders_a_move_ahead_of_the_computation_it_would_land_on`, `test::fixed_upward_portals_schedule_note_and_bang_before_midi`, `test::live_cycles_stop_only_the_expressions_they_reach`; `orcvs/src/source/tick/ordering.rs` `ordering::a_cycle_stops_every_expression_it_reaches_and_what_waits_on_it`; `orcvs/src/source/model.rs` `test::a_later_calculation_reads_an_operand_written_this_tick` |
| Atomic writes, conflicts, effect ordering | Satisfied: a write that leaves the Grid emits no partial write, later effects win each Cell independently, and commands and diagnostics keep producer order. Mover collisions: see the Self-Banging row | `orcvs/src/source/tick.rs` `test::test_a_write_whose_destination_leaves_the_grid_emits_no_partial_write`, `test::test_later_effects_win_cell_conflicts_independently`, `test::test_play_commands_and_diagnostics_keep_producer_and_emission_order`, `property::a_tick_plan_gives_each_cell_to_the_last_admitted_write_covering_it`; `orcvs/src/source/model.rs` `test::test_writes_play_commands_and_diagnostics_follow_one_producer_order` |

### MIDI terminal-output family

The FRC ships all five accepted MIDI forms:

- Raw Play `!>` with channel `00`–`0F`, velocity `00`–`7F`, and typed Note.
- Timed Play `!~` with explicit length, zero rules, and Note Off at Tick `T + length`.
- Monophonic Play `!%` with one Playback-Engine-owned voice per MIDI channel.
- Control Change `!c` with direct controller/value data bytes.
- Pitch Bend `!b` with direct LSB then MSB data bytes.

All five are satisfied (`midi-output-family/01`–`04` and `06`, resolved; `05`'s pervasion was
retired by ADR 0063). Each Expression performs exactly one Play Command, and a chord is several
Terminal Output roots one Bang activates. Bang expiry ships with the Bang row. The safety action
sends All Notes Off (CC 123), Reset All Controllers (CC 121) and a centred bend on every channel.
Both targets have a backend: native through the platform MIDI service, and the browser through Web
MIDI (#185, ADR 0059), which requests access once, answers discovery and connect synchronously from
the access it holds, and opens a port asynchronously before installing it.

| Member | Current state | Evidence |
| --- | --- | --- |
| `!>` | Satisfied: typed channel, velocity and Note; inactive roots send nothing | `lang/src/functions/mod.rs` `test::play_carries_each_operand_into_the_role_its_signature_names`, `test::play_rejects_channels_outside_the_midi_range`, `test::play_rejects_velocities_outside_the_midi_data_byte_range`, `test::play_rejects_implicit_number_note_conversions`; `orcvs/src/source/model.rs` `test::test_root_play_function_emits_one_play_command_without_a_cell_write`, `test::test_play_preserves_zero_velocity_as_an_explicit_command`; `orcvs/src/midi.rs` `tests::submits_commands_as_ordered_note_on_messages` |
| `!~` | Satisfied: Note Off at `T + length`; velocity zero and length zero follow the zero rules | `lang/src/functions/mod.rs` `test::timed_play_takes_the_same_midi_domains_as_raw_play_and_a_whole_byte_of_length`; `orcvs/src/playback/schedule.rs` `tests::a_timed_play_starts_in_tick_plan_order_and_stops_at_the_tick_its_length_names`, `tests::a_timed_play_with_velocity_zero_stops_the_note_and_schedules_nothing`, `tests::a_timed_play_with_no_length_emits_nothing_and_leaves_the_note_it_finds_standing`, `tests::a_repeated_timed_play_stops_the_instance_it_replaces_and_retires_its_expiry`; `orcvs/src/source/model.rs` `test::test_root_timed_play_function_emits_one_command_carrying_its_whole_lifetime` |
| `!%` | Satisfied: one voice per channel, owned by the Playback Engine | `orcvs/src/playback/schedule.rs` `tests::a_monophonic_play_stops_whatever_note_its_channel_was_sounding`, `tests::a_mono_voice_is_owned_per_channel_and_channels_do_not_steal_from_one_another`, `tests::a_stale_mono_expiry_cannot_stop_the_voice_claimed_after_it`, `tests::timed_and_mono_own_separately_and_neither_owns_a_raw_note`; `orcvs/src/source/model.rs` `test::test_root_monophonic_play_function_emits_one_command_of_its_own_kind` |
| `!c`, `!b` | Satisfied: data bytes reach the wire unaltered, LSB before MSB | `lang/src/functions/mod.rs` `test::every_data_byte_reaches_a_control_change_or_pitch_bend_command_unaltered`, `test::control_change_and_pitch_bend_reject_a_note_in_every_operand_position`; `orcvs/src/midi.rs` `tests::submits_control_change_and_pitch_bend_as_their_wire_bytes`; `orcvs/src/playback/schedule.rs` `tests::control_change_and_pitch_bend_are_delivered_unresolved_and_in_tick_plan_order`; `orcvs/src/source/model.rs` `test::test_root_control_change_and_pitch_bend_emit_the_command_their_operands_name` |
| Chords | Satisfied: one Bang activates several Terminal Output roots, and each Timed Play stops at its own length | `orcvs/src/source/tick.rs` `test::one_bang_activates_every_aligned_timed_play_root_as_one_chord`; `orcvs/src/playback/schedule.rs` `tests::a_chord_of_timed_plays_stops_each_note_at_its_own_length` |
| Safety action and device lifecycle | Satisfied: CC 123, CC 121 and a centred bend on every channel at stop, delivery failure and destination change; silencing clears the note schedule | `orcvs/src/midi.rs` `tests::the_safety_action_clears_notes_controllers_and_bend_on_every_channel`, `tests::delivery_failure_attempts_the_safety_action_and_reselection_reconnects`, `tests::a_destination_change_sends_the_safety_action_to_the_destination_it_leaves`; `orcvs/src/playback.rs` `tests::every_lifecycle_action_that_silences_output_clears_the_note_schedule` |
| Browser Web MIDI backend | Satisfied in automated tests (#185, ADR 0059): pending, granted and unavailable access; exact bytes to the opened output; failed and abandoned opens. Manual browser checks remain open in `midi-port-ownership/06`, `09` and `10`. A port reselected while its close is in flight may be installed before the browser has opened it, a gap `midi-port-ownership/11` (`release/v1`) owns | `console/src/web_midi.rs` `tests::a_pending_request_answers_that_access_is_awaited`, `tests::granted_access_lists_the_connected_outputs`, `tests::a_browser_without_midi_access_offers_an_empty_list_rather_than_an_error`, `tests::the_frame_after_the_browser_answers_catches_up_without_a_scan`, `tests::playback_delivers_exact_bytes_to_the_output_the_console_opened`, `tests::an_output_that_fails_to_open_leaves_the_playing_destination`, `tests::an_abandoned_open_releases_its_output`; `console/tests/wasm.rs` `web_midi_answers_discovery_and_connect_without_waiting`, `midi_output::the_browser_output_control_is_enabled_and_offers_scan` |

### Members still short of the release

- **Browser Web MIDI port reselection** — `midi-port-ownership/11` (`release/v1`). The gap is
  unconfirmed: it needs a browser that settles `close()` or `open()` before the device is released
  or acquired. A clean manual run resolves it as `wontfix`.

Every other member's Current state is satisfied, except that the Browser Web MIDI backend is
satisfied in automated tests only. Release proof that is not a member's state stays with its own
owners: native physical MIDI evidence with `v1-release/04`, the browser MIDI manual checks with
`midi-port-ownership/06`, `09` and `10` (each `release/v1`), and the candidate-wide evidence run
with `v1-release/03`.

### Explicit deferrals and omissions

- UDP `!u`, OSC `!o`, and their text/message values and transports.
- Application Command `!$` and its command value encoding.
- Cross-version Source, persistence-format, and Rust-interface compatibility while Orcvs remains
  pre-release.
- Improvement-only maintenance that is not required for correctness, safe implementation, or
  measured release evidence.
- Hidden-variable behavior is omitted rather than deferred; Identity Test is retired.
- The Sequence value, the `:` family (`:-`, `:#`, `:<`, `:&`, `:?`, `:=`) and pervasive extension
  are retired rather than deferred (ADR 0063, `cell-tracker/01`).

The Language Map, compiler-checked Function definitions, typed operand extraction, canonical Source
generation, Tick planning, and target evidence are prerequisites for proving this inventory, but
they are infrastructure or proof work rather than additional shipped language members.

## Comments

### Audit against origin/main cf01e3c — 2026-10-13

The Answer's Current state was refreshed against `cf01e3c` after the 2026-10-01 column went stale.
ADR 0063 retired the Sequence row and family; ADR 0070 renamed the directional Jumps to Copies (`&`
now spells a Read) and added the Read, Write, Track and Push Functions; ADR 0072 made a standalone
`**` the Bang Function and removed contact activation; ADR 0069 changed Equality, Delay and
Euclidean to clear their Output Portal; the conversions are no longer idempotent (`CONTEXT.md`,
Numeric Conversion Function). 57 cited tests no longer existed and were replaced; every remaining
citation was checked by script against the file named beside it. The Self-Banging collision gap
closed with `spatial-tick-planning/07`.

### Decided 2026-10-13 by Toby

The Functions ADR 0070 added — the absolute Copy `=$`, the Reads `&^ &v &< &> &$`, Track `&t`,
the Writes `@^ @v @< @> @$` and Push `@t` — join the release contract, alongside the directional
Copies. The Read, Write and Copy section now states their membership, and no addressing deferral
remains.
