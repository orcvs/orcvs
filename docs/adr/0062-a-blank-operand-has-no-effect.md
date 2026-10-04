# A blank operand has no effect

Status: accepted. Not yet implemented; tracked in `.scratch/cell-tracker/issues/02`.

**An inline input portal whose Cells are all empty makes its Function do nothing that Tick, without a diagnostic.** A value Function answers the Absence Marker and plans no write. A Terminal Output Function emits no command. A Bang still activates the Function, and it still does nothing. A portal with some Cells empty and others not, such as the `" 0"` in `.+01 02` that [ADR 0033](0033-partition-a-row-by-parse.md) describes, is malformed and diagnoses as before. Blank operands still occupy the claim, because a claim is the arity and not the content.

A blank operand is ordinary content, not an error. Clearing a slot is something a performer does by hand and a Portal does by writing spaces ([ADR 0009](0009-portals-resolve-tick-plan-destinations.md)). A blank item a Track copies into Timed Play's note slot is a rest ([ADR 0063](0063-a-list-is-cells-in-a-claim-not-a-value.md)). Treating each of these as a fault would fill the diagnostics with silence that was intended, and it would hide the malformed operands that diagnostics exist to report. Orca behaves the same way: an operator with an empty port has nothing to do.

A blank Return carries the same meaning to the portal that receives it. A nested Function that answers blank leaves its enclosing portal blank, so the enclosing Function does nothing either.
