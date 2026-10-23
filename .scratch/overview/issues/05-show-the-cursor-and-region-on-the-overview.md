# 05 — Show the Cursor and Region on the Overview

**What to build:** The performer can find their Cursor and Region on the Overview when either is off screen. The Cursor's Cell and the Region are marked over the picture in their existing Theme colours, drawn as overlays so a Cursor move or Region change rebuilds no picture. See `../spec.md` (Overlays).

**Blocked by:** 02 — Show the Overview.

**Status:** resolved

- [x] Built test-first at the Shape level: the Region mark covers the Region's Cells and the Cursor mark the Cursor's Cell at their predicted rects, drawn over the picture and under the Source View outline when ticket 03 has landed.
- [x] They reuse the Cursor and Region Theme keys; no new key.
- [x] Moving the Cursor or changing the Region does not rebuild the picture.
- [x] `cargo fmt --all -- --check`, `cargo clippy --package console --all-targets --locked -- -D warnings` and `cargo nextest run --package console --locked` pass; `mise run check_wasm` passes.

## Comments

Shipped in `console/src/console/overview.rs`. `show_overview` marks the Region and then the Cursor's Cell on the picture, from the Render Frame it already reads, with the painter clipped to the picture that `ShownOverview` keeps, straight after the image is painted. `Console::ui` draws the Source View's outline from that painter after the Source is presented, so the order is the spec's: picture, Region, Cursor, outline. Each mark is one `RectShape` over its Cells, one texel per Cell past the two-Cell margin, in the fill and border the Source draws it with: the Region in `region.background` with `region.border` at `region.border.width`, the Cursor in `cursor.background` (transparent where cleared, as both built-ins leave it) with `cursor.border` at `cursor.border.width`, both stroked inside. No key is added. Where a border is wider than the Cells it surrounds, epaint fills the rect with the border's colour, so a one-texel Cursor shows as a solid block of `cursor.border`. A Region of one Cell is the Cursor's Cell and gets no mark of its own, as Paint treats it. The Cursor keeps its mark inside a spanning Region, where the Source drops the Cursor's outline for the Region's, so the Overview always shows where the Cursor is. The marks are not in the picture, so the cache key stays the revision, the Theme and the appearance. Nothing animates.

Tests: `the_overview_marks_the_region_and_the_cursor_between_the_picture_and_the_outline` in `console::tests` drives the shipped `Console` with a four-by-two Region whose Cursor is its far corner, and finds the Region mark and the Cursor mark at the rects predicted from the picture's rect, one texel per Cell plus the margin, in the Theme's fills and borders, painted after the image and before the outline; after a select collapses the Region, the Cursor's Cell is the only mark. `the_overview_uploads_its_picture_only_when_the_source_or_appearance_changes` now also moves the Cursor and extends the Region and asserts neither uploads. `source_view_outlines` picks the outline by the Theme's outline colour, since the Cursor mark is now a stroked rect on the picture too.

With this ticket every Overview ticket is resolved, so the spec is too.
