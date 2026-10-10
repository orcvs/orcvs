# 11 — Copy a pair between Positions

Status: resolved
Blocked by: 04 — Decide how a dynamic Output Portal is ordered; 05 — Write a value in a direction or at a Position; 07 — Accept the Read, Write and Copy families; 08 — Rename Jump to Copy; 10 — Read a pair at a Position

**What to build:**

The absolute Copy `=$ src-column src-row dst-column dst-row` (ADR 0070). It reads the pair at the source Position with Copy's reading rules and writes it to the destination Position, leaving the source as it was. It takes 10 Cells where the composition `@$ c r &$ c r` takes 12. Its read is a dynamic Input Portal ordered at its Turn, as `&$`'s is; its write is a dynamic Output Portal ordered as ticket 04 decides.

## Acceptance criteria

- [x] `=$` is in the Function table with four Number operands; an unwritten one makes it invalid (ADR 0069).
- [x] It copies exactly what `@$ dst-column dst-row &$ src-column src-row` would, on the same Tick, for `**`, Function-spelling, Number and Note pairs, and diagnoses the same partial, Comment and straddling pairs. An empty pair it copies as every Copy does, clearing the destination.
- [x] A source or destination outside the Grid, or cut short by the row edge, diagnoses and writes nothing.
- [x] Tests drive writers of the source and readers of the destination before and after it in Grid order: a static writer of the source goes first, and readers of the destination that do not feed the `=$` see its write in the same Tick.
- [x] A `=$` whose source and destination are the same pair rewrites it unchanged, and one whose pairs overlap by one Cell writes on its first Tick and, on the next, diagnoses straddling input where a copied Function spelling leaves it, or copies again where the Cells still form one unit; neither is a cycle.
- [x] `=$` answers the pair it copies through its dynamic Output Portal and writes nothing one row south; nested, it returns that pair to its parent.
- [x] A `=$` nested in a value Function whose south Output Portal covers the `=$`'s source is diagnosed as a cycle under ADRs 0065 and 0068.
- [x] A Source holding `=$` gives the same result from a reused schedule as from a fresh one.
- [x] The Function reference names `=$`. ADR 0019's table and the glossary already do, from ADR 0070's acceptance.

## Comments

On its first build, every criterion is met except the empty pair in the second, which is left open for a decision.

`=$` reads an empty source pair as every Copy does (CONTEXT.md's Copy Function entry): it clears the destination and, nested, returns the empty Cells. The composition `@$ dst &$ src` does not: the nested Read returns the empty Cells, and an empty `value` makes the Write invalid (ADR 0070's amendment, "a Write carries its value"), so it diagnoses `expected a Language Unit, found "  "` and writes nothing. The Number, Note, Function-spelling and `**` pairs agree, and so do the partial, Comment and straddling ones.

Which should `=$` follow for an empty source: Copy's rule, which it follows now and which lets a rest clear a step, or the composition's, which would need a branch that refuses an empty read for `=$` alone?

## Resolution

`=$` follows the Copy family, not the composition: an empty source pair clears the destination and, nested, returns the empty Cells, as every `=` Function does. The composition `@$ dst &$ src` differs only there, because a Write's empty `value` is invalid. Built as it stood; no code changed.
