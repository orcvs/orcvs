# 17 — Prove the transparent fills and Grids a custom Theme can load

**What to build:** Tests that show a translucent or transparent value a custom Theme supplies paints as the schema says. `10` lets users load exactly these values, and today only the resolver covers them. Moved from `13`.

**Blocked by:** None — can start immediately.

**Status:** resolved

**Tags:** release/v1

- [x] A Paint-level test shows that an explicit transparent `region.cursor.background` suppresses the fallback to the Cursor fill for the Cursor inside a Region, while an omitted value falls back.
- [x] A test measures the composited pixel colour of a transparent Grid and of a partial-alpha Grid over the console surface, rather than only checking that the fill is passed through.
- [x] `cargo nextest run --package console --locked` and the `--no-default-features` arm pass.

## Comments

**2026-09-24 — split from `13` by the release-membership audit.** These two lines are release evidence for `10`'s acceptance ("an explicit transparent colour remains a supplied value and does not trigger that fallback") and for `07`'s translucent loaded Theme backdrop. Blocks `10`.

**2026-09-29 — both tests landed; no behaviour changed.** `paint::tests::an_explicit_transparent_region_cursor_fill_suppresses_the_cursor_fill_fallback` resolves a custom document inheriting `okabe-ito` with `cursor.background` set and `region.cursor.background` omitted, `"none"` and `"#FFFFFF00"`, over a transparent and a translucent `cell.background`, and paints the Cursor inside a spanning Region. Omitted and `none` fall back to the Cursor fill; the transparent value paints nothing over a transparent base and the base alone over a translucent one. Treating a zero-alpha value as absent where `Paint::derive_with_theme` (`console/src/paint.rs`) resolves `region_cursor_fill` fails it.

`console::kittest_tests::a_custom_themes_transparent_or_translucent_grid_composites_over_the_window_backdrop` loads a custom Theme with distinct `window.background` and `panel.background` and a `grid.background` of alpha 0, 0x80 and 0xFF, runs the shipped console through the kittest harness, and measures the composited colour at a point inside an empty Cell. The harness has no renderer (`snapshot`/`wgpu` stay off, per `console/Cargo.toml`), so the test tessellates the frame's Shapes with `Context::tessellate` and composites every untextured triangle covering the point, in paint order, source-over in premultiplied gamma space onto the `clear_color` backdrop — what the glow painter does at that pixel. The result is `window.background.blend(grid.background)` in all three cases; no panel fill sits beneath the Grid. Forcing `source_panel_frame`'s fill opaque fails it. No dependency or feature was added.
