# Source Cells are configurable rectangles

Status: proposed. Amends the square-Cell sentences of [ADR 0038](0038-the-console-owns-the-source-grid-transform.md), [ADR 0053](0053-one-theme-styles-the-whole-console.md) and [ADR 0058](0058-zoom-is-eguis-whole-ui-zoom.md), the default window's Cell count in [ADR 0047](0047-the-source-view-has-a-margin.md), and the Cursor Effect's motion settings in ADR 0053. Builds on [ADR 0078](0078-each-cell-edge-is-drawn-once.md), which already draws every Cell edge from each axis's own side and makes the selection outline the highest-ranked edge. Settled in `.scratch/source-cell-size/`.

Every Source Cell is a 16 point square holding an 11.5 point monospace Glyph, so a row reads as spaced-out letters rather than a line of text. Orca draws 2:3 tiles, taller than wide, and its rows read like a terminal. The square is a constant in the console and a sentence in three ADRs, so no size can even be tried. The Cursor Effect's animated glitch around the Cursor distracts from reading the Grid.

## Decision

**A Source Cell is a rectangle, 12 points wide and 18 tall by default.** That is Orca's 2:3 shape at about today's visual weight. Its width and height are set under `[cell]` in `~/.orcvs/config.toml` as `width` and `height`, in points, each optional and bounded to 8 to 64. An absent key is its default; an out-of-range or mistyped value is reported in the settings notice and falls back on its own, leaving the other key standing. The web reads no settings file and draws the default.

**Each axis snaps to whole physical pixels on its own.** A Cell's rectangle is its column times the snapped width and its row times the snapped height, from the presented Grid's corner, and the Cell a point falls in is found by dividing by each axis's own side. A size that does not land on whole pixels snaps quietly, at least one pixel per axis. The edges ADR 0078 places, the selection outline and the Sector Seams follow each axis's snapped side without change.

**The Glyph is derived from the snapped Cell.** Its size is the smaller of three quarters of the Cell's height and the Cell's width divided by 0.62, the advance of one Monaspace Neon Glyph per point of size: 13.5 points at 12 by 18, set by the height, while a narrow Cell shrinks the Glyph rather than overlapping its neighbours. It is computed from the Cell the Grid is drawn at, once per zoom factor, so the Grid and its Glyphs agree at every zoom and egui still rasterises one Glyph size per zoom factor.

**The default window keeps its size in points.** It is 1024 by 640 points of Source area plus the top bar and the compact Panel, no longer a Cell count, so 12 by 18 Cells show about 85 columns and 35 rows rather than 64 and 40.

**The Cursor Effect is removed.** The Cursor and a Region show ADR 0078's still outline in `cursor.border` and `region.border` at their widths, and nothing animates. The effect's `[cursor_effects]` settings, the `cursor.area` Theme key and the reduced-motion preference it alone read go with it. A settings file with the old Glitch keys loads with the settings reader's unknown-key notice, and a custom Theme that still sets `cursor.area` is refused with the existing notice. There is no compatibility shim, because the console is pre-release.

## Consequences

- Every per-Cell cost scales with the Cells a window shows: about 16% more at the default, and about 4× at the 8 point bound. `paint_derive` and `paint_shapes` gain cases at both, each with a floor.
- Sectors stay the Sector Seam spacing in Cells on each axis, so a Sector is a block of Cells, no longer a square.
- Theme widths stay in display points whatever the Cell size, as ADR 0053 states.

## Considered options

**Orca's own 10 by 15.** At 1.5× its height is 22.5 pixels, which snaps to 22 and breaks the 2:3 shape on one of the two common native scales. 12 by 18 keeps the shape and is whole at 1×, 1.5× and 2×.

**One constant rather than a setting.** It would leave the size untried for every performer whose display or eyes want another; a bounded setting costs one settings section.

**Scaling the Glyph by the Cell's height alone.** A narrow configured width would then overlap neighbouring Glyphs; the width term stops that.
