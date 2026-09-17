# 02 — Tint Functions and Operands by their Token colour

**What to build:** Every Cell of a Function or an Operand gets a background tint of its Token colour, at a fill strength the user sets in Theme → Source colours, so an Expression reads as a run of typed units.

**Blocked by:** 01 — Expose Source colours as Theme settings.

**Status:** ready-for-agent

- [ ] A Function Cell, nested Functions included, is tinted with the Function colour. An Operand Cell is tinted with its declared Token colour: Number, Note, Char, Atom or Sequence.
- [ ] The tint is the Token colour mixed over the Source background at the Fill tint strength.
- [ ] Theme → Source colours exposes Fill tint as a percentage control, default 16%, range 0–100. 0% paints no tint. Reset restores 16% with the colours; persistence stores it with them.
- [ ] Comment, Bang, empty unclaimed Cells and Leftover Chars are not tinted.
- [ ] The glyph keeps its Token colour on the tint.
- [ ] On the Cursor Cell, the Cursor's own fill wins over the tint.
- [ ] Adjacent tinted Cells of the same colour paint as one run rather than per Cell.
- [ ] Focused paint tests cover a tinted Function, a tinted Operand of each Token, 0%, and the Cursor Cell.

## Comments

Prototype: `?variant=A&palette=okabe` in `console/prototypes/syntax-highlighting/source-paint-prototype.html`, where the tint is 16%.
