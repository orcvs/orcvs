# 05: Set the Cell size in the settings file

**What to build:** A performer sets `width` and `height` under `[cell]` in `~/.orcvs/config.toml`, in points, and the console draws its Cells and Glyphs at that size. Each key is optional and bounded to 8 to 64 points. An out-of-range or mistyped value is reported in the usual settings notice and falls back to its own default, leaving the other key standing. A size that doesn't land on whole pixels snaps quietly. The web reads no file and uses the default. See `../spec.md`.

**Blocked by:** 04 — Draw the default Cells with a matching Glyph.

**Status:** resolved

- [x] The settings reader reads both keys, defaults each when it is absent, and falls back on its own for a bad value, as its existing tests do for other keys.
- [x] A console started with a configured Cell presents its Grid, Pan bounds and Glyphs at that size.
- [x] A configured Cell at the smallest bound under the lowest Zoom keeps its Grid and Glyphs in agreement: the Glyph is derived from the snapped Cell, which a test checks.
- [x] A narrow configured width shrinks the Glyph rather than letting it overlap its neighbours.
- [x] `paint_derive` and `paint_shapes` each run at the smallest bound (8 by 8 shows about 4× the Cells of today's 16 by 16) with a floor, so a per-Cell cost regression at small Cells is caught.
- [x] The settings file's documentation shows the `[cell]` section.

## Comments

- `Config::cell` holds the configured Cell, the default when the file has no `[cell]` section. Each key takes an integer or a float from 8 to 64 points and falls back alone. `SourceView::new` takes it, and opening another Source keeps it.
- `a_console_started_with_a_configured_cell_presents_the_source_at_it` starts a Console at 10 by 15 points and checks the drawn Cell, a click, the Glyph size, the far-edge Pan bound and the Cell after opening the Function Reference.
- `the_smallest_cell_at_the_lowest_zoom_lays_its_glyph_out_at_the_snapped_cell`: 8 points at egui's 0.2 Zoom snaps to 5 points, with a 3.75 point Glyph. The narrow-width case is in `a_glyph_is_three_quarters_of_its_cells_height_or_what_its_width_fits`.
- Benchmarks: `paint_derive/window/8x8` measured about 186 µs and `paint_shapes/window/8x8` about 1.67 ms locally. Floors are 400 µs and 3.5 ms.
