# 02 — Type BPM on the Panel

**What to build:** BPM is set from a typed field on the Panel, default 120. The Tempo menu is gone. While the field is focused, the Panel owns keys — digits do not write Source, Space does not toggle Playback.

**Blocked by:** 01

**Status:** resolved

- [x] Default BPM is 120.
- [x] The Panel has a typed BPM field, range 1..=999, no drag, no `<` `>` steppers.
- [x] Readout order is BPM · Tick · Run Clock (destination and Refresh arrive in 03).
- [x] A value commits on Enter and on focus loss. Invalid or empty input snaps back to the last accepted BPM.
- [x] Changing BPM retunes the current run in place when Playback is requested, and otherwise applies to the next run.
- [x] While the BPM field is focused, digits do not write the Source Cell under the Cursor, and Space does not toggle Playback. Enter, Escape, or a click on the Source Grid returns keys to the console.
- [x] The Tempo menu is removed. File and View stay.
- [x] BPM is not persisted. A new launch is 120 again.

## Answer

`Opts::new` now builds `Bpm::new(120)` (125ms Tick period). `Bpm` itself still admits 1..=15000; `Bpm::new(20)` constructor tests stay. Persistence is unchanged: `Console::save` stores Source only, so a new launch is 120 because the default changed.

`BpmEdit` replaces `TempoEdit`. It holds the TextEdit buffer and the last accepted `Bpm`. Commit accepts 1..=999; empty, non-numeric, 0, and >999 snap the buffer back. Enter and any non-Escape focus loss commit (and `Orcvs::set_bpm` when the value changed). Escape always reverts, even if the uncommitted text would have been valid. `set_bpm` is the existing retune-or-next-run path; the Console does not reimplement it.

Focus is last frame's: `event_handler` runs before widgets, so `Console` stores whether the BPM field has focus and skips this frame's events while it does. Digits stay in the field; Space does not toggle Playback. Escape defocuses (egui Memory) and reverts. A click on the Source Grid steals focus with no extra case. The Tempo menu is gone; File, View, and MIDI stay. Readout order is BPM · Tick · Run Clock.

Files: `orcvs/src/opts.rs`, `orcvs/src/app.rs`, `console/src/console.rs`.
