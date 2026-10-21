# 05 — Sector Seams lie over their edge

**What to build:** A Sector Seam is composited over the line that won its edge and drawn as part of that edge's piece, rather than as a separate line segment on each Cell's left and top. It is absent from the selection outline's edges. Seams still fall on every Sector boundary at the configured interval, with rectangular Cells as with square ones. See `../spec.md` and ADR 0078.

**Blocked by:** 03 — Draw every border edge once.

**Status:** resolved

- [x] Built test-first at the Paint: an edge on a Sector boundary carries the seam composited over its winning line; a selection-outline edge carries none.
- [x] Seam pieces follow the pixel rule and the junction rule; no seam piece overlaps a line piece.
- [x] The seam keeps `sector.seam.width`; where it is wider than the line it lies over, the edge takes the seam's width.
- [x] The separate seam Shapes are gone, and tests that pinned them are rewritten.
- [x] The console gate and the `--no-default-features` arm pass.

## Comments

- A Cell still carries its left and top seam colours (`CellPaint::sector_left`/`sector_top`), now on every Cell, the Cursor's included. `Paint::edges` composites the seam onto the line that won each edge, `line.blend(seam)` at the wider of the two widths in points, keeping the line's rank; where no line won, the seam stands alone at Grid line rank. The outline's edges take none. With a zero outline width the Cursor's Cell's seams show, since its edges are ordinary again.
- The composite is made in points before the pixel rule, so below one pixel the merged line fades by the wider width once, rather than the seam fading over a faded line.
- Of lines of one rank meeting at a junction, a seamed one gives it its colour, and then the most opaque. `EdgeLine::seamed` carries the seam through to the junction: alpha alone ties wherever `grid.border` is opaque. Without that tie-break the plain Grid line took every crossing a seam passes and notched the seam there, which the 2× render showed; `a_sector_seam_is_drawn_on_the_cell_edge_the_paint_asks_for` asserts a seam's crossing with a plain row line, and `a_seam_runs_through_an_opaque_grid_line_it_crosses` the opaque case.
- Where the seam is wider than the line, its extra pixels carry the line under the seam too (one colour per piece), so at 2× a default seam's second pixel reads a little stronger than the seam over the page did before.
- `sector.seam.width` is read into the Paint, and `SourceShapes::geometry` and `SourceShapes::new` no longer take a Theme.
- Shape tests read seams as edge pixels; the lattice tests leave seam lines to `a_sector_seam_is_drawn_on_the_cell_edge_the_paint_asks_for`, which runs at every device scale in `DEVICE_SCALES`.
