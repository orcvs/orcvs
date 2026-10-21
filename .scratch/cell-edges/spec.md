# Each Cell edge is drawn once

Status: ready-for-agent

## Problem Statement

A performer looking at the Source Grid sees doubled lines. Every Cell draws its own border inside its own square, and the squares touch, so each edge two Cells share is drawn twice side by side. A plain Grid line shows at twice the width the Theme asks for. A word's role border sits beside the next Cell's Grid line. Two Portal Cells show a thick bar between them. The Cursor's outline sits beside its neighbours' lines rather than replacing them. A Theme author cannot get the line weight they set, and a translucent border shows darker inside a word than around it.

## Solution

Each edge between two Cells is drawn exactly once, as one line, by whichever Cell has the more important thing to say about it: the Cursor or a Region's outline first, then Portals and Locked Cells, then Diagnostics, then role borders, and otherwise the plain Grid line. Lines land on whole screen pixels, and where lines cross, the crossing is drawn once too. A Theme's widths and colours show as written. The Cursor's outline is one clean box, the same on all four sides. It works for the rectangular, independently snapped Cells `.scratch/source-cell-size/` introduces as well as today's squares. See [ADR 0078](../../docs/adr/0078-each-cell-edge-is-drawn-once.md).

## User Stories

1. As a performer, I want each Grid line drawn once, so that the Grid looks as fine as the Theme's width says.
2. As a performer, I want the Grid line between two empty Cells to be one line, so that an empty Grid is not drawn in double strokes.
3. As a performer, I want the role border between two letters of a word to be one line, so that a word reads as one framed span.
4. As a performer, I want the lines inside a word with a translucent border to be as strong as its outline, so that the frame looks even.
5. As a performer, I want the edge between a framed Cell and an unframed one to show the frame and not also the Grid line beside it, so that a border replaces the Grid line rather than doubling it.
6. As a performer, I want two neighbouring Portal Cells to share one line, so that a Portal reads as one span rather than a row of boxes.
7. As a performer, I want a Portal, Locked or Diagnostic edge to win over a role border it meets, so that the more important fact shows.
8. As a performer, I want the Cursor drawn as one box in the Theme's Cursor colour and width, so that I can find it at a glance.
9. As a performer, I want the Cursor's box the same on all four sides, so that no Grid line shows beside it on one side only.
10. As a performer, I want the Cursor's box to replace the lines it sits on, so that its edges are not drawn beside a neighbour's.
11. As a performer, I want a Region larger than one Cell drawn as one outline around the whole Region, so that I can see what I've selected.
12. As a performer, I want the Cursor's own Cell inside a Region to keep the border the Theme gives it, so that I can still see where the Cursor is.
13. As a performer, I want the lines where Grid lines cross drawn once, so that a translucent Grid shows no dots at its crossings.
14. As a performer, I want Grid lines on whole screen pixels at 1×, 2× and fractional zoom, so that the Grid stays crisp and evenly spaced.
15. As a performer on a 1× display, I want a Grid line narrower than one pixel to fade rather than vanish or blur across two pixels, so that a fine Theme still shows a Grid.
16. As a performer, I want Sector Seams to lie over the edge they mark without doubling it, so that the registration marks read as one line.
17. As a performer, I want no Sector Seam on the Cursor's box, so that the Cursor stays distinct.
18. As a performer, I want Cell backgrounds unchanged, so that fills, widened runs and the Cursor's Cell still look as they do.
19. As a performer, I want Glyphs drawn over the lines as before, so that text is never cut by a border.
20. As a performer, I want the lines to land on the same edges whether Cells are 16 by 16 or 12 by 18, so that rectangular Cells keep crisp Grid lines on both axes.
21. As a performer, I want clicking a Cell to select that Cell exactly as before, so that the change is visual only.
22. As a performer, I want Pan, Zoom and resizing to keep the lines on Cell edges, so that the Grid never tears.
23. As a Theme author, I want `grid.border.width` to be the width a Grid line is drawn at, so that what I write is what I see.
24. As a Theme author, I want a border width of zero to mean that border is not drawn and the edge falls to the next fact, so that zero never leaves a gap in the Grid.
25. As a Theme author, I want a translucent border colour composited once, so that its opacity means what I set.
26. As a Theme author, I want the built-in Themes' widths revisited against single lines, so that the defaults still look deliberate.
27. As a Theme author using the workbench, I want the workbench to draw edges the way the console does, so that what I tune there is what the console shows.
28. As a maintainer, I want which fact wins an edge decided as a value beside the Cell's Paint, so that it can be tested without egui.
29. As a maintainer, I want where a line sits in pixels decided in one step, so that pixel placement is tested in one place.
30. As a maintainer, I want the edge pieces never to overlap, so that compositing is predictable.
31. As a maintainer, I want the cost of the shapes step measured before and after, so that a claim about it is backed by a benchmark.
32. As a maintainer, I want the change to need no change to `.scratch/source-cell-size/`'s Cell geometry, so that the two efforts land in either order.
33. As a maintainer, I want `.scratch/source-cell-size/`'s Cursor outline reworded to the edge ranking, so that the two specs agree.

## Implementation Decisions

- **Ranking.** The selection outline (the Cursor's Cell when no Region spans more than one Cell, or the spanning Region's perimeter) > the Cursor's own Cell inside a spanning Region > Output Portal, Input Portal, Locked (in ADR 0077's order) > Diagnostic > role border on a claimed Cell > Grid line. Equal facts on both sides draw one line.
- **Width zero withdraws a fact.** A border whose width is zero does not compete for the edge, and the edge goes to the next fact down. No Cell can hide its Grid line any more.
- **Composited once.** An edge's colour is the winning border composited over the facts below it, as the existing composition does, painted once over the backgrounds. A Sector Seam composites over the line that won, and is absent from the selection outline's edges.
- **The Paint decides, the shapes step places.** The Paint gains an edge answer beside its background runs: for each row line and column line the console draws, runs of edges with the same winning colour, width in points and rank. It carries no geometry, as ADR 0040 requires. The per-Cell border the Paint answers today stops being drawn and is retired once nothing reads it.
- **Pixel placement.** In the shapes step, a line `w` physical pixels wide on boundary pixel `b` covers `b - ⌊w/2⌋ .. b - ⌊w/2⌋ + w`. A line under one pixel covers one pixel at its alpha scaled by its width. A junction where lines meet is as wide as the widest vertical line there and as tall as the widest horizontal one; lines run between junctions. Inside it the highest-ranked lines keep their own widths, and the highest-ranked of the others fills only what the other lines reach. No two pieces overlap. (Rule from `console/examples/prototype_cell_borders.rs`, see ADR 0078.)
- **One Mesh of quads.** The pieces are one `Shape::Mesh`, built directly. epaint's rectangle path turns a pixel-wide rectangle into a line segment and rounds a one-pixel junction away; its segment path notches corners. Backgrounds keep their coalesced `RectShape`s, as ADR 0040 decided.
- **Cell size.** Edges are placed by each axis's own snapped side from the presented Grid's corner. Where `.scratch/source-cell-size/` ticket 03 has landed, the step reads the Cell's width and height from the viewport; where it has not, the step still takes a width and a height, both today's snapped side, and never a single square side.
- **Paint order.** Area, backgrounds, edges, Glyphs. The separate border, seam and Cursor groups are absorbed into the edges. A Cursor Effect still enabled under `.scratch/source-cell-size/`'s settings draws its own fragments after the Glyphs, outside this decision.
- **The Cursor outline.** The selection outline is the highest-ranked edge rather than a stroke inside the Cell. `.scratch/source-cell-size/`'s spec ("The Cursor outline" decision) and its ticket 02 are reworded to say so.
- **Glossary.** CONTEXT.md's Paint entry ("the per-Cell decision of … border") is updated to say a Paint answers each Cell edge's line, and a Cell edge entry is added if the term is used outside this effort.

## Testing Decisions

- A good test pins what a viewer could notice through the highest seam that answers it: which line an edge shows, where it lands in pixels, and which Shapes a Render Frame paints. Tests do not reach into private helpers.
- **The Paint is the primary seam.** Edge answers are tested as values, with no egui linked in, the way `Paint::background_runs` is: two equal neighbours share one edge; a higher fact wins against a lower one each way round; width zero falls through; runs coalesce and break where the winner changes; the selection outline outranks everything and suppresses a seam.
- **The shapes step is the pixel seam.** Shape-level tests, as in the console's existing `Shape` assertions, check that every edge piece is whole-pixel, that no two pieces overlap, that a one-pixel junction exists, and that a sub-pixel line keeps its fade, at 1×, 2× and a fractional scale, with Cells of equal and unequal width and height.
- **The console suites** keep passing; tests that pinned per-Cell Inside border rectangles (for example the "one Grid line wide" and "a Region hides the Cursor's Cell border" tests) are rewritten to the edge rule rather than deleted.
- **A benchmark** of the shapes step is added before the change, and its numbers are compared by CI, not locally.
- **Visual check.** Compare the running console against a screenshot from before the change at 1× and 2×, with a built-in Theme and with a translucent role border.

## Out of Scope

- Rectangular or configurable Cells, the derived Glyph size and the Cursor Effect's removal: `.scratch/source-cell-size/`.
- Choosing new colours for any Theme; only widths are revisited.
- Anti-aliased or rounded borders, dashed lines, and borders on anything but the Source Grid.
- Changing how backgrounds are coalesced or drawn.

## Further Notes

- The prototype is `console/examples/prototype_cell_borders.rs`, run with `cargo run -p console --example prototype_cell_borders`. It draws the egui lattice-and-overlay pattern beside each-edge-once in a user's Theme, with a pixel-exact magnifier and a translucent role border toggle. It is throwaway and lives on the local branch `prototype/cell-edges` (commit `0c84744f`), not `main`.
- Measured in the prototype at 2 px/pt with a `#FFFFFF60` role border: lattice and overlays drew the lines inside a word at 169 and its outline at 117, and crossings at 41–43 against 34 for the line; each-edge-once drew both at 117 and crossings at 34.
- With the built-in Themes' near-background role borders and opaque Portal borders, lattice and overlays and each-edge-once are indistinguishable; the difference is in translucent borders, crossings and the Cursor's sides.
