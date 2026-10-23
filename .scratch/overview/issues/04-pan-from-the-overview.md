# 04 — Pan from the Overview

**What to build:** A press on the Overview Pans the Source View. A press outside the outline centres the Source View there on the press frame and keeps following the drag; a press inside the outline drags the Source View from where it was grabbed, without a jump. The Pan stops at the margin past the Grid's edges, as every Pan does. The Cursor and Region never move, and the Source View does not snap back to the Cursor afterwards. The grab and grabbing pointer cursors show. See `../spec.md` (Pan request, Pointer semantics, Frame order).

**Blocked by:** 03 — Show the Source View on the Overview.

**Status:** ready-for-agent

- [ ] `SourceView` gains one pending-Pan request — the point of the padded Source to centre on — applied before the existing single Pan clamp, measured with the snapped Cell. The Overview never sets the Pan directly.
- [ ] The Overview senses drag only: not click, not focusable.
- [ ] Built test-first through kittest with coordinates from live geometry: a press outside the outline centres the Source View there; a drag from inside moves it by the drag with no initial jump; a drag past the Grid stops at the margin; the Cursor and Region are unchanged; a later Render Frame with no Cursor move does not Pan back to the Cursor; Tab still never focuses the Overview.
- [ ] One Render Frame of lag during a drag is accepted; the request is applied on the next Render Frame.
- [ ] `cargo fmt --all -- --check`, `cargo clippy --package console --all-targets --locked -- -D warnings` and `cargo nextest run --package console --locked` pass; `mise run check_wasm` passes.
