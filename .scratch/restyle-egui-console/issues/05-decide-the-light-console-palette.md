# 05 — Decide the light console palette

**What to build:** Decide the twenty-two light-theme token values, record them in the console theme
documentation, and expose the theme switch once they are decided.

**Blocked by:** 04 — Port the console theme mechanism.

**Status:** ready-for-human

- [ ] Twenty-two light tokens are named at exact values, filling the same roles `02` names
      for dark: page, source, grid line, sector line, ordinary, function, bang and error, number,
      note, marker, highlight, the four Cursor bloom fill and line pairs, selection fill, the
      selection stroke while the caret is hidden, and the selection and Cursor stroke.
- [ ] `console/src/theme.md` records both palettes, each marked by theme, so a later change to either
      is a documented change rather than drift.
- [ ] The Glyph colour distinctions `02` requires of dark hold for light: Numbers use a colour distinct
      from Functions, Notes and ordinary Characters, and Bang shares the diagnostic colour.
- [ ] `02`'s near-black Cell background rule is restated for light rather than inherited. It is a
      dark-theme rule and does not transfer.
- [ ] Contrast between each Glyph colour and the Cell background it sits on is stated as a measured
      ratio, not asserted by eye.
- [ ] Light captures, wide and tall, are produced and reviewed the way `03` reviews the dark ones.
- [ ] The View menu gains `ui.label("Theme")` and `egui::widgets::global_theme_preference_buttons(ui)`
      beneath the existing Diagnostics checkbox.
- [ ] `console-testing/03` is extended to pin the light values once they are decided.

## Comments

**A proposal exists and is not a decision.** `feat/egui-theming` carries a complete, hand-tuned
`LIGHT_PALETTE`: page `#EFF4F2`; source `#FAFCFB`; grid line `rgba(52, 91, 80, 64)`; sector line
`rgba(38, 104, 84, 112)`; ordinary `#303F3B`; function `#087A5A`; bang and error `#C33445`; number
`#3564A0`; note `#7553A2`; marker `rgba(53, 98, 84, 112)`; highlight `#AED8CB`; bloom core fill
`#CFEDE4` and line `rgba(20, 130, 96, 160)`; inner fill `#DAF1EA` and line `rgba(31, 137, 105, 130)`;
middle fill `#E4F5F0` and line `rgba(43, 142, 113, 105)`; outer fill `#EFF8F5` and line
`rgba(55, 146, 121, 82)`; selection fill `#CCEBE2`; selection stroke while the caret is hidden
`#187E60`; selection and Cursor stroke `#076247`.

Start from those. They are considered rather than arbitrary. But they were written in August on a
branch that never landed, no issue decides them, `theme.md` does not record them, and no capture has
ever been reviewed against them — which is exactly the gap `02`'s own correction comment was written
to close for the dark palette.

**Untagged, deliberately.** The release Gate is `v1-release/01`, whose dependency closure reaches
`restyle-egui-console/03`, `02` and `01`. Tagging this `release/v1` would put a light theme on the
release's critical path. `03`'s scope stays dark-only; light gets its own captures here.
