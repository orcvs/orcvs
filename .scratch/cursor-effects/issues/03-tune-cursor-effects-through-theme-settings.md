# 03 — Tune Cursor effects through Theme settings

**What to build:** Expose a small Theme → Cursor effects section with four controls: Cursor colour, Area colour, Glitch amount, and Glitch frequency. Changes appear immediately in the Source Grid, can be reset to theme defaults, and survive restart when persistence is enabled.

**Blocked by:** 02 — Replace the checker bloom with a living green area.

**Status:** resolved

- [x] Cursor colour changes the frame, its grain, and its fragments together. Its default is the prototype's off-white, RGB 234, 235, 229. Evidence: `CursorEffectSettings::cursor_colour` feeds `cursor_effect_shapes` frame paths; `defaults_are_the_approved_colours_and_an_irregular_live_cadence`.
- [x] Area colour changes the surrounding field, grain, and streaks together. Its default is the grid-green bloom hue, RGB 76, 190, 156. Opacity remains internally derived rather than exposed as another control. Evidence: `area_colour` in area builders; default RGB 76, 190, 156 in settings test.
- [x] One Glitch amount control adjusts fragmentation, displacement, and decorative noise density across both layers. Its default reproduces the agreed effect; zero removes decorative noise and displacement while retaining a clear Cursor. Every value preserves the selected Cell and Glyph readability. Evidence: `amount` scales displacement and fragment density in `cursor_effect_shapes`; reduced-motion path zeroes amount.
- [x] One Glitch frequency control adjusts the temporal cadence of both layers while retaining their relative pacing and irregular timing. Zero freezes temporal evolution and stops animation-driven repaints; moving the Cursor still moves the effect immediately. Setting a nonzero value resumes evolution without a catch-up burst. Evidence: `zero_frequency_freezes_the_sample_and_requests_no_repaint`; `animation_changes_only_at_deadlines_and_does_not_replay_a_backlog`.
- [x] Controls preview immediately in the actual Source Grid. Reset to theme defaults restores all four values together. Both zero-frequency and zero-amount states are understandable without extra toggles. Evidence: Theme → Cursor effects UI in `console.rs` (`amount`, `frequency`, colour pickers, reset to `CursorEffectSettings::default()`).
- [x] Keep the settings to those four controls and reset. Radius, grain scale, seed, per-edge settings, separate layer speeds, and detailed noise parameters stay internal. Colour controls use the existing colour-picker conventions. Evidence: `CursorEffectSettings` exposes only the four values plus optional internal `cell_colour`; Theme UI matches existing colour-picker widgets.
- [x] Add the small Theme settings section needed for this feature if it does not yet exist. One console-owned settings value supplies rendering and the settings UI; avoid a duplicate preference owner. Integrate with the theme mechanism available at implementation time without requiring a broader light/system-theme redesign. Evidence: `ConsoleApp::cursor_effects` owns settings; `theme.md` documents the section.
- [x] With persistence enabled, all four preferences survive native and browser restart. Missing or invalid saved values fall back to valid defaults, and older saved Source data remains usable. Without persistence, defaults work and edits apply for the current run. Evidence: `persistence.rs` `CURSOR_EFFECTS_KEY`, encode/decode round trip, `absent_or_malformed_cursor_effect_settings_use_theme_defaults`.
- [x] Theme resolution and restoring preferences do not silently overwrite user effect settings. Document how colour defaults follow the active theme and how explicit overrides and reset behave. Reduced-motion behaviour remains respected. Evidence: `theme.md`; `respecting_reduced_motion` applied at render time in `console.rs`.
- [x] Focused verification covers live edits, reset, frequency zero and resumption, amount zero, valid-value bounds, persistence round trips, and the feature-off path. Visually inspect extremes against occupied Cells and run the applicable repository checks, recording anything deferred to CI. Evidence: `cursor_effects.rs` and `persistence.rs` tests; native visual check; merge-tier browser suite deferred to CI.

## Design decisions

These are presentation preferences, not language or Playback controls. Glitch frequency must not alter BPM, Tick cadence, MIDI, or evaluation.

The separate console-theme mechanism work is related but is not a blocking edge: this ticket can deliver the four controls with today's palette. It must not duplicate that work or expand into a general theme editor.

## Answer

Theme → Cursor effects exposes opaque Cursor and Area colours plus 0–100 Glitch amount and frequency controls and one reset action. Edits apply immediately. Persistence stores the four settings separately, while missing or malformed values restore defaults and existing Source revisions remain independent.
