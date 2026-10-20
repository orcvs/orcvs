# 02: Turn the Cursor Effect off by default

**What to build:** A console with no settings file shows the Cursor and a Region as a plain, still outline, in the Theme's `cursor.border` and `region.border` at their widths. It no longer shows the animated glitch. Glitch amount and Glitch frequency default to zero. A stopped console with the Cursor on screen schedules no timed repaint. A settings file can still turn the effect back on until 06 removes it. See `../spec.md`.

**Blocked by:** 01 — Record rectangular, configurable Cells in an ADR.

**Status:** ready-for-agent

- [ ] With default settings, one Render Frame paints the Cursor's outline as a single still stroke and no living-area Shapes.
- [ ] With default settings, a stopped console requests no timed repaint for the Cursor.
- [ ] Tests that turned on reduced motion or zeroed the Glitch settings only to get a still console no longer need to. Tests of the effect itself set their own non-zero amounts.
- [ ] The settings documentation and `theme.md` show the new defaults.
