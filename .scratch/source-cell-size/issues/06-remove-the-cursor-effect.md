# 06: Remove the Cursor Effect

**What to build:** Delete the Cursor Effect outright, leaving the still outline 02 made the default. This removes the effect's module and settings type, the `[cursor_effects]` settings section, the `cursor.area` Theme key (from the built-in Themes, the catalogue, the Theme documentation and the Okabe-Ito example Theme the tests load), the reduced-motion preference and its startup probe, the effect's repaint term and its two benchmark cases. A settings file with the old Glitch keys loads with an unknown-key notice. A custom Theme that still sets `cursor.area` is refused with the existing notice, because Themes reject unknown keys. See `../spec.md`.

**Blocked by:** 02 — Turn the Cursor Effect off by default.

**Status:** resolved

- [x] No shipped code samples, animates or schedules repaints for the Cursor.
- [x] The Cursor and Region outlines are still ADR 0078's selection outline, in `cursor.border` and `region.border` at their widths.
- [x] ADR 0053, the egui skill's reduced-motion and animating-cursor notes, and the theming schema and spec no longer describe the effect.

## Comments

- Toby asked on 2026-10-21 for the remaining work to run in parallel with the stack's performance review, so this was taken on from triage then.
- Removed: `cursor_effects.rs`, the `[cursor_effects]` settings section, the `cursor.area` Theme key, the reduced-motion probes and the `MediaQueryList` web-sys feature only the probe used, the effect's repaint term (`until_next` now takes the Run Clock alone), `Paint::edges_beneath_outline` and `outline_line`, and the two `cursor effects/*` benchmarks.
- `the_removed_glitch_keys_are_unknown` checks an old settings file loads with two unknown-key notices; `an_unknown_or_wrong_case_style_property_is_refused` now includes `cursor.area`.
- `a_click_still_pans_to_follow_the_cursor_with_playback_stopped` clicks Column 14 rather than 10: at the 12 point Cell, Column 10 sat inside the 200 point console, so the test no longer exercised the follow.
