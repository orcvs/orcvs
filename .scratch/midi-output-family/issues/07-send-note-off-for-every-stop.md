# 07 — Send Note Off for every stop

**What to build:** Deliver every note stop as MIDI Note Off (`0x80 | channel`) rather than as a
Note On at velocity `00`, as Orca's `midi.js` does.

**Blocked by:** None (can start immediately)

**Status:** resolved

- [x] `OutputCommand` has a `NoteOff { channel, note }` variant, carrying no release velocity.
- [x] The engine's scheduled stops — Timed and Mono expiry, Timed velocity `00`, and Mono
      replacement — are delivered as `NoteOff`.
- [x] A Raw Play at velocity `00` is resolved into `NoteOff` by the Playback Engine, so the
      adapter encodes it without reading a velocity.
- [x] `MidiOutputAdapter` encodes `NoteOff` as `[0x80 | channel, note, 0x00]`; a test asserts the
      bytes on an upper-nibble channel.
- [x] The loopback listener decodes `0x80` as a Note Off, refuses a velocity-`00` Note On, and
      hears the program's Timed stop.
- [x] `CONTEXT.md`'s Play Command and Output Command entries state the stop's delivery.
- [x] Gates pass with `PROPTEST_CASES=32`.

## Comments

2026-10-18: A wire-encoding change, not a language change. ADR 0016 says velocity `00` is Raw Play's
explicit stop and that a Timed or Mono lifetime ends in a Note Off; it never chose the
zero-velocity Note On as the wire form, so it stands unamended. MIDI treats the two as equivalent
for a receiver, but a Note Off is what the stop is, it is what Orca sends, and a receiver that
distinguishes them (or a monitor reading the stream) no longer sees every stop as a Note On.

The release velocity is `00` because Orcvs has no spelling for one. The safety action is unchanged:
it still sends CC 123, CC 121 and a centred bend per channel and replays no schedule, per issue 06.

The web build needs no change of its own: `console/src/web_midi.rs` hands `MidiOutputAdapter`'s
bytes to Web MIDI and does not encode a message itself.

No physical MIDI smoke test was run; the evidence is the fake-adapter and loopback suites.
