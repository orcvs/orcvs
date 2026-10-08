# An unwritten operand is invalid input, and pending is how it is shown

Status: accepted. Supersedes [ADR 0066](0066-an-unwritten-operand-leaves-its-function-pending.md): Orcvs has no pending state in execution. Amends the clauses of [ADR 0032](0032-schedule-tick-execution-by-dependency.md), [ADR 0034](0034-execute-against-live-typed-expressions.md), [ADR 0061](0061-a-nested-function-returns-to-the-slot-it-occupies.md), [ADR 0063](0063-a-list-is-cells-in-a-claim-not-a-value.md) and [ADR 0067](0067-track-reads-a-portal-as-jump-does.md) that defer to ADR 0066.

**A Function at its Turn either evaluates or is invalid.** An inline operand slot whose Cells are all empty is invalid input, handled exactly as a slot with some Cells written is: the Function does not evaluate, writes nothing, returns nothing and is diagnosed. This holds whether the Source was written that way or a Portal write emptied the slot during the Tick. A nested Function that is invalid gives its parent no operand, so the parent fails under ADR 0034's nested-consumer failure clause. There is no third outcome: the Language Map's roots, the schedule and every Turn treat an unwritten slot as they treat any other invalid input.

**An invalid Function leaves what it wrote before.** It writes nothing, so a Cell it wrote on an earlier Tick keeps its characters until something overwrites it. This is ADR 0034's rule that a failed producer does not erase the encodings it leaves, and it holds for every invalid Function, whatever made it invalid. Orca does the same where an operator produces nothing: `output` returns early on an empty payload, and an operator that is deleted or covered leaves its last output standing.

**Pending is a presentation of an invalid Function, not a state.** The Language Map knows each slot's declared literal type and whether its Cells are written, so it can tell a slot that has not been written from one that is wrong. A diagnostic whose cause is an unwritten slot, in the Function itself or in a nested Function it waits on, is classified pending: the Function is waiting for a Number or a Note in that slot. The console presents a pending diagnostic as unfinished Source rather than as a fault, so typing a Function and then its operands does not read as a stream of errors. The classification changes what the performer sees, never what runs.

**An empty Cell is a character, and a Function that copies Cells copies it.** A Jump whose aligned input is empty writes the empty Cells to its destination, as Orca's `J` copies `.`, and clears it. Nested, it returns those Cells, so the operand it stands in is unwritten and its parent is invalid, presented as pending. Track copies the pair it reads the same way (ADR 0067). Copying empty Cells is a successful evaluation; only the Functions whose job is to copy Cells move empty ones.

**Bangs are the timing, and a rest is an invalid Play.** A Bang supplies when a Play sounds and the note slot supplies what it plays. A Bang that reaches a Play whose note slot is unwritten finds it invalid: it emits no Play Command and is presented as pending. A rest is therefore silent where the empty pair reaches the Play's note slot directly, by Track's or a Jump's write, or through nesting, where the Play fails with its child. A value Function that stands between Track and the Play and delivers through Cells leaves its last answer there when it is invalid, and the next Bang plays it again.

**The Absence Marker is a fault, not pending.** Equality with unequal operands, and Delay or Euclidean on a Tick they do not fire, evaluate and answer the Absence Marker. A nested Function that answers it returns nothing, and its parent's failure is diagnosed as a fault. Both make the parent fail; only the cause, and so the presentation, differs.

## Consequences

Every unwritten slot is diagnosed, including each rest on which a Bang reaches a Play. How the console keeps pending diagnostics from reading as faults, in Source Paint and in Diagnostics, decides whether this costs anything in use.

Increment and Interpolation keep their state while an operand is unwritten, because their state is the Cells they wrote, and continue from it once the operand is written again.

A transposed tracker places its arithmetic inside the Play's note slot, `!>…` holding `.+0C@t…`, so a rest reaches the Play through nesting. Wiring the same arithmetic to the Play through Cells replays the last Note on a rest.

An Expression with an unwritten slot is no longer excluded from the Language Map's roots; it is refused as a partly written one is.

## Considered options

- **Pending as an execution state (ADR 0066).** A Function with an unwritten slot did not evaluate, was not diagnosed and left its parent pending. Rejected: it added a third outcome that the Language Map's roots, the schedule and every Turn had to carry, for a distinction that matters only to how unfinished Source is shown. The presentation can draw it from the same information without changing what runs.
- **An invalid Function clears its Output Portal.** A rest would stay silent through Cells as well as nesting. Rejected: any mistake, not only an unwritten slot, would wipe the Cells south of it; Increment and Interpolation would lose their state on every error; and it would write an empty value that no Function answers.
- **An empty operand reads as zero, as in Orca.** Orca's `listen` reads an empty port as `0` or the port's default (`desktop/sources/scripts/core/operator.js`), so `A` and `C` run on zero and write every frame. Rejected: a rest would survive a copy, as Track's empty pair reaching Timed Play, but become a note as soon as arithmetic touched it. A transposed tracker `.+0C@t…` would play `0C` at every rest. Reading empty as zero for Numbers but not Notes does not help, because a returned Note enters `.+` as a Number (ADR 0061).
