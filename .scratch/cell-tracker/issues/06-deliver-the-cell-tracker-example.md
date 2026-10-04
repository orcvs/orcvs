# 06 — Deliver the editable-cell tracker and acceptance evidence

Status: needs-triage
Blocked by: 03, 05

## Work

Ship a small Source File and walkthrough that demonstrate the tracker: a
Clock-indexed Track over a List of Notes with deliberate rests. Replace the local
constructed-Sequence experiment, which 03 makes unparseable.

## Acceptance

- [ ] The checked-in Source has one editable Note per occupied item and runs from
      the ordinary console File > Open flow.
- [ ] The guide explains the index, count, List, trigger, rests, BPM, MIDI
      channel, velocity and length, with exact columns where alignment matters,
      and distinguishes Orca's `T` from Track.
- [ ] A test loads this exact Source File and checks at least two complete loops,
      including rests, through Source/Tick and Playback.
- [ ] An audible MIDI smoke test is recorded with device, channel and BPM.
