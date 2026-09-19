# The Source View is a bounded space

Status: accepted. Supersedes the fit, the pointer Zoom and the letterboxing that [ADR 0038](0038-the-console-owns-the-source-grid-transform.md) presented the Source under. [ADR 0047](0047-the-source-view-has-a-margin.md) amends the default Grid, the default window and the Pan bounds below. The rest of ADR 0038 stands: the console owns the transform, applies it when it computes each Cell's rectangle, and lays each Glyph out at the size it is drawn at. It does not reopen [ADR 0005](0005-defer-source-addressing-for-infinite-canvas.md).

The console presents the Source as a space the viewer moves through, not a picture fitted to the window. The Source View opens at Zoom 1.0 — the Source's own Cell, now 16 points with an 11.5 point Glyph — at which the default 64 by 40 Grid fills the default window exactly. A resize changes how much of the Source shows and never the Cell size.

**The Source's own Cell is 16 points so that every Zoom step is a whole number of points.** At the previous 25 points an eighth is 3.125 points, so every step but 1.0 and 2.0 was floored to a whole physical pixel and the Grid came up short of the rectangle its Zoom asked for. At 16 points an eighth is two, and every Cell is a whole number of physical pixels at 1×, 1.5× and 2× with nothing to snap.

**Pan is bounded by the Grid.** [ADR 0043](0043-the-grid-is-bounded-by-definition.md) settled that the Grid has an outside and that the outside is not a Position; the Source View now agrees with it. On an axis where the Source is larger than the console, a Pan reaches the Source's edge and stops. On an axis where it is smaller, the Source sits at the console's top-left and has nowhere to Pan. A resize or a Zoom that would open a gap past an edge settles back inside. Pan is by wheel or two-finger scroll, by middle-drag, and by Alt (Option) held with a primary drag — a spatial language is moved through directly, and a primary click alone still selects a Cell. Space was the first choice for the drag modifier and was refused because Space toggles Playback: holding it would start or stop the run before the drag began. The primary drag without a modifier belongs to Region selection — see [ADR 0046](0046-the-primary-drag-selects-a-region.md).

**Zoom is a change of Cell size in steps, from the keyboard alone.** Command `=` and command `-` step the Cell size by an eighth of the Source's own Cell — two points — from a quarter to double, which is 4 to 32 points; command `0` returns to 1.0. Those are the steps `GLYPH_SCALE_STEP` already quantises a Glyph to, so the atlas budget ADR 0038 and ADR 0040 state is now the whole range rather than its floor: fifteen sizes, with no window large enough to widen it. Bare `+` and `-` are Source characters and are never taken for a Zoom. The chords are the ones egui gives its whole-UI zoom by default; the Source View takes them, and whole-UI zoom moves to command-shift `=` and command-shift `-` so a viewer who enlarges the menus, the Panel and the Diagnostics does not lose that. A Zoom scales about the console's top-left and moves the Pan no further than the Cursor follow and the Grid's edges ask. A Cursor move or a Zoom that would leave the Cursor out of the Source View Pans just far enough to bring it back.

## Rejected alternatives

**A wider Zoom range.** Three eighths to three — Cells of 6 to 48 points — was considered and refused: 22 sizes cost about 8,300 glyph rasters, close to the fill ratio at which egui rebuilds the whole atlas, for sizes past the point of use.

**The fitted canvas ADR 0038 kept.** Fitting the window made the Cell size a property of the window rather than of the viewer's choice: a larger window drew larger Cells rather than more of them, and pinch and command-wheel Zoom floated a shrinking Grid in letterboxing with no edge to stop at. That read as an infinite canvas the language does not have.

**An infinite canvas.** ADR 0005 defers Source addressing and ADR 0043 defines the Grid as bounded. A Source View unbounded past the Grid presents Positions that do not exist.

**A scrolled document.** An `egui::ScrollArea` would have given scrollbars and edge clamping for free. It was refused because the Source is a space rather than a document: scrollbars frame it as a page read top to bottom, and the translation would have moved from the transform the console owns into a container's scroll offset.

## Consequences

`CONTEXT.md` gains Source View, Pan and Zoom.

`Scene::register_pan_and_zoom` is no longer called: with no pointer Zoom, the only thing it still offered was the wheel Pan, which is one addition. The double click on the letterboxing that returned to the fit goes with the fit.

A stored Source keeps the Grid it was stored with. Nothing resizes a Grid, so a Source stored at the previous 40 by 25 default opens at 40 by 25.
