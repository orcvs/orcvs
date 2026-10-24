# Pan settle

A refactor from the 2026-10-24 architecture review (candidate 4). `show_source_scene` reads egui input and settles the Pan in one body: drag and wheel deltas, the Cursor-move and resize-follow checks, the follow or a Region drag's edge scroll, the requested centre, then the clamp. `follow_cursor`, `followed_cell`, `overshoot` and `edge_scroll` are not tested directly, and the order between them is reachable only through `console_pass`, a whole `egui::Context` pass; about 26 Pan tests in `console/src/console/tests.rs` run that way. This puts the settle behind one method that takes a plain input value, so the order is decided and tested in one place.

Nothing here changes how any Pan behaves or what the console paints. No speedup is claimed.

## Decisions

1. **Blocked by cell-geometry 02.** That ticket moves `cursor_cell`, `followed_cell`, the requested centre and the clamp's bound onto the Source View map, in the same body this extracts from; the settle is extracted from code that already measures in Cells.
2. **The settle owns the Pan rules, in order:** the gesture deltas, the Cursor-move and resize-follow checks with the `previous_cursor` and `previous_console` bookkeeping, the follow or the Region drag's edge scroll, the requested centre, and the clamp. It answers the settled Pan, `repaint` (a Region drag's pointer is past the console's edge) and `can_pan`.
3. **The adapter only reads.** `show_source_scene` reads raw facts into the input value: the middle-drag and primary-drag deltas, `alt`, whether the console contains the pointer, the smooth-scroll delta, the pointer, the console's size, the snapped Cell, the Grid and the Cursor. The settle decides what they mean, including that Alt pressed partway through a Region drag leaves it a Region drag.
4. **What stays in `show_source_scene`:** the cursor icon (from `can_pan`), `request_repaint` (from `repaint`), the origin and Paint, and setting and clearing `region_drag`, whose anchor needs the painted `GridViewport`. The settle reads only whether a Region drag is in progress, before this frame clears it, so a release still paces its last frame.
5. **Interface:** `SourceView::settle(&mut self, input: PanInput) -> Settled`. It updates the Pan and the bookkeeping fields, which stay private. Tests build `SourceView::new`, call `settle` frame by frame and read `Settled`. Production builds the same `PanInput`, so no seam exists only for tests. `PanInput` and `Settled` are implementation names; `CONTEXT.md` does not change.
6. **Tests:** input → Pan tables for `settle` cover each rule and the order (follow before the requested centre, the requested centre before the clamp, resize follow only for a Cursor shown in full, edge-scroll pacing and its one-Cell cap). Each `console_pass` test whose only assertion is the Pan becomes a `settle` test with the same name and assertion. One `console_pass` or kittest test per wiring path stays: wheel, middle drag, Alt with a primary drag, a Region drag past the edge, the Overview's centre request, the cursor icon, and the snapped Cell. The `clamp_pan` tests stay.
7. **Out of scope:** the Overview as one call (review candidate 3), a seam over egui input, and any change to the clamp, the follow or the margin.

## Constraints

- ADR 0038: the console owns the Source Grid transform; the settle produces the Pan and `presented_grid` stays the one expression of the transform.
- ADR 0045 and ADR 0047: one clamp bounds every Pan by the margin, and runs last.
- ADR 0080: the Overview's single pending Pan request is applied before the clamp and moves no Cursor.
- Native and WASM both build; nothing needs an `egui::Context` to call `settle`.
- No shipped function takes a parameter only a test populates.
- No new ADR: the change is internal to `console` and reversible, and its constraints are already recorded.

## Tickets

- `issues/01`: settle the Pan from one input value.
