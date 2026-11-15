# Playback and outputs requirements

This document owns the Playback Engine: its lifecycle, requests, musical time, diagnostics, stop and
Rewind; the MIDI, UDP and OSC outputs and the policy each destination keeps; MIDI destination
selection; and the Built-in Synth's sound. It does not own Function spellings, operand binding or
blank operands ([language.md](language.md)), how a Tick is planned and committed
([evaluation.md](evaluation.md)), how the console draws Playback state or failures
([console.md](console.md)), or the site's page messages and Sound Module ([site.md](site.md)).
Terms are defined in [CONTEXT.md](../../CONTEXT.md); decisions are argued in [the ADRs](../adr/).

> Snapshot: incorporates ADRs through 0110 and ADR changes dated through 2026-11-15.
> A newer ADR binds where it differs until this document is updated; the
> [reading order](README.md#reading-order) says which ADRs are newer.
> Source ADRs: 0001, 0002, 0012, 0016, 0037, 0041, 0055, 0056, 0059, 0076, 0085, 0087, 0088, 0090,
> 0099, 0103, 0108.

## The seam between interpretation and Playback

### PLAY-010 Playback dispatches a committed Tick Plan and never interprets Source

Interpretation produces each Tick's Tick Plan and commits its Source writes before the Playback
Engine receives that Tick's Play Commands and Network Sends. The Playback Engine owns timing,
lifecycle and output adapters. It never parses Source or infers musical intent: it delivers every
command as supplied, and schedules a Note Off only where a Play Command carries a Tick lifetime
(Timed and Monophonic Play). The safety action (PLAY-260) is the one output it originates.

Reason: deterministic interpretation and testing, with musical interpretation and delivery concerns
kept out of the Source module.
Sources: [ADR 0001](../adr/0001-separate-source-interpretation-from-playback-engine.md),
[ADR 0090](../adr/0090-each-output-destination-keeps-its-policy-in-one-module.md).
Evidence: `orcvs/src/playback.rs` `PlaybackInner::execute_tick`; test `orcvs/src/playback.rs` `a_tick_commits_source_before_submitting_play_commands`.

### PLAY-280 A Play Command says what Source asked; an Output Command says what is delivered

A Play Command is MIDI only, one explicit variant per spelling, carrying values already validated
against their MIDI domains, with one type per operand role so that roles sharing a domain (a Control
Change's controller and value, a Pitch Bend's LSB and MSB) are not interchangeable. An Output
Command is one MIDI message carrying no lifetime, so an adapter never schedules anything. Every stop
reaches the adapter as a Note Off, encoded `[0x80 | channel, note, 0x00]`; the Playback Engine never
hands an adapter a zero-velocity Note On. Control Change and Pitch Bend reach the adapter unresolved,
in Tick Plan order.

Reason: musical time stays on the engine's side of the seam, and only the adapter knows wire
encoding.
Sources: [ADR 0016](../adr/0016-use-distinct-raw-and-timed-play-functions.md),
[ADR 0090](../adr/0090-each-output-destination-keeps-its-policy-in-one-module.md); the Note Off
encoding and role types are stated by `CONTEXT.md` (Play Command, Output Command), no ADR
(`.scratch/midi-output-family/issues/07-send-note-off-for-every-stop.md`).
Evidence: `orcvs/src/playback.rs` `OutputCommand`; test `orcvs/src/midi.rs` `submits_note_off_as_its_wire_bytes`.

### PLAY-290 A Chord Play's tones are ordinary Play Commands

A Raw or Timed Chord Play's Turn emits one ordinary Raw or Timed Play Command per tone, lowest tone
first. Velocity `00`, length `00`, Note Off scheduling, ownership and the safety action apply tone by
tone exactly as for single notes; the Playback Engine has no chord behaviour. A Timed Chord Play's
tones and a Monophonic Play on the same channel keep separate ownership.

Reason: one chord drives as one thing without giving the engine a second rule.
Sources: [ADR 0103](../adr/0103-one-chord-play-sounds-every-tone-of-its-chord.md).
Evidence: `orcvs/src/source/tick/chord_play.rs`; test `orcvs/src/playback/tests/chord_play.rs` `a_repeated_chord_replaces_the_tones_it_repeats_and_retires_their_expiries`.

## Engine lifecycle and requests

### PLAY-020 The Playback Engine owns its lifecycle behind a cloneable handle

The Playback Engine owns its clock, synchronisation and shutdown; callers coordinate none of them.
It is reached through a cloneable handle whose `start` and `stop` are idempotent: `start` does
nothing to a live run and `stop` nothing to a stopped engine.

Reason: musical-time safety stays local to the module, and its interface is the shared caller and
test surface.
Sources: [ADR 0002](../adr/0002-playback-engine-owns-lifecycle-concurrency.md),
[ADR 0041](../adr/0041-the-playback-engine-owns-its-state-in-one-task.md),
[ADR 0056](../adr/0056-playback-requests-wait-in-slots-not-a-queue.md).
Evidence: `orcvs/src/playback.rs` `PlaybackEngine`; test `start_is_idempotent_and_draining_takes_the_diagnostics` in the same file.

### PLAY-030 No Tick is admitted once `stop` returns

`stop` shuts a gate, under the mailbox lock, before it leaves its stop flag; the task checks the
gate before executing each Tick. A Tick already admitted runs to completion. "A stop has been asked"
and "Playback is playing" are distinct facts, and nothing reads the gate to decide which state the
engine is in. A full mailbox cannot drop a stop.

Reason: sending a message does not by itself prevent a Tick the task may already be starting.
Sources: [ADR 0002](../adr/0002-playback-engine-owns-lifecycle-concurrency.md),
[ADR 0041](../adr/0041-the-playback-engine-owns-its-state-in-one-task.md),
[ADR 0056](../adr/0056-playback-requests-wait-in-slots-not-a-queue.md).
Evidence: `orcvs/src/playback/gate.rs`; test `orcvs/src/playback.rs` `a_requested_stop_prevents_a_tick_before_the_message_arrives`.

### PLAY-040 A run begins with an immediate Tick, and no failure rolls back Source or stops it

A run has one Tick period at a time and executes its first Tick immediately. A zero period is
refused. Output failures and Overruns become diagnostics; they never roll back committed Source
writes or stop later Ticks.

Reason: a performance survives a failing device or a late clock.
Sources: [ADR 0002](../adr/0002-playback-engine-owns-lifecycle-concurrency.md),
[ADR 0041](../adr/0041-the-playback-engine-owns-its-state-in-one-task.md).
Evidence: `orcvs/src/playback.rs` `run_engine`; test `adapter_failure_does_not_roll_back_source_or_stop_playback` in the same file.

### PLAY-050 One task owns the engine's state and publishes what callers observe

The engine's state lives in one task, which owns its clock and waits on requests and its next
deadline together; it is not shared behind a lock. Lifecycle state, the selected destination and the
current Tick and Run Clock are published on a value read without waiting; diagnostics are drained
without blocking. No caller observation waits on the task.

Reason: the browser main thread cannot block, and owning the state removes the mechanisms sharing it
required.
Sources: [ADR 0041](../adr/0041-the-playback-engine-owns-its-state-in-one-task.md).
Evidence: `orcvs/src/playback.rs` `run_engine`, `PlaybackEngine::observation_watch`; test `beginning_a_run_publishes_tick_zero_playing_and_a_run_origin` in the same file.

### PLAY-060 MIDI selection is a capability of MIDI construction only

MIDI construction returns the lifecycle handle together with a MIDI selection handle; only it
publishes the selected destination. Output-only construction has no selection method and no
destination channel. The console opens a port on its own thread and hands the open connection over
as data; the task stays sole owner of adapter, connection, schedule and lifecycle state. Accepting a
request is not installing it: a refused delivery is an ordered diagnostic. Discovery is not
published. Requests are data dispatched by a handler fixed at construction, never closures. The
selection handle holds a weak reference: it cannot keep Playback alive or control its lifecycle.
Choosing an output device is console configuration, not part of a running Orcvs.

Reason: general Playback does not speak MIDI, and frame reads stay nonblocking.
Sources: [ADR 0041](../adr/0041-the-playback-engine-owns-its-state-in-one-task.md),
[ADR 0056](../adr/0056-playback-requests-wait-in-slots-not-a-queue.md).
Evidence: `orcvs/src/playback.rs` `PlaybackEngine::new`, `with_midi_output_adapter`; `orcvs/src/midi/selection.rs` `MidiSelectionHandle`; test `orcvs/tests/midi_selection_handle.rs` `selection_handle_cannot_outlive_the_running_orcvs`.

### PLAY-070 Requests wait in bounded slots and apply in a fixed order

Every request a Playback or MIDI selection handle makes has a slot: one stop flag, one Rewind flag,
one period to begin a run at, one period to retune to, and one destination change (a disconnect or
one open MIDI connection). A later request overwrites or merges with its slot; nothing queues. The
task takes the whole backlog at once and applies it in the order stop, Rewind, tempo, destination.
A destination change releases the request it replaces on the caller's thread; a connection offered
after the task has gone is refused and dropped; the task drops any pending connection when it ends.
The mailbox lock is held only for a constant-time update, never across a wait or a Tick.

Reason: memory is bounded by shape rather than by rate, with no request refused for lack of room.
Not: a bounded channel, which would make a full queue a failure that `stop` cannot wait out.
Sources: [ADR 0056](../adr/0056-playback-requests-wait-in-slots-not-a-queue.md),
[ADR 0076](../adr/0076-stopping-playback-holds-its-tick.md).
Evidence: `orcvs/src/playback/mailbox.rs` `Backlog`; test `orcvs/src/playback/mailbox.rs` `a_rewind_survives_a_stop_on_either_side_and_leaves_the_tempo`.

### PLAY-080 A backlog leaves the engine as the requests made in order would, with stated exceptions

A stop empties both tempo slots, so a `start` then `stop` in one backlog is no run. `start` writes
the begin slot and `retune` the retune slot; the task applies whichever matches the state it finds
after the stop. A newer `start` replaces a pending one's period, and a `retune` made while a `start`
is pending also sets that start's period. A destination change commutes with tempo and stop. A
Rewind and a stop in either order leave a stopped engine whose next Tick is Tick `0`. Accepted
differences from strict order: `start(1s)`, `retune(2s)`, `start(3s)` from stopped begins at 3s; a
`disconnect` replacing a pending installation leaves the earlier device published; a backlog may
silence a device twice or send a stop's silence to the outgoing device.

Reason: only the newest request of each kind matters, and no Tick runs between the parts of one
backlog for order to show in.
Sources: [ADR 0056](../adr/0056-playback-requests-wait-in-slots-not-a-queue.md),
[ADR 0076](../adr/0076-stopping-playback-holds-its-tick.md).
Evidence: `orcvs/src/playback/mailbox.rs`; test `repeated_starts_keep_the_newest_period` in the same file.

### PLAY-090 A run's first Tick follows exactly the backlog that began it, and the clock always progresses

Every request pending when the backlog containing the `start` was taken is applied before the first
Tick; requests made after it wait for the next Tick. The first Tick is due on arrival and takes no
further backlog. An engine no handle holds executes no Tick. Once a live run has applied 64
backlogs since its last Tick, a reached deadline is taken ahead of a pending backlog.

Reason: a caller that keeps making requests cannot defer the first Tick or starve the clock.
Sources: [ADR 0056](../adr/0056-playback-requests-wait-in-slots-not-a-queue.md).
Evidence: `orcvs/src/playback.rs` `next_playback_event`, `BACKLOGS_BEFORE_A_DEADLINE`; test `a_run_s_first_tick_follows_exactly_the_backlog_pending_when_it_began` in the same file.

### PLAY-100 No call waits on the task, and only a gone task is refused

`start` and `retune` return a result that is an error only when the task has gone; `start` also
records that refusal as a start-failure diagnostic. `stop` and `disconnect` return nothing. A MIDI
installation answers that the running Orcvs is no longer available when the task or every engine
handle has gone, and drops the connection; a request later superseded was accepted and is not
reported. Dropping the last engine handle applies whatever was pending, sends the safety action and
shuts the engine down.

Reason: the browser main thread cannot wait, and slots never fill.
Sources: [ADR 0056](../adr/0056-playback-requests-wait-in-slots-not-a-queue.md).
Evidence: `orcvs/src/playback.rs` `PlaybackEngine::start`; test `an_engine_whose_task_died_refuses_a_later_start` in the same file.

## Musical time: the Tick Grid, stop and Rewind

### PLAY-150 A stop holds the absolute Tick

Stopping ends the run, sends the safety action and discards the Note Off schedule; the absolute Tick
survives. The next run begins at the Tick after the last one executed, with its first Tick
immediate and a Tick Grid of its own: the stopped interval is neither skipped as Overruns nor owed.
A stopped Playback is what Orca calls paused; there is no separate paused state. Replacing the
running Orcvs (opening a Source, File New, the Function Reference) begins at Tick `0`.

Reason: every voice of a tracker resumes on the step it stopped on, as in Orca.
Not: a Paused state beside Stopped.
Sources: [ADR 0076](../adr/0076-stopping-playback-holds-its-tick.md) (amends ADR 0012's first-Tick
clause).
Evidence: `orcvs/src/playback.rs` `PlaybackInner::begin_run`; test `a_run_begun_after_a_stop_resumes_at_the_next_tick` in the same file.

### PLAY-160 A Rewind makes Tick `0` the next Tick

A Rewind of a stopped engine leaves it stopped with Tick `0` next. A Rewind of a live run stops it,
with the safety action and the schedule discarded, and begins the next run at once at Tick `0` at
the same period; if that safety action panics, the engine stays stopped at Tick `0`. A Rewind shuts
no gate: a Tick already admitted executes as the old run's last. Rewind is applied after stop and
before tempo (PLAY-070).

Reason: every scheduled Note Off is due at an absolute Tick, so silencing reuses the rule a stop
follows.
Not: rebasing the schedule across a live Rewind.
Sources: [ADR 0076](../adr/0076-stopping-playback-holds-its-tick.md).
Evidence: `orcvs/src/playback.rs` `PlaybackEngine::rewind`; test `a_rewind_makes_tick_zero_the_next_tick_of_a_stopped_engine` in the same file.

### PLAY-170 The Run Clock is held while stopped

The published Run Clock is held while Playback is stopped, counts on from the held value when a run
resumes, and is zero after a Rewind, together with a published Tick of `0`.

Sources: [ADR 0076](../adr/0076-stopping-playback-holds-its-tick.md).
Evidence: `orcvs/src/playback.rs` `PlaybackObservation::run_clock`; test `stopping_freezes_the_published_tick_and_run_clock` in the same file.

### PLAY-180 Each run's Ticks are due on its Tick Grid, which a stall does not move

Every Tick of a run is due at a whole multiple of the Tick period from the deadline the run began
on, until the run ends or is retuned. A deadline the clock cannot reach in time is skipped, the
deadlines passed behind it are not manufactured, and the run resumes at the first deadline still
ahead of the instant the clock woke; a deadline exactly at that instant goes with the stall.
Retuning begins a new grid without a new run, anchored on the deadline the last executed Tick was
due at, never on when the retune arrived or that Tick was seen. Both targets compute deadlines with
one shared function; only how to wait until an instant differs per target.

Reason: the grid is what a sequencer cannot give up.
Not: replaying the backlog, or restarting the period from the wake, which moves the grid for good.
Sources: [ADR 0037](../adr/0037-a-late-tick-loses-its-turn.md),
[ADR 0041](../adr/0041-the-playback-engine-owns-its-state-in-one-task.md).
Evidence: `orcvs/src/playback.rs` `next_scheduled_at`; test `a_missed_deadline_does_not_move_the_grid` in the same file.

### PLAY-190 A Tick observed a whole period late is declined as one Overrun

A Tick observed a whole Tick period or more past its deadline is declined with one Overrun diagnostic
naming that deadline; a Tick late by less is executed. An Overrun consumes no absolute Tick and is
reported once per stall, not once per deadline the stall covered. The engine, not the clock, decides
whether a Tick it was handed is still worth executing.

Reason: the count reports the event, not the clock's arithmetic, and an ordinary slow Tick costs
nothing.
Sources: [ADR 0037](../adr/0037-a-late-tick-loses-its-turn.md).
Evidence: `orcvs/src/playback.rs` `is_overrun`; test `a_late_tick_loses_its_turn_and_the_run_resumes_on_the_grid` in the same file.

## Diagnostics

### PLAY-110 What an engine retains between drains is bounded

At most 32 diagnostics are retained between two drains, each message cut to at most 256 bytes on a
character boundary and marked `…`, in an allocation made once when the engine is built.

Reason: an engine nobody drains must not grow with what its run does.
Sources: [ADR 0055](../adr/0055-playback-diagnostics-are-retained-within-a-bound.md).
Evidence: `orcvs/src/playback/diagnostics.rs` `MAX_RETAINED_DIAGNOSTICS`, `MAX_DIAGNOSTIC_MESSAGE_BYTES`; test `a_long_message_is_cut_on_a_character_boundary_within_the_bound` in the same file.

### PLAY-120 A full log evicts by class, least serious first

Classes, first evicted to last: Overrun; start and retune failures, as one tier; output failure;
clock failure. A diagnostic recorded into a full log evicts the oldest retained entry of the lowest
class present, provided that class is no higher than its own; otherwise the new diagnostic is
omitted. Retained entries keep their recorded order, so the newest diagnostic of the most serious
class since the last drain is always retained.

Built differently; see [Accepted but not built](#accepted-but-not-built-or-built-differently).

Reason: a flood of one class never displaces a more serious diagnostic.
Not: dropping the newest, or the oldest regardless of class.
Sources: [ADR 0055](../adr/0055-playback-diagnostics-are-retained-within-a-bound.md).
Evidence: `orcvs/src/playback/diagnostics.rs` `Class`; test `a_higher_class_evicts_the_oldest_of_the_lowest_class_present` in the same file.

### PLAY-130 Nothing is omitted silently

Every eviction and omission is counted against its class as Omitted Diagnostics. A drain that
omitted anything ends with one summary of the counts and resets them; only a drain builds it, and
the log never retains a summary. Each count saturates and is then a lower bound. The Overruns since
the last drain are those the drain answers plus the omitted Overrun count.

Sources: [ADR 0055](../adr/0055-playback-diagnostics-are-retained-within-a-bound.md).
Evidence: `orcvs/src/playback/diagnostics.rs` `OmittedDiagnostics`; test `an_omission_count_saturates_and_says_so` in the same file.

### PLAY-140 Diagnostics drain in recorded order, each at most once

The task and every handle append to one shared log, held for one append or one drain and never
across a Tick or a delivery. A drain takes everything retained, in the order it was recorded across
both writers.

Reason: one shared value keeps two writers' diagnostics in one order.
Sources: [ADR 0002](../adr/0002-playback-engine-owns-lifecycle-concurrency.md),
[ADR 0055](../adr/0055-playback-diagnostics-are-retained-within-a-bound.md).
Evidence: `orcvs/src/playback/diagnostics.rs` `DiagnosticLog`; test `orcvs/src/playback.rs` `diagnostics_drain_in_order_and_exactly_once`.

## MIDI commands, note ownership and the safety action

### PLAY-200 MIDI operands are direct values in their protocol domains

Channel is a Number `00`–`0F`; velocity, controller, value, LSB and MSB are Numbers `00`–`7F`;
note is a Note; length is a Number `00`–`FF`. Values are sent as written, with no base-36 scaling,
wrapping or clamping, and no Number is taken as a Note without `.^`. An operand out of its domain or
of the wrong type diagnoses and emits no MIDI. Which operands may be blank, and what blank binds to,
is in [LANG-300](language.md#lang-300-an-optional-operand-binds-a-slot-holding-no-value-to-no-value-a-plays-note-is-optional) and [LANG-320](language.md#lang-320-a-defaulted-operand-binds-a-slot-holding-no-value-as-its-fixed-number); the per-Function table is in
[docs/functions.md](../functions.md#midi-output).

Sources: [ADR 0016](../adr/0016-use-distinct-raw-and-timed-play-functions.md).
Evidence: `lang/src/functions/mod.rs`; test `lang/src/functions/mod.rs` `play_rejects_channels_outside_the_midi_range`.

### PLAY-210 A Raw Play at velocity `00` is a Note Off, and Raw notes are never owned

Raw Play leaves Note Off to Source: velocity `00` delivers a Note Off for its note. Raw notes enter
neither Timed nor Mono ownership.

Sources: [ADR 0016](../adr/0016-use-distinct-raw-and-timed-play-functions.md).
Evidence: `orcvs/src/playback/schedule.rs` `OwnedNotes`; test `a_raw_play_at_velocity_00_is_delivered_as_a_note_off` in the same file.

### PLAY-220 A Timed Play owns its note by channel and note until its expiry

After its operands validate, a Timed Play at velocity `00` stops that note in Tick Plan order and
schedules nothing. At nonzero velocity, length `00` is a no-op: nothing is sent and any prior claim
and expiry are untouched. Otherwise a command at Tick `T` with length `N` stops any prior Timed-owned
instance of that channel and note, retiring its expiry, starts the note in Tick Plan order, and
schedules its Note Off at the beginning of executed Tick `T + N`, before that Tick's new commands.
Several Timed Plays for one channel and note in one Tick leave the last owning it.

Reason: the Tick Plan carries the note's whole lifetime, so the engine schedules without inferring
intent.
Sources: [ADR 0016](../adr/0016-use-distinct-raw-and-timed-play-functions.md).
Evidence: `orcvs/src/playback/schedule.rs`; test `a_timed_play_starts_in_tick_plan_order_and_stops_at_the_tick_its_length_names` in the same file.

### PLAY-230 A Monophonic Play owns one voice per channel

Every Monophonic Play that emits a Play Command replaces the channel's Mono-owned claim, whatever
note it held. A Monophonic Play whose note is blank emits none, so the voice sounds on to its own
Note Off ([LANG-300](language.md#lang-300-an-optional-operand-binds-a-slot-holding-no-value-to-no-value-a-plays-note-is-optional)). Velocity `00`
or length `00` starts nothing, replacing the voice with silence (unlike Timed Play's length `00`).
Otherwise the replacement starts and its Note Off is scheduled at executed Tick `T + length`.
Channels are independent; the last command in a Tick owns the channel. Retriggered replacement and
legato replacement differ only in message order (PLAY-240).

Reason: the voice is the channel, so Source need not remember what it last put there.
Sources: [ADR 0016](../adr/0016-use-distinct-raw-and-timed-play-functions.md),
[ADR 0108](../adr/0108-a-plays-note-is-optional-by-type.md).
Evidence: `orcvs/src/playback/schedule.rs`; test `a_monophonic_play_stops_whatever_note_its_channel_was_sounding` in the same file.

### PLAY-240 `legato` sets the order of a Monophonic replacement's messages

With `legato` `00` (or blank, which binds `00`), the replacement retriggers: the prior note's Note
Off, then the replacement's Note On. With any other value it plays legato: the replacement's Note On,
then the prior note's Note Off. A legato replacement at the pitch already sounding sends its Note On
and no Note Off; the prior claim is retired and the note sounds on to the replacement's own end. A
legato replacement on the Tick the prior note's stop is due takes that stop over: it is delivered
after the Note On, or not at all at the same pitch. A legato replacement on a silent channel starts
as any note does. Velocity `00` or length `00` starts nothing whatever the `legato`, and still stops
the prior note. The ADR records these edge rules as awaiting the maintainer's confirmation; they
bind as written.

Reason: a mono synth hears a key still held and slides; a line whose lengths equal its steps still
slides.
Sources: [ADR 0016](../adr/0016-use-distinct-raw-and-timed-play-functions.md) (2026-11-14
amendment), [ADR 0099](../adr/0099-one-built-in-synth-channel-plays-a-303-style-voice.md).
Evidence: `orcvs/src/playback/schedule.rs`; test `a_legato_monophonic_play_starts_its_note_before_it_stops_the_one_it_replaces` in the same file.

### PLAY-250 Timed and Mono claims share one schedule and differ only in key

One schedule holds Timed claims, keyed by channel and note, and Mono claims, keyed by channel. Each
ownership change takes a generation token, and a replacement invalidates the prior token, so a stale
expiry cannot stop a later note. Neither ownership can name the other's voice. The independence is
the engine's: a Note Off carries only channel and note, so notes of different spellings at one pitch
on one channel alias at the receiver.

Sources: [ADR 0016](../adr/0016-use-distinct-raw-and-timed-play-functions.md).
Evidence: `orcvs/src/playback/schedule.rs`; test `a_stale_expiry_cannot_stop_the_note_claimed_after_it` in the same file.

### PLAY-260 One safety action returns every MIDI channel to silence and its defaults

Stopping, rewinding a live run, disconnecting, changing destination, dropping the last engine handle
and teardown after a refused delivery each send the safety action and end all Timed and Mono
ownership. The action is, for each of the sixteen channels in turn, CC 123 All Notes Off, CC 121
Reset All Controllers, then Pitch Bend `[0xE0 | channel, 0x00, 0x40]`. No Note Off is sent for
notes the engine knows it started. A channel whose message the destination refuses does not end the
action: the first refusal is reported and every remaining channel is still attempted. The action
reaches MIDI alone.

Built differently after a refused delivery; see [Accepted but not built](#accepted-but-not-built-or-built-differently).

Reason: a Control Change can latch a pedal and a Pitch Bend deflect the wheel on a device Orcvs can
no longer reach; the centred bend comes last so a device honouring CC 121 cannot move the wheel
after it.
Sources: [ADR 0016](../adr/0016-use-distinct-raw-and-timed-play-functions.md),
[ADR 0076](../adr/0076-stopping-playback-holds-its-tick.md),
[ADR 0090](../adr/0090-each-output-destination-keeps-its-policy-in-one-module.md); the
remaining-channels clause: no ADR (`.scratch/midi-output-family/issues/06-clear-controllers-and-bend-in-the-safety-action.md`).
Evidence: `orcvs/src/midi.rs` `safety_reset_messages`, `MidiOutputAdapter::safety_reset`; test `the_safety_action_clears_notes_controllers_and_bend_on_every_channel` in the same file.

### PLAY-270 Lifecycle actions discard the schedule rather than play it out

Beginning a run, stopping, rewinding a live run, disconnecting and changing destination clear every
claim and scheduled Note Off, so the safety action is one fixed run of messages, a run never begins
owning a note, and the first Monophonic Play after a resume finds its voice empty. Nothing is owned
while MIDI is disconnected.

Sources: [ADR 0016](../adr/0016-use-distinct-raw-and-timed-play-functions.md),
[ADR 0076](../adr/0076-stopping-playback-holds-its-tick.md),
[ADR 0041](../adr/0041-the-playback-engine-owns-its-state-in-one-task.md).
Evidence: `orcvs/src/playback/outputs.rs` `Outputs::release`, `disconnect`, `begin_run`; test `orcvs/src/playback.rs` `every_lifecycle_action_that_silences_output_clears_the_note_schedule`.

## Outputs and the network

### PLAY-350 The console keeps the network destination, never Source

UDP and OSC go to one host with a UDP port and an OSC port. The console stores them itself, apart
from the settings file it only reads (CONS-130), and never in Source; defaults are `127.0.0.1`,
`49161` and `49162`. A new destination applies from the next send. The Output list edits it
(CONS-104).

Sources: [ADR 0085](../adr/0085-udp-and-osc-send-the-cells-up-to-their-closer.md).
Evidence: `orcvs/src/network.rs` `NetworkDestination`; test `the_default_destination_is_orcas` in the same file.

### PLAY-360 The web build diagnoses every activated UDP or OSC Function

A browser cannot open a UDP socket, so in the web build every activated `!u` or `!o` diagnoses that
the build cannot send UDP or OSC, unlatched, and Playback goes on. Both builds parse a Source alike.

Reason: there each send is the performer's evidence that nothing left.
Sources: [ADR 0085](../adr/0085-udp-and-osc-send-the-cells-up-to-their-closer.md),
[ADR 0090](../adr/0090-each-output-destination-keeps-its-policy-in-one-module.md).
Evidence: `orcvs/src/network.rs` web adapter; test `orcvs/src/playback/outputs.rs` `the_web_build_reports_every_refused_send`.

### PLAY-370 Orcvs only sends UDP and OSC

Orcvs does not receive UDP or OSC.

Sources: [ADR 0085](../adr/0085-udp-and-osc-send-the-cells-up-to-their-closer.md).
Evidence: no receive path exists in `orcvs/src/network.rs`.

### PLAY-380 One Outputs module owns every destination and its policy

Within the Playback Engine, one Outputs module takes each Tick Plan and delivers it. It alone holds
the MIDI adapter, the note schedule, MIDI connection state, each destination's failure latch and
the network output. A further destination is one more entry in it.

Reason: destination rules otherwise spread across engine, schedule and network output.
Sources: [ADR 0090](../adr/0090-each-output-destination-keeps-its-policy-in-one-module.md).
Evidence: `orcvs/src/playback/outputs.rs` `Outputs`; test `network_sends_go_on_while_midi_is_disconnected` in the same file.

### PLAY-390 Network Sends travel beside Play Commands, and a Tick's MIDI goes first

A Tick Plan carries Network Sends in their own ordered list beside the Play Commands. Order holds
within each list; across them a Tick's MIDI is delivered before its Network Sends, whatever their
Source order.

Reason: interleaving would split one MIDI submission for no audible difference.
Sources: [ADR 0090](../adr/0090-each-output-destination-keeps-its-policy-in-one-module.md).
Evidence: `orcvs/src/playback/outputs.rs` `Outputs::deliver`; test `a_ticks_midi_is_delivered_before_its_network_sends` in the same file.

### PLAY-400 The network ignores MIDI's connection and lifecycle

Network Sends go on while no MIDI destination is connected. Stopping, rewinding, disconnecting and
the safety action send nothing on UDP or OSC, which hold no receiver state Orcvs knows how to reset.

Sources: [ADR 0085](../adr/0085-udp-and-osc-send-the-cells-up-to-their-closer.md),
[ADR 0090](../adr/0090-each-output-destination-keeps-its-policy-in-one-module.md),
[ADR 0016](../adr/0016-use-distinct-raw-and-timed-play-functions.md) (2026-10-26 amendment).
Evidence: `orcvs/src/playback/outputs.rs`; test `network_sends_go_on_while_midi_is_disconnected` in the same file.

### PLAY-410 A failing destination is reported once until its failure changes

A MIDI delivery failure and a native network send failure are each latched: reported once until a
different failure occurs, cleared by a successful delivery or send, a new destination, or the start
of a run. Whether the latest MIDI delivery and the latest Network Send failed is published on the
Playback observation, so a recovery shows as the failure clearing. On Windows a `ConnectionReset`
from a send reports an earlier datagram, so it is ignored; every other failed send is reported.

Reason: a refusing host otherwise floods the log and gives no sign of recovery.
Sources: [ADR 0090](../adr/0090-each-output-destination-keeps-its-policy-in-one-module.md),
[ADR 0085](../adr/0085-udp-and-osc-send-the-cells-up-to-their-closer.md).
Evidence: `orcvs/src/playback/outputs.rs` `record_midi_failure`, `record_network_failure`; test `a_native_network_failure_is_reported_once_until_it_changes` in the same file.

### PLAY-420 The network sits behind a seam that takes a Network Send

The network adapter takes a Network Send and its destination. The native adapter encodes and sends
one datagram with a pure encoder; the web adapter refuses without encoding.

Not: a seam taking raw datagrams, which would make the web encode OSC only to refuse it.
Sources: [ADR 0090](../adr/0090-each-output-destination-keeps-its-policy-in-one-module.md).
Evidence: `orcvs/src/network.rs` `NetworkAdapter`; test `a_refused_send_names_where_it_went` in the same file.

## UDP and OSC Functions and the Payload

### PLAY-300 A Payload runs from its spelling to the next `!` on its row

A Payload is the Cells from the end of a `!u` or `!o` spelling through the next `!` on its row,
which closes it, or to the row's end. The Parser finds the closer by search, so the extent is fixed
at parse. A Payload is never parsed, cannot contain `!`, and has no escapes. Cells after a closer
are ordinary Source.

Not: Orca's first-empty-Cell end, which would bar spaces and move with every edit after it.
Sources: [ADR 0085](../adr/0085-udp-and-osc-send-the-cells-up-to-their-closer.md).
Evidence: `lang/src/parser.rs`; test `lang/src/parser.rs` `a_payload_claims_through_the_next_bang_or_the_end_of_the_source`.

### PLAY-310 A Payload's Cells are ordinary Cells, read at its Function's Turn

Only the spelling and the closer are occupied. A Write writes into a Payload and a Read reads from
it; the Function reads its Payload after every writer of those Cells has taken its Turn, so a Write
lands in the same Tick. The extent holds for the whole Tick: a `!` written inside is sent as a
character, and a written or removed closer changes the extent from the next parse.

Sources: [ADR 0085](../adr/0085-udp-and-osc-send-the-cells-up-to-their-closer.md).
Evidence: `orcvs/src/source/portal.rs` `read_payload`; test `orcvs/src/source/tick/payload.rs` `a_write_into_a_payload_lands_in_the_same_tick`.

### PLAY-320 `!u` sends its Payload as one datagram, one byte per Cell

Each Cell sends its byte and an empty Cell a space, as one UDP datagram with no terminator. An
empty Payload sends nothing.

Sources: [ADR 0085](../adr/0085-udp-and-osc-send-the-cells-up-to-their-closer.md).
Evidence: `lang/src/functions/network.rs`; test `udp_sends_each_cell_as_one_byte_and_an_empty_payload_rests` in the same file.

### PLAY-330 `!o` sends an address, an optional type string and typed arguments

The Payload's words are separated by runs of spaces. The first word is the address and starts with
`/`. A next word starting with `,` is a type string of `i`, `f` and `s`, one per argument. Without
one, one to eight uppercase hexadecimal digits are an int32 whose bits they spell; a decimal with a
point, optional leading `-` and a digit on each side is a float32; any other word is a string. Under
a type string, an `i` word must be such hex, an `f` word such a decimal, and an `s` word is sent as
written. A bad address, type letter, type-string length or word diagnoses and sends nothing. The
exact forms are tabled in [docs/functions.md](../functions.md#network-output).

Reason: receivers dispatch on OSC type, and hex is what a Number is everywhere else in the Grid.
Sources: [ADR 0085](../adr/0085-udp-and-osc-send-the-cells-up-to-their-closer.md).
Evidence: `lang/src/functions/network.rs`; test `osc_guesses_each_argument_from_its_shape` in the same file.

### PLAY-340 Each activation sends one plain message in Tick Plan order

An activated `!u` or `!o` emits one Network Send, delivered when its Tick runs; OSC sends no bundle
and no timetag. Like every Terminal Output Function, each performs only when activated, is invalid
where a value is required, and writes no Cell.

Sources: [ADR 0085](../adr/0085-udp-and-osc-send-the-cells-up-to-their-closer.md).
Evidence: test `orcvs/src/source/tick/payload.rs` `an_unbanged_payload_function_sends_nothing`.

## MIDI destinations and browser MIDI access

### PLAY-430 The browser asks for MIDI access once and answers discovery from it

The browser backend requests MIDI access when it is built and keeps the answer; discovery and
connect read it synchronously. While pending, both answer a pending error, marked as one that can
change without the performer acting. Once granted, discovery lists the connected outputs in the
output map's order, a nameless port by its id. A browser without Web MIDI, or one that refuses,
answers an empty list rather than an error, logs the reason once, and is not asked again. A target
that is neither native nor browser lists nothing and refuses every connect.

Reason: the console's MIDI calls return to the frame that made them, and the browser main thread
cannot block.
Sources: [ADR 0059](../adr/0059-the-browser-answers-midi-discovery-from-access-it-already-holds.md).
Evidence: `console/src/web_midi.rs`; test `a_pending_request_answers_that_access_is_awaited` in the same file.

### PLAY-440 The access answer wakes the Panel, and a pending discovery is retried each frame

When access settles, the answer is stored and the Panel is asked to repaint; the callback writes no
menu state. Each frame whose last discovery answered pending discovers again in that frame, so a
granted page selects its first output without a Scan. Any other discovery failure stands until the
performer Scans.

Sources: [ADR 0059](../adr/0059-the-browser-answers-midi-discovery-from-access-it-already-holds.md).
Evidence: `console/src/web_midi.rs`, `console/src/midi.rs`; test `console/src/web_midi.rs` `the_frame_after_the_browser_answers_catches_up_without_a_scan`.

### PLAY-450 The web console waits at most 250 ms for the access answer before starting

The web entry point sends the access request and starts the console when the answer is stored or
250 ms pass, whichever is first; a browser without Web MIDI is not waited for.

Reason: a remembered permission answers well inside the bound, while a first-visit prompt must not
gate a console that is useful without MIDI.
Sources: [ADR 0059](../adr/0059-the-browser-answers-midi-discovery-from-access-it-already-holds.md).
Evidence: `console/src/main.rs` `MIDI_ACCESS_WAIT`; test `console/tests/wasm.rs` `the_midi_access_wait_ends_whether_or_not_the_browser_answers`.

### PLAY-460 Connect returns an owned request and installs nothing until the port opens

Connect returns either an open connection (native) or an owned pending open (browser) whose
completion a nonblocking poll observes. The selection keeps the request between frames rather than
repeating connect. A failed open is reported while the installed destination keeps playing and
receives no safety action.

Sources: [ADR 0059](../adr/0059-the-browser-answers-midi-discovery-from-access-it-already-holds.md).
Evidence: `console/src/web_midi.rs` `MidiConnectionRequest`; test `an_output_that_fails_to_open_leaves_the_playing_destination` in the same file.

### PLAY-470 Browser ports are owned at page scope

One page-wide lifecycle module counts pending requests and live connections against each output.
Dropping a pending request releases its claim; success transfers it to the connection. An open that
completes with no owner is closed; a claim arriving before completion shares that open; failed
opens are reported to each owner; a claim arriving during a close waits for it before reopening.
These transitions live in the shared lifecycle module, which tests drive through a fake below it.

Reason: a new backend is built before the outgoing Playback task has shut down, so backend-local
counts cannot describe a page-wide port.
Sources: [ADR 0059](../adr/0059-the-browser-answers-midi-discovery-from-access-it-already-holds.md).
Evidence: `console/src/web_midi/ports.rs`; test `console/src/web_midi.rs` `an_abandoned_open_releases_its_output`.

### PLAY-480 A browser connection names its output by id and requests no sysex

A connection crossing into the Playback Engine's task names its output by id and looks it up in the
main thread's kept access on every send, with no unsafe `Send`; a disconnected output refuses the
next send. Playback drops a connection after its safety action. MIDI access is requested without
system exclusive.

Sources: [ADR 0059](../adr/0059-the-browser-answers-midi-discovery-from-access-it-already-holds.md).
Evidence: `console/src/web_midi.rs` (`thread_local!`, `request_midi_access`); test `a_connect_to_a_gone_or_disconnected_output_is_refused` in the same file.

### PLAY-500 An explicit destination choice decides both pending and installed state

The destination selection module owns every explicit choice, including one naming the installed
destination. Choosing the installed A while B is opening drops B's request and leaves A and Playback
untouched, with no reinstall and no safety action. Choosing a pending B again keeps the same
request; after a failed open, a fresh choice retries. A native choice handed over but not yet
published is not reconnected by choosing it again.

Sources: [ADR 0059](../adr/0059-the-browser-answers-midi-discovery-from-access-it-already-holds.md).
Evidence: `console/src/midi.rs`, `console/src/web_midi.rs`; test `choosing_the_installed_output_cancels_without_safety_messages` in the same file.

### PLAY-510 A Scan asks the browser again while access is unanswered

While access is pending, each Scan sends a fresh access request; frames never do. Any grant settles
access as granted; a refusal counts only when it answers the latest request. A settled answer is
not asked again.

Reason: a browser can leave a request unanswered for good, and Scan is the performer's explicit ask.
Sources: [ADR 0059](../adr/0059-the-browser-answers-midi-discovery-from-access-it-already-holds.md)
(2026-11-01 amendment).
Evidence: `console/src/web_midi/requests.rs`; test `a_stale_refusal_leaves_the_latest_request_pending` in the same file.

### PLAY-520 The Built-in Synth is a destination listed after every MIDI port

While a framing page announces a Built-in Synth, it is offered after every MIDI port and chosen like
any destination. Automatic selection picks it only when no port exists; while MIDI access is
pending, automatic selection picks the first destination listed until discovery settles, after
which the first destination replaces it; a destination the performer chose stays chosen. The page
messages and the Linked Console's pick are in [site.md](site.md) (SITE-040, SITE-041); an
unavailable synth in SITE-046; a console no page frames in SITE-048. How the list is drawn is in
[CONS-104](console.md#cons-104-the-output-list-has-one-labelled-section-per-output-type).

Sources: [ADR 0087](../adr/0087-the-site-plays-the-console-through-a-built-in-synth.md),
[ADR 0059](../adr/0059-the-browser-answers-midi-discovery-from-access-it-already-holds.md)
(2026-11-01 amendment).
Evidence: `console/src/built_in_synth.rs`; test `the_synth_is_listed_after_the_ports_only_while_a_page_announces_it` in the same file.

### PLAY-540 Each message carries its Tick's planned time

Playback hands each destination, with every message, how late its Tick is, so the destination can
recover the Tick's planned instant; a destination may ignore it. The Built-in Synth's fixed delay
and its handling of late messages are in [site.md](site.md) (SITE-042, SITE-043).

Reason: a note sounds on its Tick, not when it arrives.
Sources: [ADR 0087](../adr/0087-the-site-plays-the-console-through-a-built-in-synth.md).
Evidence: `orcvs/src/playback/outputs.rs` `Outputs::deliver`; test `orcvs/src/midi.rs` `a_submission_hands_each_message_how_late_its_tick_was`.

## The Built-in Synth's sound

### PLAY-550 One Rust synth core renders the Built-in Synth

The synth core takes MIDI bytes and a time and fills a buffer, with no audio I/O. On the web it runs
as its own WASM module in an AudioWorklet on the browser's audio thread. Native plays the same core
through cpal. The console holds no audio code.

The native path is not built; see [Accepted but not built](#accepted-but-not-built-or-built-differently).

Reason: neither page work nor console frames can starve the audio, and native can reuse the core.
Not: a synth on the page's main thread, which snaps notes to buffer edges.
Sources: [ADR 0087](../adr/0087-the-site-plays-the-console-through-a-built-in-synth.md).
Evidence: `synth/src/lib.rs` `Synth`; `synth/src/web.rs`; `synth/web/synth-worklet.js`; test `synth/src/lib.rs` `a_message_sounds_from_the_frame_it_is_scheduled_at`.

### PLAY-570 Channels `00`–`07` play tones, `08` the acid voice, `09`–`0F` the kit

Tones are on `00`–`07` (sine, triangle, saw, square, two FM voices, two AM voices), the acid voice
on `08` (PLAY-610), and an 808-style kit on `09`–`0F`, so MIDI channel 10 is a drum channel as in
General MIDI. Tone channels share one short envelope; a Note Off releases a tone over 150 ms; one
channel can sound a chord. The channel table is in [docs/functions.md](../functions.md#built-in-synth).

Sources: [ADR 0087](../adr/0087-the-site-plays-the-console-through-a-built-in-synth.md),
[ADR 0099](../adr/0099-one-built-in-synth-channel-plays-a-303-style-voice.md) (2026-11-13
amendment).
Evidence: `synth/src/tone.rs`, `synth/src/lib.rs`; test `synth/src/lib.rs` `channels_zero_to_seven_play_tones_eight_the_acid_voice_and_nine_to_fifteen_the_kit`.

### PLAY-580 The kit answers General MIDI drum keys

The kit is synthesised, with no recordings, and picks each instrument by General MIDI drum key;
every key it does not list is silent. A drum rings out its own decay and ignores Note Off. A closed
or pedal hat chokes an open hat on its channel. The key table is in
[docs/functions.md](../functions.md#built-in-synth).

Sources: [ADR 0087](../adr/0087-the-site-plays-the-console-through-a-built-in-synth.md)
(2026-11-13 amendment).
Evidence: `synth/src/drum.rs`; test `synth/src/lib.rs` `the_keys_of_a_family_sound_alike`.

### PLAY-590 Velocity is a channel's only level, and sixteen voices sound at once

Velocity sets loudness; there is no volume control. Sixteen voices sound at once, and a Note On
past that takes the oldest voice. All Sound Off (CC 120) and All Notes Off (CC 123) release the
channel's tones and acid voice, while drums ring out. The synth acts on Note On, Note Off and the
Control Changes in PLAY-600, PLAY-610 and PLAY-620, and ignores Pitch Bend and every other message.

Sources: [ADR 0087](../adr/0087-the-site-plays-the-console-through-a-built-in-synth.md),
[ADR 0088](../adr/0088-the-sound-module-shows-the-synths-channels-and-effects-as-cells.md),
[ADR 0099](../adr/0099-one-built-in-synth-channel-plays-a-303-style-voice.md).
Evidence: `synth/src/lib.rs` `VOICES`; test `velocity_sets_loudness` in the same file.

### PLAY-600 Each channel has its own eight effects on standard controllers

Each channel's Control Changes set that channel's effects: `MD` vibrato (CC 1), `EQ` tilt (CC 74),
`LN` length (CC 75), `OF` offset (CC 85), and sends `R1` (CC 91), `R2` (CC 92), `CH` (CC 93) and `DL`
(CC 94) into one small reverb, one large reverb, one chorus and one echo that every channel shares,
each returning only what was sent to it. `MD` and `EQ` process the channel before its sends; `OF`
and `LN` shape each new sound. At load values a channel sounds as with no effect, and values reset on
every load. On `08`, CC 74 and 75 are the acid voice's cutoff and decay instead of tilt and length.
CC 2 to 8 set nothing. Load values and names are tabled in
[docs/functions.md](../functions.md#built-in-synth).

Reason: one channel's controllers never change another channel, as on a musician's other gear.
Sources: [ADR 0088](../adr/0088-the-sound-module-shows-the-synths-channels-and-effects-as-cells.md)
(2026-11-14 amendment), [ADR 0099](../adr/0099-one-built-in-synth-channel-plays-a-303-style-voice.md)
(first 2026-11-14 amendment).
Evidence: `synth/src/controller.rs`; test `synth/src/lib.rs` `each_effect_changes_the_channel_it_is_set_on`.

### PLAY-610 Channel `08` plays one 303-style voice

The acid voice is mono: a chord sounds as its last note; a Note On while the channel's note is held
slides to the new pitch over about 60 ms without restarting either envelope; a Note On with nothing
held starts both; a Note Off for a note no longer sounding changes nothing. It is a bandlimited saw
or square into a resonant 24 dB/oct lowpass capped below self-oscillation, with a filter envelope
that jumps on each new note and decays. A Note On at velocity `78` or above is accented: louder,
filter opened further, shortest decay. On that channel only, CC 70 (waveform), 71 (resonance), 74
(cutoff), 75 (envelope decay) and 79 (envelope depth) set it; it takes one of the sixteen voices.

Reason: acid basslines are what a reader expects a sequencer to make, and velocity and note overlap
already say accent and slide.
Sources: [ADR 0099](../adr/0099-one-built-in-synth-channel-plays-a-303-style-voice.md).
Evidence: `synth/src/acid.rs`; test `synth/src/lib.rs` `the_acid_channel_holds_one_note_at_its_pitch`.

### PLAY-620 Reset All Controllers clears only a channel's Modulation

On the Built-in Synth, CC 121 on a channel clears its `MD` to `00` and leaves the acid voice's
controls and every other effect as they are, following RP-015. Stop's safety action therefore clears
every channel's vibrato and keeps the sound a Source set up.

Sources: [ADR 0099](../adr/0099-one-built-in-synth-channel-plays-a-303-style-voice.md) (second
2026-11-14 amendment).
Evidence: `synth/src/controller.rs` `RESET`; test `synth/src/lib.rs` `reset_all_controllers_leaves_the_acid_voices_sound_alone`.

## Accepted but not built, or built differently

- PLAY-260: the code keeps the schedule after a refused delivery. The schedule records only what an
  output adapter accepted, so a refused submission leaves every Timed and Mono claim standing and the
  due stops are delivered again at the next executed Tick (test `orcvs/src/playback.rs`
  `a_refused_submission_leaves_the_schedule_standing_for_the_next_tick`). ADR 0016 says teardown
  after a refused delivery ends that ownership. With the MIDI adapter nothing is delivered until a destination change
  clears the schedule; an adapter that survives a refusal sees the retry. Ruling without ADR:
  [midi-output-family/02](../../.scratch/midi-output-family/issues/02-schedule-timed-play-note-off.md)
  (resolved). Until an ADR or an amendment to ADR 0016 decides between them, PLAY-260 stands as
  written. Tracked in [requirements-snapshot/05](../../.scratch/requirements-snapshot/issues/05-decide-teardown-after-a-refused-delivery.md).
- PLAY-120: the code adds a network-failure class, ranked just above Overrun and counted in Omitted
  Diagnostics, which ADR 0055 does not name. Tracked in
  [requirements-snapshot/11](../../.scratch/requirements-snapshot/issues/11-record-the-network-failure-diagnostic-class.md).
- PLAY-550: native has no Built-in Synth. Tracked in
  [orcvs-site/14](../../.scratch/orcvs-site/issues/14-give-native-the-built-in-synth.md).

## Open decisions

### Built, awaiting a record

Preserve this as built until an ADR records it or changes it.

- The network destination reaches the engine on its own newest-wins channel, outside the request
  slots ADR 0056 lists (PLAY-350). ADR 0056 predates network output and does not say whether it is
  a slot. Tracked in
  [requirements-snapshot/06](../../.scratch/requirements-snapshot/issues/06-decide-the-network-destination-path.md).
  Proposed for the record in [ADR 0112](../adr/0112-the-network-destination-is-a-value-the-engine-reads-at-each-send.md),
  not yet accepted, which would also amend ADR 0085's storage clause to the eframe Storage key the
  console uses.

### Undecided

- Whether a Tick must still land on time while a MIDI destination change is installed (the outgoing
  connection's safety action runs in the Tick-owning task). Being triaged in
  [midi-port-ownership/08](../../.scratch/midi-port-ownership/issues/08-a-tick-still-lands-while-a-destination-change-is-processed.md).
