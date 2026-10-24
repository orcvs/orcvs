# 03 — Draw the Overview over the extent

**What to build:** The Overview in `console/src/console/overview.rs` maps Cells onto its picture with `emath::RectTransform::from_to(extent, picture)`, wrapped to take and return `CellPoint` and `CellRect`. `outline_source_view` strokes `ShownSourceView::outline()` through it, `pan_source_view` converts the pointer to Cells, fixes the grab against `outline()` and asks `SourceView::centre_on`, and `mark_region_and_cursor` maps the Region's and the Cursor's Cells through it. `draw_picture` lays its texels out from the extent. Both take the `ShownSourceView` in place of `(visible, grid, cell)`.

**Blocked by:** 02

**Status:** resolved

- [x] `overview::outline`, `overview::MARGIN`, the `pannable` guards and every `padded_source` call in `overview.rs` are gone.
- [x] The outline, grab and marks test families each keep one test whose expectation is worked by hand from texel counts, not through the map.
- [x] Every existing Overview test passes in what it asserts.
- [x] `draw_picture` stays public for the benchmark, and `paint_overview` measures the same walk.
- [x] `cargo fmt --all -- --check`, `cargo clippy --package console --all-targets --locked -- -D warnings` and `cargo nextest run --package console --locked` pass, and `mise run check_wasm` passes.

## Comments

- 2026-10-24: `check_wasm`'s WASM clippy pass (`cargo clippy --workspace --all-targets --target wasm32-unknown-unknown --locked -- -D warnings`) ran clean locally. Its two `trunk build` steps need `trunk`, which the cloud container lacks, so the CI `wasm` job is their evidence.
