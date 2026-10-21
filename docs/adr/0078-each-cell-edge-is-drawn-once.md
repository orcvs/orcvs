# Each Cell edge is drawn once

Status: proposed. Narrows [ADR 0040](0040-the-console-paints-from-a-value.md): a `Paint` still carries no geometry, and now answers its borders per Cell edge as well as per Cell. Its refusal of a single Mesh for backgrounds stands; this record takes a Mesh for edges only, for reasons that do not apply to fills. It also settles who draws the Cursor's border for `.scratch/source-cell-size/`, whose decision is [ADR 0079](0079-source-cells-are-configurable-rectangles.md). Settled in `.scratch/cell-edges/`.

Every Cell of the Source Grid strokes its own border inside its own rectangle, and the Cells tile with no gap, so every edge two Cells share carries two strokes side by side. A plain Grid line shows at twice the Theme's width: 0.5 points is 2 physical pixels on a 2× display, not 1. A bordered Cell's border sits beside its neighbour's Grid line, and two bordered neighbours put two borders side by side, so two Portal Cells at 1 point show a 4 pixel bar between them. A translucent role border drawn by both neighbours composites twice.

The console's own Inside stroke is upstream's choice for tiling: epaint rounds an Inside rectangle's outer edge to pixels "so that a user can tile rectangles with `StrokeKind::Inside`, and get no pixel overlap" (`epaint` 0.36.2 `tessellator.rs`). No overlap is not one line. The egui projects surveyed — `egui_extras`' Table, `egui_table`, `egui_plot`, `egui_tiles`, egui-spreadsheet, egui-snarl — never stroke cells; they draw each separator once as a full-length line, and paint highlights afterwards, with painter order as the only priority. That pattern alone still composites a translucent edge twice where two highlighted neighbours share it, and every crossing twice, which a prototype measured: with a 38% white role border, the lines inside a three-letter word read 169 against 117 for its outline, and every Grid crossing shows a brighter dot.

## Decision

**An edge of the Source Grid is drawn once, by the Cells either side of it.** A Cell edge is the boundary between two Positions, or between a Position and the outside of the Grid. Each edge shows exactly one line: the line of the highest-ranked fact either Cell carries on that side.

**The ranking, highest first:**

1. The selection outline: the Cursor's Cell when no Region spans more than one Cell, or the perimeter of a Region that does.
2. The Cursor's own Cell inside a spanning Region.
3. Output Portal, Input Portal and Locked, ordered among themselves as [ADR 0077](0077-a-committed-tick-records-its-portal-accesses.md) orders them.
4. Diagnostic.
5. The role border of a Cell its role claims.
6. The Grid line.

Equal facts on both sides draw one line. Unequal facts of the same rank draw the one the existing composition already prefers.

**A width of zero withdraws a fact from an edge rather than hiding the edge.** A Theme that sets a border's width to zero says that border is not drawn, so the edge goes to the next fact down, ending at the Grid line. Hiding a Cell's own Grid line is no longer expressible, because no Cell owns a Grid line.

**An edge's colour is composited once.** The winning border is composited over the facts below it on that edge, as `ordinary_border` composites today, and the result is painted once over the Cell backgrounds. Nothing is painted under it, so no colour is counted twice.

**A Sector Seam composites over its edge's line, except where the selection outline owns the edge.** It is not a competitor in the ranking: the seam's colour lies over whatever line won, as it lies over the border today, and is absent from the selection outline's edges, as it is absent from the Cursor's Cell today.

**The `Paint` decides; the shapes step places.** Which fact wins an edge, its colour and its width in points are a value, testable without a `Context`, derived beside `Paint::background_runs` and coalesced the same way into runs along each row line and column line. Where a line sits in physical pixels is the shapes step's, as where a Cell sits is `GridViewport`'s.

**Lines are placed on whole physical pixels, and junctions are their own pieces.** A line `w` physical pixels wide on the boundary pixel `b` covers `b - ⌊w/2⌋` to `b - ⌊w/2⌋ + w`: a 1 pixel line falls in the pixel right of or below its boundary, a 2 pixel line straddles it. A line thinner than one pixel covers one pixel at its alpha scaled by its width, as epaint's own thin-line path fades rather than thins. Where lines meet, the junction is as wide as the widest vertical line there and as tall as the widest horizontal one, and every line runs between junctions. Inside it the highest-ranked lines meeting there keep their own widths: each reaches from its own side through the crossing of their widest bands, or through the whole junction along an axis none of them crosses, in the colour of the highest-ranked of them. The other lines reach into the junction the same way, and the highest-ranked of them fills only what they reach and the top lines do not, so a corner the top lines turn away from stays empty. A 1 pixel outline turns its corner at 1 pixel through 2 pixel Grid lines rather than in a 2 by 2 block of its colour, and no Grid pixel lands inside the outlined Cell at any of its corners. No two pieces overlap. From the prototype:

```text
band(boundary, w) = boundary - w/2 .. boundary - w/2 + w   (integer physical pixels)
width_px(w_pt)    = w_pt * ppp < 1 ? (1, alpha * w_pt * ppp) : (round(w_pt * ppp), alpha)
junction          = width: max(vertical width_px), height: max(horizontal width_px)
arm(line)         = the line's own band, from its side to the far edge of the
                    top lines' crossing
top lines         = the lines of the highest rank meeting it, over their arms
rest              = the highest-ranked of the other lines, over the other arms
                    the top lines' arms leave
```

**The pieces are one `Shape::Mesh`, built straight from quads.** epaint turns a filled rectangle no more than two feathering widths across into a line segment and rounds that segment's ends inward by a quarter pixel, so a 1 by 1 pixel junction is rounded to nothing — the prototype showed a hole at every crossing — and a line segment drawn per edge leaves its own notches at corners. ADR 0040 refused a Mesh for backgrounds because it moves snapping into this crate and gives up per-rectangle anti-aliasing. Neither cost applies here: every piece is an axis-aligned, whole-pixel quad, which has no edge to anti-alias, and the snapping is this decision's content.

**A Cell has a width and a height.** Edges are placed from the presented Grid's corner by each axis's own snapped side, so the step takes the rectangular, per-axis-snapped Cells of `.scratch/source-cell-size/` without change. Nothing here assumes a square.

## Consequences

- A Grid line shows at the Theme's width rather than twice it, so every Theme's Grid reads lighter. Theme defaults are revisited against the new look.
- The Cursor and a spanning Region show one outline of the Theme's colour and width, centred on its edges and the same on all four sides; no Grid line or neighbour's border shows beside it.
- `ordinary_border` stops returning the Grid line for a Cell, and a Cell's `border_width` of zero no longer hides anything.
- The theme workbench copies the console's drawing and moves to the same edges.
- The shapes step gains a benchmark before the change, because `paint_derive` measures the derivation only.

## Rejected alternatives

**Keep per-Cell Inside strokes and halve every width.** The line still sits on one side of its boundary, a bordered Cell's border still sits beside its neighbour's, and an odd pixel width cannot be halved.

**A lattice of full-length lines with every bordered Cell's outline painted over it.** It is the surveyed egui pattern and the smallest change, and for the built-in Themes' opaque borders it is indistinguishable. It composites every translucent edge two bordered neighbours share twice, and every crossing twice, and painter order cannot stop a wider low-ranked border showing past a narrower high-ranked one.

**Lines as `line_segment`s, one per edge or per run.** epaint's segment snapping shrinks each end by a quarter pixel before rounding, so segments either leave a notch at a corner or, extended to cover it, overlap by a pixel and composite twice.

**The selection outline as a separate stroke inside the Cell.** Against lines that fall right of and below their boundaries, an Inside outline covers the Grid line on its left and top and leaves it showing outside its right and bottom.
