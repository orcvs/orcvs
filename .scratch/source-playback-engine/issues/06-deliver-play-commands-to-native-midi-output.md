# 06 — Deliver Play Commands to native MIDI output

**What to build:** Let a desktop user select a native MIDI destination and hear the exact Play Command batches produced during Playback, while keeping device availability and delivery failures outside Source interpretation.

**Blocked by:** 05 — Run Live Editing through the Playback Engine.

**Status:** ready-for-agent

- [ ] Native desktop builds can enumerate available MIDI output destinations and let the user select one.
- [ ] Each submitted Play Command becomes the corresponding MIDI Note On message with its hexadecimal channel, velocity, and note preserved.
- [ ] Ordered commands from one Tick are delivered as one batch without inferred sustain, deduplication, or note lifetime.
- [ ] Velocity `00` is delivered unchanged so the user controls note-off explicitly from Source.
- [ ] Device loss or delivery failure is visible to the user while Playback and Source interpretation continue.
- [ ] Reconnecting or selecting another available destination restores subsequent output without restarting Playback.
- [ ] Stopping Playback or losing the selected destination attempts all-notes-off.
- [ ] Native MIDI dependencies remain isolated so non-native builds do not compile or initialize the native adapter.
