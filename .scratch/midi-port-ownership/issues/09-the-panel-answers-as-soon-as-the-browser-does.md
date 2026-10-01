# 09 — The Panel answers as soon as the browser does

**What to build:** A performer running Orcvs in the browser sees the MIDI menu catch up with the browser's permission answer on its own. When the browser grants MIDI access, the Panel repaints, discovery runs again, the destination list fills, and the first destination is selected automatically, as on the desktop. When the browser refuses access or has no Web MIDI, the "waiting for the browser to grant MIDI access" status clears and the list reads `None` with no error. No Scan and no click on the Output readout is needed in either case.

Today the console runs discovery once at startup, in the same call that sends the access request, so that discovery always finds access pending. Settling the request only updates the kept access and neither wakes the Panel nor runs discovery again. The pending status therefore stays on screen, in the error colour, until the performer Scans, even when the browser has refused and access can never come for the page.

Discovery needs to report "access still pending" as its own kind of answer rather than as a failure message, so the console can tell a temporary answer from a real failure without comparing strings. The native backend never gives that answer.

**Blocked by:** None — can start immediately.

**Status:** ready-for-human

**Tags:** release/v1

- [ ] Settling the access request wakes the Panel, so the next frame runs even without performer input.
- [x] While the last discovery answered pending, each frame runs discovery again until it doesn't; a settled answer stops the retries.
- [x] A granted page lists its connected outputs and auto-selects the first without a Scan.
- [x] A refusing page, or a browser without Web MIDI, shows no status and reads `None` without a Scan, so manual step 7 of `midi-port-ownership/06` passes as written.
- [x] A fake-adapter test builds the console's MIDI selection while access is pending, settles the fake to granted and to unavailable, runs one frame's worth of Panel MIDI logic without a Scan, and asserts both the destination list and the status line.
- [x] Native discovery behaviour and its tests are unchanged.
- [x] ADR 0059 no longer says nothing re-runs discovery when the Promise settles, and describes the wake and retry instead.
- [x] The manual steps in `midi-port-ownership/06` that tell the performer to Scan after allowing access describe the automatic listing instead.

## Comments

**2026-09-29 — implemented.** `MidiError::pending` is the discovery answer a backend gives while its MIDI service has not answered; the browser backend gives it while access is pending, and no native backend does. `MidiDeviceSelection::observe_frame` is the Panel's per-frame MIDI work: it discovers again while the last discovery answered pending, then auto-selects. The browser keeps a Promise that resolves once the access answer is stored, and the console spawns a task that awaits it and requests a repaint.

Evidence:

- `web_midi::tests::the_frame_after_the_browser_answers_catches_up_without_a_scan` builds the selection while the fake is pending, settles it to granted and to unavailable, and runs one frame with no Scan: granted lists and selects the first output with no status; unavailable lists nothing with no status.
- `midi::tests::a_failed_discovery_is_not_repeated_by_later_frames` holds that a native-style failure is asked once and kept until a Scan.
- `console/tests/wasm.rs` also asserts the headless browser's pending answer is the pending kind.

Left open: *settling the access request wakes the Panel*. The wake compiles for `wasm32` but only a browser shows that a frame runs with no input. Manual steps 3, 4 and 7 of `midi-port-ownership/06` check it: after allowing access, the port is listed and selected without moving the mouse; after denying it in site settings and reloading, `None` shows with no status.

### Issue audit against d3fd1b27 — 2026-10-01

The "Today…" paragraph describes the code before #185. The web entry point now sends the access
request before the console starts (`console/src/main.rs:90`, `request_access_within`). The ticked
lines hold: `MidiError::pending` (`orcvs/src/midi.rs:68-76`), the retry in `observe_frame`
(`console/src/midi.rs:164-184`) and the wake (`console/src/console/repaint.rs:36-44`). Only the
manual line remains, checked by 06's steps 3, 4 and 7.
