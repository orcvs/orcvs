# 01 — Publish the MIDI failure on the observation

Status: resolved
Type: task

**What to build:** the Playback observation carries `midi_failed`, the MIDI latch Outputs keeps,
beside `network_failed` ([spec](../spec.md), ADR 0092).

- [x] Set when a MIDI delivery or safety action fails.
- [x] Cleared by a successful delivery and an installed destination, not by the start of a run,
      which clears only the reporting latch.
- [x] Published after every step that can change either latch.
