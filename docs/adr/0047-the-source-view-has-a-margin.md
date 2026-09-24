# The Source View has a margin

Status: accepted. Amends [ADR 0045](0045-the-source-view-is-a-bounded-space.md): the default Grid, the default window and how far a Pan reaches change; Zoom, the Pan gestures and the Cursor follow stand. [ADR 0054](0054-a-grid-is-always-256-by-256.md) amends it: every Grid is 256 by 256, so the 128 by 80 default below and the Consequence that a stored Source keeps its own Grid no longer hold; the margin, the default window and Pan reach stand.

**The default Grid is 128 by 80, twice the default window on each axis.** ADR 0045 sized the default Grid to fill the default window exactly, so a fresh console had nowhere to Pan and the Source View's own movement was invisible until a viewer Zoomed in or shrank the window. The default window keeps its 1024 by 640 point console, now 64 by 40 Cells of view onto a larger Grid, and opens on its top-left quarter.

**The Grid sits two Cells in from the console, and a Pan reaches two Cells past each edge.** Pinned flush to the console's edges, the Grid's edge was indistinguishable from the console's, and a Cursor on the first or last row or column sat against the chrome. The margin makes the Grid's edge visible as an edge. It is counted in Cells, so it scales with the Zoom, and it is taken from the snapped Cell side, so it is a whole number of physical pixels wherever the Cells are. On an axis where the Grid and its margins are smaller than the console, the Grid rests one margin in from the top-left and has nowhere to Pan. The margin is not Source: it holds no Positions, and a click on it selects nothing — ADR 0043's outside is still outside.

**The pointer shows a drag Pan.** A grab hand while Alt (Option) is held over the console, and a grabbing hand while an Alt-held primary drag or a middle-drag is Panning. A middle-drag has no hover state to announce before it starts, and a wheel Pan is not a grab, so neither shows the open hand.

## Rejected alternatives

**A margin in points.** A fixed number of points would shrink to a sliver at `MAX_ZOOM` and swamp the Grid at `MIN_ZOOM`; a margin in Cells reads the same at every Zoom.

**Centring a Grid smaller than the console.** ADR 0045 refused re-centring along with the fit; a small Grid resting at the top-left keeps the origin where a viewer expects it.

## Consequences

A stored Source keeps the Grid it was stored with, so a Source stored at 64 by 40 opens at 64 by 40, one margin in, with room to Pan only past the margin.
