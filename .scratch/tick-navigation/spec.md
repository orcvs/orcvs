# Tick navigation: step, scrub and jump to any Tick

Status: needs-triage

## Problem Statement

Playback only moves forward. Space pauses and resumes and Command Shift R
rewinds to Tick `0` (ADR 0076), but there is no way to step one Tick at a time,
step back, or go straight to Tick 1000 to hear how a pattern lands there.

Toby asked for it on 2026-10-19: "we should be able to jump to ANY frame count
of the source, everything is deterministic."

## Why it is possible

Evaluation is a function of the Source and the absolute Tick. Clock, Delay,
Euclidean and Random read the absolute Tick, and Random derives its result from
its seed, the Tick and its Position rather than any outside generator
(`CONTEXT.md`, Random Function). So from a known Source at Tick `0`, executing
Ticks `0` to `N - 1` always produces the same Source at Tick `N`.

## What makes it hard

1. **Tick writes mutate the Source.** The Source at Tick `N` is not the Source
   the user sees at Tick `0`. Going back needs a base Source to replay from,
   and the current Source is not one.
2. **Live edits break the replay.** An edit made at Tick `500` is part of the
   history. Replaying from the base without it reaches a different Tick `600`
   from the one the user heard.
3. **Cost.** Jumping to Tick `N` costs `N` Ticks of evaluation. Checkpoints of
   the Source every so many Ticks bound it.
4. **Sound.** Replayed Ticks must not send MIDI. A jump sends the safety action
   a stop already sends, and the Note Offs owed at the old Tick are discarded,
   as a Rewind discards them.

## Open questions

1. **What is the base?** The default proposed here is the Source at the moment
   Playback last left Tick `0`: a Rewind or an Open sets it, and Tick `0`'s
   first execution captures it.
2. **What happens to live edits?** The default proposed here records each user
   edit with the Tick it landed before, and a replay applies it at that Tick,
   so a jump reproduces what was heard. The alternative is that an edit makes
   the current Source and Tick the new base, and no Tick before it can be
   reached again.
3. **Which commands?** The default proposed here is step forward and step back
   by one Tick while stopped, and a jump to a typed Tick from the Panel's T
   field. Chords are picked so the web can bind them.
4. **Does a jump work while playing?** The default proposed here stops, jumps
   and resumes, as a live Rewind restarts the run.
5. **How far apart are checkpoints?** Settled by a benchmark of replay cost
   over the shipped examples.

## Relation to Undo

`../source-undo/spec.md` is separate. If edits are recorded for replay, the
same record is a candidate for the Undo history, and whichever lands second
reuses it rather than building another.
