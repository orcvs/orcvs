# 13 — Split Console::ui into panel modules

**What to build:** The console's frame function composes named units instead of doing all of them inline in one ~420-line function: focus, the File/View/Help menus and `FileCommand` dispatch, the discard-confirmation modal, the mode control, the Notices menu, window title and close handling, the bottom panel with BPM and MIDI, the Source view, repaint scheduling, and diagnostics. Input translation (`translate_event`), the Source view (`SourceView`), the glyph table (`glyph_scale`) and shape building live in their own modules. The two `too_many_arguments` suppressions (`console.rs`, `cursor_effects.rs`) are reviewed against the resulting responsibilities; cohesive parameter types are used where they express an actual shared concept. 04 gates this only to avoid conflicting edits in the same code.

**Blocked by:** 03 — Prove the panel's Playback wake-up survives an Open; 04 — Make the browser build's MIDI Output agree with its reporting path; 12 — Move console.rs inline tests into sibling test modules; 14 — Drive the panel layout test through Console::ui; 26 — Share console test setup and stop polling with sleeps.

**Status:** ready-for-agent

- [ ] File workflows, input routing, Source presentation and repaint scheduling have explicit owners and interfaces. `Console::ui` composes those operations; extraction follows responsibilities rather than an arbitrary line limit.
- [ ] Review `show_source_scene`, `show_source`, `geometry` and both argument-count suppressions. Group parameters only when they share a domain concept; justify any retained narrow suppression instead of hiding unrelated arguments in a bag.
- [ ] Preserve focus routing, file confirmation behavior, menu ordering and repaint timing with the existing behavioral tests.
- [ ] The frame no longer clones the presented Theme or the MIDI destination list; it borrows them.
- [ ] kittest and paint tests pass unchanged.
- [ ] The paint benchmark shows no regression beyond noise.

## Comments

**2026-09-25 — audited against `origin/main` `199c3331`.** `Console::ui` has grown from ~370 to ~420 lines with the menu-bar restructure (f00cee01), File New/Open/Save, the mode control (6c8f054a) and Notices; those units are now named. 03's code change landed, so it no longer blocks this ticket.

**2026-09-25 — acceptance-criteria review against `199c3331`.** Replaced line-count and suppression quotas with responsibility-based boundaries and observable behavior preservation.
