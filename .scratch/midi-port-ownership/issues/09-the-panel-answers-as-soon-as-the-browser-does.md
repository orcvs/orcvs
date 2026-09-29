# 09 — The Panel answers as soon as the browser does

**What to build:** A performer running Orcvs in the browser sees the MIDI menu catch up with the browser's permission answer on its own. When the browser grants MIDI access, the Panel repaints, discovery runs again, the destination list fills, and the first destination is selected automatically, as on the desktop. When the browser refuses access or has no Web MIDI, the "waiting for the browser to grant MIDI access" status clears and the list reads `None` with no error. No Scan and no click on the Output readout is needed in either case.

Today the console runs discovery once at startup, in the same call that sends the access request, so that discovery always finds access pending. Settling the request only updates the kept access and neither wakes the Panel nor runs discovery again. The pending status therefore stays on screen, in the error colour, until the performer Scans, even when the browser has refused and access can never come for the page.

Discovery needs to report "access still pending" as its own kind of answer rather than as a failure message, so the console can tell a temporary answer from a real failure without comparing strings. The native backend never gives that answer.

**Blocked by:** None — can start immediately.

**Status:** ready-for-agent

**Tags:** release/v1

- [ ] Settling the access request wakes the Panel, so the next frame runs even without performer input.
- [ ] While the last discovery answered pending, each frame runs discovery again until it doesn't; a settled answer stops the retries.
- [ ] A granted page lists its connected outputs and auto-selects the first without a Scan.
- [ ] A refusing page, or a browser without Web MIDI, shows no status and reads `None` without a Scan, so manual step 7 of `midi-port-ownership/06` passes as written.
- [ ] A fake-adapter test builds the console's MIDI selection while access is pending, settles the fake to granted and to unavailable, runs one frame's worth of Panel MIDI logic without a Scan, and asserts both the destination list and the status line.
- [ ] Native discovery behaviour and its tests are unchanged.
- [ ] ADR 0059 no longer says nothing re-runs discovery when the Promise settles, and describes the wake and retry instead.
- [ ] The manual steps in `midi-port-ownership/06` that tell the performer to Scan after allowing access describe the automatic listing instead.
