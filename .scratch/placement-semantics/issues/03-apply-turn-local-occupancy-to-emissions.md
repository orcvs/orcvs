# 03 — Apply Turn-local occupancy to Directional Bang emissions

**What to build:** An activated Directional Bang Function emits into Cells vacated before its
Turn, refuses Cells still occupied, and contests destinations in dependency order without a
special scheduling exception for a mutually blocked emitter/mover pair.

**Blocked by:** 02 — Let movers enter Cells vacated by earlier Turns.

**Status:** resolved

**Parent:** [01 — Reconcile placement with value readiness](01-reconcile-placement-with-value-readiness.md).

Use the shared admission contract delivered by issue 02 and the [placement semantics spec](../spec.md).
Parent issue 01 is a scope reference, not a prerequisite. The independent investigation in issue
04 does not gate this work or authorize a change to value-input readiness.

- [x] Remove emission contact dependencies that represent occupancy alone and retire the
      mutual-pair exception. Derive the behavior from the operation declaration while preserving
      value-input, Input Portal, nesting, activation, Halt, and ordinary overwrite dependencies.
- [x] Rewrite the vacating-mover regression to assert successful emission into the Cells left
      before its Turn. Retain explicit occupied-destination refusal and assert complete Tick
      Grids and diagnostic messages through `Source::execute`.
- [x] Verify both contender orders for an emission and a mover wanting the same empty Cell,
      mutually blocked horizontal and vertical pairs, out-of-Grid refusals, and preservation of
      unrelated effects. Confirm that newly emitted code first moves on the next Tick.
- [x] Pin the dependency-ordered contest whose local arrangement is unchanged: the east-emitting
      Function at column 2 and westbound mover at column 6 on row 1 contest column 5. A
      southbound mover at column 2 on row 0 supplies collision Bang early, and the emission wins.
      A northbound mover at column 2 on row 2 supplies collision Bang later, so the westbound
      mover enters first and the emission is refused. Use separate 10-by-3 test Grids, one Tick
      each, and assert full rows and diagnostics. Document this as intended dependency ordering.
- [x] Verify emission into empty claimed operand Cells: the consumer waits for the emission to
      settle and reads the resulting encoding, including its ordinary invalid-operand outcome.
      Pair this with occupancy refusal that waits for no occupant to leave, making rules 3a and
      3b independently observable. Retain failed/no-write supplier and ordinary late-overwrite
      regressions.
- [x] Extend the property to well-formed mixtures of movers, Directional Bang Functions, and
      empty Cells over successive Ticks. State explicitly that activation comes only from mover
      collision contact. Include constructive activation cases with successful and refused
      emissions so that a run of entirely inert emitters cannot stand in for activation coverage.
      Assert no placement-caused executed-computation rejection or dependency-cycle diagnostic.
- [x] Keep value-produced activation covered by focused tests. Preserve and name the known
      Equality/emission operand cycle tracked in issue 04, whose current expected outcome is
      rejection. Do not broaden the no-rejection property to arbitrary computation graphs or
      weaken literal-consumer edges to make it pass.
- [x] Complete the ADR reconciliation for delivered emission behavior, dependency-sensitive
      contest winners, and guard scope. Record both slices' evidence against the parent spec,
      including deliberate expectation changes and the remaining independent cycle question.
      Leave parent issue 01 unchanged.
- [x] Run scoped repository gates for changed crates and dependents with 32 local property
      cases. Record exact commands, results, and deferred checks; review the complete diff.

## Comments

### 2026-10-02 — Approved slice, with ADR review coverage

Issue 02 supplies the shared admission contract. This slice removes scheduling constraints that
exist only to defend the stale guard. Bang suppliers above and below the contenders demonstrate
why Source position alone cannot predict the winner. The generated property's emitter coverage
is intentionally limited to collision activation; value-produced activation remains separately
tested and its operand feedback cycle has an explicit owner.

### 2026-10-04 — Unpinned contact case inherited from issue 02

Review of issue 02 found one contact classification no test pins. `Execution::write` drops a
placed unit when any later write overlaps it, but only move and emission writes join
`source_writes`. If an ordinary value write replaces part of a placement made earlier in the
Tick, a blocked mover touching the remaining Cells sees them through neither record and
classifies them as `Empty`, so it gets no partial-contact diagnostic. Value writes were not
tracked before issue 02 either, and no Grid reproducing the turn order has been constructed, so
the case may be unreachable. When this slice changes emission contact, either pin the case
through `Source::execute` or record why the ordering cannot arise.

### 2026-10-11 — Implemented on PR 213

`producer_edges` gives an emitting bundle no contact edges: an emission tests occupancy at its
Turn and delivers nothing to an occupant, so the emitter → occupant edge and the mutual
emitter/mover exception (`reserves_over`) are gone. Literal-consumer, Input Portal, nesting,
activation, Halt and ordinary overwrite edges are unchanged. ADR 0060's status line now records
both slices as delivered.

Expectation changed deliberately: the vacating-mover regression is now
`an_emission_enters_cells_a_mover_vacated_earlier_in_the_tick` (the mover leaves first and the
emission succeeds). Added, all through `Source::execute`:
`an_emission_waits_for_no_occupant_to_leave` (rule 3b),
`an_emission_contest_follows_the_order_its_activation_imposes` (the 10-by-3 above/below supplier
pair), `an_emission_into_a_claimed_operand_settles_before_its_consumer` (rule 3a, invalid-operand
outcome), and `an_emission_reserving_its_activators_operand_is_a_cycle` (issue 04's case, still a
cycle). The property is now `placements_never_meet_an_ordering_defect_or_a_cycle` with the four
Directional Bang Functions in its alphabet, beside
`collision_activated_emissions_never_meet_an_ordering_defect_or_a_cycle`, which aims a mover at
an emitter whose destination is empty, static or another mover. Emitter activation in both comes
only from mover collision. The contest test pins one successful and one refused activated emission.

The 2026-10-04 contact case was not constructed: it needs an ordinary value write ordered after a
placement and before a blocked mover touching the rest of it, and no Grid found here orders a
value writer between two independent placements whose Cells it reaches. This slice does not
change `Execution`'s contact classification, so the case is neither introduced nor fixed here.

`PROPTEST_CASES=32 cargo nextest run --package orcvs --locked` — 887 passed.
`cargo clippy --package orcvs --all-targets --locked -- -D warnings` and
`cargo fmt --all -- --check` — passed.
