# Value inputs wait; placement tests occupancy at its Turn

Status: accepted. Delivery is split between the [mover](../../.scratch/placement-semantics/issues/02-let-movers-enter-vacated-cells.md) and [emission](../../.scratch/placement-semantics/issues/03-apply-turn-local-occupancy-to-emissions.md) slices; emission scheduling remains pending after the mover slice. The cycle clause is amended by [ADR 0065](0065-an-error-never-stops-the-performance.md).

Orcvs keeps one dependency-ordered Tick against working Source. Snapshot ownership identifies
computations; it does not reserve their original Cells after they move. This amends the placement
and overwrite-ordering clauses of ADRs 0006, 0014, 0032, and 0034 while preserving ADR 0036's
reservation-based value ordering. ADR 0031's row-major evaluator remains superseded.

**Rule 3a: value inputs wait for their suppliers.** Every applicable potential writer whose
reservation covers a value input settles before its consumer executes. The consumer reads the
surviving encoding under its current signature. A spatial supplier that fails or produces no write
still settles; a nested input waits for its child's Return, the two-Cell encoding the receiving
operand decodes by its declared literal type exactly as it decodes a spatial write, as
[ADR 0061](0061-a-nested-function-returns-to-the-slot-it-occupies.md) decides. A nested Function's
own Output Portal write is a potential writer like any other. Input Portal read dependencies remain.

**Rule 3b: placement tests occupancy at its Turn.** An Advance or Emit reads the working Grid
at its scheduled Turn, after prior Bang cleanup and earlier admitted effects. It does not wait
for an occupant to leave or for other placements contesting the same Cells. Occupancy alone
creates no ordering edge, and a refused placement does not retry. Value-input, activation, Halt,
and ordinary overwrite dependencies still apply; Source position breaks ties only among ready
Turns. A placement covering an operand remains a potential supplier under rule 3a.

Advance tests newly entered Cells, excluding overlap with its own Span. Success atomically clears
its origin and writes the shifted spelling; refusal writes Bang at its origin and retains existing
contact activation and alignment diagnostics. Emit tests its complete destination, writes the
declared mover on success, and diagnoses without emitting on refusal. Both require an in-Grid
empty destination. Newly anchored Function code first executes from the next Snapshot.

A successful placement into vacated Cells does not overwrite their former owner's computation.
The executed-computation guard applies to ordinary overwrites reaching attempted Snapshot
computations, including failed attempts. A later overwrite may replace a mover's new spelling
without diagnosing when those Cells were empty in the Snapshot: the mover has finished its Turn,
and the new spelling is Source content, not another scheduled computation. Other Snapshot
computations covered by that overwrite retain their protection. Overwrite/placement destination
overlap alone introduces no dependency: an earlier nonempty overwrite blocks placement; a later
overwrite replaces what was placed.

An emission/mover contest follows the actual schedule. A Bang supplier above the contenders can
make the emission win; a supplier below can leave the emitter waiting while the independent mover
wins. This is intended even when the local arrangement of the contenders is identical. A follower
moves into space its leader has already vacated, but cannot demand the leader run first.

We choose these rules to preserve same-Tick value propagation with local placement refusal and
one execution order. Snapshot-only admission would forbid following into newly vacated space;
simultaneous vacancy solving would require additional contest, swap, and cycle rules. Neither is
adopted, and matching Orca is not a compatibility requirement.

Genuine dependency cycles still stop every Expression they reach, and no other. The activation/operand cycle
where an emission reserves its Bang supplier's literal input is a [separate investigation](../../.scratch/placement-semantics/issues/04-investigate-emission-activation-operand-cycles.md), not permission to drop input-readiness edges. Bang lifetime, bounded original-anchor replacement, and Source/Playback separation are unchanged.
