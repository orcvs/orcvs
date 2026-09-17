# 03 — Show a Pending Operand as its tint, with no letter

**What to build:** An operand a Function has claimed but nothing fills shows its declared Token only as the background tint. No placeholder letter is drawn.

**Blocked by:** 02 — Tint Functions and Operands by their Token colour.

**Status:** ready-for-agent

- [ ] An empty claimed operand Cell of every Token (Number, Note, Char, Atom, Sequence) shows the Token tint and no character.
- [ ] The placeholder spellings (`h`, `n`, `c`, `F`, `*`) and the blank-character table that produces them are deleted, with the test that pins them.
- [ ] An operand slot cut off by the end of its row also shows its declared Token tint, for each of the slot's Cells that exist in the Grid. Today that slot records no Cells and carries no Token; establish where that fact belongs in the Language Map rather than adding a per-Cell classifier beside it.
- [ ] `typed-source-paint/02` is resolved by this ticket in the "painted, not spelled" direction, with a pointer to this ticket as its answer.
- [ ] Focused tests cover an empty Number and Note slot, and a slot cut off at the row edge.

## Comments

Pending Operand is distinct from Pending Operand Encoding: this is only the claimed-and-empty case.
