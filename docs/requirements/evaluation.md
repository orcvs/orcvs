# Evaluation requirements

> Snapshot: checked against `c8cc197` (2026-11-14); incorporates ADRs through 0109.
> ADRs accepted after 0109 bind where they differ, until this document is updated.
> Source ADRs: 0003, 0004, 0006, 0009, 0012, 0014, 0020, 0028, 0032, 0034, 0036, 0057, 0060, 0065, 0067 (scheduling amendments), 0068, 0071, 0072, 0073, 0076 (Tick count only), 0077. Reviewed, not applicable or wholly superseded: 0031 (by 0032), 0066 (by 0069), 0037 (owned by [playback](playback.md)).

This document states how one Tick turns a Source Snapshot into a committed Tick Plan: Turns, Dependencies, Reservations, Portals and their writes, activation, placement, the Tick-family time and feedback inputs, error containment, the commit, and the Tick Portals record. It does not state Function spellings, operand types or what each Function reads ([language](language.md)), the Playback clock, the Tick period and Overrun (PLAY-180 Tick Grid and PLAY-190 Overrun in [playback](playback.md)), or output delivery ([playback](playback.md)), how the console paints Portals ([console](console.md)), or the site ([site](site.md)). Terms are defined in [CONTEXT.md](../../CONTEXT.md); decisions are argued in [the ADRs](../adr/).

## State and determinism

### EVAL-010 The Source Snapshot is the whole language state

Every Tick starts from a Source Snapshot that is a complete executable program and the accumulated output of earlier Ticks. Neither the Evaluator nor the Playback Engine keeps hidden state between Ticks; any stateful behaviour is reproducible from the Source Snapshot and the absolute Tick. The Evaluator's only memory is one Operand Stack per evaluation (EVAL-081). Pending Operand Encodings and Returns are local to the Tick.

Reason: Source stays live-inspectable and self-modifying in the tracker style; nothing about a Bang is remembered outside Source.
Sources: [ADR 0003](../adr/0003-source-snapshot-is-complete-language-state.md), [ADR 0028](../adr/0028-specify-the-orcvs-evaluation-machine.md), [ADR 0072](../adr/0072-a-standalone-bang-is-a-function.md).
Evidence: `lang/src/stack.rs` `Stack`; test `orcvs/src/source/tick/bang_function.rs` `identical_snapshots_at_the_same_tick_plan_the_same_tick` (assertions inspected).

### EVAL-011 Identical Source Snapshots at the same absolute Tick plan the same Tick

The Tick Plan — activation routing, Source writes, ordered Play Commands, ordered Network Sends and diagnostics — is a function of the Source Snapshot and the absolute Tick alone. A Source reloaded from the same Cells plans what the live one plans.

Reason: reproducibility is what makes planning from a snapshot equal to planning in place (EVAL-090) and lets a performer predict the Grid.
Sources: [ADR 0003](../adr/0003-source-snapshot-is-complete-language-state.md), [ADR 0072](../adr/0072-a-standalone-bang-is-a-function.md), [ADR 0057](../adr/0057-a-tick-commits-only-the-revision-it-planned.md).
Evidence: `orcvs/src/source/tick/execution.rs` `execute`; tests `orcvs/src/source/tick/bang_function.rs` `identical_snapshots_at_the_same_tick_plan_the_same_tick`, `orcvs/src/source/planning.rs` `a_tick_planned_from_a_snapshot_is_the_tick_planned_under_the_lock`, `orcvs/src/source/tick/tick_functions.rs` `moving_a_random_changes_its_stream_and_identical_inputs_reproduce_it`.

### EVAL-012 Newly written Function code first takes a Turn from the next Source Snapshot

Only Functions the Parser found in the starting Source Snapshot take Turns. A Function spelling written during Tick T — emitted, moved, copied or written — is ordinary Source until the next Tick parses it, nested or not. Values and Bangs produced during T are visible to dependent original computations in T. The one exception is Function Replacement at an original anchor (EVAL-040).

Reason: a schedule fixed from the Snapshot cannot order code that did not exist when it was built.
Not: executing newly anchored or Function-valued output in the same Tick; ADR 0034 defers it.
Sources: [ADR 0003](../adr/0003-source-snapshot-is-complete-language-state.md), [ADR 0004](../adr/0004-source-functions-plan-spatial-effects.md), [ADR 0006](../adr/0006-bang-activation-includes-self-banging-functions.md), [ADR 0014](../adr/0014-spatial-functions-preserve-performative-behaviour.md), [ADR 0020](../adr/0020-order-tick-effects-by-source-position.md), [ADR 0034](../adr/0034-execute-against-live-typed-expressions.md), [ADR 0060](../adr/0060-value-inputs-wait-placement-tests-occupancy.md).
Evidence: test `orcvs/src/source/tick/movers.rs` `an_emitted_self_banging_function_first_moves_on_the_following_tick`; test `orcvs/src/source/tick/replacement.rs` `live_replacement_checks_retained_arity_and_never_runs_new_anchors`.

## Scheduling a Tick

### EVAL-020 A Tick parses once, orders Turns by Dependency, gives each Producer at most one Turn, and publishes atomically

At Tick T: parse the Source Snapshot, establish data, activation, lock and overwrite Dependencies, take Turns in that order against working Source, and publish the resulting Tick Plan as one atomic commit. Parsing owns Function identities, operand positions, nested connections and Bang validity, including missing or invalid operand slots a current-Tick write may supply; Source is not parsed again between Turns. A Producer below its consumer reaches it in the same Tick; a fixed upward Portal is an ordinary Dependency, not delayed feedback. An admitted write is visible to every later Turn, and no Turn is revisited. Scheduling stays behind the Source Tick interface; Playback receives T's ordered commands.

Reason: a Note and a Bang supplied to a Play during T must sound in T wherever their producers stand.
Not: one row-major pass; reading a frozen Snapshot.
Sources: [ADR 0032](../adr/0032-schedule-tick-execution-by-dependency.md), [ADR 0034](../adr/0034-execute-against-live-typed-expressions.md), [ADR 0020](../adr/0020-order-tick-effects-by-source-position.md), [ADR 0004](../adr/0004-source-functions-plan-spatial-effects.md), [ADR 0014](../adr/0014-spatial-functions-preserve-performative-behaviour.md).
Evidence: `orcvs/src/source/tick/ordering.rs` `Progress`; tests `orcvs/src/source/tick/ordering.rs` `a_dependency_goes_before_grid_order`, `orcvs/src/source/tick/live_editing.rs` `live_cross_boundary_chain_uses_lower_producers`, `orcvs/src/source/tick/halt.rs` `halt_is_interpreted_once_per_tick`.

### EVAL-021 Source Position breaks ties between ready Turns and overrides no Dependency

Among Turns whose Dependencies have settled, the one earlier in row-major Position goes first. Each Producer has one Position, its anchor.

Reason: independent Turns need one deterministic order, and Position is the one the Source shows.
Sources: [ADR 0020](../adr/0020-order-tick-effects-by-source-position.md), [ADR 0032](../adr/0032-schedule-tick-execution-by-dependency.md), [ADR 0060](../adr/0060-value-inputs-wait-placement-tests-occupancy.md).
Evidence: `orcvs/src/source/tick/ordering.rs` `take_ready`; test `independent_nodes_take_their_turns_in_grid_order` in the same file.

### EVAL-022 A value input waits for every potential writer whose Reservation covers it

Before a computation takes its Turn, every writer whose Reservation covers one of its input Cells settles, whether it writes, fails or writes nothing; the computation then reads the surviving encoding. A nested input waits for its child's Return. A nested Function's Output Portal write is a potential writer like any other, and Input Portal reads add Dependencies the same way. A missing Bang is no activation, not an unsettled Dependency.

Reason: the consumer must see every same-Tick write that can reach it.
Sources: [ADR 0060](../adr/0060-value-inputs-wait-placement-tests-occupancy.md), [ADR 0032](../adr/0032-schedule-tick-execution-by-dependency.md), [ADR 0034](../adr/0034-execute-against-live-typed-expressions.md).
Evidence: `orcvs/src/source/tick/dependencies.rs`; tests `orcvs/src/source/tick/live_editing.rs` `partial_writers_settle_before_consumption`, `live_pending_note_decodes_only_after_all_writers_settle`, `orcvs/src/source/tick/movers.rs` `a_mover_supplies_a_claimed_operand_before_its_consumer`.

### EVAL-023 A Reservation orders Turns; the admitted write decides what happened

Every value Function, root or nested, reserves the two-Cell pair at each of its static write Portals; a Source-writing Function reserves the Spans its Source Bundle names. Reservations are settled before any Function evaluates and are the only Cells a Dependency names. Whether a Cell was written, a computation suppressed or replaced, or a root activated is decided by the write a Portal admits, which always lies within the Reservation.

Reason: the schedule is fixed before evaluation, so it must know every Cell a write can reach.
Sources: [ADR 0036](../adr/0036-reserve-result-cells-before-their-width-exists.md), [ADR 0061](../adr/0061-a-nested-function-returns-to-the-slot-it-occupies.md), [ADR 0063](../adr/0063-a-list-is-cells-in-a-claim-not-a-value.md).
Evidence: `orcvs/src/source/tick/dependencies.rs`; tests `orcvs/src/source/tick/conflicts.rs` `write_claims_answer_overlapping_reservations`, `orcvs/src/source/tick/nesting.rs` `a_nested_output_portal_orders_an_earlier_consumer_after_its_writer`.

### EVAL-024 A static writer covering a Snapshot computation goes before it; a late write or lock refuses only that Turn's effect

When a static writer's Reservation covers a computation from the Source Snapshot (a Contact), the writer is ordered first, so its write suppresses or replaces that computation. If a write or a Halt lock nonetheless reaches a computation that has already taken its Turn, that write or lock is refused and diagnosed ("lock reached a root that has taken its Turn" for a lock), and the Tick continues. Dynamic writes follow EVAL-028 instead.

Reason: an ordering defect costs one Turn, never the performance.
Sources: [ADR 0034](../adr/0034-execute-against-live-typed-expressions.md), [ADR 0060](../adr/0060-value-inputs-wait-placement-tests-occupancy.md), [ADR 0065](../adr/0065-an-error-never-stops-the-performance.md).
Evidence: `orcvs/src/source/tick/dependencies.rs` `refusal`; tests `orcvs/src/source/tick/live_editing.rs` `a_late_spatial_write_is_refused_and_the_tick_continues_even_when_the_earlier_turn_failed`, `orcvs/src/source/tick/halt.rs` `a_late_halt_lock_is_refused_and_the_tick_continues`.

### EVAL-025 Placement tests occupancy in working Source at its Turn and creates no Dependency

An Advance or Emit reads working Source at its Turn, after every earlier admitted effect. It does not wait for an occupant to leave or for a contesting placement, and a refused placement is not retried. Occupancy alone adds no Dependency, and neither does overlap between an overwrite's and a placement's destinations: an earlier non-empty overwrite blocks the placement, a later overwrite replaces what was placed. Value-input, activation, Halt and ordinary overwrite Dependencies still apply, and a placement covering an operand is a potential writer under EVAL-022.

Reason: same-Tick value propagation with local placement refusal under one execution order.
Not: Snapshot-only admission; simultaneous vacancy solving.
Sources: [ADR 0060](../adr/0060-value-inputs-wait-placement-tests-occupancy.md); ruled 2026-10-13, no ADR (`.scratch/bang-function/issues/08-decide-whether-a-mover-still-precedes-the-root-it-contacts.md`) for a moving bundle ordering no Contacts.
Evidence: `orcvs/src/source/tick/execution.rs` `deliver_source_effect`; tests `orcvs/src/source/tick/movers.rs` `an_emission_waits_for_no_occupant_to_leave`, `a_mover_is_blocked_by_an_earlier_overwrite`.

### EVAL-026 A placement into vacated Cells overwrites no computation

A Snapshot computation does not keep its Cells after it moves: a placement into Cells vacated earlier in the Tick succeeds and suppresses nothing. The guard against overwriting an attempted computation protects Snapshot computations, failed attempts included; a later overwrite of a mover's new spelling in Cells that were empty in the Snapshot replaces Source content without a diagnostic.

Reason: a follower may move into space its leader has vacated, but cannot demand the leader go first.
Sources: [ADR 0060](../adr/0060-value-inputs-wait-placement-tests-occupancy.md).
Evidence: test `orcvs/src/source/tick/movers.rs` `an_emission_enters_cells_a_mover_vacated_earlier_in_the_tick`, `a_movers_new_cells_can_be_overwritten_after_its_turn`.

### EVAL-027 A dynamic Input Portal's writers are found at its Turn

Once a Function's operands settle, its dynamic Input Portal is known, and every writer of those Cells that has not taken its Turn goes first; the Function's Turn waits and is taken again after them. The loop that orders Turns continues through the Tick so a Dependency found at a Turn, or a root activated during the Tick, joins the order with its own Dependencies. A writer that waits on the reader closes a cycle (EVAL-071, EVAL-073). A Function never waits on its own write. Which Functions have dynamic Input Portals, and how operands select the Cells, is stated in [language](language.md).

Reason: a reader sees writers east or south of it in the same Tick without reserving every Cell it might read.
Not: ordering a reader after every writer to the row edge, which manufactures cycles.
Sources: [ADR 0067](../adr/0067-track-reads-a-portal-as-jump-does.md).
Evidence: `orcvs/src/source/tick/ordering.rs` `continue_ordering`, `join`; tests `orcvs/src/source/tick/ordering.rs` `a_turn_that_waits_is_taken_again_after_its_writers`, `a_root_that_joins_brings_its_dependencies_into_the_rest_of_the_tick`.

### EVAL-028 A dynamic writer takes its Turn as early as its inputs allow; its write onto a completed Turn is next-Tick feedback

Dynamic writers are ordered among themselves by what feeds them, Position breaking ties (a nested writer before the writer it feeds), and each goes before every computation that neither feeds it nor feeds a dynamic writer ordered before it; a Dependency it finds at its Turn overrides that order. A static writer that does not feed a dynamic writer comes after it and wins a pair both reach. A dynamic write onto Cells a completed Turn read, or onto a Function whose Turn is taken, is feedback seen on the next Tick; onto a Function still to come it suppresses or replaces it now. A dynamic write that suppresses or replaces a Function of its own Expression is applied and diagnosed at the writer ("dynamic output reached its own Expression"). A Bang or Halt lock a dynamic write delivers to a Turn already taken is lost for that Tick and diagnosed at its target as missed (Missed Bang); nothing is stopped. Writers-first ordering closes no cycle; a dynamic writer cycles only through dependencies it actually has. Which computations are dynamic writers is a scheduling input shared across Ticks; where a write lands belongs to one Tick.

Reason: readers anywhere in the Grid see a Write's value in the same Tick, and no Turn is taken twice.
Not: re-taking a reader's Turn after a later write.
Sources: [ADR 0067](../adr/0067-track-reads-a-portal-as-jump-does.md), [ADR 0072](../adr/0072-a-standalone-bang-is-a-function.md).
Evidence: `orcvs/src/source/tick/ordering.rs` `writers_first`; tests `orcvs/src/source/tick/ordering.rs` `a_dynamic_writer_goes_before_what_does_not_feed_it`, `orcvs/src/source/tick/write.rs` `a_reader_that_feeds_the_write_sees_it_on_the_next_tick`, `a_write_onto_a_function_that_has_taken_its_turn_is_seen_next_tick`.

## Portals and writes

### EVAL-030 A Portal is a Cell destination resolved within one Tick

A Portal is internal to the Tick that resolves it: no Atom carries one, it is not Source content, and it does not survive the Tick or reach a saved Source. The Tick Portals record (EVAL-100) observes what Turns did with Portals; it is not a Portal.

Reason: destination resolution can change without touching Function evaluation, effect ordering or commit.
Sources: [ADR 0009](../adr/0009-portals-resolve-tick-plan-destinations.md), [ADR 0077](../adr/0077-a-committed-tick-records-its-portal-accesses.md).
Evidence: `orcvs/src/source/portal.rs` (module).

### EVAL-031 A write validates its whole destination before any Cell of it is emitted

A Portal whose site falls outside the Grid resolves no Cells: a read there diagnoses, and a write, placement or lock there is refused. A write whose destination leaves the Grid, is cut short by the row edge, or does not match the reserved pair width is refused whole and diagnosed; no part of it lands. The Source a successful write leaves may intentionally hold a malformed or misaligned arrangement, visible to later Turns and after commit.

Reason: a partial write would leave Cells no Reservation described.
Sources: [ADR 0004](../adr/0004-source-functions-plan-spatial-effects.md), [ADR 0009](../adr/0009-portals-resolve-tick-plan-destinations.md), [ADR 0020](../adr/0020-order-tick-effects-by-source-position.md), [ADR 0036](../adr/0036-reserve-result-cells-before-their-width-exists.md).
Evidence: `orcvs/src/source/portal.rs` `Portal::admit`; tests `orcvs/src/source/tick/conflicts.rs` `test_a_write_whose_destination_leaves_the_grid_emits_no_partial_write`, `orcvs/src/source/tick/live_editing.rs` `live_rejected_complete_write_changes_no_cell`, `orcvs/src/source/tick/outputs.rs` `a_destination_at_the_row_edge_costs_one_expression_its_turn_not_the_tick`.

### EVAL-032 Admitted writes resolve Cell by Cell in execution order, then emission order

Effects follow execution order (EVAL-020, EVAL-021), then the order their Producer emits them; Play Commands and diagnostics keep the same order. Each Cell takes the last write to reach it, so a later write replaces the Cells it covers and leaves the rest of an earlier write standing. Pending Operand Encodings compose the same way.

Reason: overlapping writes are deterministic without intermediate decoding.
Sources: [ADR 0004](../adr/0004-source-functions-plan-spatial-effects.md), [ADR 0020](../adr/0020-order-tick-effects-by-source-position.md), [ADR 0034](../adr/0034-execute-against-live-typed-expressions.md).
Evidence: `orcvs/src/source/tick/execution.rs` `write`; tests `orcvs/src/source/tick/conflicts.rs` `test_later_effects_win_cell_conflicts_independently`, `the_cells_of_two_overlapping_writes_are_contested_one_by_one`, `test_play_commands_and_diagnostics_keep_producer_and_emission_order`; property `orcvs/src/source/tick/live_editing.rs` `live_writes_agree_with_an_independent_cell_overlay`.

### EVAL-033 A Source Bundle is validated whole and writes its own Span first

A Source-writing Function validates every destination of its Source Bundle before any write; one invalid destination refuses the bundle. A clear is spaces written to Source, not a value. Writes run own Span first, then the displaced write, so in a horizontal move the shifted spelling wins the shared Cell.

Reason: a move is one atomic effect with no half-applied state.
Sources: [ADR 0004](../adr/0004-source-functions-plan-spatial-effects.md), [ADR 0009](../adr/0009-portals-resolve-tick-plan-destinations.md), [ADR 0020](../adr/0020-order-tick-effects-by-source-position.md).
Evidence: `orcvs/src/source/tick/execution.rs` `deliver_source_effect`, `fire`; tests `orcvs/src/source/tick/movers.rs` `a_self_banging_function_advances_its_whole_span_by_one_cell`, `a_move_that_leaves_the_grid_replaces_its_own_span_with_bang`.

### EVAL-034 A spatially written operand stays a Pending Operand Encoding until its Function takes its Turn

Writes into an operand are not decoded as they land and raise no intermediate decode failure. When the receiving Function takes its Turn, its current signature decodes the surviving characters, then applies type and domain checks: in a Note operand, `E4` → `EA` → `E5` decodes as `E5`; `EA` left standing diagnoses there.

Reason: identical final characters mean the same thing however they arrived.
Sources: [ADR 0034](../adr/0034-execute-against-live-typed-expressions.md), [ADR 0032](../adr/0032-schedule-tick-execution-by-dependency.md).
Evidence: `orcvs/src/source/tick/inputs.rs`; tests `orcvs/src/source/tick/live_editing.rs` `live_pending_note_decodes_only_after_all_writers_settle`, `orcvs/src/source/tick/nesting.rs` `a_return_and_a_portal_decode_the_same_characters_the_same_way`.

### EVAL-035 A failure in one delivery leaves the others standing

A refused spatial write keeps the Function's answer for its parent's Return. A parent's evaluation error does not undo its child's write. A producer that fails writes nothing, and its spatial consumers proceed on the encodings that remain; a nested consumer whose child returns nothing fails.

Reason: an error costs the computation it occurs in, not its neighbours.
Sources: [ADR 0034](../adr/0034-execute-against-live-typed-expressions.md), [ADR 0065](../adr/0065-an-error-never-stops-the-performance.md).
Evidence: `orcvs/src/source/tick/execution.rs` `deliver_value`, `project_value`; tests `orcvs/src/source/tick/live_editing.rs` `live_child_write_survives_parent_failure_and_rejected_portal_keeps_its_return`, `live_failed_suppliers_preserve_spatial_data_but_not_nested_answers`, `orcvs/src/source/tick/outputs.rs` `an_activated_consumer_uses_surviving_cells_after_supplier_failure`.

### EVAL-036 A result writes only its own encoding

A result never clears Cells outside the pair it writes.

Reason: what stands beside a result is Source the result does not own.
Sources: [ADR 0009](../adr/0009-portals-resolve-tick-plan-destinations.md).
Evidence: test `orcvs/src/source/tick/tick_functions.rs` `increment_writes_its_current_encoding_and_leaves_a_stale_tail` (assertions inspected).

## Replacement and suppression

### EVAL-040 A Function Replacement at an original anchor runs in the same Tick on the retained operands

A write that lands a Function Atom on the anchor of a computation still to take its Turn replaces which Function runs there for the rest of the Tick. The replacement keeps the original operand positions and nested connections, recruits and discards none, and its own signature decodes the retained characters and checks arity and type at its Turn (`.v` → `.^` in `.vC4` reads Number `C4` and diagnoses its range). The write is positioned by the Grid, not aligned to parsed items, so the resulting arrangement may be malformed. A replacement is refused, naming the fact that differs, unless it agrees with the Function the computation is running on each of: whether it answers a value, where its activation comes from, whether it can emit Bang, and the Source write it declares.

Reason: the schedule was derived from those four facts, so a replacement that changes one would run on edges that no longer describe it.
Not: replacements that change activation or output kind, newly anchored code, or changed destinations; ADR 0034 defers them.
Sources: [ADR 0034](../adr/0034-execute-against-live-typed-expressions.md), [ADR 0032](../adr/0032-schedule-tick-execution-by-dependency.md), [ADR 0036](../adr/0036-reserve-result-cells-before-their-width-exists.md).
Evidence: `orcvs/src/source/tick/dependencies.rs` `Refusal::Replacement`; `lang/src/atom.rs`; tests `orcvs/src/source/tick/replacement.rs` `a_replacement_that_changes_only_the_activation_source_is_refused`, `live_the_second_replacement_at_one_anchor_replaces_the_first`, `live_function_replacement_keeps_nesting_and_reinterprets_only_literals`, `original_anchor_function_replacement_retains_inputs`.

### EVAL-041 A value written over a computation replaces it as an input or suppresses its Turn

A value landing on a nested Function replaces that computation as its parent's input; its former operands are unused (`Add(2, Multiply(3, 4))` receiving `05` at Multiply is `Add(2, 5)`). A value landing on a root's Function suppresses its Turn and every effect it would have made. Former operand characters stay unless a write covers them.

Reason: the Cells hold the value now, and identical Cells mean the same thing however they arrived.
Sources: [ADR 0034](../adr/0034-execute-against-live-typed-expressions.md).
Evidence: test `orcvs/src/source/tick/live_editing.rs` `a_value_replaces_nested_computation_and_preserves_next_tick_source`.

### EVAL-042 A nested Function takes its Turn only through its root

Activation of a root includes its nested Functions. A nested Function's Output Portal does not activate it; an inactive or suppressed parent suppresses the child, its Return and its Output Portal write.

Reason: a nested Function belongs to its Expression.
Sources: [ADR 0006](../adr/0006-bang-activation-includes-self-banging-functions.md), [ADR 0034](../adr/0034-execute-against-live-typed-expressions.md).
Evidence: tests `orcvs/src/source/tick/live_editing.rs` `live_inactive_ownership_and_silent_terminal_output_are_independent`, `orcvs/src/source/tick/replacement.rs` `live_deep_and_top_level_replacement_suppress_descendant_portals`.

## Activation, Halt and placement

### EVAL-050 A root that answers an effect is inert until a Bang reaches it, except a Self-Banging Function and the Bang Function

A Terminal Output, Directional Bang or Halt root takes a Turn only when a Bang activates it during the Tick. A Self-Banging Function and the Bang Function take their Turn with nothing delivered. Whether a root that answers a value is likewise inert is an open decision (see Open decisions).

Reason: autonomous motion is visible in the Function spelling rather than in a hidden mode.
Sources: [ADR 0006](../adr/0006-bang-activation-includes-self-banging-functions.md), [ADR 0072](../adr/0072-a-standalone-bang-is-a-function.md), [ADR 0032](../adr/0032-schedule-tick-execution-by-dependency.md).
Evidence: `lang/src/atom.rs` `ActivationSource`, `Function::is_intrinsically_active`; test `orcvs/src/source/tick/movers.rs` `an_inert_directional_bang_function_emits_nothing`.

### EVAL-051 A Bang activates the complete roots aligned with its Span in the Tick it is produced

For a Bang anchored at `(x, y)`, the aligned root anchors are `(x, y-1)`, `(x, y+1)`, `(x-2, y)` and `(x+2, y)`; only a complete root anchored there is activated, and it takes its Turn later in the same Tick. A `**` touching an operand activates nothing, nor does a `**` rejected in a typed operand. What a Bang is, and how pair-passing Functions carry `**`, is stated in [language](language.md).

Reason: a Note and a Bang supplied in Tick T must sound in T.
Sources: [ADR 0006](../adr/0006-bang-activation-includes-self-banging-functions.md), [ADR 0032](../adr/0032-schedule-tick-execution-by-dependency.md), [ADR 0071](../adr/0071-a-bang-travels-as-its-glyph.md).
Evidence: `orcvs/src/source/tick/dependencies.rs` `bang_root`; tests `orcvs/src/source/tick/bang_activation.rs` `a_bang_activates_its_aligned_neighbours_and_no_further_root`, `a_bang_rejected_in_a_typed_operand_never_activates`.

### EVAL-052 Several Bangs give a root one Turn, and a Turn already taken is not revisited

A root activated during the Tick joins the order with its Dependencies. However many Bangs reach it, it takes one Turn; a Bang reaching it after its Turn is a Missed Bang.

Reason: each Producer takes at most one Turn.
Sources: [ADR 0006](../adr/0006-bang-activation-includes-self-banging-functions.md), [ADR 0004](../adr/0004-source-functions-plan-spatial-effects.md), [ADR 0072](../adr/0072-a-standalone-bang-is-a-function.md).
Evidence: `orcvs/src/source/tick/execution.rs` `activate`; test `orcvs/src/source/tick/ordering.rs` `a_root_that_joins_brings_its_dependencies_into_the_rest_of_the_tick`.

### EVAL-053 An active Halt locks the root one row south before that root's Turn

A Halt's lock is an Effect of its Turn, ordered before its target's Turn, and holds only if a Bang that executed activated the Halt first. The lock withholds every effect of the target Expression, a Bang Function included, and changes no Source. An empty target is a no-op; an occupied target that is not an Expression root, other than Comment text, diagnoses; a Halt aimed at Comment text locks nothing silently. A Halt takes at most one Turn per Tick, and a Halt locked by another locks nothing.

Reason: Halt acts on activation itself rather than moving a value.
Sources: [ADR 0006](../adr/0006-bang-activation-includes-self-banging-functions.md), [ADR 0072](../adr/0072-a-standalone-bang-is-a-function.md), [ADR 0014](../adr/0014-spatial-functions-preserve-performative-behaviour.md).
Evidence: `orcvs/src/source/tick/execution.rs` `lock_portal`; tests `orcvs/src/source/tick/halt.rs` `an_active_halt_locks_the_complete_root_one_row_south`, `an_empty_halt_target_is_a_noop`, `an_occupied_non_root_halt_target_diagnoses`, `a_halt_aimed_at_comment_text_locks_nothing_and_does_not_diagnose`, `an_active_halt_locks_a_bang_function`, `a_suppressed_halt_does_not_lock_its_own_target`.

### EVAL-054 A Self-Banging Function advances one Cell into empty Cells or leaves `**`

At its Turn a Self-Banging Function tests only the Cells newly entered by a one-Cell step in its direction, ignoring its own Span. If they are inside the Grid and empty it writes spaces over its Span, then its spelling at the shifted destination. Otherwise it writes `**` over its own Span and activates nothing this Tick; that `**` is a Bang Function on the next. Contact with part of another Language Unit is also diagnosed; complete contact with a non-root adds no diagnostic.

Reason: direction and all cross-Tick state stay visible in Source.
Sources: [ADR 0006](../adr/0006-bang-activation-includes-self-banging-functions.md), [ADR 0014](../adr/0014-spatial-functions-preserve-performative-behaviour.md), [ADR 0060](../adr/0060-value-inputs-wait-placement-tests-occupancy.md), [ADR 0072](../adr/0072-a-standalone-bang-is-a-function.md), [ADR 0020](../adr/0020-order-tick-effects-by-source-position.md).
Evidence: `orcvs/src/source/tick/execution.rs` `deliver_source_effect`; tests `orcvs/src/source/tick/movers.rs` `a_mover_diagnoses_partial_contact_with_a_blocked_movers_bang`, `orcvs/src/source/tick/bang_function.rs` `a_blocked_mover_activates_nothing_and_its_bang_plays_on_the_next_tick`.

### EVAL-055 A Directional Bang Function emits only into a complete, empty, in-Grid destination

When activated, a Directional Bang Function at `(x, y)` writes its Self-Banging Function at `(x, y-1)`, `(x, y+1)`, `(x-2, y)` or `(x+2, y)`. If any destination Cell is occupied at its Turn or outside the Grid, it diagnoses and writes nothing.

Reason: a producer that stays standing has no Cells of its own to show a refusal.
Sources: [ADR 0006](../adr/0006-bang-activation-includes-self-banging-functions.md), [ADR 0060](../adr/0060-value-inputs-wait-placement-tests-occupancy.md).
Evidence: tests `orcvs/src/source/tick/movers.rs` `an_active_directional_bang_function_emits_its_self_banging_function`, `a_refused_emission_diagnoses_and_writes_no_cell`.

### EVAL-056 Collisions are decided by Turn order alone

Contending placements each take their own refusal; whoever's Turn comes first finds the Cells empty. Converging movers close their gap by two Cells a Tick: an odd gap leaves one Cell to the earlier mover and the later bangs (`>> <<` → ` >>**`); an even gap bangs both (`>><<` → `****`); a follower whose Turn precedes its leader's bangs (`>>>>` → `** >>`). A Directional Bang Function contesting a mover keeps its refusal; a mover moving back into it is blocked. An emission/mover contest follows the actual schedule, so a Bang supplier's position can decide it.

Reason: no rule beyond the evaluation machine; mutual consumption would need Turns that depend on each other.
Sources: [ADR 0006](../adr/0006-bang-activation-includes-self-banging-functions.md), [ADR 0060](../adr/0060-value-inputs-wait-placement-tests-occupancy.md).
Evidence: tests `orcvs/src/source/tick/movers.rs` `an_odd_gap_leaves_the_earlier_mover_the_cell_and_the_later_one_bangs`, `converging_movers_resolve_pairwise_in_source_order`, `a_mover_flush_against_a_directional_bang_function_bangs_rather_than_costing_the_tick`, `an_emission_contest_follows_the_order_its_activation_imposes`.

### EVAL-057 The Bang Function's Turn activates its aligned roots and clears its two Cells

A standalone `**` takes its Turn like any intrinsically active Producer: it activates the roots aligned with its Span (EVAL-051) and writes spaces over its two Cells. A write that covers any of its Cells before its Turn suppresses it, so a producer that bangs again or answers a Clear suppresses its own last `**`. A producer that takes no Turn, or is invalid, writes nothing, so its last `**` fires once more. A Halt above it locks it and it stands without firing; a reader of its Cells waits for its Turn and reads what it leaves; a mover meets it in Source order (EVAL-056).

Reason: whether a `**` fires depends only on the Source Snapshot and the absolute Tick.
Sources: [ADR 0072](../adr/0072-a-standalone-bang-is-a-function.md), [ADR 0073](../adr/0073-a-function-with-nothing-to-write-answers-nothing.md).
Evidence: `orcvs/src/source/tick/execution.rs` `fire`; tests `orcvs/src/source/tick/bang_function.rs` `a_typed_bang_plays_once_on_the_next_tick_and_is_cleared`, `identical_snapshots_at_the_same_tick_plan_the_same_tick`, `orcvs/src/source/tick/halt.rs` `an_active_halt_locks_a_bang_function`, `orcvs/src/source/tick/movers.rs` `a_copy_reads_a_dynamic_write_over_cells_a_fired_bang_cleared`.

## Time and feedback inputs

### EVAL-060 The absolute Tick is an interpretation input supplied from outside Source

Each Tick is planned at an unsigned absolute Tick supplied by Playback; each executed Tick is the previous plus one. Where a run starts counting, and what stop and Rewind do to it, is stated in [playback](playback.md); when each Tick is due and when one is declined are PLAY-180 (Tick Grid) and PLAY-190 (Overrun). Tick-family Functions read it rather than counting their own activations.

Reason: the same Snapshot at the same Tick answers the same step.
Sources: [ADR 0012](../adr/0012-time-and-feedback-are-explicit-tick-inputs.md), [ADR 0076](../adr/0076-stopping-playback-holds-its-tick.md).
Evidence: `lang/src/tick.rs` `Tick`, `TickInputs`; tests `orcvs/src/source/tick/tick_functions.rs` `the_interpreter_is_handed_the_shared_tick_and_each_roots_own_anchor`, `the_tick_functions_answer_about_the_absolute_tick_they_are_planned_at` (assertions inspected).

### EVAL-061 Clock answers `floor(Tick / division) % steps`

Arithmetic is in an integer wider than a byte. A zero `division` or `steps` diagnoses, `division` first. Spelling and Roles: [docs/functions.md](../functions.md).

Reason: the cycle is `division * steps` Ticks; a higher division is slower, as on a clock divider.
Sources: [ADR 0012](../adr/0012-time-and-feedback-are-explicit-tick-inputs.md).
Evidence: `lang/src/functions/tick.rs`; tests in the same file `clock_counts_one_step_per_division_ticks_and_wraps_at_its_steps`, `clock_counts_a_cycle_of_more_ticks_than_a_byte_holds`, `clock_diagnoses_a_zero_division_before_zero_steps`.

### EVAL-062 Every Bangs exactly when `Tick % (division * steps) == 0`, Tick 0 included

On every other Tick it answers a Clear. The product is taken in a wider integer; zero operands diagnose as for Clock.

Reason: `steps` of `01` is one Bang per `division` Ticks, not one every Tick.
Sources: [ADR 0012](../adr/0012-time-and-feedback-are-explicit-tick-inputs.md), [ADR 0073](../adr/0073-a-function-with-nothing-to-write-answers-nothing.md).
Evidence: `lang/src/functions/tick.rs`; tests `every_bangs_once_per_cycle_beginning_at_the_first_tick`, `every_with_one_step_bangs_once_per_division_ticks_rather_than_every_tick`, `every_counts_a_cycle_product_wider_than_a_byte`.

### EVAL-063 Euclidean Bangs where Tidal's `bjorklund (hits, steps)` places an onset at `Tick % steps`

This holds for every `steps` `01`–`FF` and `hits` `00`–`steps`; every pattern with an onset starts on one. Zero `steps` diagnoses before `hits > steps` does, so `00 00` diagnoses; zero hits is silent; equal hits and steps Bang every Tick.

Reason: a pattern copied from Tidal sounds the same in Orcvs.
Not: Orca's `U` rotation.
Sources: [ADR 0012](../adr/0012-time-and-feedback-are-explicit-tick-inputs.md).
Evidence: `lang/src/functions/tick.rs`; tests `euclidean_matches_tidals_bjorklund_for_every_pattern_up_to_32_steps` (assertions inspected), `euclidean_matches_tidals_bjorklund_up_to_a_byte_of_steps`, `orcvs/src/source/tick/tick_functions.rs` `a_tick_function_with_no_cycle_diagnoses_and_writes_nothing`.

### EVAL-064 Increment and Slew keep their state in Source at one Portal

Their Input and Output Portal are the same site, one row south. The previous value is read from working Source at their Turn, including earlier same-Tick writes; empty Cells read as `00`, and Cells that do not decode as a Number diagnose and write nothing. Operand faults are reported before Portal faults. Feedback is Source input, never a `TickInputs` channel or state carried by the Evaluator.

Reason: the complete cross-Tick state stays visible as one Atom.
Sources: [ADR 0012](../adr/0012-time-and-feedback-are-explicit-tick-inputs.md), [ADR 0034](../adr/0034-execute-against-live-typed-expressions.md).
Evidence: `lang/src/functions/tick.rs`; tests `orcvs/src/source/tick/tick_functions.rs` `an_empty_portal_initialises_increment_and_slew_as_zero`, `feedback_reads_a_portal_write_from_an_earlier_turn`, `a_previous_that_is_not_a_number_diagnoses_and_writes_nothing` (assertions inspected), `lang/src/functions/tick.rs` `cell_operand_faults_precede_an_invalid_portal_input`.

### EVAL-065 Increment answers `(previous + amount) % modulus` in a wider integer

A zero `modulus` diagnoses.

Reason: `FF + 02` cannot wrap before the modulus applies.
Sources: [ADR 0012](../adr/0012-time-and-feedback-are-explicit-tick-inputs.md).
Evidence: `lang/src/functions/tick.rs`; tests `increment_adds_in_a_wider_integer_before_the_modulus`, `increment_diagnoses_a_zero_modulus`.

### EVAL-066 Slew moves toward `target` by at most `amount` and never overshoots

Below the target it steps up by `amount` or lands on it; above, down the same way; equal, it answers the target. Each distance is computed only in the branch where its subtraction is non-negative; `amount` `00` holds the value.

Reason: a jump in the target becomes a ramp without underflow.
Sources: [ADR 0012](../adr/0012-time-and-feedback-are-explicit-tick-inputs.md).
Evidence: `lang/src/functions/tick.rs`; tests `slew_moves_toward_target_without_overshoot`, `slew_holds_when_its_amount_is_zero`.

## Errors and cycles

### EVAL-070 No error rejects a whole Tick

Every Tick publishes the effects of every Expression that does not depend on a fault, whatever else in the Grid is wrong, and whether the fault is in Source or in Orcvs.

Reason: Orcvs is played live and a performer makes mistakes while it plays.
Sources: [ADR 0065](../adr/0065-an-error-never-stops-the-performance.md).
Evidence: tests `orcvs/src/source/tick/outputs.rs` `a_row_edge_fragment_diagnoses_its_own_expression_and_leaves_the_tick_playing`, `orcvs/src/source/tick/movers.rs` `a_blocked_pair_of_moves_leaves_the_rest_of_the_grid_running`.

### EVAL-071 A same-Tick cycle stops the Expressions on it and those that depend on them

An Expression depends on a stopped one when any of its computations reads Cells a stopped computation writes, is activated by its Bang, locked by its Halt, or covered by its write; the stopped set grows until nothing else depends on it. Nothing in a cycle known before execution takes a Turn, no order is chosen for it, and nothing is delayed to a later Tick. Every other Expression takes its Turns exactly as if the stopped ones were absent.

Reason: one mistake must not silence the Grid, and no part of a cycle runs in an order the Source does not determine.
Not: breaking the cycle at an arbitrary edge; delaying it a Tick.
Sources: [ADR 0065](../adr/0065-an-error-never-stops-the-performance.md), [ADR 0034](../adr/0034-execute-against-live-typed-expressions.md), [ADR 0060](../adr/0060-value-inputs-wait-placement-tests-occupancy.md), [ADR 0014](../adr/0014-spatial-functions-preserve-performative-behaviour.md).
Evidence: `orcvs/src/source/tick/ordering.rs` `cycle_closure`, `stall`; tests `orcvs/src/source/tick/cycles.rs` `a_cycle_leaves_an_independent_expression_publishing`, `live_cycles_stop_only_the_expressions_they_reach` (assertions inspected).

### EVAL-072 Each cycle is diagnosed once, and each Expression it starves is diagnosed at its root

A cycle is diagnosed ("same-Tick dependency cycle") at the first computation on it in Parser order. A stopped Expression holding no computation on a cycle is diagnosed at its root as waiting on one, only if activation could reach it this Tick.

Reason: a performer can tell a cycle from what it starved.
Sources: [ADR 0065](../adr/0065-an-error-never-stops-the-performance.md).
Evidence: tests `orcvs/src/source/tick/cycles.rs` `each_separate_cycle_is_diagnosed_once`, `an_expression_a_cycle_stops_is_diagnosed_at_its_root_as_waiting`, `an_inactive_root_a_cycle_writes_over_is_not_reported_as_waiting` (assertions inspected).

### EVAL-073 A cycle found at a Turn keeps every completed Turn's Effects

When a Dependency found at a Turn (EVAL-027) closes a cycle, every remaining computation in the Expressions it reaches and their dependants stops, including nested siblings whose inputs are ready. Completed Turns keep their Effects, and a consumer of a completed Turn is not stopped for that reason alone. Diagnosis follows EVAL-072.

Reason: discarding one write would leave its consequences standing; rolling back all of them would need replanning.
Sources: [ADR 0068](../adr/0068-a-discovered-cycle-preserves-completed-turns.md).
Evidence: tests `orcvs/src/source/tick/track.rs` `a_cycle_discovered_by_a_nested_track_stops_its_sibling` (assertions inspected), `a_late_cycle_preserves_a_completed_nested_operands_write`.

## The Evaluator

### EVAL-080 The Evaluator evaluates one Function at a time over operands already resolved

A Function's resolved operands are pushed onto its Operand Stack last to first, so it pops them in signature order; its answer is returned to its caller, never pushed. The Evaluator has no registers, heap, persistent variables or program counter across Expressions; it is a stack-based expression evaluator whose memory is Source, not a virtual machine.

Reason: the last-to-first push is what makes operands arrive in signature order.
Sources: [ADR 0028](../adr/0028-specify-the-orcvs-evaluation-machine.md).
Evidence: `lang/src/stack.rs` `Stack::with_operands`, `extract`; test `operands_pop_back_in_signature_order` in the same file.

### EVAL-081 An Operand Stack's depth is bounded by the widest signature in the Function table

An operand list whose length differs from the signature is refused before the stack exists. Capacity is read from the table at compile time, independent of Expression length and nesting depth, and a push past it diagnoses rather than panics.

Reason: the bound follows the declaration, not the program.
Sources: [ADR 0028](../adr/0028-specify-the-orcvs-evaluation-machine.md).
Evidence: `lang/src/stack.rs` `Stack::new`, `push`; tests `no_function_declares_more_operands_than_the_inline_storage_holds`, `an_exhausted_operand_stack_diagnoses_rather_than_panicking`, `orcvs/src/source/tick/nesting.rs` `a_tick_over_a_chain_deeper_than_any_signature_interprets_every_link`.

### EVAL-082 The instruction set is declared once and dispatch is exhaustive

Every Function declares whether it answers a value or performs an effect, never both and never neither; the Evaluator never pushes an effect onto the Operand Stack. Every Function's spelling, name, kind, activation source, Bang capability, operand Roles and types and Portals are declared in one Function table, and the parser's literal list, operand extraction and Role names are derived from it. Dispatch is a hand-written match with no wildcard arm, so a new Function fails to compile until dispatched. Agreement between a body and its declared kind is a test obligation, not a compile check. The table's form is open.

Reason: an operand order exists in one place, so transposing it is an edit to the table.
Sources: [ADR 0028](../adr/0028-specify-the-orcvs-evaluation-machine.md).
Evidence: `lang/src/atom.rs` `define_functions!`; `lang/src/interpreter.rs`.

### EVAL-083 Each non-commutative Function has a test with distinct operand values

A complete transposition of two Role declarations compiles; only a test whose operands differ, so that a transposition changes the answer, catches it.

Reason: nothing derived from the single declaration can carry that guarantee.
Sources: [ADR 0028](../adr/0028-specify-the-orcvs-evaluation-machine.md).
Evidence: tests `lang/src/functions/math.rs` `the_non_commutative_functions_read_left_and_right_in_signature_order`, `lang/src/functions/mod.rs` `play_carries_each_operand_into_the_role_its_signature_names` (and the Timed, Monophonic, Control Change and Pitch Bend counterparts), `lang/src/functions/chord.rs` `each_operand_reaches_the_role_its_signature_names`.

## Commit

### EVAL-090 A Tick is planned with no Source lock held and committed only against the revision it planned

Planning takes one revision's Grid, Cells (shared copy-on-write), Language Map and revision identity under one short read guard and releases it. The commit takes the write lock, compares the current revision identity with the planned one, and writes only when they match; comparison and writes happen under that one guard. Every Source write mints a new revision identity.

Reason: editing and rendering do not wait for planning, and EVAL-011 makes snapshot planning equal to planning in place.
Sources: [ADR 0057](../adr/0057-a-tick-commits-only-the-revision-it-planned.md).
Evidence: `orcvs/src/source/planning.rs` `PlannedTick::commit`; tests `a_captured_snapshot_holds_no_source_lock`, `an_edit_between_plan_and_commit_is_never_overwritten` (assertions inspected).

### EVAL-091 A refused plan publishes nothing and the Tick is planned again at the same absolute Tick

A plan whose revision no longer matches is dropped whole: its Cell writes, Play Commands, Network Sends, diagnostics, locks and Tick Portals. Playback advances its Tick counter only after planning returns, so a refusal consumes no musical Tick, and the Tick that commits is planned from a revision holding the edit.

Reason: an edit made while a Tick plans is never overwritten (Live Editing).
Not: merging a plan onto a new revision; skipping the Tick.
Sources: [ADR 0057](../adr/0057-a-tick-commits-only-the-revision-it-planned.md).
Evidence: tests `orcvs/src/source/planning.rs` `a_refused_plan_carrying_play_commands_delivers_none_of_them`, `consecutive_accepted_ticks_apply_in_order_across_refused_plans`.

### EVAL-092 After three refused attempts a Tick is planned and committed under one write guard

That attempt cannot be refused, so a Tick ends in one committed plan after at most four plannings, however fast the Source is edited.

Reason: unbounded optimism would let a continuous writer stall Playback.
Sources: [ADR 0057](../adr/0057-a-tick-commits-only-the-revision-it-planned.md).
Evidence: `orcvs/src/source/mod.rs` `OPTIMISTIC_TICK_ATTEMPTS`; tests `orcvs/src/source/planning.rs` `a_tick_conflicting_on_every_optimistic_attempt_commits_once_under_the_lock`, `ticks_under_continuous_editing_terminate_and_count_once`.

### EVAL-093 An uncontended Tick copies no Cells

A planning snapshot shares the Source's Cells; they are copied once, by an edit under the write lock, only when that edit lands while an attempt still holds them.

Reason: snapshot planning costs a reference count, not a Grid copy.
Sources: [ADR 0057](../adr/0057-a-tick-commits-only-the-revision-it-planned.md).
Evidence: `orcvs/src/source/buffer.rs`; tests `orcvs/src/source/planning.rs` `a_snapshot_shares_the_cells_rather_than_copying_them`, `orcvs/tests/allocation.rs` `a_playback_tick_allocates_no_more_than_the_locked_tick`.

### EVAL-094 A Render Frame reads a Source revision and never advances Playback

A Render Frame is not a Tick: deriving one reads a captured Source revision, with its Tick Portals record, and plans, commits and counts nothing. Only Playback requests a Tick, on its own clock (PLAY-180), however often the console repaints.

Reason: Render Frames run at the UI's rate, independently of musical time; the Playback Engine alone owns musical time and the Tick count.
Sources: [ADR 0002](../adr/0002-playback-engine-owns-lifecycle-concurrency.md), [ADR 0001](../adr/0001-separate-source-interpretation-from-playback-engine.md), [ADR 0057](../adr/0057-a-tick-commits-only-the-revision-it-planned.md); CONTEXT.md Render Frame.
Evidence: `orcvs/src/render_frame.rs` `RenderFrame::derive`; test `orcvs/src/render_frame.rs` `concurrent_ticks_cannot_mix_source_revisions_within_a_render_frame` (assertions inspected: a frame reads one revision while Ticks run elsewhere).

## Tick Portals

### EVAL-100 Every planned Tick records its Tick Portals beside its Tick Plan

Execution is the record's only writer and records unconditionally: there is no recorder a caller installs and no switch, so the record a test reads is the record Playback commits. The record travels beside the Tick Plan, not in it, and the Playback Engine's input is unchanged. A Tick reserves room for one access per computation.

Reason: only execution knows which Cells a dynamic read selected; the console can paint only what a revision carries.
Not: an installable recorder; folding the record into the Tick Plan.
Sources: [ADR 0077](../adr/0077-a-committed-tick-records-its-portal-accesses.md).
Evidence: `orcvs/src/source/tick/execution.rs` `record`; test `orcvs/src/source/tick/halt.rs` `a_lock_is_recorded_with_its_site_and_what_it_did_there`.

### EVAL-101 Each entry names its owner as execution held it

The owner is the anchor of the Function whose Turn made the access, the Cells its subtree holds (its spelling through its last operand, nested Functions included), and the Function that Turn ran after any Function Replacement, all captured during execution rather than from committed Source. Input and Output Spans are constructed only inside Tick execution, from a resolved Input Portal or an admitted write, and neither converts into the other.

Reason: a Tick may rewrite the Cells its owners stood in.
Sources: [ADR 0077](../adr/0077-a-committed-tick-records-its-portal-accesses.md).
Evidence: `orcvs/src/source/portal.rs` `InputSpan`, `OutputSpan`; test `orcvs/src/source/tick/movers.rs` `reads_and_writes_keep_their_order_and_owners_when_a_mover_rewrites_the_source`.

### EVAL-102 A Turn records its read, its writes, its lock and its Source Bundle parts in the order they ran

A read (Declared or Selected) is recorded once its Turn passes the writer wait, as the resolved Input Span or the Portal Error; a retried Turn records one read, and a Turn refused before interpretation records nothing. A write is recorded per site delivery is asked to deliver to, as the admitted Output Span or the Write Refusal (Portal Error with the attempted Position, a replacement that changes a schedule fact, or an ordering defect), before any Contact replaces a Function. A lock records its site and verdict; a Source Bundle records `Vacate`, `Place`, `LeftBang` and `Fire` parts and why a destination was refused. Turns appear in settlement order; within a Turn: read, writes in site order, then lock or bundle parts.

Reason: the record says what each Turn did, including refusals.
Sources: [ADR 0077](../adr/0077-a-committed-tick-records-its-portal-accesses.md).
Evidence: `orcvs/src/source/tick/execution.rs` `record_read`, `record_write`, `record_bundle`; tests `orcvs/src/source/tick/replacement.rs` `a_refused_replacement_keeps_its_reason_in_the_record_and_paints_nothing`, `orcvs/src/source/tick/halt.rs` `a_missed_lock_is_recorded_with_the_root_it_missed`.

### EVAL-105 The record commits through the one commit path and every committed Tick replaces it

The optimistic and locked attempts hand the same commit one executed Tick holding the plan and its record; a refused attempt drops both. Each commit stores its record with the revision it leaves, an empty record when the Tick accessed no Portal.

Reason: no record can be committed beside a plan it does not observe.
Sources: [ADR 0077](../adr/0077-a-committed-tick-records-its-portal-accesses.md).
Evidence: tests `orcvs/src/source/planning.rs` `a_refused_plan_publishes_no_tick_portals_and_the_locked_attempt_does`, `orcvs/src/source/mod.rs` `a_committed_tick_that_reads_nothing_replaces_the_record_with_an_empty_one`.

### EVAL-106 A Source revision answers the last record as Current or Retained

The record is `Current` for the revision its commit left and `Retained` for a revision an edit made since; a revision keeps the record and the set of Cells edited since the commit as it was read. Stopping Playback keeps the record; a newly opened or loaded Source has none until its first Tick; the record is never persisted. Edited Cells are one bit per Cell, marked by every write except a Tick commit, cleared by each commit and shared copy-on-write with revisions. Which entries a `Retained` record paints is stated in [console](console.md).

Reason: a paused tracker keeps showing what its last Tick read.
Not: dropping the record at the first edit.
Sources: [ADR 0077](../adr/0077-a-committed-tick-records-its-portal-accesses.md).
Evidence: `orcvs/src/source/mod.rs` `freshness`, `tick_portals`; tests `a_source_answers_no_tick_portals_before_its_first_tick`, `a_revision_an_edit_made_after_the_commit_answers_the_record_as_retained`, `an_older_revision_keeps_the_record_it_was_read_with` in the same file.

## Accepted but not built, or built differently

- None found against a binding rule here. EVAL-063's test sweeps every pattern to 32 steps and samples above; ADR 0012 covers 01–FF (coverage, not behaviour).

## Open decisions

- Whether a root that answers a value is inert until Bang (ADR 0006: "An ordinary root Expression is inert until Bang activation") or takes a Turn every Tick (ADR 0032's first-implementation policy). Not binding either way. Observed implementation: every value Function is declared intrinsically active (`lang/src/atom.rs` `ActivationSource`), as `docs/functions.md` describes. Awaiting a maintainer ruling (requirements snapshot decision D5); closing it needs an ADR.
- Emission/activation cycles through operand Reservations: whether an emission whose destination reserves its Bang supplier's operand should form a cycle. Not binding beyond EVAL-071. Investigated in [`.scratch/placement-semantics/issues/04`](../../.scratch/placement-semantics/issues/04-investigate-emission-activation-operand-cycles.md).
- Function-valued output, replacements that change activation or output kind, and changed destinations introducing Dependencies are deferred by ADR 0034 and decide nothing here.
