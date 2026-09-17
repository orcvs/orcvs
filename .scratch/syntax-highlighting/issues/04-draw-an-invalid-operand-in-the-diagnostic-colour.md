# 04 — Draw an Invalid Operand in the diagnostic colour

**What to build:** When an operand's Cells fail to bind as its declared Token, those Cells keep their Token tint and draw their glyphs in the Diagnostic colour. A lone `|` draws in the Diagnostic colour too.

**Blocked by:** 02 — Tint Functions and Operands by their Token colour.

**Status:** ready-for-agent

- [ ] `.+c40G`: both operands keep the Number tint; `c4` and `0G` draw in the Diagnostic colour.
- [ ] `.+**01` and `.+||02`: the rejected `**` and `||` keep the Number tint and draw in the Diagnostic colour; `01` and `02` are unchanged.
- [ ] A lone `|` draws in the Diagnostic colour with no tint.
- [ ] The fact comes from the Parser's own record that the operand bound no Atom, exposed through the Language Map to the Render Frame. The Expression-wide Diagnostic Span and the generic one-Cell lexical diagnostic are not used to infer it, and no new per-Cell classification vocabulary is minted.
- [ ] A valid Operand beside an invalid one in the same Expression is unaffected.
- [ ] Evaluation-time operand diagnostics (e.g. a zero divisor) are out of scope: they are Tick outcomes, not Source facts.
- [ ] Focused tests cover each case above at the Render Frame and at the paint.

## Comments

The Token stays the declared one: `c4` in a Number slot is still Number, which is why the tint stays.
