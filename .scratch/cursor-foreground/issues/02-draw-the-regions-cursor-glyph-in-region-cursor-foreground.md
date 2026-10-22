# 02 — Draw the Region's Cursor glyph in `region.cursor.foreground`

**What to build:** While a Region spans more than one Cell, a Theme can colour the glyph on the Cursor's own Cell separately from the single-Cell Cursor's. A Theme document sets `region.cursor.foreground`, and the glyph on the Region's Cursor Cell draws in it.

It falls back the way the Region's Cursor fill does. `region.cursor.background` falls back to `cursor.background` when it is `"none"`, and `region.cursor.foreground` does the same:

1. `region.cursor.foreground` when it holds a colour, transparent included;
2. otherwise `cursor.foreground` when that holds a colour;
3. otherwise the glyph's role colour.

The other Cells of the Region keep their role colours. The fill and the lasso are unchanged.

All four built-in Themes set it to `"none"`, so nothing a built-in paints changes.

**Blocked by:** 01 — Draw the glyph under the Cursor in `cursor.foreground`

**Status:** resolved

- [x] `region.cursor.foreground` is in the Theme property catalogue as an optional colour, decoded and inherited exactly as `01`'s `cursor.foreground` is, and all four built-ins define it as `"none"`.
- [x] The fallback has one statement, beside the Region's Cursor fill fallback, which both painting and the contrast check read.
- [x] Paint tests show each step of the fallback on the Region's Cursor Cell: its own colour; `cursor.foreground` when it is `"none"`; the role colour when both are `"none"`; and an explicit transparent value suppressing the fallback. They also show that the other Region Cells keep their role colours.
- [x] The contrast gate measures the Region's Cursor state with the resolved glyph colour, and a test shows a failing pairing is reported.
- [x] The theming schema and the console's Theme record list the property for every built-in, and the record tests pass.
- [x] `cargo fmt --all -- --check`, `cargo clippy --package console --all-targets --locked -- -D warnings`, `cargo nextest run --package console --locked` and the `--no-default-features` arm pass.

## Comments

**2026-10-20 — landed.** `region.cursor.foreground` joins the style catalogue's optional colours, so it decodes, inherits and clears exactly as `cursor.foreground` does, and the four built-ins set it to `none`. `Theme::region_cursor_glyph`, beside `Theme::region_cursor_fill`, states the fallback once: `region.cursor.foreground`, then `cursor.foreground`, then `None` for the role colour. `style::cell_foreground` applies it to the Region's Cursor Cell alone (`is_cursor && region_spans`, as `style::cell_background` does), and both `Paint::derive_with_theme` and `contrast::painted` call it, so the Region's Cursor state measures the resolved glyph. Tests: `theme::tests::the_region_cursor_glyph_falls_back_to_the_cursor_glyph_only_when_cleared`, `paint::tests::the_regions_cursor_glyph_falls_back_to_the_cursor_glyph_then_the_role` (each step from written Source, transparent suppressing the fallback, the other Region Cells, the fill and the grid line unchanged), `paint::tests::a_theme_document_paints_the_region_cursor_foreground_it_sets`, `contrast::tests::the_region_cursor_state_measures_the_resolved_region_cursor_glyph`; the resolve, decoder, example and record tests cover the new property. Dropping the `cursor.foreground` fallback fails the theme, paint and contrast fallback tests; dropping the Region-Cursor guard fails both paint tests. `schema.md`'s catalogue and composition step 5, `theme.md`'s three built-in records and `okabe-ito-copy.toml` (78 properties) list it.
