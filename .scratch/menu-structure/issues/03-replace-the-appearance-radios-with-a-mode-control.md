# 03 — Replace the appearance radios with a mode control

**What to build:** A three-state icon control — 💻 Follow the OS, 🌙 Dark, ☀ Light — right-aligned in the top bar, replacing the Appearance radios in View.

**Blocked by:** 02 — Restructure the menu bar.

**Status:** resolved

**Tags:** release/v1

- [x] Three icon-only selectable buttons, each with hover text; Follow the OS's hover names the current system appearance, as egui's `ThemePreference::radio_buttons` does.
- [x] A choice goes through the existing `AppearanceChange::Mode` path, applied after the frame.
- [x] The mode persists as egui's `ThemePreference` already does; no config key is added.
- [x] The glyphs render under the console's fonts on native and web; if one does not, the control uses a glyph that does rather than falling back to text.
- [x] View no longer holds any appearance control.
- [x] Tests cover each of the three choices and that a restored preference is the one shown selected.
- [x] Scoped gates for `console` pass, and the egui skill's guidance is followed.

## Comments

- 💻, 🌙 and ☀ are not in the console's only font. `egui` and `eframe` are built without
  `default_fonts`, so `Console::new` installs MonaspaceNeon alone and a missing glyph draws as its
  replacement glyph. The control uses ◐ (Follow the OS), ☾ (Dark) and ☼ (Light), which the font
  holds; the same bytes are embedded on native and web.
  `kittest_tests::the_mode_glyphs_are_in_the_console_font` compares each glyph's atlas rectangle
  with the replacement glyph's, because `Fonts::has_glyph` answers false for every character when
  a family has one face.
- Each button's accessible name is Follow the OS, Dark or Light (a second `widget_info` over the
  glyph), so a screen reader and the tests find it by meaning rather than by glyph.
- Hover text: "Follow the operating system's appearance." plus "The operating system's appearance is
  dark/light/unknown."; "Always use the dark appearance."; "Always use the light appearance."
