# An unwritten operand leaves its Function pending

Status: accepted. Supersedes [ADR 0062](0062-a-blank-operand-gives-a-blank-answer.md): Orcvs has no Blank Answer. Amends the blank-item clause of [ADR 0063](0063-a-list-is-cells-in-a-claim-not-a-value.md). Amends the clauses of [ADR 0032](0032-schedule-tick-execution-by-dependency.md), [ADR 0034](0034-execute-against-live-typed-expressions.md) and [ADR 0061](0061-a-nested-function-returns-to-the-slot-it-occupies.md) that defer to ADR 0062.

**A Function is pending until every input Cell of its operands is written.** An inline operand slot whose Cells are all empty leaves its Function pending: the Function does not evaluate, writes nothing, returns nothing and is not diagnosed. This holds whether the Source was written that way or a Portal write emptied the slot during the Tick. A nested Function that is pending leaves its parent pending in turn. A slot with some Cells written and others empty, such as the `" 1"` in `.+ 101`, is malformed and diagnoses as before, and so does a slot the row edge cuts short.

An empty slot is input that has not arrived, not a value. Typing a Function and then its operands passes through states where some slots are empty, and diagnosing each of them would fill the diagnostics with Source that is unfinished rather than wrong.

**Bangs are the timing.** A Bang supplies when a Play sounds and the note slot supplies what it plays. A Note left in a Play's note slot plays again only when a Bang activates the Play. A rest is a step on which no Bang reaches the Play, or on which its note slot is empty, so the Play is pending and the Bang plays nothing.

**An empty Cell is a character, and a Function that copies Cells copies it.** A Jump whose aligned input is empty writes the empty Cells to its destination, as Orca's `J` copies `.`, and clears it. Nested, it returns those Cells, so the operand it stands in is empty and its parent is pending. Track copies the pair it reads the same way ([ADR 0067](0067-track-reads-a-portal-as-jump-does.md)): an empty pair clears the Cells south of it, as Orca's `T` does, and a Play whose note slot it clears is pending when a Bang reaches it. No Function answers an empty value; only the Functions whose job is to copy Cells move empty ones.

**Pending is not the Absence Marker.** Equality with unequal operands, and Delay or Euclidean on a Tick they do not fire, evaluate and answer the Absence Marker. A nested Function that answers it returns nothing, and its parent diagnoses the missing Return under ADR 0034's nested-consumer failure clause. A pending Function does not evaluate, so it answers nothing and its parent is pending rather than diagnosed.

## Consequences

Increment and Interpolation keep their state while an operand is empty, and continue from it once the operand is written again.

A value Function with an empty operand writes nothing, so a Cell it wrote on an earlier Tick keeps its characters until something overwrites it.
