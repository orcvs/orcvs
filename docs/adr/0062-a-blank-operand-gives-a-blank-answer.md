# A blank operand gives a blank answer

Status: superseded by [ADR 0066](0066-an-unwritten-operand-leaves-its-function-pending.md), which is itself superseded by [ADR 0069](0069-an-unwritten-operand-is-invalid-input.md): an unwritten operand is invalid input, and Orcvs has neither a Blank Answer nor a pending state.

**A blank answer is two spaces.** It is what a value Function answers when one of its inline input portals is blank, and it is delivered like any other answer: the Function writes the two spaces through its Output Portal, and when it is nested its Return is those two blank Cells. Blankness therefore travels one way, whether it arrives at a slot through nesting or through a Portal, and the slot that receives it is blank in turn.

**An inline input portal whose Cells are all empty makes its value Function give a blank answer, without a diagnostic.** A Terminal Output Function with a blank operand emits no command. A Bang still activates the Function, and it still gives a blank answer. A portal with some Cells empty and others not, such as the `" 0"` in `.+01 02` that [ADR 0033](0033-partition-a-row-by-parse.md) describes, is malformed and diagnoses as before. Blank operands still occupy the claim, because a claim is the arity and not the content.

A blank operand is ordinary content, not an error. Clearing a slot is something a performer does by hand and a Portal does by writing spaces ([ADR 0009](0009-portals-resolve-tick-plan-destinations.md)). Track copies a blank item into Timed Play's note slot as it copies any other ([ADR 0063](0063-a-list-is-cells-in-a-claim-not-a-value.md)). Treating each of these as a fault would fill the diagnostics with silence that was intended, and it would hide the malformed operands that diagnostics exist to report. Orca behaves the same way: an operator with an empty port has nothing to do. *Corrected by the 2026-10-08 amendment below.*

**A blank answer writes, so a consumer fed through a Portal does not replay a stale value.** A value root whose Output Portal feeds Timed Play's note slot below it clears that slot when its own operand goes blank, and the Timed Play that a Bang then activates emits no Play Command. A blank answer that planned no write would leave the previous Note in the slot, and the next Bang would play it again. A nested blank Return makes the enclosing value Function give a blank answer too: it writes spaces at its own Output Portal and returns blank to its parent.

**The Absence Marker is not blank.** Equality with unequal operands, and Delay or Euclidean on a Tick they do not fire, answer the Absence Marker, which keeps its existing meaning: no write, so the destination keeps its characters, and no activation. This decision gives the Absence Marker no Source encoding and does not turn it into a blank answer. Only an all-empty inline operand, or a blank Return or Portal write that leaves one, produces a blank answer.

## Consequences

A blank answer from a Function whose Output Portal is also its input clears its state. A blank operand to Increment or Interpolation writes spaces over the feedback Cell they share with their output, and on the following valid evaluation they read the cleared Cell as their initial `00`, so the count or glide restarts. A performer who blanks an operand for one Tick loses that state; keeping it would need a no-write exception for exactly the Functions whose Output Portal is an input, the exception [ADR 0061](0061-a-nested-function-returns-to-the-slot-it-occupies.md) declines for nesting.

## Amendment, 2026-10-08: Orca reads an empty port as zero

The claim above that "Orca behaves the same way: an operator with an empty port has nothing to do" is wrong. Orca's `listen` (`desktop/sources/scripts/core/operator.js`) reads an empty port as `0` or the port's default, so `A` and `C` run on zero and write every frame, and `T` copying an empty Cell writes `.`. Only the I/O operators `:`, `%`, `!`, `?` and `=` return early when a port they need is empty. [ADR 0069](0069-an-unwritten-operand-is-invalid-input.md) records why Orcvs does not read empty as zero.
