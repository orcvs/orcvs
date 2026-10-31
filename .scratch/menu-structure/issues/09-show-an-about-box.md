# 09 — Show an About box

**What to build:** Help ends with About, which opens a modal over the console showing the Orcvs
logo, the name and the version, as the splash does (ADR 0083). Native and the web show the same
box from the same build.

**Blocked by:** None (can start immediately).

**Status:** resolved

- [x] Help holds About after the site links, below a separator.
- [x] About opens an `egui::Modal` over the whole console holding the splash's logo, `O R C V S`
      and `v` with the crate version, tinted for the mode as the splash tints them.
- [x] Escape and a click outside close it.
- [x] It holds the keys while it shows, as the discard confirmation does.
- [x] A kittest drives the shipped `Console` through Help → About and asserts the version shows,
      that a character or an arrow reaches neither the Source nor the Cursor, and that Escape
      closes it.
- [x] Scoped gates for `console` pass, and the egui skill's guidance is followed.

## Comments

- The logo is loaded from `assets/logo-splash.png` when About opens and dropped when it closes,
  so the box holds no texture while closed. `console::logo` loads it for both the splash and the
  box.
- eframe's PNG helper (`eframe::icon_data`) is native-only, so the web could not decode the logo
  through it. `console` now names `image` 0.25 with `png` alone, the crate and feature eframe
  already depends on for both targets: the lockfile gains an edge, not a crate.
- No ADR: the box reuses the splash's logo and the discard confirmation's modal, and decides
  nothing hard to reverse.
