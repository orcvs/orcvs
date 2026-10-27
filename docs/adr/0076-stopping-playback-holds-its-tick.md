# Stopping Playback holds its Tick

Status: accepted; the web's lack of a Rewind is amended by [ADR 0089](0089-the-panel-plays-and-rewinds.md), whose Panel control rewinds on every target. Amends [ADR 0012](0012-time-and-feedback-are-explicit-tick-inputs.md)'s clause that the first Playback Tick is absolute Tick `0`: only the first run, and the first after a Rewind, begins there. Amends [ADR 0056](0056-playback-requests-wait-in-slots-not-a-queue.md)'s mailbox, which holds a Rewind flag beside the stop flag. [ADR 0002](0002-playback-engine-owns-lifecycle-concurrency.md)'s synchronous `stop` and [ADR 0016](0016-use-distinct-raw-and-timed-play-functions.md)'s safety action are unchanged. Settled in `.scratch/playback-pause/`.

Space started a run at absolute Tick `0` and stopped it, so every start began every Clock `~.`, Delay `~*` and Euclidean `~%` again at the first step of its cycle. Orca's Space pauses: its frame counter survives, and the next play continues from the frame after the last one run, so each voice of a tracker resumes on the step it stopped on. Orca resets the frame to `0` with a separate chord, Command Shift R. The Orca tracker example has to behave as Orca's does, so Orcvs follows it.

## Decision

**A stop holds the absolute Tick.** Stopping Playback ends the run as before: no further Tick is admitted once `stop` returns, the safety action of ADR 0016 is sent, and the schedule of Timed and Monophonic Note Offs is discarded rather than played out. What survives is the counter. The next run begins at the Tick after the last one executed, so the absolute Tick an interpretation reads continues where it stopped, and every Function that reads it resumes at the step it stopped on. A stopped Playback is what Orca calls paused; Orcvs has no separate paused state.

**The safety action on a stop is the one a stop already sends.** Orca releases its held notes and sends CC 123 on every channel when it pauses. Orcvs's stop sends All Notes Off, Reset All Controllers and a centred Pitch Bend on all sixteen channels, which covers what Orca's does. The Note Offs a run owed are discarded with the schedule, so a run never begins owning a note, and the first Monophonic Play after a resume finds its channel's voice empty.

**A run still has a grid of its own.** The resumed run's first Tick executes immediately and its Tick Grid is anchored on that moment, as every run's is under ADR 0037; a stop is not a stall, so the deadlines the stopped interval covered are neither skipped as Overruns nor owed.

**A Rewind makes Tick `0` the next Tick.** Applied to a stopped engine, it leaves Playback stopped and the next run begins at Tick `0`. Applied to a live run, it stops that run, with the safety action and the discarded schedule, and begins the next at once at Tick `0`, at the period the run had. Every scheduled Note Off is due at an absolute Tick, and a schedule carried across a Rewind would stop notes at Ticks the new run reaches late or not at all, so the live Rewind silences rather than rebasing the schedule. A live run whose safety action panics is not begun again: the `ClockFailure` reporting it withdraws the console's request to play, so the Rewind leaves it stopped at Tick `0`. A Rewind shuts no gate: a Tick already admitted when it is asked executes as the old run's last.

**The mailbox holds one Rewind flag, applied after the stop.** The backlog is applied in the order stop, Rewind, tempo, destination. Rewinds collapse into one flag, and a stop leaves it standing: a Rewind and a stop made in either order leave a stopped engine whose next Tick is Tick `0`, and no Tick runs between the parts of one backlog for the order to show in. The tempo slots are left alone, because a Rewind of a live run begins another one for a `retune` to act on.

**The Run Clock is held and counted on.** It reads the wall-clock time Playback has spent playing since Tick `0`: held while stopped, counting on from the held value when a run resumes, and zero again after a Rewind. The Panel shows a stopped Playback as it already did, with the rest marker, the held Tick at T and the held Run Clock at C; after a Rewind it shows Tick `0` and a Run Clock of zero, though Tick `0` has not sounded.

**Command Shift R rewinds on native.** It is Orca's chord, and Command is Ctrl outside macOS, as for the File chords. A held chord rewinds once. The browser reserves Command Shift R for a reload that bypasses its cache, so the web binds no Rewind, as it binds no File chord the browser reserves; a reload of the web console begins at Tick `0` regardless. The console has no Playback menu, so the Rewind has no menu item.

## Consequences

- A performer pausing and resuming hears every voice continue on its step, and the Orca tracker example aligns with Orca's.
- Opening a Source, File New and the Function Reference replace the running Orcvs, Playback Engine included, so each begins at Tick `0` without a Rewind.
- `PlaybackEngine::rewind` and `InputEvent::Rewind` join the public surface of the `orcvs` crate, which no one outside this workspace consumes.

## Considered options

- **A Paused state beside Stopped.** It names a difference that does not exist: both have stopped Ticking, sent the safety action and discarded the schedule, and either resumes the same way. A separate state would split every check of whether Playback is live into two.
- **Rewinding by stopping and starting from the console.** ADR 0056 makes a `stop` empty the tempo slots, so a stop and a start made in one frame are no run at all; the engine is the only party that can end a live run and begin the next in one step.
- **Rebasing the schedule on a live Rewind.** It keeps the notes sounding across the Rewind, as Orca's per-note countdown does, at the cost of a second rule for what an absolute expiry means. Silencing reuses the one rule a stop already follows.
- **Binding Command Shift R on the web too.** The browser reserves the chord for a hard reload, and a page cannot count on receiving it first, so a viewer pressing it to rewind could reload the console instead.
