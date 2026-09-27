# 11 — Decide the fate of the colour-blindness simulation

**What to build:** A decision, then its implementation: either the colour-vision simulation and distinguishability checks become part of Theme contrast validation, or they leave shipped code. Today about 490 lines of `console/src/contrast.rs` ship uncalled, kept compiling by fourteen dead-code expectations, and the Theme by-key accessors are kept the same way.

**Blocked by:** None — can start immediately.

**Status:** resolved

- [x] The decision and its reason are recorded in this ticket.
- [ ] ~~If kept: Theme validation calls the simulation and reports indistinguishable roles as a notice, with tests.~~ Not applicable: the simulation was removed from production.
- [x] If removed: the simulation lives only under test or in a separate tool, and no dead-code expectation remains in the contrast module. Exception: the one on the `ContrastReport::scope` field, which belongs to `validate`'s report rather than the simulation (see below).
- [x] The Theme by-key accessors get the same treatment.

## Comments

**2026-09-25 — audited against `origin/main` `199c3331`.** Still valid; `contrast.rs` is unchanged since the baseline. Context for the decision: since 8a5f9ef6 the web build loads no Theme documents, so Theme resolution and `ThemeDocument` are native-only — validation added here would run only natively.

**2026-09-27 — decided and resolved.** Decision: keep the colour-vision simulation as a test-only check on the built-in Themes, and remove it from production. Reason: no shipped path calls `contrast::distinguish`; its only consumer is `shipped_theme_colour_vision_gate` over the built-ins. Applying it to user Themes would run natively only, since the web build loads no Theme documents, and would turn the ungated-tritanopia judgement made for the built-ins' palette into a policy applied to every user Theme.

Implementation: the simulation (`CONFUSION_FLOOR`, `ColourVision`, `GlyphChannel`, `ConfusionResult`, `ConfusionReport`, `CONFUSION_SCOPE`, `distinguish`, `simulate`, `lab`, `difference` and `Role::glyph_channel`) and its twelve tests moved to `console/src/contrast/colour_vision.rs`, declared `#[cfg(test)] mod colour_vision;` in `contrast.rs`. Its items are private to that module. Thirteen of the module's fourteen dead-code expectations went with it. The one left, on `ContrastReport`'s `scope` field that only tests read, belongs to `validate`'s report rather than the simulation, and is narrowed to that field; the report carries its scope in the data by `.scratch/theming/issues/08`'s decision, so removing the field is a design change outside this ticket; the one wasm-target expectation on `mod contrast` in `lib.rs` is unchanged, because it covers native-only Theme loading. The by-key readers `Theme::color`, `color_ref`, `grid_width` and `chrome_width` had only test callers and are now a `#[cfg(test)] impl Theme`. The writers `color_mut`, `set_grid_width` and `set_chrome_width` are called by `theme::resolve` and stay shipped, under the same wasm-target expectation `resolve` carries.
