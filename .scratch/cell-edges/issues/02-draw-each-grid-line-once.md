# 02 — Draw each Grid line once

**What to build:** An empty Grid shows each Grid line once, at `grid.border.width`, on whole physical pixels, with every crossing drawn once. The lines and junctions are non-overlapping pieces of one Mesh placed by ADR 0078's pixel rule. Cells that carry a role, Portal, Locked, Diagnostic or Cursor border keep today's per-Cell border for now, drawn over the lines. See `../spec.md`.

The step takes the Cell as a width and a height snapped per axis, never one square side. If `.scratch/source-cell-size/` ticket 03 has landed it reads both from the viewport; if not, both are today's snapped side.

**Blocked by:** 01 — Measure the shapes step.

**Status:** resolved

- [x] Built test-first at the shapes step: on an empty Grid every edge is one piece, pieces never overlap, every crossing has a junction piece, and every piece is whole-pixel, at 1×, 2× and a fractional scale, with equal and unequal Cell width and height.
- [x] A line under one physical pixel covers one pixel at its alpha scaled by its width.
- [x] A one-pixel junction is present in the Shapes (epaint's rectangle path would round it away; the Mesh is built from quads).
- [x] Unclaimed Cells draw no per-Cell border any more; the Paint's Grid line answer is the lattice's.
- [x] Tests that pinned a Grid line as a per-Cell rectangle are rewritten to the lattice, not deleted.
- [x] The running console at 1× and 2× is compared with a screenshot from before: plain Grid lines are half their previous visible width and crisp.
- [ ] The benchmark from 01 is run by CI on the pull request.
- [x] `cargo fmt --all -- --check`, `cargo clippy --package console --all-targets --locked -- -D warnings`, `cargo nextest run --package console --locked` and the `--no-default-features` arm pass; `mise run check_wasm` passes.

## Comments

**2026-10-20 — what shipped.**

- `SourceShapes` gains a `lines` group, painted after the backgrounds and before the borders. It holds one `Shape::Mesh` that `grid_lines` builds from the boundaries of the drawn range in physical pixels, read per axis from `GridViewport::cell_rect`. Each piece is a quad: one junction per crossing and one piece per edge between two junctions. `PixelLine` applies ADR 0078's band and width rules, fading a sub-pixel line with `gamma_multiply` as epaint's thin-stroke path does. A Grid line width of zero draws no Mesh.
- `ordinary_border` answers `Option<Stroke>`, and gives `None` to a Cell with no Diagnostic or Portal fact. `CellVisuals` and `CellPaint` carry `border: Option<Stroke>`. Ticket 03 replaces the remaining per-Cell strokes.
- `.scratch/source-cell-size/` ticket 03 has not landed, so the boundaries come from today's square `cell_size`. `grid_lines` takes each axis's boundaries separately, and `unequal_cell_sides_draw_each_grid_line_once` drives it with 12 by 18 pixel Cells.
- **Tests.** New ones: `an_empty_grid_draws_each_grid_line_once`, which counts every covered pixel at 1×, 1.5× and 2× and at one and two pixel widths, plus `unequal_cell_sides_…`, `a_grid_line_under_one_pixel_…`, `a_one_pixel_crossing_is_drawn` and `the_grid_lines_lie_over_the_backgrounds`. Nine tests that pinned per-Cell border rectangles now assert the lattice instead. Two mutations were each caught: dropping the junction quad, and placing bands without straddling the boundary.
- **Visual check.** The release capture was rendered locally on lavapipe at 1×, 1.5× and 2× (the capture's `PIXELS_PER_POINT` was varied locally and not committed), before and after this change. A pixel row through empty Cells measured every plain Grid line at 2 px wide before and 1 px after at all three scales, with the same colour and the same 16, 24 and 32 px spacing. The magnified crops are in the project files as `cell-edges/02-grid-lines-before-after.png`. Where a Sector Seam crosses the row it still shows beside the line as a separate segment; ticket 05 fixes that.
