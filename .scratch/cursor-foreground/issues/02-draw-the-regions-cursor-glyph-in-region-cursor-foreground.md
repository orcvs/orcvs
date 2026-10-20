# 02 — Draw the Region's Cursor glyph in `region.cursor.foreground`

**What to build:** While a Region spans more than one Cell, a Theme can colour the glyph on the Cursor's own Cell separately from the single-Cell Cursor's. A Theme document sets `region.cursor.foreground`, and the glyph on the Region's Cursor Cell draws in it.

It falls back the way the Region's Cursor fill does. `region.cursor.background` falls back to `cursor.background` when it is `"none"`, and `region.cursor.foreground` does the same:

1. `region.cursor.foreground` when it holds a colour, transparent included;
2. otherwise `cursor.foreground` when that holds a colour;
3. otherwise the glyph's role colour.

The other Cells of the Region keep their role colours. The fill and the lasso are unchanged.

All four built-in Themes set it to `"none"`, so nothing a built-in paints changes.

**Blocked by:** 01 — Draw the glyph under the Cursor in `cursor.foreground`

**Status:** ready-for-agent

- [ ] `region.cursor.foreground` is in the Theme property catalogue as an optional colour, decoded and inherited exactly as `01`'s `cursor.foreground` is, and all four built-ins define it as `"none"`.
- [ ] The fallback has one statement, beside the Region's Cursor fill fallback, which both painting and the contrast check read.
- [ ] Paint tests show each step of the fallback on the Region's Cursor Cell: its own colour; `cursor.foreground` when it is `"none"`; the role colour when both are `"none"`; and an explicit transparent value suppressing the fallback. They also show that the other Region Cells keep their role colours.
- [ ] The contrast gate measures the Region's Cursor state with the resolved glyph colour, and a test shows a failing pairing is reported.
- [ ] The theming schema and the console's Theme record list the property for every built-in, and the record tests pass.
- [ ] `cargo fmt --all -- --check`, `cargo clippy --package console --all-targets --locked -- -D warnings`, `cargo nextest run --package console --locked` and the `--no-default-features` arm pass.
