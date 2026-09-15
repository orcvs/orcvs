# 02 — Replace the checker bloom with a living green area

**What to build:** Surround the eroded Cursor with the approved C + area treatment: a faint grid-green field of connected grain, irregular empty patches, and short horizontal signal fragments. Both the white frame and green area continue evolving while the Cursor is stationary. The area supports focus without obscuring the Source.

**Blocked by:** 01 — Animate the eroded Cursor frame.

**Status:** resolved

- [ ] Remove the four cell-aligned brightness bands and hashed whole-Cell breakup responsible for the checker appearance. The new field is continuous across Cell boundaries, with locally related noise and large irregular gaps.
- [ ] Use the current bloom's grid-green hue by default, matching the approved green prototype: RGB 76, 190, 156, with subdued opacity. Preserve the off-white Cursor frame and existing Glyph colours.
- [ ] The area reaches roughly seven Cells from the Cursor and fades irregularly toward its boundary. Keep this reach internal; this slice adds no tuning controls.
- [ ] Grain and horizontal fragments visibly change while stationary and with Playback stopped. The broad field evolves more slowly than the fine grain and streaks; it does not freeze after a movement burst or flash as one surface.
- [ ] The area is painted beneath Grid lines and Glyphs, with the Cursor remaining the strongest focus mark. Verify readability over densely occupied Source as well as empty Cells.
- [ ] Zoom, pan, letterboxing, and viewport clipping preserve the effect's relationship to the selected Cell. A Cursor just outside the viewport still contributes the portion of its area that reaches into view.
- [ ] Construction and animation work are bounded to the visible effect rather than the whole Grid. Offscreen work and missed animation updates do not accumulate. Share the console animation timing ownership established by ticket 01.
- [ ] Focused tests cover continuous field geometry, paint ordering, edge clipping, stationary evolution, and bounded work. Fixed presentation samples support repeatable verification without forcing deterministic user-visible patterns across launches.
- [ ] Visually verify native and WASM renders at normal size, enlarged detail, viewport edges, and several stationary realizations against the approved C + area and green prototypes. Preserve readable Glyphs and the independently changing frame from ticket 01.
- [ ] Update appearance documentation to replace the historical static four-band bloom description. Extend the painting benchmark to cover the combined effect, run applicable repository checks, and defer benchmark comparison and full cross-platform gates to CI as the repository contract specifies.

## Design decisions

The rejected smooth radial halo and evenly arranged filaments are not the target. The chosen effect uses uneven density, black absences, fine connections, and horizontal interruption. Area animation must be visible independently of Cursor-frame animation; changing only the frame is insufficient.

## Answer

The cell-aligned bloom no longer affects shipped Paint derivation. A bounded green field now paints beneath the Grid with continuous grain, connected strands, irregular gaps, horizontal fragments, and a slower broad-field epoch. The actual Cursor Position drives it even when only the field reaches the viewport.
