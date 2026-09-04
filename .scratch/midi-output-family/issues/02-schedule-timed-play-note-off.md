# 02 — Schedule Timed Play Note Off

**What to build:** Implement `!~ channel velocity note length` and make Playback Engine schedule its
explicit Note Off at Tick `T + length`.

**Blocked by:** 01 — Generalize Play Commands for MIDI output; tick-functions/01.

**Status:** resolved

**Tags:** release/v1

- [x] Positive velocity and length emit Note On now and Note Off at the beginning of Tick `T + length`.
- [x] Velocity `00` emits the explicit stop and schedules no expiry.
- [x] Length `00` emits no MIDI output.
- [x] Channel is `00`–`0F`, velocity `00`–`7F`, Note is typed, and length is `00`–`FF`.
- [x] Stale expiries cannot stop a separately owned future note incorrectly.
- [x] Stop, disconnect, destination change, and final-handle drop clear scheduled ownership safely.
- [x] Cancellation and exact-Tick expiry tests are deterministic without wall-clock sleeps.

## Comments

`!~` is one row in `define_functions!` — `[channel: MidiChannel, velocity: Velocity, note: Note,
length: Length]` — and `functions::timed_play` destructures it into `PlayCommand::Timed`. The
length is carried unread across the crate boundary, because a Tick plans one Tick and the Note
Off this command owes is due at another one. `Length` is a new domain type over the whole byte:
it converts where the MIDI domains validate, and exists so that `PlayCommand`'s claim that every
field carries its own domain stays true and a length cannot stand in a data-byte position.
`Function::Play` became `Function::RawPlay` now that two Play spellings exist.

### The schedule is the Playback Engine's, keyed per channel and note

`PlaybackInner` holds a `TimedNotes`: a claim per `TimedVoice` (channel and note, since Timed
Play is polyphonic and ADR 0016 gives one voice per channel to Mono alone), and the expiries due
at each Tick. Each claim carries a generation token. An expiry is scheduled at the Tick it comes
due and is never revisited when the note it would stop is replaced or stopped early; instead the
stale entry is refused when it arrives, because the claim it names is no longer the claim
standing. That is what stops `!~007FC403`, an explicit stop, and a second `!~007FC405` on the
same voice from having the first command's expiry cut the third command's note short.

Delivery per executed Tick is one submission: the stops due at this Tick first, then the Tick
Plan's own commands in order. ADR 0016's requirement is an ordering — a scheduled Note Off
arrives before that Tick's new Play Commands — and two submissions would leave that order to the
adapter to keep. The engine resolves the Tick Plan only while connected, so nothing is owned
that was never sounded.

### An output adapter can no longer be handed a lifetime

`OutputAdapter::submit` now takes `&[OutputCommand]` rather than `&[PlayCommand]`. A Play Command
says what the Source asked for; an Output Command says what is delivered, and only the Playback
Engine spans the two. `OutputCommand::NoteOn` is the single variant today, exactly as
`PlayCommand::Raw` was at issue 01: Control Change and Pitch Bend join it in issue 04, carried
through unresolved. The alternative — leaving `Timed` representable at the adapter — needed
either an `unreachable!` inside a Tick under the Playback lock, which ADR 0028 rules out, or an
arm that assembled a Note On and lost the schedule silently. A scheduled stop is delivered as
MIDI's zero-velocity Note On, the same explicit stop Raw Play already gives the Source.

### What clears the schedule

Beginning a run, stopping, disconnecting, and changing destination, plus the final-handle drop
that routes through `stop`. Both selection paths — `PlaybackEngine::select_midi_destination` and
the `MidiSelectionHandle` the shell holds — now call one `PlaybackInner::select_destination`, so
what a destination change owes is stated once rather than twice.

A refused submission changes nothing at all. The schedule describes notes that are sounding, so
a Tick is resolved against a copy of it and the copy is adopted only once the adapter has
accepted the delivery: every claim and every expiry stands, and the stop the refused Tick drained
is drained again by the next executed Tick. `MidiOutputAdapter` alone would not have needed this,
since it gives up its connection on a failed send and a discarded stop would have been a stop
into a dead connection either way. But `OutputAdapter` is a trait and this module is generic over
it, so for an adapter that survives a refusal — `InMemoryOutputAdapter` is one — committing
before the submission means a hanging note with no retry and no diagnostic. The copy is two maps
of the notes currently sounding, taken once per executed Tick.

### Left open

A Timed expiry stops whatever stands on its voice, so a Raw Play of the same channel and note
started between a claim and its expiry is stopped by that expiry. One note is sounding on the
voice and one stop ends it, which is arguably what the wire means, but ADR 0016 says nothing
either way. Recorded for whoever revisits the ADR rather than settled here.

### Tests

Every Tick is driven through `clock_tick` at a stated absolute Tick, so nothing sleeps and
nothing depends on a clock. A Source-resident Bang persists across Ticks until
`spatial-tick-planning/02`, so an activated terminal root repeats every Tick; the tests retire
the Bang by clearing its Cells after the Tick that matters, which leaves the Ticks after it
carrying the schedule alone. `a_scheduled_stop_is_due_at_an_absolute_tick_rather_than_at_a_clock_tick`
drives a declined overrun Tick between the start and the stop, pinning that an expiry counts
executed Ticks rather than clock ticks. The destination-change test uses the paused-clock MIDI
fake, advancing one Tick period at a time.

Three properties needed a Tick that carries more than one thing, and each was proven by watching
the test fail without the code that provides it. The delivery order is pinned by a Tick that
carries both a due stop and a Raw Play of the voice that stop names: appending the stops last
instead silences the note that Tick sounds. Ownership per channel *and* note is pinned by a
second note on one channel and by one note on two channels: a key that compares either field
alone stops the standing note to start the new one, cutting it short. And two commands for one
voice within one Tick pin that ownership is resolved in Tick Plan order rather than once per
Tick. The refused-submission test drives the failure at the Tick a stop is due and reads the
stop from the Tick after it.
