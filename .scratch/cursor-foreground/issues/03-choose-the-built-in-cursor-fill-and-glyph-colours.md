# 03 — Choose the built-in Cursor fill and glyph colours

**What to build:** Decide whether each built-in Theme draws a solid Cursor and, where it does, the glyph colour that reads on it. The two built-ins, Grayscale Dark and Grayscale Light, leave `cursor.background` `"none"`, so the single-Cell Cursor is a frame over the role's fill; each sets `cursor.foreground` and `region.cursor.background`. A Cursor glyph colour only earns its place with a fill behind it, so the fill and the glyph colour are chosen together. The values are picked by eye in the theme workbench and confirmed in the console with the tracker example open.

**Blocked by:** 02 — Draw the Region's Cursor glyph in `region.cursor.foreground`

**Status:** ready-for-human

- [ ] For each of the two built-ins, Grayscale Dark and Grayscale Light, a recorded decision: keep the framed Cursor with all four Cursor colours `"none"`, or set `cursor.background` with a `cursor.foreground` that reads on it, and whether the Region's Cursor differs.
- [ ] The console's Theme record and the built-in literals carry the chosen values, and the record tests pass.
- [ ] The shipped-Theme contrast gate passes for every built-in in the Cursor and Region-Cursor states.
- [ ] `cargo fmt --all -- --check`, `cargo clippy --package console --all-targets --locked -- -D warnings`, `cargo nextest run --package console --locked` and the `--no-default-features` arm pass.

**2026-10-25 — narrowed.** `.scratch/theme-independence/issues/07` retired Okabe–Ito, Orcvs Light, My Grayscale Dark and My Grayscale Light, so the decision covers the two grayscale defaults alone.
