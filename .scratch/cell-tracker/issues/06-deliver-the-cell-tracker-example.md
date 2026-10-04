# 06 — Deliver the cell tracker example

Status: ready-for-agent
Blocked by: 03, 05

**What to build:**

Ship the first checked-in tracker Source File and guide, demonstrating an
editable Clock-driven List with intentional rests through the normal console
File > Open flow.

## Acceptance criteria

- [ ] The example has one editable Note per occupied Item and preserves deliberate rests in a repeating eight-Item List.
- [ ] The guide explains index, literal count, current-Tick Item reads, next-Tick count changes, trigger, BPM, MIDI channel, velocity and length, with exact columns for spatial alignment.
- [ ] The guide distinguishes Track from Orca’s Track and explains that a blank Item prevents a new trigger without cancelling an earlier Timed Play lifetime.
- [ ] A test loads the exact shipped Source File and checks at least two complete loops through Source/Tick and Playback, including rests and expected Note Off behavior.
- [ ] The example works after Sequence removal and through the console editing and file workflows verified by ticket 05.
- [ ] Record an audible MIDI smoke test with device, channel and BPM; do not mark delivery complete on automated evidence alone.
