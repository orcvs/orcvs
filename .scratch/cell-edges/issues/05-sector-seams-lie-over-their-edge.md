# 05 — Sector Seams lie over their edge

**What to build:** A Sector Seam is composited over the line that won its edge and drawn as part of that edge's piece, rather than as a separate line segment on each Cell's left and top. It is absent from the selection outline's edges. Seams still fall on every Sector boundary at the configured interval, with rectangular Cells as with square ones. See `../spec.md` and ADR 0078.

**Blocked by:** 03 — Draw every border edge once.

**Status:** ready-for-agent

- [ ] Built test-first at the Paint: an edge on a Sector boundary carries the seam composited over its winning line; a selection-outline edge carries none.
- [ ] Seam pieces follow the pixel rule and the junction rule; no seam piece overlaps a line piece.
- [ ] The seam keeps `sector.seam.width`; where it is wider than the line it lies over, the edge takes the seam's width.
- [ ] The separate seam Shapes are gone, and tests that pinned them are rewritten.
- [ ] The console gate and the `--no-default-features` arm pass.
