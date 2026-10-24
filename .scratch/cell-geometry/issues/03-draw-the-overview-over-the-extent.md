# 03 — Draw the Overview over the extent

**What to build:** The Overview in `console/src/console/overview.rs` maps Cells onto its picture with `emath::RectTransform::from_to(extent, picture)`, wrapped to take and return `CellPoint` and `CellRect`. `outline_source_view` strokes `ShownSourceView::outline()` through it, `pan_source_view` converts the pointer to Cells, fixes the grab against `outline()` and asks `SourceView::centre_on`, and `mark_region_and_cursor` maps the Region's and the Cursor's Cells through it. `draw_picture` lays its texels out from the extent. Both take the `ShownSourceView` in place of `(visible, grid, cell)`.

**Blocked by:** 02

**Status:** ready-for-agent

- [ ] `overview::outline`, `overview::MARGIN`, the `pannable` guards and every `padded_source` call in `overview.rs` are gone.
- [ ] The outline, grab and marks test families each keep one test whose expectation is worked by hand from texel counts, not through the map.
- [ ] Every existing Overview test passes in what it asserts.
- [ ] `draw_picture` stays public for the benchmark, and `paint_overview` measures the same walk.
- [ ] `cargo fmt --all -- --check`, `cargo clippy --package console --all-targets --locked -- -D warnings` and `cargo nextest run --package console --locked` pass, and `mise run check_wasm` passes.
