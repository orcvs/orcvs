# 04 — Coalesce Cell backgrounds

**What to build:** Draw Cell backgrounds only where they differ from the Source fill, and coalesce
consecutive same-coloured Cells in a row into one rectangle.

**Blocked by:** 02 — Paint the Source Grid instead of building a Cell button field.

**Status:** resolved

- [x] Cell backgrounds are painted only where `cell_visuals` gives something other than
      `PALETTE.source`. The `CentralPanel` frame already fills the panel with `PALETTE.source`
      (`console.rs:537-538`) and clips every Scene shape to that same rect
      (`egui/src/containers/panel.rs:1227-1250`), so no pan or zoom can expose unfilled area and an
      ordinary Cell needs no rectangle.
- [x] The skip condition is exactly `cursor_visible || (!selected && bloom.is_none())`. The branch
      order in `cell_visuals` (`style.rs:83-92`) puts `cursor_visible` before the bloom arm, so the
      Cursor Cell paints `PALETTE.source` even though `cursor_bloom` answers `Some(Core)` for it.
      Skipping it is correct, and the blink alternates rectangle and no rectangle.
- [x] Consecutive Cells in a row sharing a background colour widen one rectangle rather than
      producing one each.
- [x] Every rectangle is snapped with `Rect::round_to_pixels`. This is mandatory, not a nicety: with
      snapping, one wide rectangle is pixel-identical to N adjacent ones at every
      `pixels_per_point` measured, for opaque and alpha fills alike; without it, coalescing changes
      the result by up to twenty-one per cent at a fractional `pixels_per_point`, and it is the split
      version that carries the dark seam.
- [x] Run endpoints are computed with the same column-to-x function the per-Cell path uses, so a
      coalesced edge is bit-identical rather than an accumulated sum.
- [x] Coalesced backgrounds stay in the background shape sequence issue 02 establishes, entirely
      beneath the Glyphs. A widened run reaching into a later Cell must not paint over that Cell's
      Glyph.
- [x] Visual parity holds against every case in `cell_visuals`, checked at a fractional
      `pixels_per_point` as well as at 1.0.

## Comments

**Grid lines are deliberately not in this issue, and the reason is a contradiction found while
planning it.** The earlier draft required spanning lines drawn once per Grid line instead of a
stroke per Cell, and separately required that a bloom border not composite over a Grid line. Both
cannot hold with this effort's strict-parity rule, because **the current renderer already
double-composites every interior Grid line**. `egui::Button` draws its border through
`Frame::paint` as a `RectShape` with `StrokeKind::Inside` (`egui/src/containers/frame.rs:435-441`),
and at `GRID_LINE_WIDTH = 0.5` that stroke takes epaint's thin-line branch — `width <= 0.9 *
feathering` (`tessellator.rs:1004`) — which paints a ridge two feather-widths wide centred on the
Cell boundary. Both neighbours' ridges cover the same pixels. Measured interior edges carry 1.68x
the ink of the Grid's outer edge at `pixels_per_point` 1.0 and 1.86x at 1.5; at 2.0 the stroke takes
the thick branch instead and the seam is double width at single intensity. The same overlap already
blends a neighbour's `grid_line` into a bloom Cell's own border.

So "correct alpha compositing" is a **styling change**, not parity, and this effort's spec sends
styling changes elsewhere. Worse, no spanning line reproduces today's line: `Painter::line_segment`
snaps the stroke centre to a pixel centre (`tessellator.rs:1656-1699`) while an Inside rect stroke
snaps the rect outside to a pixel boundary, a different sub-pixel phase. Only width 1.0 at exactly
`pixels_per_point` 2.0 matches, and that width is wrong everywhere else.

Per-Cell stroked borders therefore stay. Issue 02 draws them; this issue leaves them alone.

A uniform, singly-composited Grid line is a real improvement and worth having — Orca has no per-Cell
borders at all, and the glossary already has **Marker** for reading distance by eye. It belongs to a
styling effort that can change how the console looks on purpose, alongside the `restyle-egui-console`
work, not to a change about how it draws.

The background win is smaller than the earlier draft implied and still worth taking. Bloom is
Chebyshev over `DEFAULT_HIGHLIGHT_DOT_SPACING = 7` (`opts.rs:6`, `render_frame.rs:119-131`), so the
neighbourhood reaches 15x15 — up to 225 of the default 1000 Cells, less the `signal_breakup`
dropouts. Around three quarters of the Grid needs no background rectangle, not all but a handful.

Do not reach for a hand-built `epaint::Mesh`. No egui terminal renderer surveyed does; they coalesce
runs and stay on the ordinary painter API. Coalescing removes most of the rectangles outright, and
the per-rectangle constant only matters once the count stops falling.

The vertex-count argument for coalescing is the smaller half. `hxy-view` measured the larger one:
every shape becomes its own `epaint::Mesh` during tessellation, and those `Mesh::reserve_vertices`
and `reserve_triangles` calls were its top cumulative allocators. Fewer shapes beats fewer vertices
per shape.

That measurement carries a warning for this effort's verification strategy. `hxy-view` records that
the win did not appear in its `egui_kittest` harness at all, because that harness never drives the
tessellator — it only surfaced under the real rendering pipeline. This effort asserts against the
Render Frame rather than pixels, by the spec's own rule, and therefore has the same blind spot. That
is a reason to claim structure rather than speed, which the spec already does, not a reason to add a
harness.

---

Resolved. A Cell is filled only where `cell_visuals` asks for something other than `PALETTE.source`,
and consecutive Cells in a row that want the same fill share one `RectShape`. On the 8x8 Grid the
painting tests use, 53 filled Cells become 25 rectangles; on the default Grid most Cells ask for no
rectangle at all.

Five things a later reader needs.

**The per-Cell border had to leave the background shape.** Issue 02 carried the fill and the border
on one `RectShape`. A run widened across several Cells covers the borders of every Cell inside it,
so the fill can no longer be the shape that also strokes. The backgrounds are now fills carrying
`Stroke::NONE`, the borders are stroke-only `RectShape`s, and the paint order is fills, borders,
Glyphs, seams, Cursor. `every_background_is_painted_before_every_glyph_and_the_cursor_after_both`
gained the border half of that ordering and a check that a run wider than one Cell actually exists,
so it is asserting about coalesced shapes rather than about per-Cell ones.

**Two sub-pixel differences follow from that, and neither is a palette change.** Before, an ordinary
Cell painted an opaque `PALETTE.source` fill *after* its left neighbour's border and over the
feathered edge of its left neighbour's fill; now it paints nothing. So a Grid line loses the sliver
of ink its right-hand neighbour used to paint back over it, and a bloom Cell's outer edge blends
into the panel fill rather than being half re-covered by a neighbour's own Source fill — about a
quarter of the difference between `bloom_core_fill` and `source` on one pixel column, under a Grid
line that already double-composites there (see the note above). Parity here is asserted against the
Render Frame, by this effort's spec, and these are below what it can see.

**Issue 03's flooring does not make `round_to_pixels` redundant, and the mutation check proves it.**
`presented_grid` floors the Cell side to whole physical pixels, so a Cell corner is on a pixel
boundary up to the error of multiplying a column index by a Cell side no binary float holds exactly.
At `pixels_per_point` 1.5 on the test Grid that error is real: the Cell side is 24.666666, column
one's far edge computes as 50.666664 and snaps to 50.666668, and column seven's near edge computes
as 173.99998 where column six's far edge is 174.0. Removing the snap from `background_run` fails
`consecutive_cells_sharing_a_background_are_one_rectangle` at that scale — and passes at 1.0, which
is why the test runs at both. epaint rounds rects again at tessellation
(`round_rects_to_pixels`, `tessellator.rs:1778`, `:1830-1861`), but nothing in a Render Frame can
see it do so, and its thin-rect branch returns before the rounding, so the snap belongs in the
Shape.

**Requirement four is implemented but is not independently observable.** Computing the run's far
edge by accumulation (`covered.max + cell_size`) instead of from `GridViewport::cell_rect` fails no
test, because the snap absorbs a drift that would need on the order of a hundred thousand Cells to
reach half a pixel. The endpoints come from `cell_rect` as asked; the claim that this is what makes
a coalesced edge bit-identical only holds in the absence of the snap.

**The skip condition is pinned as a pure function, not restated in the painter.** The console
compares `visuals.background != PALETTE.source` so it cannot drift from `cell_visuals`, and
`the_cell_needing_no_background_is_exactly_the_one_filled_with_the_source` asserts over every
combination of Glyph, bloom, selection and caret phase that this is exactly
`cursor_visible || (!selected && bloom.is_none())`. Swapping the `cursor_visible` and bloom arms in
`cell_visuals` fails it.

One test had to widen its Grid. `a_painted_cell_takes_exactly_the_visuals_its_render_cell_asks_for`
ran on 8x8, where the bloom reaches every Cell, so its new "this Cell is filled by not being
filled" half held vacuously — painting every Cell's background left it passing. It runs on 20x20
now, wider than the fifteen Cells the bloom reaches, and counts the unfilled Cells so it cannot
become vacuous again. It also runs at `pixels_per_point` 1.5 as well as 1.0.

Grid lines are untouched, as the note above requires, and issue 05 is untouched.
