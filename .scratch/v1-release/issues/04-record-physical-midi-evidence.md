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
      Control Change, Pitch Bend, stop/all-notes-off, disconnect, and reconnect. Record that the
      candidate ships All Notes Off (CC 123), Reset All Controllers (CC 121) and a centred bend on
      every channel (`orcvs/src/midi.rs:221-251`, delivered by `midi-output-family/06`), so the
      hardware observation and the fake adapter describe the same implementation.
- [ ] Change the destination mid-run on the physical device and record whether playback continued
      without an audible gap or an Overrun diagnostic. This is a hardware observation; the automated
      guarantee is `midi-port-ownership/08`, which is outside the release.
- [ ] Link the deterministic fake-adapter results that prove exact bytes, ordering, zero cases,
      scheduling, ownership, and failure cleanup.
- [ ] Hardware evidence supplements rather than replaces automated protocol and lifecycle tests.

## Comments

### Issue audit against d3fd1b27 — 2026-10-01

The shipped safety action is all-notes-off, CC 121 and a centred bend on every channel
(`orcvs/src/midi.rs:226-237`). The mid-run destination-change line cites midi-port-ownership/08,
which is untagged and outside the Gate closure, so nothing guarantees it lands first.

### Audit against origin/main cf01e3c — 2026-10-13

`midi-output-family/06` is resolved, so the safety-action line no longer describes pending work and
now states the shipped action. The reference in the 2026-10-01 comment, `orcvs/src/midi.rs:226-237`,
has moved: All Notes Off is `:221`, Reset All Controllers `:226`, and `safety_reset_messages`
`:244-251`. The mid-run destination-change line still cites `midi-port-ownership/08`, which remains
untagged and outside the Gate closure.

### Decided 2026-10-13 by Toby

The mid-run destination-change box is a standalone hardware observation. Its automated
counterpart, `midi-port-ownership/08`, stays untagged and outside the Gate closure (now
`needs-triage`), so this ticket does not wait on it.
