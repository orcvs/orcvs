# 02 — Schedule Timed Play Note Off

**What to build:** Implement `!~ channel velocity note length` and make Playback Engine schedule its
explicit Note Off at Tick `T + length`.

**Blocked by:** 01 — Generalize Play Commands for MIDI output;
`.scratch/tick-functions/issues/01-thread-tick-and-position-into-interpretation.md`.

**Status:** ready-for-agent

- [ ] Positive velocity and length emit Note On now and Note Off at the beginning of Tick `T + length`.
- [ ] Velocity `00` emits the explicit stop and schedules no expiry.
- [ ] Length `00` emits no MIDI output.
- [ ] Channel is `00`–`0F`, velocity `00`–`7F`, Note is typed, and length is `00`–`FF`.
- [ ] Stale expiries cannot stop a separately owned future note incorrectly.
- [ ] Stop, disconnect, destination change, and final-handle drop clear scheduled ownership safely.
- [ ] Cancellation and exact-Tick expiry tests are deterministic without wall-clock sleeps.
