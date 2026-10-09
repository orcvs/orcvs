# 06 — Deliver the cell tracker example

Status: ready-for-agent
Blocked by: 03, 05

**What to build:**

Ship the first checked-in tracker Source File and guide, demonstrating a
Clock-driven Track over editable pairs, some deliberately empty, through the
normal console File > Open flow.

## Acceptance criteria

- [ ] The example has one editable Note per occupied pair and keeps deliberately empty pairs in a repeating eight-pair row.
- [ ] The guide explains `index` and `count` as ordinary operands, that Track reads its selected pair from working Source at its Turn, so a same-Tick write to `index`, `count` or the pair is seen that Tick (ADR 0067), and the trigger, BPM, MIDI channel, velocity and length, with exact columns for spatial alignment.
- [ ] The guide distinguishes Track from Orca's `T`: Track's read is ordered after every writer of its selected pair that has not yet taken its Turn, and a writer that waits on Track forms a diagnosed same-Tick cycle. It explains that Track copies an empty pair's Cells like any others, so a Timed Play whose note slot they empty is invalid (ADR 0069): it emits no new trigger and does not cancel an earlier Timed Play lifetime. It also explains that a value Function placed between Track and the Play and delivering through Cells leaves its last answer in the slot when it is invalid, so the next Bang plays it again.
- [ ] A test loads the exact shipped Source File and checks at least two complete loops through Source/Tick and Playback, including empty pairs and expected Note Off behavior. It asserts each Tick's diagnostics exactly. A Banged Tick whose selected pair is empty emits no Play Command and one Tick diagnostic, anchored at the Timed Play, that its note slot is empty (ADR 0069); this repeats on every loop. Every other Tick, an unbanged one on an empty pair included, emits no Tick diagnostic. While the note slot is empty the Language Map also diagnoses the Play. The test does not pin the Map's diagnostics of the pairs themselves, which ADR 0067 leaves to the Parser.
- [ ] The example works through the console editing and file workflows verified by ticket 05.
- [ ] Record an audible MIDI smoke test with device, channel and BPM; do not mark delivery complete on automated evidence alone.

## Comments

Rewritten for ADR 0067 and ADR 0069, which replaced the List claim, its literal count fixed when the Source is parsed, its Items, the Blank Answer and the pending state.
