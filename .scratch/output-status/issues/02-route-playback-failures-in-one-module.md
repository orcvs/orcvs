# 02 — Route Playback failures in one module

Status: resolved
Type: task
Blocked by: 01

**What to build:** `console::playback_status` routes each frame's diagnostics to the MIDI, network
and engine statuses the Panel shows ([spec](../spec.md), ADR 0092).

- [x] MIDI selection and the network setting no longer read Playback diagnostics.
- [x] Engine failures show in their own status until the next run begins.
- [x] The `console_midi::AVAILABLE` routing branch and `report_playback_failures` are gone.
- [x] The browser test reports a network failure through the Playback status.
