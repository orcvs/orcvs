# 01 — Publish the MIDI failure on the observation

Status: ready-for-agent
Type: task

**What to build:** the Playback observation carries `midi_failed`, the MIDI latch Outputs keeps,
beside `network_failed` ([spec](../spec.md), ADR 0092).

- [ ] Set when a MIDI delivery or safety action fails.
- [ ] Cleared by a successful delivery, an installed destination and the start of a run.
- [ ] Published after every step that can change either latch.
