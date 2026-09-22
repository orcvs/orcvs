# 04 — Decide the light built-in Theme

**What to build:** A light built-in Theme, with every named property at an exact value, recorded in the console theme documentation. The theme switch is exposed once it exists.

**Blocked by:** 03 — Derive the console's chrome from the Theme; 01 — Decide where Source colour authority lives; 08 — Validate a Theme’s composited contrast.

**Status:** ready-for-human

**Tags:** release/v1

- [ ] Every named property in `../schema.md` has an exact value, and the Theme declares itself light.
- [ ] The exact-value record includes the border-width keys added under ADR 0053. Shared values with the dark built-in are recorded explicitly rather than left as implicit toolkit defaults.
- [ ] `console/src/theme.md` records it beside the Okabe–Ito Theme, so a later change to either is a documented change rather than drift.
- [ ] `restyle-egui-console/02`'s near-black Cell background rule is restated for light rather than inherited. It is a rule about the dark Theme and does not transfer.
- [ ] It passes `08`'s validator, or its report names each failure as a deliberate exception.
- [ ] Prepare the existing light-palette proposal as a complete Theme, with its contrast report and wide and tall visual captures. The user reviews and accepts the concrete result before switching is exposed; agreement to prepare it is not approval of its colours.
- [ ] After acceptance of the complete light definition, replace `02`/`03`'s duplicate registration with `style()` of the selected dark Theme for `egui::Theme::Dark` and of the selected light Theme for `egui::Theme::Light`, through `set_style_of`. Source resolves the same effective Theme; preserve stored preferences and do not force `set_theme`.
- [ ] Switching acceptance tests cover picker changes, restored dark/light references, OS mode changes and loaded-Theme selections once loading is available. Every presented frame uses the same resolved Theme for Source and chrome. This issue owns these distinct-theme checks, not `03`.
- [ ] The View menu gains the mode (follow the OS, Dark, Light), beside the dark and light Theme pickers prepared in `03` over `06`'s foundation. Expose these controls together only after acceptance of the light Theme and coherent Source/chrome switching.
- [ ] `console-testing/03` is extended to pin the light Theme's values.

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
ever been reviewed against them — which is exactly the gap
`restyle-egui-console/02`'s own correction comment was written to close for the dark palette.

**Untagged, deliberately.** The release Gate is `v1-release/01`, whose dependency closure reaches
`restyle-egui-console/03`, `restyle-egui-console/02` and `restyle-egui-console/01`. Tagging
this `release/v1` would put a light theme on the release's critical path.
`restyle-egui-console/03`'s scope stays dark-only; light gets its own captures here.

**2026-09-21 — re-scoped by ADR 0053.**

This was a light *chrome palette* beside a separate Source scheme. Under ADR 0053 there is only one Theme, so this becomes the light built-in Theme: slots and named keys together. The `feat/egui-theming` proposal above is still the starting point for the chrome keys. Map it onto the named keys (`page` → `panel.background`, `grid line` → `grid.border`, `sector line` → `sector.seam`, `selection fill` → `selection.background`, and so on). Its glyph values are a starting proposal for the slots.

**2026-09-21 — in the release.** The "Untagged, deliberately" comment above is stale. The whole theming effort is in `release/v1`, this issue included, and `v1-release/03` depends on it.

**2026-09-21 — review sequence confirmed.** The user confirmed preparing the existing light proposal as a complete Theme with contrast results and visual captures for review before exposing switching. `03` supplies coherent rendering and prepares pickers; this issue owns acceptance and exposing switching.

**2026-09-22 — one Orcvs format confirmed.** The user chose one versioned Orcvs Theme format with named style properties. Base16 is inspiration only; importing/conversion is deferred. This supersedes the earlier sixteen-slot, palette/template and published-scheme requirements. Native startup loading, web file imports, exact inheritance, strict validation and the confirmed appearance controls remain in scope.
