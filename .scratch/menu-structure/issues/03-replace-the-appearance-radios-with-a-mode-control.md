# 03 — Replace the appearance radios with a mode control

**What to build:** A three-state icon control — 💻 Follow the OS, 🌙 Dark, ☀ Light — right-aligned in the top bar, replacing the Appearance radios in View.

**Blocked by:** 02 — Restructure the menu bar.

**Status:** ready-for-agent

**Tags:** release/v1

- [ ] Three icon-only selectable buttons, each with hover text; Follow the OS's hover names the current system appearance, as egui's `ThemePreference::radio_buttons` does.
- [ ] A choice goes through the existing `AppearanceChange::Mode` path, applied after the frame.
- [ ] The mode persists as egui's `ThemePreference` already does; no config key is added.
- [ ] The glyphs render under the console's fonts on native and web; if one does not, the control uses a glyph that does rather than falling back to text.
- [ ] View no longer holds any appearance control.
- [ ] Tests cover each of the three choices and that a restored preference is the one shown selected.
- [ ] Scoped gates for `console` pass, and the egui skill's guidance is followed.
