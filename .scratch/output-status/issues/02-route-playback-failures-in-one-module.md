# 02 — Route Playback failures in one module

Status: ready-for-agent
Type: task
Blocked by: 01

**What to build:** `console::playback_status` routes each frame's diagnostics to the MIDI, network
and engine statuses the Panel shows ([spec](../spec.md), ADR 0092).

- [ ] MIDI selection and the network setting no longer read Playback diagnostics.
- [ ] Engine failures show in their own status until the next run begins.
- [ ] The `console_midi::AVAILABLE` routing branch and `report_playback_failures` are gone.
- [ ] The browser test reports a network failure through the Playback status.
