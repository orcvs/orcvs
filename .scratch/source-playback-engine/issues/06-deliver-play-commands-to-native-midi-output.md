# 06 — Deliver Play Commands to native MIDI output

**What to build:** Let a desktop user select a native MIDI destination and hear the exact Play Command batches produced during Playback, while keeping device availability and delivery failures outside Source interpretation.

**Blocked by:** 05 — Run Live Editing through the Playback Engine.

**Status:** resolved

- [x] Native desktop builds can enumerate available MIDI output destinations and let the user select one.
- [x] Each submitted Play Command becomes the corresponding MIDI Note On message with its hexadecimal channel, velocity, and note preserved.
- [x] Ordered commands from one Tick are delivered as one batch without inferred sustain, deduplication, or note lifetime.
- [x] Velocity `00` is delivered unchanged so the user controls note-off explicitly from Source.
- [x] Device loss or delivery failure is visible to the user while Playback and Source interpretation continue.
- [x] Reconnecting or selecting another available destination restores subsequent output without restarting Playback.
- [x] Stopping Playback or losing the selected destination attempts all-notes-off.
- [x] Native MIDI dependencies remain isolated so non-native builds do not compile or initialize the native adapter.

## Answer

Desktop builds now use a target-gated `midir` backend behind a device-independent MIDI adapter. The MIDI menu enumerates and selects destinations, reports enumeration and delivery failures, and permits reselection while Playback continues. Each Play Command is submitted in Tick order as `[Note On | channel, note, velocity]`; zero velocity is unchanged. Stop, destination changes, and failed delivery attempt all-notes-off across all channels.

## Comments

- Regression tests use a fake MIDI backend and assert raw ordered bytes, zero velocity, enumeration and selection, all-notes-off, delivery failure, and reconnection without MIDI hardware.
- `cargo test --workspace` and `cargo check -p console` pass. The WebAssembly dependency graph contains no `midir` package.
- Strict workspace Clippy remains blocked by pre-existing warnings in `lang` (`needless_borrow`, `new_without_default`, `len_without_is_empty`, and `useless_conversion`).
