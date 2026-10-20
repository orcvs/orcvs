# 06 — Bring the Themes and the workbench to single edges

**What to build:** The built-in Themes' Grid, border and seam widths are chosen against single lines, and the theme workbench draws edges the way the console does, so a Theme tuned in the workbench looks the same in the console. See `../spec.md` and ADR 0078.

**Blocked by:** 03 — Draw every border edge once; 04 — The selection outline owns its edges; 05 — Sector Seams lie over their edge.

**Status:** ready-for-human

- [ ] A human decides each built-in Theme's widths, including whether `grid.border.width` defaults move and whether its 0 to 1 point range should widen, looking at the running console at 1× and 2×.
- [ ] The workbench's drawing applies the ranking, width-zero, composite-once and pixel rules, and its specimen is rebuilt.
- [ ] The workbench and console are compared with real screenshots of the same Source and Theme, and match.
- [ ] `theme.md` and the theming schema state the new defaults.
- [ ] The console gate passes.
