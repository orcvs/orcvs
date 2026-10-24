# 02 — Present the Source View as one value

**What to build:** `ShownSourceView` in `console/src/console/source_view.rs`, replacing `ShownSource`. `show_source_scene` builds it after the clamp from the console rect, the `GridViewport` it presents and the Grid, and stores it on `SourceView` in place of `pub(super) origin`. It answers `screen()` (the map from 01), `shown()` (the console in Cells, unclipped), `outline()` (that region clipped to the extent, `None` where nothing can Pan), `console()` and `extent()`. `SourceView::new` holds a degenerate value whose `outline()` is `None`. `Console::ui` reads `source_view.shown()`; `show_source_panel` returns no geometry.

Inside `source_view`, `cursor_cell`, `followed_cell`, the requested centre and `clamp_pan`'s bound go through the map and the extent, and `outline()` and the clamp share one private slack predicate in place of `pannable`. `SourceView::centre_on` takes a `CellPoint`. The clamp stays in points, and follow → requested centre → clamp keeps its order. `SOURCE_MARGIN_CELLS` becomes private, and `panel::source_reserve` calls a `source_view` function for the height of one Row between its margins.

Overview and Diagnostics callers are moved only as far as compiling needs: 03 and 04 finish them.

**Blocked by:** 01

**Status:** ready-for-agent

- [ ] `ShownSource`, `SourceView.origin` and `padded_source`'s callers inside `source_view` are gone; `SOURCE_MARGIN_CELLS` is private to `source_view`.
- [ ] `ShownSourceView` is built from what Paint drew: `screen()` maps Cell `(0, 0)` to `presented_grid(..).rect.min`.
- [ ] `outline().is_some()` exactly when the clamp has somewhere to Pan, through one predicate, tested at the boundary on each axis.
- [ ] Every existing Pan, follow, resize-follow, Region-drag and Overview-Pan test passes unchanged in what it asserts.
- [ ] Pure tests build `ShownSourceView` from a literal `GridViewport`, console rect and Grid.
- [ ] `cargo fmt --all -- --check`, `cargo clippy --package console --all-targets --locked -- -D warnings` and `cargo nextest run --package console --locked` pass, and `mise run check_wasm` passes.
