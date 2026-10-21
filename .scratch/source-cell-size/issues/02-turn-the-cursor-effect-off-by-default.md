# 02: Turn the Cursor Effect off by default

**What to build:** A console with no settings file shows the Cursor and a Region as ADR 0078's still outline and no animated glitch: Glitch amount and Glitch frequency default to zero. At amount zero the effect already builds no Shape (cell-edges 04), so this ticket changes the defaults, the repaint they cause and the tests and docs that assumed the old defaults. A settings file can still turn the effect back on until 06 removes it. See `../spec.md`.

**Blocked by:** 01 — Record rectangular, configurable Cells in an ADR.

**Status:** resolved

- [x] With default settings, one Render Frame paints no Cursor Effect Shapes; the Cursor's and a Region's outline are the edges ADR 0078 draws.
- [x] With default settings, a stopped console requests no timed repaint for the Cursor.
- [x] Tests that turned on reduced motion or zeroed the Glitch settings only to get a still console no longer need to. Tests of the effect itself set their own non-zero amounts.
- [x] The settings documentation and `theme.md` show the new defaults.

## Comments

- `CursorEffectSettings::default()` is amount 0 and frequency 0. `the_default_is_still_and_schedules_nothing` pins that it schedules no interval; the effect's own tests use `animated()` (60, 55).
- `a_stopped_console_with_default_settings_requests_no_timed_wake` was red before the flip and no longer sets reduced motion.
- The console tests that set reduced motion or a zero amount only to stay still no longer do. The reduced-motion tests themselves keep it, because they test that preference.
- The draw-loop culling test compares the edge Mesh's vertex counts: at default settings no Cursor Effect Shape is left to tell a culled frame from a full one.
- `config.rs` and `theme.md` show the zero defaults.
