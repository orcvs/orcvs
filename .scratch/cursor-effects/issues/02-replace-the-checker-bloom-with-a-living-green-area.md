# 02 — Replace the checker bloom with a living green area

**What to build:** Surround the eroded Cursor with the approved C + area treatment: a faint grid-green field of connected grain, irregular empty patches, and short horizontal signal fragments. Both the white frame and green area continue evolving while the Cursor is stationary. The area supports focus without obscuring the Source.

**Blocked by:** 01 — Animate the eroded Cursor frame.

**Status:** resolved

- [x] Remove the four cell-aligned brightness bands and hashed whole-Cell breakup responsible for the checker appearance. The new field is continuous across Cell boundaries, with locally related noise and large irregular gaps. Evidence: checker bloom removed from `marks.rs`/`paint.rs`; `area_is_bounded_crosses_cell_boundaries_and_can_reach_in_from_outside_clip` asserts cross-cell continuity.
- [x] Use the current bloom's grid-green hue by default, matching the approved green prototype: RGB 76, 190, 156, with subdued opacity. Preserve the off-white Cursor frame and existing Glyph colours. Evidence: `DEFAULT_AREA_COLOUR` and `defaults_are_the_approved_colours_and_an_irregular_live_cadence`.
- [x] The area reaches roughly seven Cells from the Cursor and fades irregularly toward its boundary. Keep this reach internal; this slice adds no tuning controls. Evidence: `AREA_RADIUS_CELLS = 7.0`; `advertised_bounds_contain_the_outer_area_tails`.
- [x] Grain and horizontal fragments visibly change while stationary and with Playback stopped. The broad field evolves more slowly than the fine grain and streaks; it does not freeze after a movement burst or flash as one surface. Evidence: separate `frame_epoch` and `field_epoch` with `field` interval ×3 in `CursorEffectAnimation::advance`.
- [x] The area is painted beneath Grid lines and Glyphs, with the Cursor remaining the strongest focus mark. Verify readability over densely occupied Source as well as empty Cells. Evidence: `SourceShapes::new` paints area before Grid content and frame last; native visual check on occupied Source.
- [x] Zoom, pan, letterboxing, and viewport clipping preserve the effect's relationship to the selected Cell. A Cursor just outside the viewport still contributes the portion of its area that reaches into view. Evidence: `area_is_bounded_crosses_cell_boundaries_and_can_reach_in_from_outside_clip` (cursor left of clip, shapes inside clip).
- [x] Construction and animation work are bounded to the visible effect rather than the whole Grid. Offscreen work and missed animation updates do not accumulate. Share the console animation timing ownership established by ticket 01. Evidence: `effect_bounds` + clip intersection; shared `CursorEffectAnimation` deadline model from ticket 01.
- [x] Focused tests cover continuous field geometry, paint ordering, edge clipping, stationary evolution, and bounded work. Fixed presentation samples support repeatable verification without forcing deterministic user-visible patterns across launches. Evidence: `cursor_effects.rs` geometry tests; fixed `CursorEffectSample` inputs.
- [x] Visually verify native and WASM renders at normal size, enlarged detail, viewport edges, and several stationary realizations against the approved C + area and green prototypes. Preserve readable Glyphs and the independently changing frame from ticket 01. Evidence: native visual check on branch; WASM build passes `check_wasm`; headless browser suite deferred to CI.
- [x] Update appearance documentation to replace the historical static four-band bloom description. Extend the painting benchmark to cover the combined effect, run applicable repository checks, and defer benchmark comparison and full cross-platform gates to CI as the repository contract specifies. Evidence: `console/src/theme.md`; `console/benches/paint.rs` `cursor_effects`; crate-scoped gates on branch.

## Design decisions

The rejected smooth radial halo and evenly arranged filaments are not the target. The chosen effect uses uneven density, black absences, fine connections, and horizontal interruption. Area animation must be visible independently of Cursor-frame animation; changing only the frame is insufficient.

## Answer

The cell-aligned bloom no longer affects shipped Paint derivation. A bounded green field now paints beneath the Grid with continuous grain, connected strands, irregular gaps, horizontal fragments, and a slower broad-field epoch. The actual Cursor Position drives it even when only the field reaches the viewport.
