# 04 — Decide the light built-in Theme

**What to build:** A light built-in Theme, with every named property at an exact value, recorded in the console theme documentation. The theme switch is exposed once it exists.

**Blocked by:** 03 — Derive the console's chrome from the Theme; 01 — Decide where Source colour authority lives; 08 — Validate a Theme’s composited contrast.

**Status:** ready-for-human

**Tags:** release/v1

- [x] Every named property in `../schema.md` has an exact value, and the Theme declares itself light.
- [x] The exact-value record includes the border-width keys added under ADR 0053. Shared values with the dark built-in are recorded explicitly rather than left as implicit toolkit defaults.
- [x] `console/src/theme.md` records it beside the Okabe–Ito Theme, so a later change to either is a documented change rather than drift.
- [x] `restyle-egui-console/02`'s near-black Cell background rule is restated for light rather than inherited. It is a rule about the dark Theme and does not transfer.
- [x] It passes `08`'s validator, or its report names each failure as a deliberate exception.
- [x] Prepare the existing light-palette proposal as a complete Theme, with its contrast report and wide and tall visual captures. The user reviews and accepts the concrete result before switching is exposed; agreement to prepare it is not approval of its colours.
- [ ] After acceptance of the complete light definition, replace `02`/`03`'s duplicate registration with `style()` of the selected dark Theme for `egui::Theme::Dark` and of the selected light Theme for `egui::Theme::Light`, through `set_style_of`. Source resolves the same effective Theme; preserve stored preferences and do not force `set_theme`.
- [ ] Switching acceptance tests cover picker changes, restored dark/light references, OS mode changes and loaded-Theme selections once loading is available. Every presented frame uses the same resolved Theme for Source and chrome. This issue owns these distinct-theme checks, not `03`.
- [ ] The View menu gains the mode (follow the OS, Dark, Light), beside the dark and light Theme pickers prepared in `03` over `06`'s foundation. Expose these controls together only after acceptance of the light Theme and coherent Source/chrome switching.
- [x] `console-testing/03` is extended to pin the light Theme's values.

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

**2026-09-23 — prepared, awaiting the user's decision on the colours.**

`orcvs-light` now exists as a complete built-in beside `okabe_ito()`
(`console/src/theme.rs::orcvs_light`). Every named property in `../schema.md`
has an exact value, the widths ADR 0053 added included, and the values it
shares with the dark built-in are written out rather than inherited.
`console/src/theme.md` records all of them with a Provenance column saying,
for each, whether it is the `feat/egui-theming` proposal's, an adjustment of
it, or decided here.

Three of the proposal's tokens have no named key to land on: `marker` and
`highlight` name Glyphs `retired-glyph-vocabulary` removed, and the four
`bloom_*` fill/line pairs became the Cursor Effect's single `cursor.area`
field, which takes `bloom_core_line`'s colour. The proposal is silent on
Comment, Sequence, Diagnostic and Output Portal; those four are decided here
and flagged as such.

`08`'s validator measures all 84 reachable painted states. **Every one clears
the 4.5:1 floor and there are no deliberate exceptions** —
`contrast::accepted_failures("orcvs-light")` is empty, and
`contrast::tests::orcvs_light_has_nothing_to_except` pins that it is empty
because nothing fails. Reaching that took three darkenings, each recorded in
`theme.md`: Function `#087A5A` → `#077055`, Bang `#C33445` → `#AD2A3B`, and
Output Portal `#8A5D00` → `#7A5200`. No floor was lowered.

Captures for review are in `../evidence/`: `light-wide.png` (1200 × 700),
`light-tall.png` (700 × 1200) and `light-chrome.png` (the Settings menu open).
That directory's README records the procedure and says what it was patched to
produce them — nothing shipped selects this Theme, and `style::install` still
registers one Okabe–Ito style in both egui appearance slots.

The three unticked boxes above are exactly the ones this issue makes
conditional on the user accepting these colours: `set_style_of` per
appearance, the switching acceptance tests, and the View menu's mode and
pickers. None of them is started.

**2026-09-23 — the glyph hues are re-picked from Okabe–Ito.**

The user's decision, after a review of the definition above: the light
built-in's Source glyph hues come from the same Okabe–Ito palette and the same
role-to-hue assignment the dark built-in uses, and the `feat/egui-theming`
proposal's glyph hues are dropped. The chrome keys are untouched — the proposal
remains their provenance.

The review's finding was that the proposal put Function `#087A5A` and Bang
`#C33445` at near-identical relative luminance, the canonical deuteranope
confusion pair. Measuring it, rather than assuming it, says two things:

- The luminance observation is exact — the two measure 1.01:1 against each
  other — and the conclusion drawn from it is not. Every dichromacy keeps
  lightness *and* one chromatic axis, so those two colours still measure 14.39
  apart under simulated deuteranopia. Equal luminance is a fact about two
  colours, not a verdict on them.
- The definition failed anyway, worse and elsewhere. Its Diagnostic `#A34A00`
  and Output Portal `#7A5200` measure **0.70** apart under protanopia and its
  Number `#3564A0` and Note `#7553A2` **2.16** under deuteranopia, against a
  floor of 5.0. Those are pairs a red–green colour-blind reader reads as one
  colour, and every one of them cleared `08`'s contrast floor.

So the re-pick is right, and the reason to make it is stronger than the one
given. The new values, their OKLCh hue and lightness, the state that set each
lightness, the full contrast table and the colour-vision numbers are in
`console/src/theme.md`. In summary: Number `#006D9B`, Note `#706900`, Function
`#007555`, Bang `#90426F`, Sequence `#0072B2` *unchanged*, Diagnostic
`#652800`, Output Portal `#6F4A00`. Every OKLCh hue angle is held to within
0.7° of Okabe–Ito's. Five moved only as far as a near-white ground required;
Sequence did not move at all; Diagnostic and Output Portal moved further,
because the contrast floor is a lightness *ceiling* on a light ground and it
compresses Okabe–Ito's yellow, orange and vermillion — which a red–green
dichromacy merges into one hue — into one lightness.

`error` and `warning` follow Bang to `#90426F`, as they already did.

**The colour-vision check ships**, in `console/src/contrast.rs` beside the
contrast floor, rather than staying a recorded one-off: the defect it found was
invisible to every existing gate, and a recorded measurement would not have
stopped the next retune reintroducing it. `contrast::distinguish` simulates
protanopia, deuteranopia and tritanopia (Viénot, Brettel & Mollon 1999) over
the same composited colours `validate` measures, and reports CIEDE2000 between
every pair of Source glyph channels. `contrast::CONFUSION_FLOOR` is 5.0 and
gates the two red–green dichromacies; both built-ins clear it with no
exception — Okabe–Ito at 6.65, Orcvs Light at 7.19. That module's `# Scope`
section no longer excludes colour vision, and says what the new function does
and does not cover.

**Tritanopia is measured and deliberately not gated**, which is this issue's
one named exception. The Okabe–Ito assignment is published as safe for
red–green deficiency and makes no tritan claim: the *shipped dark* built-in's
Bang and Diagnostic measure 0.60 apart under it. A tritan gate would fail the
Theme the console ships today, and no arrangement of these seven hues inside a
light ground's lightness ceiling rescues it either. Orcvs Light measures 1.54,
better than the dark built-in and better than the definition it replaces (0.39),
and not good. `shipped_theme_colour_vision_gate` pins both numbers so it stays
a checked boundary.

The captures are redone, at `pixels_per_point: 2` and over a Source that now
reaches Atom Invalid, Sequence Invalid and a visible Output Portal Reservation
including the doubled Portal-over-role tint — the states nearest the contrast
floor, which the first set could not show. `../evidence/README.md` records the
layout, why each line is there, and the procedure, and states that both
temporary patches were reverted before the commit. It also carries one
simulated copy of the wide capture per dichromacy.

Still nothing shipped selects this Theme.
