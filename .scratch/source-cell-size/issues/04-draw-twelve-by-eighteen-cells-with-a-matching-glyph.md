# 04: Draw 12 by 18 Cells with a matching Glyph

**What to build:** The visible change. The default Source Cell is 12 points wide and 18 tall, and the Glyph is three quarters of the Cell's height (13.5 points). The default window is fixed at today's size in points (1024 by 640 points of Source area plus chrome) instead of being derived from a Cell count, so it shows about 85 columns and 35 rows. Diagnostics shows the Cell as width by height. Clicks near Sector Seams and the Grid edge land on the Cell drawn there. See `../spec.md`.

**Blocked by:** 03 — Give the Cell a width and a height.

**Status:** resolved

- [x] The default Cell is 12 by 18 points, and is exactly that at 1×, 1.5× and 2×.
- [x] The Glyph's size is derived in one place from the snapped Cell (the smaller of 0.75 × height and width ÷ 0.6), and the console lays out the Source's Glyphs at it.
- [x] The default window is the same size in points as before, and a test pins it without referring to a Cell count.
- [x] The Diagnostics readout reads `12.0×18.0 pt/cell` at the default, and the kittests that assert the old `16.0 pt/cell` text are updated.
- [x] `paint_derive` and `paint_shapes` each gain a case at the 12 by 18 Cell over the default window, which shows about 16% more Cells than today, each with a floor. The Glyph size is computed once per Zoom level, not per frame, so the text layout cache keeps hitting.
- [x] The real console has been checked against a screenshot from before the change, with clicks near Sector Seams and the Grid edge.

## Comments

- `GridViewport::glyph_size` derives the Glyph from the snapped Cell; `shapes` lays the Source out at it. The value is a pure function of the Cell, so it is the same font size every frame at a given Zoom and the galley cache keeps hitting.
- `DEFAULT_SOURCE_AREA` fixes the window at 1024 by 640 points of Source area; `the_default_window_is_a_fixed_size_in_points` pins it with no Cell count.
- Benchmarks: `paint_derive/window/12x18` measured about 55 µs and `paint_shapes/window/12x18` about 499 µs locally, against 42 µs and 422 µs for 16 by 16 over the same window. Floors are 120 µs and 1 ms.
- Checked the release capture at 2× against one from ticket 03: the Glyphs fill the narrower Cells, the Grid lines and seams sit on the Cell boundaries, and Diagnostics reads `12.0×18.0 pt/cell`. Clicks are covered by the unequal-axis sweep in `grid_viewport`, which resolves every Cell's corner and centre to that Cell at 200 Cell sizes.
