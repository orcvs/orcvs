# 06 — Let the paint answer instead of take

**What to build:** `show_source` derives a `Paint`, converts it to Shapes, extends the painter, and answers the click. It stops taking `&mut Orcvs`. Every test that reached into the flat shape list to recover a colour moves to the value layer.

The test migration cannot be a separate commit. The moment `show_source`'s signature changes, every test that captures a shape list through it stops compiling, and each commit is expected to pass its gates. This ticket is large for that reason and no other.

**Blocked by:** 05

**Status:** ready-for-agent

### The shape step

- [ ] `SourceShapes { backgrounds, borders, glyphs, seams, cursor }` is built from a `&Paint` plus `&GridViewport`, `&GlyphTable`, the scale and `pixels_per_point`.
- [ ] It lives in `console.rs`, not in `paint.rs`. `05` bars geometry from that module and this type takes a `GridViewport`, so the shape step stays beside the painting it serves.
- [ ] `into_shapes()` concatenates in that order, which is the order `show_source` chains today: backgrounds, borders, glyphs, seams, cursor strokes.
- [ ] The five fields are built eagerly. Shape construction is never lazy: `Painter::extend` runs the iterator inside `ctx.graphics_mut`, a full `Context` write lock, so a lazily built Shape is one built while holding it.
- [ ] `into_shapes()` answers `impl Iterator<Item = Shape>` over the chained owned `Vec`s, not a `Vec<Shape>`. A sixth Vec costs a ~2000-element allocation and a full re-move per Render Frame that today's `chain` avoids. All five fields are still complete before the first Shape leaves, so the ordering test is unaffected.
- [ ] `background_run`'s device-scale snapping stays here, applied to the column ranges `Paint::background_runs()` answers.
- [ ] Its comment says what it is for. It is not load-bearing against seams: `RectShape::filled` leaves `round_to_pixels: None`, `TessellationOptions::round_rects_to_pixels` defaults true, and `tessellate_rect` applies `Rect::round_to_pixels` — the same `emath` function — at `tessellator.rs:1829-1862`. epaint snaps the rect whether or not this code does. The snapping earns its place by making the rectangle a test can predict; say so, and stop the comment claiming a rendering fault it does not prevent.

### The seam out

- [ ] `show_source` answers `Option<Position>` — the click, and nothing else. It does **not** answer the `Paint`: nothing in production would read it, `Paint::derive` is already reachable, and a return value only a test reads is a test-only seam in shipped code.
- [ ] `show_source` no longer takes `&mut Orcvs`. There is no `select()` call inside it.
- [ ] `show_source_scene` no longer takes `&mut Orcvs` either. It holds one today for nothing but the forward to `show_source`, so `Console::ui` cannot own the `select` while it still takes one.
- [ ] `show_source_scene` answers `PresentedSource { viewport: GridViewport, clicked: Option<Position> }`.
- [ ] `Console::ui` calls `orcvs.select(...)` with the answered Position.
- [ ] There is no `PaintedSource` and no `SourcePaint` type.
- [ ] One `painter.extend`, never a `Painter::add` per Shape — `add` takes a full `Context` write lock.

### The tests

- [ ] No test asserting what colour a Cell is constructs a live `Orcvs` or an egui `Context`. This is the bar; there is no target count, because a count invites satisfying it by merging tests.
- [ ] The only test still reading a flat `Vec<Shape>` is the one asserting `into_shapes()`'s concatenation order, and it runs through `console_pass` so it also covers the three-line wiring inside `show_source` that nothing else proves. It stays there because paint order is observable there and nowhere else, not as a concession — `egui_kittest`'s own regression tests assert against `output().shapes` the same way.
- [ ] `every_background_is_painted_before_every_glyph_and_the_cursor_after_both` asserts over `SourceShapes` fields.
- [ ] `consecutive_cells_sharing_a_background_are_one_rectangle` asserts over column ranges from `Paint::background_runs()`.
- [ ] `the_cell_needing_no_background_is_exactly_the_one_filled_with_the_source` moves to the value layer and calls `cell_visuals()` directly for its comparison. It is the one assertion tying the skip decision to the function it must not drift from — do not delete it as covered elsewhere.
- [ ] The harness's hand-written `retain()` subtracting the panel's own fill (`console.rs:1356-1362`) is deleted.
- [ ] The three shape-filtering helpers — `stroked_cell_rects`, `background_runs`, `background_at` — are deleted or reduced to whatever the one remaining flat-list test needs.

### Out of scope

- [ ] The 266-line distance between `visuals.background != PALETTE.source` and the `source_panel_frame()` that makes it correct is untouched. That is a separate locality finding.
- [ ] Nothing about what appears on screen changes: same colours, same geometry, same order.

## Verification

`cargo fmt --all -- --check`, then `cargo clippy --package console --all-targets --locked -- -D warnings` and `PROPTEST_CASES=32 cargo nextest run --package console --locked`.

`mise run check_wasm` is deferred to CI and named on the `Not run` line: nothing here is platform-conditional, there is no `cfg` and no new dependency. No benchmark is owed — the shape sequence handed to `painter.extend` is unchanged.
