# 04 — Pan from the Overview

**What to build:** A press on the Overview Pans the Source View. A press outside the outline centres the Source View there on the press frame and keeps following the drag; a press inside the outline drags the Source View from where it was grabbed, without a jump. The Pan stops at the margin past the Grid's edges, as every Pan does. The Cursor and Region never move, and the Source View does not snap back to the Cursor afterwards. The grab and grabbing pointer cursors show. See `../spec.md` (Pan request, Pointer semantics, Frame order).

**Blocked by:** 03 — Show the Source View on the Overview.

**Status:** resolved

- [x] `SourceView` gains one pending-Pan request — the point of the padded Source to centre on — applied before the existing single Pan clamp, measured with the snapped Cell. The Overview never sets the Pan directly.
- [x] The Overview senses drag only: not click, not focusable.
- [x] Built test-first through kittest with coordinates from live geometry: a press outside the outline centres the Source View there; a drag from inside moves it by the drag with no initial jump; a drag past the Grid stops at the margin; the Cursor and Region are unchanged; a later Render Frame with no Cursor move does not Pan back to the Cursor; Tab still never focuses the Overview.
- [x] One Render Frame of lag during a drag is accepted; the request is applied on the next Render Frame.
- [x] `cargo fmt --all -- --check`, `cargo clippy --package console --all-targets --locked -- -D warnings` and `cargo nextest run --package console --locked` pass; `mise run check_wasm` passes.

## Comments

Shipped in `console/src/console/source_view.rs` and `console/src/console/overview.rs`. `SourceView::centre_on` holds one pending request: the point of the padded Source to centre the console on, counted in Cells from the padded Source's top-left so `show_source_scene` measures it with the Cell it snaps that Render Frame. The Source scene takes it after any Cursor follow of that frame and immediately before its one `clamp_pan`, so a request past the Grid settles at the margin like every other Pan. A later request before that frame replaces the earlier one.

`Overview::pan_source_view` runs from `Console::ui` after the Source is presented and the outline drawn, from the same visible Source region, so the grab is decided against the Pan this Render Frame settled. On the press frame (a drag-only widget starts its drag on press) the grab is zero when the pointer is outside the outline and the offset from the pointer to the outline's centre when inside; each frame the primary button is held it asks the Source View to centre on the pointer plus the grab, and the next Render Frame applies it. It sets the grabbing cursor while held and the grab cursor on hover, and requests a repaint so the request lands without further input. Where the console shows the whole padded Source there is nowhere to Pan and it does nothing, as the Source View's own grab hand does. The Overview still senses drag only, so it stays out of the Tab order and never selects; the Cursor-follow rule is unchanged.

Tests: four `console::kittest_tests` cases drive the shipped `Console` with every coordinate read from the Overview node's rect and the live origin. A press outside the outline centres the Source View on the pressed point one frame later with the button still held, the Overview is not focused, the same press then follows a drag, and after release further frames leave the Pan alone with the Cursor out of view; a press inside the outline does not move the Source View and a drag then moves it by the drag scaled from picture to padded Source; a drag far past the picture's corner stops with the console's far corner on the padded Source's; the grab and grabbing hands show and clear. The first two also assert the Cursor and a three-Cell Region are unchanged. `the_overview_is_one_labelled_node_that_tab_never_focuses` still covers Tab.

The spec's Pointer semantics said the inside grab offset runs "from the outline's centre to the pointer", which added to the pointer would double the jump; it now reads from the pointer to the outline's centre, as shipped. A drag held while a key moves the Cursor keeps the drag's Pan for that frame, and the Cursor is not followed afterwards, since the follow answers a Cursor move and the move was already seen; the drag is the performer's own gesture, so this is left as the Cursor-follow rule has it.
