# 05 — Derive the paint as a value

**What to build:** `console/src/paint.rs`, holding the per-Cell paint description, the coalescing fold, and their tests. Nothing else changes: `show_source` is untouched by this ticket and keeps its current loop.

That is deliberate. This ticket lands entirely new code with nothing broken, so `06` is a rewire rather than an invention.

**Blocked by:** 03, 04

**Status:** ready-for-agent

- [ ] `Paint::derive(&RenderFrame) -> Paint` takes nothing else — no `Orcvs`, no egui `Context`, no `GridViewport`.
- [ ] `Paint` stores `Vec<CellPaint>` flat plus the `Grid`, and `at(Position)` indexes through `Grid::index`. The module doc says why this is unlike `RenderFrame`'s row nesting: `Paint`'s primary access is `at(Position)`, and the nesting on `RenderFrame` exists only to serve painting.
- [ ] `CellPaint` is flat — `background: Option<Color32>`, `border`, `foreground`, two seam colours, `character`. It does not also hold a `CellVisuals`.
- [ ] `cell_visuals()` in `style.rs` is unchanged and called once inside the derive.
- [ ] The background skip is the derive's: `background` is `None` exactly where `cell_visuals` asks for `PALETTE.source`.
- [ ] `Paint::cursor() -> Position`. There is no `is_cursor` bool on `CellPaint`: `RenderFrame::derive` takes one `selected` and asserts the Grid owns it, so exactly one exists and a per-Cell bool would re-open a state the layer below closed.
- [ ] Sector seams are suppressed on the Cursor's Cell by the derive, so its `CellPaint` carries no seams and no later step learns the rule.
- [ ] `Paint::background_runs()` is the coalescing fold, pure over the per-Cell backgrounds, answering column ranges within a row — not rectangles. A run ends at a different colour, at no colour, and at the end of its row.
- [ ] No geometry anywhere in the module. No `Rect`, no `GridViewport`.
- [ ] Tests construct a `RenderFrame` and read `Paint` directly. None builds an egui `Context`, a `RawInput`, a `CentralPanel` or a running `Orcvs`.
- [ ] `CONTEXT.md` gains **Paint**: the per-Cell decision of how one Render Frame is drawn — background, border, foreground, seams and the character shown — derived from a Render Frame and carrying no geometry. `_Avoid_`: shapes, draw list, painter. No crate name, no file path, no egui reference.

Do not name the type `PaintedCells` or anything else in the past tense. The painting has not happened; this is the description a later step turns into shapes.

## Verification

`cargo fmt --all -- --check`, then `cargo clippy --package console --all-targets --locked -- -D warnings` and `PROPTEST_CASES=32 cargo nextest run --package console --locked`.
