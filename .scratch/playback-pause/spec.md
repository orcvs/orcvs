# Playback pause: Space pauses and resumes, Command Shift R rewinds to Tick 0

Status: resolved

## Problem Statement

Space stopped Playback, and the next Space began a new run at absolute Tick
`0`. Clock `~.`, Delay `~*` and Euclidean `~%` read the absolute Tick, so every
start put every voice back on the first step of its cycle. Orca's Space pauses
instead: its frame counter `f` survives the pause (`clock.js` 50–88,
`client.js` 98), and the next play continues from the frame after the last one
run. Orca resets the frame to `0` with a separate chord, Ctrl or Command Shift R
(`client.js` 100, `clock.js` `setFrame`). The Orca tracker example has to behave
as Orca's does.

## Decision

Toby chose it on 2026-10-18, on a decision card: "Make Space pause and resume
Playback, as Orca's does?" → Pause. "Space keeps the Tick and resumes at the
same step; a separate key resets to Tick 0. Its own ticket and PR." ADR 0076
records it.

1. A stop holds the absolute Tick. The next run begins at the Tick after the
   last one executed, so every per-voice Clock resumes on its step.
2. A stop sends the safety action it already sends (All Notes Off, Reset All
   Controllers, centred Pitch Bend on all sixteen channels) and discards the
   schedule of owed Timed and Monophonic Note Offs. Orca silences on pause too
   (`clock.js` 76–88, `io/midi.js` `silence` and `allNotesOff`).
3. A Rewind makes Tick `0` the next Tick. Command Shift R on native, which is
   Ctrl Shift R outside macOS, as Orca binds it.
4. There is no Paused state beside Stopped: a stopped Playback is a paused one.

## Decisions

Defaults picked by Claude where the card left them open:

- **A Rewind of a live run restarts it.** It stops the run, with the safety
  action and the schedule discarded, and begins the next at once at Tick `0`
  at the same period. Orca's `setFrame(0)` keeps playing and keeps its notes,
  which count down per note; Orcvs's Note Offs are due at absolute Ticks, so a
  schedule kept across a Rewind would stop notes at Ticks the new run reaches
  late or never.
- **A Rewind of a stopped engine leaves it stopped.** The Panel shows Tick `0`
  and a Run Clock of zero, though Tick `0` has not sounded, so it shows where
  the next run begins.
- **The Run Clock is held across a pause and counts on when Playback resumes.**
  It is the time spent playing since Tick `0`, and a Rewind zeroes it.
- **The Panel shows a paused Playback as it showed a stopped one:** the rest
  marker, the held Tick at T, the held Run Clock at C. Nothing new is drawn.
- **A resumed run's grid is its own.** Its first Tick executes immediately, as
  every run's does; the stopped interval is neither owed nor counted as
  Overruns.
- **The mailbox holds one Rewind flag,** applied after the stop and before the
  tempo, which a `stop` leaves standing and which leaves the tempo slots alone.
- **A held chord rewinds once:** a key repeat is not a Rewind.
- **The web binds no Rewind.** The browser reserves Command Shift R for a hard
  reload, as the console's File chords already avoid the chords the browser
  reserves; a reload of the web console begins at Tick `0` anyway.
- **No menu item.** The console's menus list no Playback commands, so the
  Rewind joins none.
- **Opening a Source, File New and the Function Reference rewind by
  replacement:** each builds a new running Orcvs and Playback Engine, as
  before.
- **The engine keeps its `stop` and `PlaybackState::Stopped` names.** Renaming
  them to pause would touch every lifecycle test and ADR for no change in
  meaning.

## Out of scope

- A Rewind on the web, and a Playback menu.
- Rebasing scheduled Note Offs across a Rewind so notes keep sounding.
