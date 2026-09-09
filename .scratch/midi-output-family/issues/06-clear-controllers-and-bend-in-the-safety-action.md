# 06 — Clear controllers and bend in the safety action

**What to build:** Widen the output adapter's safety action from All Notes Off alone to one that also
returns controllers and the pitch wheel to their defaults.

**Blocked by:** 04 — Send Control Change and Pitch Bend.

**Status:** resolved

**Tags:** release/v1

- [x] The safety action sends CC 121 Reset All Controllers per channel alongside the existing CC 123.
- [x] The safety action sends an explicit centred bend `[0xE0 | channel, 0x00, 0x40]` per channel.
- [x] Byte order within the safety action is fixed and asserted, not incidental.
- [x] Every safety trigger carries the widened action: playback stop, disconnect, destination change,
      and delivery-failure teardown.
- [x] A test drives a `!b` bend and a latching `!c` through a stop and asserts the bytes that follow.
- [x] `CONTEXT.md`'s Playback Engine entry states what the safety action resets, not just that it
      fires.
- [x] A partial failure mid-action behaves as CC 123 already does: the first error is reported and
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

### What was built

One widened action in one place. `MidiOutputAdapter::send_safety_reset` — the former
`send_all_notes_off` — loops the sixteen channels and sends the triple
`safety_reset_messages` states for each: `[0xB0 | channel, 123, 0x00]`,
`[0xB0 | channel, 121, 0x00]`, `[0xE0 | channel, 0x00, 0x40]`. The four safety triggers already
converged on that one function before this change and still do, so none of them carries a copy of
the action: playback stop and disconnect reach it through `PlaybackInner::send_safety_reset` and
the `OutputAdapter::safety_reset` trait method, a destination change through
`MidiOutputAdapter::select`, and delivery-failure teardown through the error arm of
`MidiOutputAdapter::submit`.

The order is a helper with a doc comment rather than three lines inside the loop, because "byte
order is fixed and asserted" is a claim about the action and not about the loop that drives it.
All Notes Off first, so the channel is silent before anything else on it changes. CC 121 next,
because it is the message that clears what a Control Change latched. The centred bend last: it is
sent at all because device support for CC 121 varies, and it is sent *after* CC 121 because a
device that does honour CC 121 may move the wheel itself, so the unambiguous `0x2000` has to be
the last word rather than the first.

Partial failure is unchanged in shape and now spans forty-eight messages rather than sixteen:
every message is attempted, `first_error.get_or_insert` keeps the first refusal, and that one is
returned. `a_refused_safety_message_reports_the_first_error_and_attempts_the_rest` refuses sends
1 and 20 with errors naming their own index, so a run that stopped early delivers too few messages
and a run that kept the last refusal names the wrong one. Both were confirmed by mutation:
returning at the first error fails it, and so does transposing CC 123 and CC 121, which fails the
two byte-order tests and nothing else.

`the_safety_action_clears_notes_controllers_and_bend_on_every_channel` asserts all forty-eight
messages — the ends of the run spelled out literally, then each channel's triple against a
test-local expectation rather than against the adapter's own arithmetic.
`a_stop_clears_the_bend_and_the_latched_controller_a_source_left_standing` is the Source-path
test the checklist asks for: `!c01407F` latches sustain on channel `01`, `!b03007F` deflects the
wheel on channel `03`, and after `playback.stop()` the two messages the run produced are followed
by the forty-eight the action owes, with channel `01`'s and channel `03`'s triples read out by
position.

### The name

`all_notes_off` is no longer what the action does, so `OutputAdapter::all_notes_off` is now
`OutputAdapter::safety_reset`, with `send_all_notes_off` and `InMemoryOutputAdapter`'s
`all_notes_off_count` renamed to match. `CONTEXT.md` already called this the safety action; the
code was the only place still naming it after one of its three messages. The rename is confined to
the `orcvs` crate — `shell` never referred to it — and touches no wire behaviour.

### The Orca Note-Off difference is intended

Decided rather than left open, per this issue's closing note. Orca sends explicit Note Offs for
every note in its tracked stack in addition to CC 123; Orcvs sends CC 123 and discards its
schedule, and that stays.

All Notes Off is the message the protocol provides for silencing a channel. A device that ignores
it is a device that would equally ignore the CC 121 sent beside it, so replaying the schedule buys
coverage only on a receiver already outside what this action assumes. Against that, replaying it
makes the safety action vary with what happened to be sounding: a fixed run of forty-eight
messages becomes a variable one, its length and content decided by the schedule's state at the
moment of the stop, which is exactly the thing a byte-order assertion cannot pin. The schedule
also records only what an adapter accepted, so after a delivery failure — one of the four triggers
— it is precisely the state Orcvs cannot trust. Orca's stack is not a better source of truth here;
it is a second one.

The difference is now stated in ADR 0016 alongside the widened action rather than only here, since
that is where the safety action's content is recorded.

### What was not verified

No physical MIDI smoke test. The evidence is the deterministic fake-adapter suite in
`orcvs/src/midi.rs`, which asserts the bytes a device would receive; nothing here was confirmed
against hardware.
