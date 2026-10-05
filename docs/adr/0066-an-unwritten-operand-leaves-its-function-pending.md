# An unwritten operand leaves its Function pending

Status: accepted. Supersedes [ADR 0062](0062-a-blank-operand-gives-a-blank-answer.md): Orcvs has no Blank Answer. Amends the clauses of [ADR 0032](0032-schedule-tick-execution-by-dependency.md), [ADR 0034](0034-execute-against-live-typed-expressions.md) and [ADR 0061](0061-a-nested-function-returns-to-the-slot-it-occupies.md) that defer to ADR 0062.

**A Function is pending until every input Cell of its operands is written.** An inline operand slot whose Cells are all empty leaves its Function pending: the Function does not evaluate, writes nothing, returns nothing and is not diagnosed. This holds whether the Source was written that way or a Portal write emptied the slot during the Tick. A nested Function that is pending leaves its parent pending in turn. A slot with some Cells written and others empty, such as the `" 1"` in `.+ 101`, is malformed and diagnoses as before, and so does a slot the row edge cuts short.

An empty slot is input that has not arrived, not a value. Typing a Function and then its operands passes through states where some slots are empty, and diagnosing each of them would fill the diagnostics with Source that is unfinished rather than wrong.

**A rest is a Tick on which no Bang activates the Play.** A Bang supplies the timing and the note slot supplies what plays when one arrives. A Note left in a Play's note slot is not replayed until a Bang activates the Play, so no value is needed to clear it. A Play whose note slot is empty is pending, so a Bang that reaches it plays nothing.

**Pending is not the Absence Marker.** Equality with unequal operands, and Delay or Euclidean on a Tick they do not fire, evaluate and answer the Absence Marker. A nested Function that answers it returns nothing, and its parent diagnoses the missing Return under ADR 0034's nested-consumer failure clause. A pending Function does not evaluate, so it answers nothing and its parent is pending rather than diagnosed.

## Consequences

Increment and Interpolation keep their state while an operand is empty, and continue from it once the operand is written again.

A Jump whose aligned input is empty still clears its destination. An operand it clears leaves the Expression that reads it pending.
