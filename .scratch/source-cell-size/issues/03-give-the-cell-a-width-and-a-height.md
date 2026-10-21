# 03: Give the Cell a width and a height

**What to build:** A prefactor with no visible change. The console's Cell becomes a width and a height everywhere it is read: the Cell geometry (`GridViewport`), the Source View's Pan bounds, margin, Cursor follow and edge scroll, the Cursor Effect and the Diagnostics source reserve. The shapes step reads each axis's side from the viewport; its edge Mesh already places each axis on its own. The Diagnostics readout text and the default window are left for 04. The Cell geometry is given the Cell size by its caller rather than reading a constant. The default stays 16 by 16 and the Glyph stays 11.5 points, so every existing behaviour test passes unchanged in what it asserts. See `../spec.md`.

**Blocked by:** 02 — Turn the Cursor Effect off by default.

**Status:** resolved

- [x] Built test-first at the Cell geometry. A point still resolves to the Cell drawn under it across a sweep where width and height step independently, so reading one axis with the other's side fails. Each axis snaps to a whole physical pixel on its own at every Zoom step. A Cell whose width or height is zero or not finite answers no click.
- [x] Pan margin, Cursor follow and edge scroll use the width across and the height down, which a console test with an unequal Cell proves.
- [x] The Cursor Effect uses the width along horizontal edges and the height along vertical ones.
- [x] No pixel of a 16 by 16 console changes. The existing console and kittest suites pass.

## Comments

- `GridViewport::cell_size` is a `Vec2`, width by height, and `CELL_SIZE` is `Vec2::splat(16.0)`. `presented_grid` takes the Cell from its caller. `snapped_cell` snaps each axis through `snapped_cell_side`, and `cell_index` takes the axis's own side.
- Test-first: the corner and centre sweep in `a_cell_corner_answers_its_own_cell_whatever_the_cell_measures` steps width and height apart, and failed while the row axis still read the width. `a_cell_degenerate_on_one_axis_answers_no_cell` covers a zero or non-finite side on either axis. `every_cell_is_a_whole_number_of_physical_pixels` now runs 16 by 16, 12 by 18 and 10 by 15 through egui's whole zoom range, per axis.
- The Source View carries the Cell it draws at (`SourceView::cell`, `CELL_SIZE` by default), which ticket 05's settings set. The Pan bounds, margin, Cursor follow, edge scroll and the Diagnostics source reserve read it per axis. `an_unequal_cell_is_measured_across_by_its_width_and_down_by_its_height` drives a 12 by 18 console; swapping either axis in the margin or the edge-scroll cap fails it.
- The Cursor Effect takes the Cell as a `Vec2`: Cell-lengths along horizontal edges are widths and along vertical ones heights. Its lasso test runs at 16 by 16 and 12 by 18.
- The Diagnostics readout still shows one side (the width) until ticket 04 shows width by height. The test-only fitted viewport fits Cells of `CELL_SIZE`'s shape.
