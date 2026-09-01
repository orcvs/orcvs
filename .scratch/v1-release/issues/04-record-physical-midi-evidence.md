# 04 — Record physical MIDI evidence

**What to build:** Demonstrate the exact candidate's native MIDI adapter against one physical MIDI
device after deterministic fake-adapter coverage has proved the complete five-Function software
contract.

**Blocked by:** 03 — Run the exact-candidate verification workflow.

**Status:** ready-for-human

**Tags:** release/v1

- [ ] Record candidate SHA, OS, MIDI device, connection and playback procedure, expected
      observation, actual result, date, and reviewer.
- [ ] Exercise port enumeration and selection, Raw Play, Timed expiry, Monophonic replacement,
      Control Change, Pitch Bend, stop/all-notes-off, disconnect, and reconnect.
- [ ] Link the deterministic fake-adapter results that prove exact bytes, ordering, zero cases,
      scheduling, ownership, and failure cleanup.
- [ ] Hardware evidence supplements rather than replaces automated protocol and lifecycle tests.
