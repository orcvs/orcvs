# 01 — Draw the glyph under the Cursor in `cursor.foreground`

**What to build:** A Theme can set the colour of the glyph under the Cursor. A Theme document sets `cursor.foreground`, and the glyph on the single-Cell Cursor draws in that colour, whatever its role or overlay. Today a Theme can fill the Cursor's Cell (`cursor.background`) but cannot change the glyph on it, so a solid Cursor fill can swallow the glyph standing on it. Terminals and editors pair a cursor text colour with a block cursor fill for this reason.

`cursor.foreground` is optional, on the same terms as `cursor.background`:

- The string `"none"` clears it, and the glyph keeps its role's colour: the colour it shows today, Diagnostic and Portal composition included.
- A supplied colour, transparent included, replaces the glyph colour on the Cursor's Cell. A transparent supplied colour hides the glyph; it does not fall back.
- A document that omits it inherits its parent's value.

It changes only the glyph. The Cursor's fill and frame, and every other Cell, are unchanged. An empty Cursor Cell draws no glyph either way.

The catalogue groups the optional values as fills. Once a glyph colour joins them that name is wrong, so the group becomes the optional colours, with no change in behaviour.

All four built-in Themes set `cursor.foreground` to `"none"`, so nothing a built-in paints changes.

**Blocked by:** None — can start immediately.

**Status:** resolved

- [x] `cursor.foreground` is in the Theme property catalogue as an optional colour: the decoder accepts a colour or `"none"`, and a custom Theme that omits it inherits its parent's value.
- [x] All four built-in Themes define it explicitly as `"none"`.
- [x] The catalogue's optional-value group, and the type that carries a `"none"`-or-colour value, are named for colours rather than fills, and every existing test still passes unchanged in behaviour.
- [x] Paint tests show: a supplied `cursor.foreground` draws the glyph on the single-Cell Cursor over a Function, an Invalid operand and an Output Portal Cell; `"none"` leaves each of those at its role colour; a transparent supplied colour hides the glyph; no other Cell's glyph changes; the Cursor's fill and frame are unchanged.
- [x] A custom Theme document setting `cursor.foreground` decodes, resolves and paints it (a resolve-and-paint test, not only a decoder test).
- [x] The shipped-Theme contrast gate measures the Cursor state with the glyph colour actually drawn, so a custom Theme whose `cursor.foreground` fails against its Cursor fill raises the usual contrast notice. A test shows a failing pairing is reported and a passing one is not.
- [x] The theming schema lists the property with the optional colours and states the rule in its Cursor/Region composition step, and the console's Theme record lists it for every built-in, with the record tests passing.
- [x] `cargo fmt --all -- --check`, `cargo clippy --package console --all-targets --locked -- -D warnings`, `cargo nextest run --package console --locked` and the `--no-default-features` arm pass.

## Comments

**2026-10-20 — landed.** `cursor.foreground` is in the style catalogue's optional group, so the decoder takes a colour or `"none"`, `resolve` inherits it when omitted, and the four built-ins set it to `none`. The group and its type are named for colours: `OptionalFill` is `OptionalColor`, the decoder's newtype `OptionalColorValue` and its list `OPTIONAL_COLORS`, whose `"none"` refusal now names all three properties. `resolve` reads each optional colour through one rule. `style::cell_visuals_with_cursor_colour` draws the selected Cell's glyph in `cursor.foreground` when it holds a colour, and leaves the fill, the frame and every unselected Cell alone; `contrast::painted` calls the same function, so the Cursor state measures the drawn glyph. Tests: `paint::tests::cursor_foreground_draws_the_glyph_on_the_single_cell_cursor` (a Function, an Invalid operand and an Output Portal answer written from Source; supplied, transparent and cleared; fill, frame and every other Cell unchanged), `paint::tests::a_theme_document_paints_the_cursor_foreground_it_sets` (decode, resolve, paint), `style::tests::cursor_foreground_draws_the_selected_glyph_over_every_fact_and_portal` (Locked included), `contrast::tests::the_cursor_state_measures_the_cursor_foreground_against_the_cursor_fill`; the resolve, decoder, example and record tests cover the new property. Removing the `selected` guard fails all four new tests. `schema.md`'s catalogue and composition step 5, `theme.md`'s three built-in records and `okabe-ito-copy.toml` (77 properties) list it.
