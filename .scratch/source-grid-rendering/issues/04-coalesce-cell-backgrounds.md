# 04 — Coalesce Cell backgrounds

**What to build:** Draw Cell backgrounds only where they differ from the Source fill, and coalesce
consecutive same-coloured Cells in a row into one rectangle.

**Blocked by:** 02 — Paint the Source Grid instead of building a Cell button field.

**Status:** ready-for-agent

- [ ] Cell backgrounds are painted only where `cell_visuals` gives something other than
      `PALETTE.source`. The `CentralPanel` frame already fills the panel with `PALETTE.source`
      (`console.rs:537-538`) and clips every Scene shape to that same rect
      (`egui/src/containers/panel.rs:1227-1250`), so no pan or zoom can expose unfilled area and an
      ordinary Cell needs no rectangle.
- [ ] The skip condition is exactly `cursor_visible || (!selected && bloom.is_none())`. The branch
      order in `cell_visuals` (`style.rs:83-92`) puts `cursor_visible` before the bloom arm, so the
      Cursor Cell paints `PALETTE.source` even though `cursor_bloom` answers `Some(Core)` for it.
      Skipping it is correct, and the blink alternates rectangle and no rectangle.
- [ ] Consecutive Cells in a row sharing a background colour widen one rectangle rather than
      producing one each.
- [ ] Every rectangle is snapped with `Rect::round_to_pixels`. This is mandatory, not a nicety: with
      snapping, one wide rectangle is pixel-identical to N adjacent ones at every
      `pixels_per_point` measured, for opaque and alpha fills alike; without it, coalescing changes
      the result by up to twenty-one per cent at a fractional `pixels_per_point`, and it is the split
      version that carries the dark seam.
- [ ] Run endpoints are computed with the same column-to-x function the per-Cell path uses, so a
      coalesced edge is bit-identical rather than an accumulated sum.
- [ ] Coalesced backgrounds stay in the background shape sequence issue 02 establishes, entirely
      beneath the Glyphs. A widened run reaching into a later Cell must not paint over that Cell's
      Glyph.
- [ ] Visual parity holds against every case in `cell_visuals`, checked at a fractional
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
