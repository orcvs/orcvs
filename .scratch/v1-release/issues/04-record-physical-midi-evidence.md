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
      Control Change, Pitch Bend, stop/all-notes-off, disconnect, and reconnect. Record which
      safety action the candidate ships. `midi-output-family/06` widens it from CC 123 alone to
      CC 121 and a centred bend, and it blocks `03`, so the hardware observation and the fake
      adapter describe the same implementation.
- [ ] Link the deterministic fake-adapter results that prove exact bytes, ordering, zero cases,
      scheduling, ownership, and failure cleanup.
- [ ] Hardware evidence supplements rather than replaces automated protocol and lifecycle tests.
