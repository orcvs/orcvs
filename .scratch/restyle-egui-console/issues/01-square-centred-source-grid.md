# 01 — Square, centred Source Grid viewport

**What to build:** The console renders the largest square Source Grid viewport that fits the available console area. Cells remain square at every window size, and surplus rectangular space is centred as letterboxing without changing editing, Cursor, Playback, persistence, native, or WASM behaviour.

**Blocked by:** None — can start immediately.

**Status:** resolved

**Tags:** release/v1

- [x] Cell width and height remain equal for wide, tall, and square available areas.
- [x] The rendered Grid viewport remains square and centred in surplus space.
- [x] Resizing cannot stretch one Cell axis independently of the other.
- [x] Existing console interaction behaviour remains intact.

## Comments

### Resolved on `01-square-centred-source-grid`, 2026-09-04

**The geometry is a seam of its own.** `shell/src/grid_viewport.rs` answers, for
an available area and a Grid dimension, the square Cell size and the centred
viewport rectangle: `grid_viewport(available, columns, rows)`. One Cell size
serves both axes, so no console shape can stretch one axis past the other, and
the viewport is centred on the available area, so the surplus of the longer axis
falls away as letterboxing on both sides. The wide, tall, square, and degenerate
cases are settled there without a window. The type is `GridViewport` rather than
`SourceViewport` because egui already calls the OS window a viewport
(`ViewportCommand` is used in the same file).

**The console follows that geometry rather than cropping to the window.** The
console previously opened on `top_right_source_view`: a window-shaped slice of
the Source, pinned to the Source's top right corner and offset 15 points, so a
wide window showed a cropped band rather than the Grid. `show_source_scene` now
asks the geometry for the viewport and hands the Scene the Source region that
presents it — the available area measured in Source coordinates, centred on the
Source. Because that region is recomputed every frame while the viewer has not
panned or zoomed, a resize re-fits instead of cropping.

**Panning and zooming still work, and the reset is now sticky.** `SourceView`
records whether the viewer has moved the view: a pan or zoom pins it, and a
double click hands it back to the fitted viewport. The old double click already
produced the same square, centred, letterboxed picture — it set the view to the
Source's own bounds, which the Scene fits and centres exactly as this change
does — so the delta is not geometry but stickiness: that reset was a one-shot
fit that cropped again on the next resize, while clearing `adjusted` makes the
fit hold across every later resize.

**The Scene's zoom range has to admit the fit at both ends.** A console taller
than the Source's own 1600 points fits it at more than the 2.0 maximum, and
`fit_to_rect_in_scene` clamps the scale it computes, so the Grid would stop
filling the console. The floor clamps the same way in the other direction: a
console under 200 points on its shorter axis fits an 800 point Source below
0.25, and the clamp there spilled the Grid out of the console and put Cells
under the wrong pointer positions. So both ends give to the fit, and only to the
fit — a console with no area answers a scale of zero, which is no fit to reach,
and keeps the floor. The viewer's own zoom stays bounded by 0.25 and 2.0
wherever those bounds still contain the fit.

**Hit-testing goes through the same geometry.** Nothing in the console maps a
pointer position back to a Cell: each Cell is an egui button inside the Scene's
transformed layer, so a click is placed by exactly the transform that draws it.
The tests drive the real path in a headless egui context and assert through
clicks — an interior Cell, the first and last Cell of the Grid, and a click on
the letterboxing that changes nothing — at wide, tall, square, and above-maximum
scales, and they cover the pin/un-pin state machine directly: a resize re-fits
while the view is unpinned, a zoom pins it against the next resize, and a double
click hands it back.

**Consequence for `restyle-egui-console/03`: Grid text is slightly softer.** The
console used to open at zoom 1.0, drawing Glyphs at their rasterised size. It
now opens at the fitted scale, which is 1.0 only when the shorter console axis
is exactly 800 points. `epaint::TextShape::transform` scales the laid-out galley
mesh rather than re-laying it out, so at almost every window size Glyphs are
sampled out of the font atlas at a non-integer ratio and read a little soft;
egui records the same effect for `Scene` (emilk/egui#4813). This is inherent to
fitting a fixed-size Source into an arbitrary console — only rebuilding the Grid
without a `Scene`, laying Cells out at the fitted size and choosing a font size
per frame, would avoid it. It is the change a viewer is most likely to notice,
so `03` should judge it against the prototype rather than be surprised by it.

**Vocabulary gap for `/domain-modeling`.** `CONTEXT.md` is language-only: it has
no Console or presentation section, and neither "viewport" nor "Cell size" has
an entry, so this change names its geometry in code without a glossary term
behind it. Per `docs/agents/domain.md` the term is not invented here; the gap is
recorded for a domain-modelling pass.

**No palette, Cursor, Playback, or persistence behaviour changed.** The only
other visible change is a Cell size row in the Diagnostics window, which reports
the answered square Cell size.

### The default shape, 2026-09-04

**Square Cells settle the geometry; they do not settle the shape.** A square
Grid in a wide console is all fit and no fill: the default was 32 by 32, an 800
point square Source, opened in an 800 by 600 window, so the console spent a
quarter of its width on letterboxing before the viewer touched anything. The fit
was doing what it was asked; it was being asked with two shapes that disagreed.

**One ratio, stated once, and derived from there.** The default Grid is now 40
by 25. Cells are square, so those counts are the ratio: 8 by 5, which reads left
to right in time and is the proportion a console is most often given. The
default window follows from the Grid rather than standing beside it —
`DEFAULT_VIEW_SIZE` is the Source's own points plus the height the top panel
takes — so the two cannot drift apart, and changing the Grid moves the window
with it. The Cell count is 1000, against 1024 before, so the workspace is the
same size in a more usable shape.

**The console opens at a scale of exactly one.** Measured from the running app,
the default window gives the console 1000 by 625 points, the viewport is that
same rectangle, and the Cell size is 25: the Grid fills the console with no
letterboxing and Glyphs are drawn at the size they are rasterised at. That is
the resampling softness recorded above, gone from the case a viewer sees first.

**What this does not do.** The window is 1000 by 657, because it carries the top
panel: the console is 8 by 5, the window is not, and no Grid size makes both so
while the window carries chrome. The fit is exact at that one size and
letterboxes at every other, so this makes the default exact rather than making
letterboxing impossible. A Grid whose Cell counts follow the console would fill
any window at any shape and retire the question, at the cost of dynamic Grid
dimensions across the Source, persistence and Cursor bounds. That is a different
design and is not this one.

**Two tests hold the two halves together.** One asserts the default console
presents the default Grid at `CELL_SIZE` with the viewport equal to the console
area; the other asserts the top panel takes exactly the height the default
window holds back, so neither constant can move without the other.
