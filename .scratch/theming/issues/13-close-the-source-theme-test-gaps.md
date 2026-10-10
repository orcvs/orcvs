# 13 — Close the Source Theme test gaps, and stop converting seam colour per Cell

**What to build:** The tests `06` resolved without, and the one per-Cell colour conversion left in the Paint loop.

**Blocked by:** None — can start immediately.

**Status:** resolved

- [x] *(Moved to `17`.)* A Paint-level test shows that an explicit transparent `region.cursor.background` suppresses the fallback to the Cursor fill for the Cursor inside a Region. The code at `console/src/paint.rs:242` (`Some(TRANSPARENT).or(..)`) is right; only the resolver covers the three optional-fill states today.
- [x] *(Moved to `17`.)* A test measures the composited pixel colour of a transparent Grid and of a partial-alpha Grid over the console surface. `console.rs:5643` checks only that `source_panel_frame` passes the fill through.
- [x] A test holds hit-testing unchanged across Cell border widths from 0 to 1 point, at more than one egui zoom factor (ADR 0058), stepped as `a_zoomed_console_paints_whole_pixel_cells_at_the_themes_widths` (`console/src/console/kittest_tests.rs:1242`) does.
- [x] `sector_line` (`console/src/style.rs:552`), called per seam Cell from `paint.rs:303` and `:309`, stops un-premultiplying and re-premultiplying `sector.seam` for every Cell. `../schema.md:207-208` rules out per-Cell colour interpolation. The per-Cell strength is a hash, so this is a once-per-Paint unpack or a premultiplied-space scale, not a table. `paint_derive` stays within `benches/floors.toml`.
- [x] `contrast.rs:344` and `paint.rs:242` share one derivation of the Region Cursor fill instead of each restating `region_cursor_background.or(cursor_fill)`.
- [x] `cargo nextest run --package console --locked` and the `--no-default-features` arm pass.

## Comments

**2026-09-23 — opened by the audit of the merged pull requests against their issues.** Residuals of `06`, which is resolved with these lines pointing here.

**2026-09-24 — split.** The two transparent-value tests moved to `17`, which joins `release/v1` and blocks `10`. The remaining lines (hit-testing across border widths, the per-seam colour conversion, the shared Region Cursor fill derivation) stay out of the release as improvements.

**2026-09-29 — audit at `cad296df`.** All three open lines still hold; only their line references drifted and are corrected in place: `sector_line` 562→`console/src/style.rs:552`, its per-seam calls 312/318→`console/src/paint.rs:303`/`:309`, and the duplicated `region_cursor_background.or(cursor_fill)` 397/246→`console/src/contrast.rs:344` and `console/src/paint.rs:242`. No test varies the Cell border width while clicking: the width tests in `console/src/console/tests.rs` check strokes only.

**2026-09-30 — the hit-testing line names its zoom.** "At more than one zoom" was written while the Source view had a zoom of its own. ADR 0058 leaves egui's whole-UI zoom as the console's only zoom, so the line now asks for more than one egui zoom factor, stepped by the command chords as `a_zoomed_console_paints_whole_pixel_cells_at_the_themes_widths` steps to 1.3. What it tests is unchanged.

**2026-10-10 — resolved.** `hit_testing_holds_across_cell_border_widths_and_zoom_factors` in `console/src/console/kittest_tests.rs` loads a Theme with `grid.border.width`, `cell.selection.border.width` and `cursor.border.width` at 0, 0.25, 0.5, 0.75 and 1, steps egui's zoom by the chords to 1.0 and 1.3, holds the presented Grid unchanged across the widths, and clicks each of four Cells at its centre and just inside two opposite corners. A deliberately broken hit test that shifted the pointer by the border width failed it at width 0.75. `sector_line` scales the premultiplied bytes of `sector.seam` by the strength with integer rounding, so no seam Cell leaves premultiplied space; `sector_line_scales_the_premultiplied_base_by_its_strength` pins it over every strength. In `sector_line_strength_only_attenuates_the_base_colours_alpha` the alpha at 8% moves from 8 to 9: 8% of 110 is 8.8, which the old conversion truncated. `paint_derive` in `console/benches/paint.rs` already times the seam colours; its comparison against `benches/floors.toml` is left to CI. `Theme::region_cursor_fill` is the one derivation of the Region Cursor fill, read by `Paint::derive_with_theme` and `contrast::painted`, and `the_region_cursor_fill_falls_back_to_the_cursor_fill_only_when_cleared` covers its five states.
