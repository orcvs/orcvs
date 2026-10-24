# 01 — Add Cell points and the Source View map

**What to build:** In `console/src/grid_viewport.rs`, beside `presented_grid`, `CellPoint` and `CellRect`, each a newtype with a private field, and a map from Cells to screen points built from a `GridViewport`: `corner + cells × cell`, the same multiply-add `GridViewport::cell_rect` uses, so Cell `(0, 0)` lands on `viewport.rect.min` bit for bit. The map converts a `CellPoint` and a `CellRect` to screen points and a screen point back to Cells, and answers its Cell size. `CellPoint` and `CellRect` convert to and from egui types only through a map.

Nothing calls it yet outside its tests; `GridViewport` and Paint are unchanged.

**Blocked by:** None — can start immediately.

**Status:** resolved

- [x] `CellPoint` and `CellRect` exist with private fields and no public conversion to or from `Pos2`/`Rect` except through the map.
- [x] The map's forward conversion of every whole Cell equals `GridViewport::cell_rect`'s corner for that Position, exactly, at fractional Cell sizes and pixels-per-point.
- [x] A screen point converted to Cells and back is within float precision of where it started; the doc states it never chooses a Position.
- [x] A zero or non-finite Cell answers without NaN, and the degenerate behaviour is documented.
- [x] Pure tests, with no `egui::Context`.
- [x] `cargo fmt --all -- --check`, `cargo clippy --package console --all-targets --locked -- -D warnings` and `cargo nextest run --package console --locked` pass, and `mise run check_wasm` passes.

## Comments

- 2026-10-24: `check_wasm`'s WASM clippy pass (`cargo clippy --workspace --all-targets --target wasm32-unknown-unknown --locked -- -D warnings`) ran clean locally. Its two `trunk build` steps need `trunk`, which the cloud container lacks, so the CI `wasm` job is their evidence.
