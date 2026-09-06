# 03 — Own Monophonic voices per channel

**What to build:** Implement `!% channel velocity note length` with one Mono-owned voice per MIDI
channel. Written as "per output adapter and MIDI channel", after ADR 0016 as it then stood; the
owner is the Playback Engine, which this ticket settled and corrected the ADR to say.

**Blocked by:** 02 — Schedule Timed Play Note Off.

**Status:** resolved

**Tags:** release/v1

- [x] Every command stops the prior Mono-owned note on its channel first.
- [x] Velocity `00` or length `00` replaces ownership with silence and starts nothing.
- [x] Positive commands own the replacement and schedule Note Off at Tick `T + length`.
- [x] Generation tokens prevent stale expiry from stopping a replacement.
- [x] Channels are independent and the last same-Tick command owns its channel.
- [x] Raw and Timed Play notes never enter Mono ownership.
- [x] Stop and adapter lifecycle safety clear every Mono voice.

## Comments

`!%` is one row in `define_functions!` with Timed Play's operand list — `[channel: MidiChannel,
velocity: Velocity, note: Note, length: Length]` — and `functions::monophonic_play` destructures
it into `PlayCommand::Mono`. A variant of its own rather than a flag on `Timed`, because the two
carry identical operands and differ entirely in what they own: a shared variant would put that
difference in a field every consumer had to branch on, where a variant makes the match arms the
branch. Nothing in the lang crate knows the difference; it is a spelling that arrives, and the
Playback Engine is where it means something.

### The seam: Playback Engine, and ADR 0016 corrected rather than left contradicted

ADR 0016 as first written gave the Mono voice and its expiry to the selected output adapter.
Issue 02 had already moved Timed Play's schedule to the Playback Engine because an
`OutputCommand` carries one message and no lifetime, and left CONTEXT.md flagging the
contradiction for this issue to settle. It is settled the way issue 02 pointed: Mono ownership
sits beside the Timed schedule, and ADR 0016's Monophonic paragraph now says so and says why —
an adapter handed a lifetime has nothing to schedule with, and giving it one would put musical
time on the far side of the seam ADR 0001 draws. ADR 0008 and ADR 0019 each carried the same
"per output adapter and MIDI channel" phrasing and are corrected in step, so the old seam is not
left standing anywhere. CONTEXT.md's Monophonic Play entry now states the settled seam instead of
recording an open question, and its Playback Engine, Play Command, Output Command, and Terminal
Output Function entries follow.

### One schedule, two keys

`TimedNotes` became `OwnedNotes` and its key became a `Voice` enum: `Timed { channel, note }` and
`Mono { channel }`. Everything that follows a claim — the generation token, the Tick its stop is
due at, the staleness check on arrival, the drain-before-the-Tick-Plan ordering, and every
lifecycle action that clears the lot — is identical for the two spellings, and the only thing
that differs is what counts as the same voice. Two schedules would have restated all of that
twice and let one of them drift.

Distinct variants are also what keeps the two ownerships apart in both directions: a Mono command
cannot find a Timed claim to replace, and a Timed expiry cannot stop a Mono note, because neither
key can name the other's voice. A Mono note and a Timed note may therefore sound the same pitch
on the same channel and expire independently, which is what ADR 0016 asks for and what the wire
will carry as two Note Ons and two Note Offs.

The map's value gained a note beside the claim. A Mono key names a channel and not a note, so the
note a stop needs cannot be read back off the key; recording it in the claim means only a claim
still standing can name a note to stop, and a Timed voice restating its note there is the price
of one schedule rather than two.

### Where Monophonic Play parts from Timed Play

Every `!%` releases its channel before it does anything else, and the note it stops comes from the
claim rather than from the command's own note operand. That is the whole of what monophony means
here: the voice is the channel, and a Source that wants the previous note stopped does not have to
remember what it was.

Velocity `00` and length `00` both leave the channel silent and end the command. Timed Play's
length `00` is a no-op instead, and the two rules are consistent rather than merely different: a
Timed command claims the note it names, so a note that never starts claims nothing and disturbs
nothing, while a Monophonic command claims its channel whether or not it sounds, so one that
starts nothing has replaced the voice with silence. The ADR paragraph and the CONTEXT.md entry
both now say that reason rather than only the rule.

### Left open

Issue 02 recorded that a Timed expiry stops whatever stands on its voice, so a Raw Play of the
same channel and note started between a claim and its expiry is stopped by it. Monophonic Play
adds no new case of that and settles none of it: a Mono expiry stops the note its own claim
recorded, so it cannot reach a Raw or Timed note, but a Mono Note Off still goes out on a channel
and note a Raw Play may also be sounding. Recorded for whoever revisits ADR 0016, as issue 02 left
it.

Terminal Output Functions stay Scalar. `!%` broadcasting across a Sequence is the same open
question `!>` and `!~` already carry, and CONTEXT.md still records that as the state of the code
rather than a decision.

### Tests

Every Tick is driven through `clock_tick` at a stated absolute Tick, so nothing sleeps and nothing
depends on a clock. A Source-resident Bang persists across Ticks, so the tests retire it by
clearing its Cells after the Tick that matters, which leaves the Ticks after it carrying the
schedule alone.

Each non-obvious property was proven by watching the tests fail without the code that provides
it, five mutations in all. Keying the Mono voice by channel *and* note fails three tests: the one
that changes note on a held channel, the one that stops with a note operand that is not the note
sounding, and the two-commands-in-one-Tick test. Releasing the channel only when the command goes
on to start something fails the velocity-`00`, length-`00`, and stale-expiry tests. Dropping the
generation-token comparison in `expired_at` fails both stale-expiry tests and both replacement
tests, Timed and Mono alike. Appending the due stops after the Tick Plan instead of before it
fails the length-`01` self-replacement test, which pins that one Tick carrying both a due stop
and a command for the voice it names delivers one stop rather than two and does not silence the
note it starts. Reversing the Tick Plan order fails the two-commands-in-one-Tick test. And keying
Mono in Timed's own key space fails `timed_and_mono_own_separately_and_neither_owns_a_raw_note`,
which sounds one channel and one note through all three spellings and counts two stops where a
shared key delivers one.

The lifecycle test is now a loop over a Timed and a Mono Expression, asserting stop, disconnect,
begin-run, and final-handle drop of each, because the two share one schedule and a clear that
reached only one of them would leave the other hanging.
