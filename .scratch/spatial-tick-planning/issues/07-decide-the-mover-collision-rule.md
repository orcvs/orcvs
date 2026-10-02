# 07 — Decide the Mover Collision Rule

**What to build:** A settled rule for what happens when two Self-Banging Functions want the same
Cells, recorded in ADR 0006. The rule that ships today was chosen to fix a defect without changing
semantics; this decides whether it is the rule the language keeps.

**Blocked by:** None (can start immediately).

**Status:** resolved

**Tags:** release/v1

**Sources of truth:** ADR 0006 states the move, its refusal, and root contact; ADR 0020 orders Tick
effects by Source position; ADR 0031 evaluates Turns against working Source; ADR 0018 states how a
run of `*` Cells is read; `CONTEXT.md` defines Self-Banging Function and Bang.

## The rule that ships

Two movers approaching each other close the gap by two Cells a Tick, so the parity of that gap never
changes and there are two cases. Source order decides both, and each blocked mover takes ADR 0006's
ordinary refusal — it replaces its current Span with `**`.

```text
>> <<     odd gap, one Cell of overlap    ->  " >>**"  then "  >> "
>><<      even gap, flush                 ->  " ****"  then "     "
```

The odd case always worked and matches Orca, which resolves the same event by evaluation order. The
even case is admitted because `order_turns` omits contact dependencies from an advancing
Function to an intrinsically active owner. Ready Turns are then chosen by Source position.

## The alternative

Both movers are consumed, and one Bang anchors at the first Cell they both want.

```text
>> <<     ->  "  **"        <<  the conflicted Cell is 2; the Bang is [2,3]
>><<      ->  " ** "        <<  the conflicted Cells are 1 and 2; the Bang is [1,2]
```

This is the reading that treats the collision as a write conflict rather than as two independent
refusals, and it says something the shipping rule does not: at an even gap the two movers did not
fail separately, they wanted the same two Cells.

It was not taken now because it needs two Turns to depend on each other before either runs. ADR 0031
runs one Turn at a time against working Source, and a Turn reads Cells rather than another
Function's intent. That makes this a change to the evaluation machine, not a rule inside it.

## Rejected outright

Writing a Bang at each destination Span and letting them overlap. That spells `***`, and ADR 0018
already reads `***` as one Bang plus one invalid `*` — `map.bangs().count()` is 1. Which Cell becomes
litter follows the parse, not the collision. `****` is the spelling that means two whole Bangs.

- [x] ADR 0006 states the collision rule outright, rather than leaving it to be derived from the
      refusal sentence and the schedule. A reader should not have to run the Grid to learn what two
      movers do.
- [x] The decision covers the one-Cell overlap of an odd gap and not only the flush pair. The two
      cases differ in how many Cells are contested, and the shipping rule gives one a survivor and
      the other none.
- [x] The decision covers three or more movers converging. Today they resolve in Source order with
      no cycle from mover-to-mover contact, because those contact dependencies are omitted and
      ready Turns are chosen by Source position:
      `">>>>  <<    "` gives `"** >><<     "` and then `"   ****     "`.
- [x] The decision covers a Directional Bang emission that contests Cells with a mover. An emission
      never vacates its own Span, so "both are consumed" has no meaning for it, and any rule that
      consumes both needs an answer for the pair that is not symmetric.
- [x] The decision states what a mover blocked by a static Cell does. That case keeps the current
      refusal under either answer, so choosing the alternative leaves the language with two refusal
      rules and a reader needs to know which applies when.
- [x] *(Not applicable: the rule is kept.)* If the rule changes, ADR 0006's "replaces its current Span with `**`" is amended rather than
      contradicted, and `two_moves_that_want_the_same_cells_each_bang_in_their_own_span` is rewritten
      to the new rule rather than deleted.

## Comments

**2026-09-24 — decided: the shipping rule is the v1 rule.** Each blocked mover takes ADR 0006's ordinary refusal and replaces its current Span with `**`, with Source order deciding which moves first. "Both consumed" is not adopted. The remaining lines are now documentation and regression tests for the kept rule, not open questions: ADR 0006 states the rule outright, and tests cover the odd and even gaps, three or more movers converging (`">>>>  <<    "`), a Directional Bang emission contesting a mover, and a mover blocked by a static Cell. This issue joins `release/v1` and blocks `v1-release/03`, under the definition of done's spatial "conflicts" line.

### Audit at cad296df — 2026-09-29

The decision is taken. What remains is the ADR 0006 wording and three regression tests. No box
below is ticked, because each asks for the decision to be *stated*, and ADR 0006 still carries
only the generic refusal sentence (no collision, convergence or Source-order text).

Test coverage in `orcvs/src/source/tick.rs`:

- Even gap: covered. `two_moves_that_want_the_same_cells_each_bang_in_their_own_span` (`:1497`,
  a two-Cell gap) and `two_movers_reserving_each_other_each_bang_rather_than_costing_the_tick`
  (`:2019`, flush `>><<`).
- Mover blocked by a static Cell: covered.
  `a_move_blocked_by_one_complete_language_unit_bangs_without_diagnosing` (`:2052`).
- Odd gap (`">> <<"`): missing. The comment at `:1500-1503` defers to
  `a_self_banging_function_moves_once_per_tick_and_bangs_where_it_stops` (`:1481`), but that test
  has one mover and no second one.
- Three or more movers (`">>>>  <<    "`): missing. No fixture exists in `orcvs/src` or
  `orcvs/tests`.
- A Directional Bang emission contesting a mover: missing.
  `a_refused_emission_diagnoses_and_writes_no_cell` (`:1621`) blocks the emission on static Cells
  only.

Remaining work: amend ADR 0006 so it states the kept rule for all five cases, then add the three
missing tests.

### Independent implementation audit — 2026-09-29

Corrected the scheduler explanation: `orcvs/src/source/tick.rs:1083-1089` skips
contact dependencies to intrinsically active owners without comparing Source positions.
`tick.rs:1128-1142` orders ready nodes by anchor. The kept collision decision and remaining
ADR/regression work are unchanged. This audit did not execute the example grids.

### Resolved — 2026-10-02

ADR 0006 now states the kept rule for every case above, and `orcvs/src/source/tick.rs` pins each one: `an_odd_gap_leaves_the_earlier_mover_the_cell_and_the_later_one_bangs`, `converging_movers_resolve_pairwise_in_source_order`, `an_emission_and_a_mover_wanting_one_cell_follow_source_order`, `a_mover_flush_against_an_emitter_bangs_rather_than_costing_the_tick` and `an_emission_waits_for_no_mover_that_would_vacate_its_destination`, beside the existing even-gap and static-Cell tests.

Writing the emission test found a defect under the kept rule. A mover flush against a Directional Bang Function, standing in its destination (`*><<`, `>>*<`, `*v` over `^^`), made a same-Tick dependency cycle: the emitter's reservation ordered it before the mover, and the mover's contact ordered it before the emitter. The whole Grid's Tick was rejected every Tick, even with the emitter inert. `order_turns` now drops the emitter's edge for exactly that mutual pair, as it already drops the edge between two flush movers. The emitter's edge to an occupant that can vacate stays: removing it lets `^^` leave first and the emission write over an executed computation, rejecting the Tick.
