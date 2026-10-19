# 01: Carry Portal roles on each Cell, with an Input Portal colour

**What to build:** Prefactor. Each Render Cell carries a small set of Portal
roles in place of its single Output Portal flag, and both built-in themes gain
an Input Portal colour that the contrast checks cover. Nothing paints
differently yet: only the Output Portal role is ever set, from the existing
static highlight.

**Blocked by:** None (can start immediately)

**Status:** ready-for-agent

- [ ] Render Cells expose Portal roles (Output Portal, Input Portal) instead of
      a boolean.
- [ ] Both built-in themes define Input Portal; the contrast checks cover every
      Source role under it, with and without the Cursor.
- [ ] Precedence follows the theming schema: Portal over every non-Function
      fact, Diagnostic included; Output Portal over Input Portal; a bound
      Function keeps its own paint; a Bang keeps its glyph and takes the Portal
      background; Cursor and Region over every Portal.
- [ ] Every existing paint and contrast test passes unchanged, including Output
      Portal over an invalid operand.
