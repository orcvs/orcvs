# 01: Record rectangular, configurable Cells in an ADR

**What to build:** A new ADR recording that Source Cells are rectangles with a configurable width and height, Orca's 2:3 at 12 by 18 points by default. It also records that the Glyph's size is three quarters of the Cell's height, that each axis snaps to whole physical pixels on its own, that the Cursor Effect is off by default, and that the effect is to be removed later. The ADR names the sentences it amends in ADRs 0038, 0053 and 0058. CONTEXT.md's Cursor Effect, Sector and Theme entries are updated to match. See `../spec.md`.

**Blocked by:** None (can start immediately).

**Status:** ready-for-agent

- [ ] The ADR states the default Cell, the settings keys and their bounds, the derived Glyph, the per-axis snap, and why Orca's 10 by 15 is not the default (see the spec's Further Notes).
- [ ] ADRs 0038, 0053 and 0058 each carry an "amended by" note at the sentences the spec lists, including both of ADR 0058's 11.5 point sentences.
- [ ] CONTEXT.md no longer calls a Sector a square block, and its Cursor Effect entry says the effect is off by default.
