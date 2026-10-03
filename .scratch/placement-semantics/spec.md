# Value inputs wait; placement tests occupancy at its Turn

**Status:** ready-for-agent

## Problem Statement

A composer can place adjacent Self-Banging Functions travelling west or north and have the
whole Grid's Tick rejected indefinitely. The leading Function moves successfully, but its
follower is refused by the executed-computation guard when it enters the vacated Cells. Those
Cells are empty in working Source, while the Source Snapshot still associates them with the
leader's computation. Unrelated calculations and musical effects lose their Tick too.

Directional Bang Functions expose the same inconsistency. Scheduling currently treats an
emission destination as an overwrite dependency, except for a special case for an emission and
a mover blocked against each other. An occupant that could leave is instead ordered after the
emission to avoid the stale guard, making emission refusal a consequence of that implementation
constraint.

The language needs a small, explicit distinction between two kinds of read. A value input waits
for its potential writers to settle. A placement tests occupancy when its Turn arrives. Saying
only that both observe earlier effects hides the difference between readiness and visibility.

## Solution

Keep one dependency-ordered Tick operating on working Source. Value inputs remain ordered after
their potential suppliers. Self-Banging Functions and Directional Bang Functions test placement
against the Cells present at their own Turn, with no dependency introduced merely to wait for
an occupant to leave or to settle another placement contender.

A successful movement can therefore leave room for a later movement or emission. A placement
that finds its destination occupied takes its ordinary refusal immediately. Collision outcomes
follow from the execution order and these admission rules; they need no separate collision
resolver. Real dependencies still take precedence over Source position.

Record the distinction in an ADR before implementing it, and reconcile the affected existing
ADRs. The design follows Orcvs's value propagation, activation, and Tick publication contracts.
Agreement with Orca in some examples is supporting comparison, not the language's authority.

## User Stories

1. As a composer, I want a westbound follower to enter Cells its leader has vacated, so that an ordinary train does not freeze the Grid.
2. As a composer, I want a northbound follower to obey the same vacancy rule, so that movement semantics do not need a separate vertical exception.
3. As a composer, I want a follower whose Turn precedes its leader's Turn to bang when blocked, so that movement depends on actual occupancy rather than anticipated motion.
4. As a composer, I want ready Turns to use Source position consistently, so that independent placements resolve deterministically.
5. As a composer, I want genuine dependencies to take precedence over Source position, so that movement does not restore the superseded row-major evaluator.
6. As a composer, I want the earlier placement to occupy a contested empty Cell, so that the later placement can take its normal refusal.
7. As a composer, I want facing movers with no gap to bang at their own Spans, so that a collision does not invent a shared or partial Bang.
8. As a composer, I want three or more converging movers to follow the same rules, so that adding another mover does not introduce a special collision mode.
9. As a composer, I want a blocked movement to affect only its prescribed local effects, so that unrelated calculations and musical effects can still run.
10. As a composer, I want a Directional Bang Function to emit into space vacated before its Turn, so that its destination is judged from current Source.
11. As a composer, I want an emission into Cells still occupied at its Turn to diagnose and emit nothing, so that it never overwrites the occupant.
12. As a composer, I want an emission and a mover blocked against each other to take their respective refusals, so that their contact alone cannot reject the Tick.
13. As a composer, I want movement at a Grid edge to bang at its origin, so that an out-of-Grid destination never creates a partial move.
14. As a composer, I want an out-of-Grid emission to leave its producer intact and diagnose, so that emission refusal remains distinct from movement refusal.
15. As a composer, I want generated Self-Banging Functions to first move on the following Tick, so that writing a Function does not give it an extra Turn immediately.
16. As a composer, I want collision Bang activation to retain its existing same-Tick dependency behavior, so that blocked movement can still trigger a contacted root.
17. As a composer, I want incomplete or misaligned contact to retain its alignment diagnostic, so that accepting vacated Cells does not weaken Language Unit rules.
18. As a composer, I want value inputs to wait for every applicable potential writer, so that a consumer sees settled operand encoding even when the supplier is later in Source order.
19. As a composer, I want a supplier that emits nothing to settle its dependency while preserving surviving operand characters, so that absence is not an unfinished input.
20. As a composer, I want a placement that can write into an operand to settle before that operand is consumed, so that occupancy admission and value readiness remain distinct.
21. As a composer, I want prior Bang display to retain its existing cleanup and activation lifetime, so that this change does not replay old pulses.
22. As a composer, I want genuine dependency cycles and illegal late overwrites to retain their rejection behavior, so that movement fixes do not weaken Tick integrity.
23. As a maintainer, I want scheduling and execution to derive placement semantics from the same operation declaration, so that another contact shape does not require another exception.
24. As a maintainer, I want executable examples through the production Source interface, so that the documented rules are checked against the behavior composers receive.
25. As a composer, I want a contest to follow the actual activation dependencies, so that moving a Bang producer across the contenders has a specified outcome even when their local arrangement is unchanged.
26. As a composer, I want an ordinary overwrite and a placement to resolve according to their Turns, so that mixed spatial effects obey the same working-Source contract.
27. As a composer, I want a later ordinary overwrite to replace a mover at its new Cells, so that moving does not grant those Cells protection from subsequent writes.

## Implementation Decisions

- **Snapshot identities.** The Source Snapshot supplies the initial parsed computations,
  positions, and operand relationships. Each scheduled computation executes at most once.
  Newly anchored Function code waits for the next Snapshot. Preserve the existing bounded
  original-anchor replacement contract; this work does not redefine it.
- **Execution order.** Dependencies determine which Turns are ready. Source position breaks
  ties among ready Turns. A placement neither waits for a hoped-for vacancy nor retries after
  refusing its destination. This is scheduled execution, not an unconditional row-major pass.
- **Rule 3a — value inputs wait for their suppliers.** Every applicable potential writer whose
  reservation covers a value input settles before its consumer executes. The consumer reads
  the resulting encoding under its existing signature. When a spatial supplier fails or produces
  no write, surviving characters remain available under the existing rules; nested inputs still
  require their typed answers. Input Portal read dependencies remain in force.
- **Rule 3b — placement tests occupancy at its Turn.** An Advance or Emit reads working Source
  when it executes. It does not wait for an occupant to leave or for other placements targeting
  the same Cells. Occupancy alone creates no ordering dependency. Existing prior-Bang cleanup
  happens before evaluation, so this test observes the cleaned and subsequently mutated Source.
- **Dependency-ordered contests.** The same local arrangement of an emitter and a mover can
  produce different winners when the emitter's Bang supplier changes position. An emitter
  waiting for a later supplier is not ready ahead of an independent mover merely because the
  emitter's anchor is earlier in Source order. This is intended language behavior.
- **Mixed overwrite and placement.** When an ordinary overwrite and a placement target Cells
  empty in the Snapshot, that destination overlap alone introduces no ordering dependency
  between them. Their actual Turns decide the result, subject to any other genuine dependencies.
  An earlier nonempty overwrite can block the placement. A later overwrite can replace the
  content a successful placement left there; empty-only admission does not reserve the Cells
  against subsequent writes.
- **Atomic Advance.** Test only newly entered Cells, excluding overlap with the mover's own
  Span. When the complete destination is in bounds and those Cells are empty, atomically clear
  the old Span and write the shifted spelling, preserving the existing overlap write order.
  Otherwise write Bang over the old Span and preserve existing contact activation and alignment
  diagnostics. A failed placement makes no partial destination write.
- **Atomic Emit.** Test the complete initial destination. When it is empty and in bounds, write
  the declared Self-Banging Function there. Otherwise diagnose and emit nothing. The Directional
  Bang Function remains at its origin and retains its existing activation requirement.
- **Declarations constrain behavior.** Use the existing Advance and Emit declaration as the
  authority for placement admission, write shape, and refusal. Scheduling and execution must
  derive their treatment from that authority. Keep the implementation at the existing Function
  declaration and Source Tick modules. Independent flags for emptiness, guard bypass, or
  refusal are not additional language choices. No new public interface or Direction type is
  required by this spec.
- **Dependencies describe what must settle.** Retain value-input, Input Portal, nesting,
  activation, Halt, and ordinary overwrite dependencies. Placement contact with a Function's
  Cells does not by itself make that Function an overwrite consumer. Retain the dependencies
  required for a mover's possible collision Bang to reach a root. A placement reservation that
  covers an operand still makes the placement a potential supplier under rule 3a. Retire the
  special treatment of mutually blocked emitter/mover pairs in favor of the general rule.
- **Guard scope.** A successful placement into currently empty Cells is not an overwrite of
  the computation that occupied those Cells in the Snapshot. Its former owner's attempted
  Turn cannot reject that placement. Preserve executed-computation rejection for ordinary
  overwrite paths reaching an attempted Snapshot computation, including a failed attempted
  computation. A later ordinary overwrite of a mover's new Cells is admitted without a
  late-write diagnostic when those Cells were empty in the Snapshot: the mover has finished
  its Turn, and its new spelling is Source content rather than another scheduled computation.
  This grants no exemption to other Snapshot computations covered by the same overwrite.
  Preserve genuine dependency-cycle rejection, whole-Tick publication, and the Source/Playback
  separation.
- **ADR reconciliation.** Record the decision and explicitly amend the relevant clauses of
  ADRs 0006, 0014, 0032, and 0034. Clarify that Snapshot ownership is not a permanent occupancy
  claim, and state rules 3a and 3b independently. Preserve ADR 0036's reservation-based value
  ordering. ADR 0031 is superseded and cannot justify a return to row-major execution.
- **Known expectation change.** The existing regression that deliberately runs an emission
  before a vacating mover to protect the stale guard must be rewritten to assert the new
  behavior. In the established northward fixture, the mover leaves first and the activated
  emission succeeds in the newly empty Cells. Retain this as an explicit language change in
  the ADR and delivery evidence, rather than claiming all prior expectations remain valid.

## Testing Decisions

- Use one existing production seam: `Source::execute`. The existing `tick_by_tick` helper
  supplies Source, executes consecutive Ticks, and returns committed Grid rows and Tick Plans.
  Exercise the actual scheduler and executor together. A new shipped test parameter, injected
  destination, synthetic effect result, or replacement execution interface is unnecessary.
- Good tests assert externally visible behavior: complete Source rows after each Tick,
  diagnostic messages or their absence, and relevant Play Commands or unrelated results.
  They do not assert helper layout, specific graph edges, or an implementation's branch order.
  Small Grid shapes remain test-only, as in the existing movement tests.
- Add failing regressions for adjacent west and north trains with space ahead, over multiple
  Ticks, and for a west train approached by an eastbound mover. Include an unrelated Addition
  or musical effect to prove that movement does not discard its Tick. Reach the Grid edge in
  a later Tick to prove that ordinary refusal still occurs.
- Preserve coverage for east and south followers blocked before their leaders move, odd gaps,
  even gaps, static obstacles, partial contact, horizontal self-overlap, and every Grid edge.
  The west/north distinction is a consequence for independent ready movers, not a separate
  direction policy; tests must not assume Source order can override real dependencies.
- Cover an emission into a destination vacated earlier in the Tick, an emission while its
  destination is still occupied, an emission that wins an empty contested destination, and a
  mover that occupies that destination first. Cover mutually blocked mover/emitter pairs in
  horizontal and vertical orientations with unrelated effects still committed.
- Pin the dependency-ordered contest explicitly. Keep an east-emitting Directional Bang
  Function at column 2 and a westbound mover at column 6 of row 1. Both want column 5. In one
  fixture a southbound mover at column 2 of row 0 collides with the emitter and supplies Bang;
  the emission wins and the westbound mover bangs. In the other, a northbound mover at column
  2 of row 2 supplies the collision Bang; the westbound mover takes its Turn before that
  supplier, enters column 5, and the later emission is refused. Use a 10-by-3 test Grid and
  assert complete rows and the emission-refusal diagnostic. These are two arrangements tested
  for one Tick each, not two activations within a single Tick. No synthetic Bang or Portal is
  needed, and the suppliers introduce no operand dependencies.
- Pin mixed overwrite/placement contests in both execution orders with destinations initially
  empty. An Addition above a westbound mover can fill its newly entered Cell first, making the
  mover bang. A southbound mover can move into an empty row before a northward Jump below
  overwrites the complete new Span with an ordinary value read from a valid operand. In the
  latter case assert the cleared origin, replacement value, and absence of a late-write
  diagnostic. These tests distinguish a destination contest from an overwrite of the mover's
  original Snapshot Cells, which retains its existing ordering and guard rules.
- Distinguish rules 3a and 3b with actual Source programs. A value consumer must observe its
  settled input even when a potential writer is later in Source order. A placement must refuse
  an occupied destination at its Turn without waiting for that occupant to leave. A placement
  whose reservation reaches an empty claimed operand must settle before the consumer reads
  its resulting encoding, including when that encoding is invalid for the operand.
- Preserve regressions for partial and competing value writes, spatial suppliers that fail or
  produce no write, nested typed answers, Input Portal reads, collision activation, Halt, and
  prior Bang cleanup. Preserve generated-code deferral over more than one Tick. Retain genuine
  cycle and late-overwrite rejection tests; a blanket removal of execution guards must fail.
- Add a property over well-formed arrangements containing only Self-Banging Functions,
  Directional Bang Functions, and empty Cells: over successive Ticks, placements and their
  local refusals do not yield executed-computation rejection or a dependency-cycle diagnostic.
  Include adjacent and offset arrangements. **Emitter activation in this property comes only
  from mover collision contact.** Generate constructive collision arrangements as well as
  arbitrary placements, including the above/below supplier pair, and assert cases with both
  successful and refused activated emissions so that inert emitters cannot satisfy all coverage.
  This property does not claim coverage of arbitrary value-produced activation. Keep arbitrary
  value-computation graphs out because genuine cycles remain valid reasons to reject their
  Ticks. A generated test run is evidence, not a proof over every Source.
- Preserve the separately tracked activation/operand cycle as a named example, not a silently
  filtered property input. Equality above an eastbound mover followed by a north-emitting
  Directional Bang Function creates an activation edge to the emitter and a potential-write
  edge back to Equality's first operand. The current result is a same-Tick dependency-cycle
  diagnostic and no committed effects. Deciding whether that conservative cycle should instead
  be admitted belongs to the separate investigation; placement work must not drop value-input
  readiness to make the no-rejection property pass.
- Prior art includes the existing odd-gap, converging-mover, contested-emission, flush-pair,
  generated-mover, aligned-root-contact, partial-writer, failed-supplier, and late-overwrite
  regressions. Update the vacating-mover regression rather than deleting its coverage.
- Run the repository's scoped Rust verification for changed crates and dependents, using 32
  local property cases. Run the issue-tracker checks when updating this effort. Apply additional
  gates only when their inputs change, and report deferred CI coverage explicitly.

## Out of Scope

- Simultaneous movement intentions, vacancy-dependency solving, swaps, movement-loop resolution,
  or refusing every contender because their destinations overlap.
- Snapshot-only occupancy admission, direction-independent trains, a separate movement phase,
  repeated evaluation, or placement retries within a Tick.
- Restoring Orca's row-major evaluator, Bang lifetime, single-Cell syntax, or general compatibility.
- New Function spellings, new movement directions, general Function-valued output, or broader
  Function replacement semantics.
- A generic collision framework, persistent vacancy metadata, dynamic reparsing between Turns,
  unrelated Source-module restructuring, new dependencies, or performance claims.
- Changes to the console presentation, Playback timing, persistence format, or platform support.
- Changing admission of activation cycles through emission reservations over value operands;
  that question has its own investigation and is not required to deliver this placement rule.

## Further Notes

The [mover semantics walkthrough](../../lang/mover-semantics-prototype.html) motivated this
decision. This spec selects scheduled placement against working Source, with the explicit
value-readiness/occupancy distinction agreed in discussion. The alternatives in that walkthrough
are not additional implementation modes.

This follows [the earlier collision decision](../spatial-tick-planning/issues/07-decide-the-mover-collision-rule.md).
Keep its resolved history; this effort reconciles placement with the dependency model and
deliberately replaces its vacating-mover workaround. It is not automatically added to a release
scope by inheriting that earlier issue's tag.

Review of the walkthrough confirmed the central defect, but found inaccuracies: the cited August
implementation read the pre-Tick Snapshot rather than working Source; the old flush-pair failure
was a same-Tick dependency cycle, not the late-write diagnostic; and the hypothetical converging
frame places the rear Bang one Cell too far west. Its historical 684-test prototype claim does
not cover the current branch's contrary vacating-mover expectation. Use the selected contract and
newly executed tests as delivery evidence, rather than copying prototype frames uncritically.

Implementation and ADR delivery are tracked in [issue 01](issues/01-reconcile-placement-with-value-readiness.md).
The approved delivery slices are [movers](issues/02-let-movers-enter-vacated-cells.md), followed by
[emissions](issues/03-apply-turn-local-occupancy-to-emissions.md). The
[activation/operand cycle investigation](issues/04-investigate-emission-activation-operand-cycles.md)
is independent and does not block either slice.

The ADR review examples were executed against the current branch through `Source::execute`.
Both activation-contest outcomes, the two mixed overwrite orders, and the activation/operand
cycle already reproduce there. They are contracts to preserve and make explicit; west/north
train admission and emission into a just-vacated destination remain the intended behavior changes.
