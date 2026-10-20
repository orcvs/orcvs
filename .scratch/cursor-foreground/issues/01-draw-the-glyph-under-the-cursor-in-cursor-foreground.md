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

**Status:** ready-for-agent

- [ ] `cursor.foreground` is in the Theme property catalogue as an optional colour: the decoder accepts a colour or `"none"`, and a custom Theme that omits it inherits its parent's value.
- [ ] All four built-in Themes define it explicitly as `"none"`.
- [ ] The catalogue's optional-value group, and the type that carries a `"none"`-or-colour value, are named for colours rather than fills, and every existing test still passes unchanged in behaviour.
- [ ] Paint tests show: a supplied `cursor.foreground` draws the glyph on the single-Cell Cursor over a Function, an Invalid operand and an Output Portal Cell; `"none"` leaves each of those at its role colour; a transparent supplied colour hides the glyph; no other Cell's glyph changes; the Cursor's fill and frame are unchanged.
- [ ] A custom Theme document setting `cursor.foreground` decodes, resolves and paints it (a resolve-and-paint test, not only a decoder test).
- [ ] The shipped-Theme contrast gate measures the Cursor state with the glyph colour actually drawn, so a custom Theme whose `cursor.foreground` fails against its Cursor fill raises the usual contrast notice. A test shows a failing pairing is reported and a passing one is not.
- [ ] The theming schema lists the property with the optional colours and states the rule in its Cursor/Region composition step, and the console's Theme record lists it for every built-in, with the record tests passing.
- [ ] `cargo fmt --all -- --check`, `cargo clippy --package console --all-targets --locked -- -D warnings`, `cargo nextest run --package console --locked` and the `--no-default-features` arm pass.
