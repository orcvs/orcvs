# 01 — Record that a standalone Operand Literal is not diagnosed

Status: ready-for-agent
Blocked by: None — can start immediately

**What to build:**

An ADR answering the question ADR 0067 leaves to the Parser, and reversing ADR 0021's clause that "a standalone operand literal without typed Expression context is invalid". Today the Language Map reports each pair east of Track as an unknown Function, two diagnostics per Cell, because a refused Function advances one Cell.

The decision: a two-Cell Language Unit that spells a Number (`00`–`FF`) or a Note (`C4`, `c4`, `C/`) where an Expression could start is not diagnosed, and the row walk consumes it as one Unit. Every other spelling that names no Function, such as `Z ` or `id`, is still diagnosed. The cost, stated in the ADR: operands a deleted or replaced Function leaves behind are no longer diagnosed. They already paint as ordinary Source, and Orca leaves them inert.

The row walk reads a pair from wherever it stands, as it reads a Function name, so a pair's alignment follows the Units before it: in `1C4.` the walk consumes `1C` as a Number and then reads from `4`, so `C4` is never read as a Note. The ADR states this rather than leaving alignment to the Parser.

## Acceptance criteria

- [ ] A new ADR records the decision, its cost and the rejected options (keep diagnosing; diagnose no spelling that names no Function, which silences mistyped Function names), amends ADR 0067's "a question for the Parser" paragraph, and amends ADR 0021's standalone-literal clause, with each amended ADR's status line pointing at the new one.
- [ ] `CONTEXT.md`'s Operand Literal entry states the decision instead of calling it open.

## Comments

### Decided 2026-10-16 by Toby

Settled while grilling `.scratch/source-diagnostics/`: option (b) of three. Implementation is ticket 04.
