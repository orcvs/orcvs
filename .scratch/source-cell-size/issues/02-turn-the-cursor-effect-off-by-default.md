# 02: Turn the Cursor Effect off by default

**What to build:** A console with no settings file shows the Cursor and a Region as a plain, still outline, in the Theme's `cursor.border` and `region.border` at their widths. It no longer shows the animated glitch. Glitch amount and Glitch frequency default to zero. A stopped console with the Cursor on screen schedules no timed repaint. A settings file can still turn the effect back on until 06 removes it. See `../spec.md`.

**Blocked by:** 01 — Record rectangular, configurable Cells in an ADR.

**Status:** ready-for-agent

- [ ] With default settings, one Render Frame paints the Cursor's Cell as a single still stroke in `cursor.border` at `cursor.border.width`, and no living-area Shapes.
- [ ] With default settings, a Region larger than one Cell is stroked once around the whole Region in `region.border` at `region.border.width`.
- [ ] The Cursor is never drawn with two borders: where the outline is drawn, it replaces the Paint's own Cursor-Cell border, and `selection.border` is not the visible Cursor colour.
- [ ] A Theme width of zero for the Cursor or Region border draws no outline.
- [ ] With default settings, a stopped console requests no timed repaint for the Cursor.
- [ ] Tests that turned on reduced motion or zeroed the Glitch settings only to get a still console no longer need to. Tests of the effect itself set their own non-zero amounts.
- [ ] The settings documentation and `theme.md` show the new defaults.
