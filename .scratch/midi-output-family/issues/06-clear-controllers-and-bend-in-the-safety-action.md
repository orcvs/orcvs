# 06 — Clear controllers and bend in the safety action

**What to build:** Widen the output adapter's safety action from All Notes Off alone to one that also
returns controllers and the pitch wheel to their defaults.

**Blocked by:** 04 — Send Control Change and Pitch Bend.

**Status:** ready-for-agent

**Tags:** release/v1

- [ ] The safety action sends CC 121 Reset All Controllers per channel alongside the existing CC 123.
- [ ] The safety action sends an explicit centred bend `[0xE0 | channel, 0x00, 0x40]` per channel.
- [ ] Byte order within the safety action is fixed and asserted, not incidental.
- [ ] Every safety trigger carries the widened action: playback stop, disconnect, destination change,
      and delivery-failure teardown.
- [ ] A test drives a `!b` bend and a latching `!c` through a stop and asserts the bytes that follow.
- [ ] `CONTEXT.md`'s Playback Engine entry states what the safety action resets, not just that it
      fires.
- [ ] A partial failure mid-action behaves as CC 123 already does: the first error is reported and
      the remaining channels are still attempted.

## Comments

### Release scope is decided

2026-09-09: the release-candidate audit put this question to the owner of v1 scope and the answer
was to widen the release. The issue now carries `release/v1` and blocks `v1-release/03`, not
`v1-release/04`. The safety action must ship before the candidate SHA is cut, so the deterministic
fake-adapter evidence and the recorded physical smoke describe the same implementation. The
definition of done requires MIDI terminal output to cover device lifecycle, which is this issue.

This file was also renumbered from `05` to `06`: the resolved
`05-extend-terminal-output-functions-over-sequences.md` already claimed that reference.

### Why this is not part of 04

`04` added `!c` and `!b`, which is the first time an Orcvs Source can set device state that outlives
the note that carried it. Before it, a Source wrote notes only, and CC 123 was a sufficient safety
action. `04` recorded the gap under "Left open" rather than closing it, because widening the safety
action changes behaviour for every Source rather than only for Sources that use the two new
spellings, and because `CONTEXT.md` records the safety action as a domain rule. That is a decision
with its own record, not a detail of the ticket that exposed it.

### The failure

A Source writes `!b03007F` and the user stops playback. Orcvs sends `[0xB3, 0x7B, 0x00]`. The wheel
on channel 3 stays deflected, so the next note on that channel sounds sharp — including notes from
another program sharing the port, and including Orcvs's own notes after a restart. Nothing in Orcvs
can clear it. The same holds for `!c01407F`: CC 64 is sustain, and CC 123 does not release a held
pedal on every device.

### Why CC 121

CC 121 is the message the protocol provides for exactly this. The MIDI Association describes it as
what a sequencer sends so that stopping in the middle of a pitch bend does not leave the bend stuck
in the receiving synth; it also releases the sustain pedal and returns the mod wheel to zero, which
a bend centre alone does not. The explicit centred bend is sent as well because device support for
CC 121 varies, and `0x2000` is unambiguous where CC 121's coverage is not.

### The objection, and why it does not hold

CC 121 clears state on channels the Source never wrote, including channels driven by other gear on
the same port. That is true, and it is already true of the CC 123 loop this widens: Orcvs sends the
existing safety action on all sixteen channels, and so does Orca. The precedent is set, and a safety
action that only half-silences the device is the worse of the two positions.

### Orca does not do this

Orca's stop (`clock.js`) calls `midi.allNotesOff()` — the same CC 123 loop over sixteen channels —
and then `midi.silence()`, which sends an explicit Note Off for each note in its own tracked stack.
It sends no CC 121 and centres no wheel. Its `MidiCC.clear()` empties the pending queue without
sending anything, and `io.silence()` never calls the `cc` module at all, so Orca carries this fault
too. That is not an argument for keeping it: Orcvs already departs from Orca where Orca is wrong for
this project, per ADR 0010 on base-36 and ADR 0016 on scaling and clamping.

### Worth deciding alongside

Orca sends explicit Note Offs for its tracked notes in addition to CC 123; Orcvs sends CC 123 and
discards its schedule. CC 123 should cover it on a compliant device, and this ticket does not change
it — but whoever holds the safety action open should say whether that difference is intended.
