# 01 — Paint a Function border from the Theme

**What to build:** A Theme can draw a border around Function Cells. A Theme document sets `source.function.border` (a colour) and `source.function.border.width` (a Grid width, 0–1 point), and the console strokes that border around every occupied Function Cell. Today a Source role carries only a glyph and a fill, so the only per-Cell borders are the Grid line and the Diagnostic, Portal and Locked overlay borders. This is the tracer bullet for role borders: one role through every layer.

The role border is a new layer in the Cell border composition, above the Grid line and below every overlay border: Grid line, then role border, then Diagnostic, then the painting Portal or Locked. Two rules keep it from taking over the Grid:

- It applies only when its colour is visible (alpha above zero). A transparent role border leaves the Grid line's colour **and width** in place, so `grid.border.width` keeps governing every Cell no role border is drawn on.
- It applies only to occupied Cells. An empty Cell keeps the plain Grid line.

When it applies, it decides the stroke's width the way the overlay channels do: the winning channel's width is the stroke's width.

The four built-in Themes set the Function border transparent at 0.5 point, the same pattern as the Diagnostic and Portal borders, so nothing a built-in paints changes.

**Blocked by:** None — can start immediately.

**Status:** resolved

- [x] `source.function.border` and `source.function.border.width` are in the Theme property catalogue, so the document decoder accepts them, rejects out-of-range widths as it does every Grid width, and a custom Theme inherits them from its parent.
- [x] All four built-in Themes define both properties explicitly, at transparent and 0.5 point.
- [x] The theming schema's property catalogue lists both properties, and its composition step for borders states the new layer, the visible-only rule and the occupied-only rule.
- [x] The values tables in the console's Theme record list both properties for every built-in, and the record tests that pin each built-in to its table pass.
- [x] Paint tests show: a visible Function border strokes an occupied Function Cell at its own width; a transparent one leaves the Grid line's colour and width; an empty Cell never takes it; a Diagnostic, Portal or Locked border wins over it on the same Cell; a Function border width of zero hides that Cell's stroke only when the border is visible.
- [x] A custom Theme document setting both properties loads, and the console draws the border (a resolve-and-paint test, not only a decoder test).
- [x] The contrast gate is unchanged; its existing exclusion of border visibility covers the new property.
- [x] `cargo fmt --all -- --check`, `cargo clippy --package console --all-targets --locked -- -D warnings`, `cargo nextest run --package console --locked` and the `--no-default-features` arm pass.

## Comments

**2026-10-20 — landed.** `source.function.border` and `source.function.border.width` are in the style catalogue, so the decoder, `resolve` and the by-key accessors take them; the four built-ins set them transparent at 0.5. The Cell visuals take an `occupied` flag (the Cell's content), and `style::ordinary_border` composes the role border over the Grid line, beneath Diagnostic and the painting Portal or Locked, only on an occupied Cell whose role border is visible. `SourcePaintVisuals` caches by occupancy as well as fact and Portal. Tests: `paint::tests::a_visible_function_border_strokes_the_function_cells_at_its_own_width`, `a_transparent_function_border_leaves_the_grid_line_and_its_width`, `a_theme_document_paints_the_function_border_it_sets`; `style::tests::the_locked_border_composes_over_a_functions_own_border`, `an_empty_cell_never_takes_its_roles_border` (fails with the occupancy check removed). The grayscale shared-grey test now reads the real Function border. `schema.md`'s catalogue and composition step 4, and `theme.md`'s three built-in records, list the property.
