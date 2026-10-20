# 06: Remove the Cursor Effect

**What to build:** Delete the Cursor Effect outright, leaving the still outline 02 made the default. This removes the effect's module and settings type, the `[cursor_effects]` settings section, the `cursor.area` Theme key (from the built-in Themes, the catalogue, the Theme documentation and the Okabe-Ito example Theme the tests load), the reduced-motion preference and its startup probe, the effect's repaint term and its two benchmark cases. A settings file with the old Glitch keys loads with an unknown-key notice. A custom Theme that still sets `cursor.area` is refused with the existing notice, because Themes reject unknown keys. See `../spec.md`.

**Blocked by:** 02 — Turn the Cursor Effect off by default.

**Status:** ready-for-agent

- [ ] No shipped code samples, animates or schedules repaints for the Cursor.
- [ ] The Cursor and Region outlines still use `cursor.border` and `region.border` at their widths, and the Paint's own Cursor-Cell border is not shown in their place.
- [ ] ADR 0053, the egui skill's reduced-motion and animating-cursor notes, and the theming schema and spec no longer describe the effect.
