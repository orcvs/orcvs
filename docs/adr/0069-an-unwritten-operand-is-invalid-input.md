# An unwritten operand is invalid input

Status: accepted. Supersedes [ADR 0066](0066-an-unwritten-operand-leaves-its-function-pending.md): Orcvs has no pending state. Amends the clauses of [ADR 0032](0032-schedule-tick-execution-by-dependency.md), [ADR 0034](0034-execute-against-live-typed-expressions.md), [ADR 0061](0061-a-nested-function-returns-to-the-slot-it-occupies.md), [ADR 0063](0063-a-list-is-cells-in-a-claim-not-a-value.md) and [ADR 0067](0067-track-reads-a-portal-as-jump-does.md) that defer to ADR 0066.

**A Function at its Turn either evaluates or is invalid.** An inline operand slot whose Cells are all empty is invalid input, handled exactly as a slot with some Cells written is: the Function does not evaluate, writes nothing, returns nothing and is diagnosed. This holds whether the Source was written that way or a Portal write emptied the slot during the Tick. A nested Function that is invalid gives its parent no operand, so the parent fails under ADR 0034's nested-consumer failure clause. There is no third outcome: the Language Map's roots, the schedule and every Turn treat an unwritten slot as they treat any other invalid input.

**An invalid Function leaves what it wrote before.** It writes nothing, so a Cell it wrote on an earlier Tick keeps its characters until something overwrites it. This is ADR 0034's rule that a failed producer does not erase the encodings it leaves, and it holds for every invalid Function, whatever made it invalid. Orca does the same where an operator produces nothing: `output` returns early on an empty payload, and an operator that is deleted or covered leaves its last output standing.

**Pending describes an invalid Function; it is not a state.** A Function whose spelling is known but whose arguments are not yet valid is pending in the ordinary sense of the word: written arguments that are malformed and Cells not yet written are alike. Nothing in execution, scheduling or diagnostics distinguishes the two. Source Paint already shows an unwritten operand in its declared type's colour rather than as a fault, from the Source alone.

**An empty Cell is a character, and a Function that copies Cells copies it.** A Jump whose aligned input is empty writes the empty Cells to its destination, as Orca's `J` copies `.`, and clears it. Nested, it returns those Cells, so the operand it stands in is unwritten and its parent is invalid. Track copies the Cells at its Input Portal to its Output Portal in the same way (ADR 0067), whatever they hold. Copying empty Cells is a successful evaluation; only the Functions whose job is to copy Cells move empty ones.

**A Bang activates a Function; its operands decide whether it can run.** A Bang that reaches a Play whose note slot is unwritten finds an invalid Play, which emits no Play Command. A value Function that writes into that slot through Cells and is itself invalid leaves its last answer there, and the next Bang plays it.

**The Absence Marker is an answer.** Equality with unequal operands, and Delay or Euclidean on a Tick they do not fire, evaluate and answer the Absence Marker. A nested Function that answers it returns nothing, and its parent fails under ADR 0034.

## Consequences

Every unwritten slot is diagnosed. The console shows no Source or Tick diagnostic messages today, so this changes nothing a performer sees.

Increment and Interpolation keep their state while an operand is unwritten, because their state is the Cells they wrote, and continue from it once the operand is written again.

An Expression with an unwritten slot is no longer excluded from the Language Map's roots; it is refused as a partly written one is.

## Considered options

- **Pending as an execution state (ADR 0066).** A Function with an unwritten slot did not evaluate, was not diagnosed and left its parent pending. Rejected: it added a third outcome that the Language Map's roots, the schedule and every Turn had to carry, to separate unwritten arguments from malformed ones, a distinction nothing in the language needs.
- **An invalid Function clears its Output Portal.** Rejected: any mistake would wipe the Cells south of it; Increment and Interpolation would lose their state on every error; and it would write an empty value that no Function answers.
- **An empty operand reads as zero, as in Orca.** Orca's `listen` reads an empty port as `0` or the port's default (`desktop/sources/scripts/core/operator.js`), so `A` and `C` run on zero and write every frame. Rejected: an empty Cell copied by Track would stay empty through a Jump but become `00` as soon as arithmetic touched it, so a transposition `.+0C@t…` would write `0C` wherever Track copied an empty pair. Reading empty as zero for Numbers but not Notes does not help, because a returned Note enters `.+` as a Number (ADR 0061).
