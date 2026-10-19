# 01: Carry Portal roles on each Cell, with an Input Portal colour

**What to build:** Prefactor. Each Render Cell carries a small set of Portal
roles in place of its single Output Portal flag, and both built-in themes gain
an Input Portal colour that the contrast checks cover. Nothing paints
differently yet: only the Output Portal role is ever set, from the existing
static highlight.

**Blocked by:** None (can start immediately)

**Status:** resolved

- [x] Render Cells expose Portal roles (Output Portal, Input Portal) instead of
      a boolean.
- [x] Both built-in themes define Input Portal; the contrast checks cover every
      Source role under it, with and without the Cursor.
- [x] Precedence follows the theming schema: Portal over every non-Function
      fact, Diagnostic included; Output Portal over Input Portal; a bound
      Function keeps its own paint; a Bang keeps its glyph and takes the Portal
      background; Cursor and Region over every Portal.
- [x] Every existing paint and contrast test passes unchanged, including Output
      Portal over an invalid operand.

## Comments

2026-10-19 — `RenderCell::portal_roles()` answers `PortalRoles`, a `u8` set of
`PortalRole::{Output, Input}`; derivation sets Output from the static highlight
and never Input. The console resolves the painting Portal once (Output over
Input; the two do not composite) and its once-per-Paint table holds 10 facts by
3 Portal answers. `input_portal.{foreground,background,border,border.width}`
mirror Output Portal's keys in the schema, both built-ins and the document
decoder: Okabe–Ito `#AC77FA` / `#AC77FA1A`, Orcvs Light `#6407AF` /
`#6407AF1A`, both OKLCh hue 300°, borders transparent at 0.5. Okabe–Ito's one
unassigned hue, blue `#0072B2`, could not clear contrast on black without moving
the dark built-in's closest protanopia pair; `console/src/theme.md` records the
figures. The contrast matrix crosses every Source role and Cursor placement
with all four Portal role sets (143 states), and the colour-vision gate gains
an Input Portal channel (108 pairs); no recorded closest pair moves.

