# 01 — Add Source Cells and the Source View map

**What to build:** In `console/src/grid_viewport.rs`, beside `presented_grid`, a Source Cell point and a Source Cell rect, each a newtype with a private field, and a map from Source Cells to screen points built from a `GridViewport`: `corner + cells × cell`, the same multiply-add `GridViewport::cell_rect` uses, so Source Cell `(0, 0)` lands on `viewport.rect.min` bit for bit. The map converts a Source Cell point and rect to screen points and a screen point back to Source Cells, and answers its Cell size. Source Cells convert to and from egui types only through a map.

Nothing calls it yet outside its tests; `GridViewport` and Paint are unchanged.

**Blocked by:** None — can start immediately.

**Status:** ready-for-agent

- [ ] Source Cell point and rect types exist with private fields and no public conversion to or from `Pos2`/`Rect` except through the map.
- [ ] The map's forward conversion of every whole Source Cell equals `GridViewport::cell_rect`'s corner for that Position, exactly, at fractional Cell sizes and pixels-per-point.
- [ ] A screen point converted to Source Cells and back is within float precision of where it started; the doc states it never chooses a Position.
- [ ] A zero or non-finite Cell answers without NaN, and the degenerate behaviour is documented.
- [ ] Pure tests, with no `egui::Context`.
- [ ] `cargo fmt --all -- --check`, `cargo clippy --package console --all-targets --locked -- -D warnings` and `cargo nextest run --package console --locked` pass, and `mise run check_wasm` passes.
