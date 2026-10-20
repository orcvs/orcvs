# 04: Draw 12 by 18 Cells with a matching Glyph

**What to build:** The visible change. The default Source Cell is 12 points wide and 18 tall, and the Glyph is three quarters of the Cell's height (13.5 points). The default window keeps today's size in points and shows more columns. Diagnostics shows the Cell as width by height. Clicks near Sector Seams and the Grid edge land on the Cell drawn there. See `../spec.md`.

**Blocked by:** 03 — Give the Cell a width and a height.

**Status:** ready-for-agent

- [ ] The default Cell is 12 by 18 points, and is exactly that at 1×, 1.5× and 2×.
- [ ] The Glyph's size is derived from the Cell height in one place, and the console lays out the Source's Glyphs at it.
- [ ] The default window is the same size in points as before.
- [ ] The Diagnostics readout reads `12.0×18.0 pt/cell` at the default.
- [ ] The real console has been checked against a screenshot from before the change, with clicks near Sector Seams and the Grid edge.
