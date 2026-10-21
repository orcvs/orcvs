# 04: Draw 12 by 18 Cells with a matching Glyph

**What to build:** The visible change. The default Source Cell is 12 points wide and 18 tall, and the Glyph is three quarters of the Cell's height (13.5 points). The default window is fixed at today's size in points (1024 by 640 points of Source area plus chrome) instead of being derived from a Cell count, so it shows about 85 columns and 35 rows. Diagnostics shows the Cell as width by height. Clicks near Sector Seams and the Grid edge land on the Cell drawn there. See `../spec.md`.

**Blocked by:** 03 — Give the Cell a width and a height.

**Status:** ready-for-agent

- [ ] The default Cell is 12 by 18 points, and is exactly that at 1×, 1.5× and 2×.
- [ ] The Glyph's size is derived in one place from the snapped Cell (the smaller of 0.75 × height and width ÷ 0.6), and the console lays out the Source's Glyphs at it.
- [ ] The default window is the same size in points as before, and a test pins it without referring to a Cell count.
- [ ] The Diagnostics readout reads `12.0×18.0 pt/cell` at the default, and the kittests that assert the old `16.0 pt/cell` text are updated.
- [ ] The paint benchmarks run at the 12 by 18 Cell over the default window, which shows about 16% more Cells than today. The Glyph size is computed once per Zoom level, not per frame, so the text layout cache keeps hitting.
- [ ] The real console has been checked against a screenshot from before the change, with clicks near Sector Seams and the Grid edge.
