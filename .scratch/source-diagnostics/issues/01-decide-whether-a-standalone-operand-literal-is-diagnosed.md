# 01 — Record that a standalone Operand Literal is not diagnosed

Status: ready-for-agent
Blocked by: None — can start immediately

**What to build:**

An ADR answering ADR 0067's open question. Today the Language Map reports each pair east of Track as an unknown Function, two diagnostics per Cell, because a refused Function advances one Cell.

The decision: a two-Cell Language Unit that spells a Number (`00`–`FF`) or a Note (`C4`, `c4`, `C/`) where an Expression could start is not diagnosed, and the row walk consumes it as one Unit. Every other spelling that names no Function, such as `Z ` or `id`, is still diagnosed. The cost, stated in the ADR: operands a deleted or replaced Function leaves behind are no longer diagnosed. They already paint as ordinary Source, and Orca leaves them inert.

## Acceptance criteria

- [ ] A new ADR records the decision, its cost and the rejected options (keep diagnosing; diagnose no spelling that names no Function, which silences mistyped Function names), and amends ADR 0067's open question.
- [ ] `CONTEXT.md`'s Operand Literal entry states the decision instead of calling it open.
- [ ] Cell-tracker 06's diagnostic criterion no longer needs to leave the pairs east of Track unpinned; note this on that ticket.

## Comments

### Decided 2026-10-16 by Toby

Settled while grilling `.scratch/source-diagnostics/`: option (b) of three. Implementation is ticket 04.
