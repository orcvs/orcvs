# 01 — Decide a Cell's colours in one place

**What to build:** A `CellColours` value in `console/src/style.rs`, built once per Paint from the Theme, owns a Cell's whole colour decision. It holds the prepared Cursor, Region-Cursor, Region and base fills and the per-Paint unselected visuals cache that `SourcePaintVisuals` keeps today, and answers one method, `colours(fact, portals, placement)`, with the Cell's background, foreground and border. `placement` is the existing `CursorPlacement` (moved beside `CellColours` if contrast is its only home today), so Paint stops deriving `is_cursor`, `selected`, `in_region` and `region_spans` booleans for colour. Paint (`paint.rs`, the per-Cell walk) and contrast (`contrast.rs`, `painted`) both call it, and `cell_background` is deleted rather than wrapped.

Paint keeps geometry and edges: re-ranking the Cursor's border to `EdgeRank::RegionCursor` inside a spanning Region stays in Paint, as edge arbitration under ADR 0078. Contrast keeps blending the painted background over the window and Grid surfaces.

**Blocked by:** None — can start immediately.

**Status:** resolved

- [x] `CellColours::colours` is the only caller of the selected and unselected visuals, and neither Paint nor contrast prepares a fill or calls a composition helper itself.
- [x] `cell_background` and its seven arguments are gone.
- [x] The per-Paint cache is kept: the unselected visuals are looked up once per distinct fact and Portal roles in a Paint, as now.
- [x] Style tests cross the whole decision per `CursorPlacement` (`Plain`, `Cursor`, `Region`, `RegionCursor`) over an ordinary, a tinted and a translucent background; the existing expected-colour and Paint integration tests pass unchanged.
- [x] The contrast gate's measurements are unchanged for every shipped Theme.
- [x] The empty and Region-spanning Paint benchmarks still exist and still measure the same walk.
- [x] `cargo fmt --all -- --check`, `cargo clippy --package console --all-targets --locked -- -D warnings` and `cargo nextest run --package console --locked` pass.
