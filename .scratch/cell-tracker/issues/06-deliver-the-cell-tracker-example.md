# 06 — Deliver the cell tracker example

Status: resolved
Blocked by: 03, 05

**What to build:**

Ship the first checked-in tracker Source File and guide, demonstrating a
Clock-driven Track over editable pairs, some deliberately empty, through the
normal console File > Open flow.

## Acceptance criteria

- [x] The example has one editable Note per occupied pair and keeps deliberately empty pairs in a repeating eight-pair row.
- [x] The guide explains `index` and `count` as ordinary operands, that Track reads its selected pair from working Source at its Turn, so a same-Tick write to `index`, `count` or the pair is seen that Tick (ADR 0067), and the trigger, BPM, MIDI channel, velocity and length, with exact columns for spatial alignment.
- [x] The guide distinguishes Track from Orca's `T`: Track's read is ordered after every writer of its selected pair that has not yet taken its Turn, and a writer that waits on Track forms a diagnosed same-Tick cycle. It explains that Track copies an empty pair's Cells like any others, so a Timed Play whose note slot they empty is invalid (ADR 0069): it emits no new trigger and does not cancel an earlier Timed Play lifetime. It also explains that a value Function placed between Track and the Play and delivering through Cells leaves its last answer in the slot when it is invalid, so the next Bang plays it again.
- [x] A test loads the exact shipped Source File and checks at least two complete loops through Source/Tick and Playback, including empty pairs and expected Note Off behavior. It asserts each Tick's diagnostics exactly. A Banged Tick whose selected pair is empty emits no Play Command and one Tick diagnostic, anchored at the Timed Play, that its note slot is empty (ADR 0069); this repeats on every loop. Every other Tick, an unbanged one on an empty pair included, emits no Tick diagnostic. While the note slot is empty the Language Map also diagnoses the Play. The test does not pin the Map's diagnostics of the pairs themselves, which ADR 0067 leaves to the Parser.
- [x] The example works through the console editing and file workflows verified by ticket 05.
- [ ] The audible MIDI smoke test is recorded in `13`.

## Comments

Rewritten for ADR 0067 and ADR 0069, which replaced the List claim, its literal count fixed when the Source is parsed, its Items, the Blank Answer and the pending state.

### Decided 2026-10-13 by Toby

The audible MIDI smoke test (device, channel and BPM) moved to `13`, a `ready-for-human` ticket
blocked by this one, because it needs a person with a MIDI device. Delivery is still not complete on
automated evidence alone.

### 2026-10-17 — Resolved

Shipped `examples/cell-tracker.orcvs` and `examples/cell-tracker.md`: a Clock `~.0208` writes
Track's `index`, Track `&t  08` reads eight pairs (`C4 D4 __ G4 __ A4 C5 __`), and a Delay `~*0102`
bangs a Timed Play `!~0064  04` whose note slot Track writes. Each step is two Ticks; odd Ticks are
unbanged, so a held D4 sounds through the empty step after it.

`orcvs/src/playback/tests/shipped_trackers.rs` loads the file with `include_bytes!` and runs 32
Ticks through a hand-driven Playback run, asserting every Tick's diagnostics and delivery (Note Ons,
expiries and retrigger stops) exactly, and that the Language Map diagnoses the Play exactly while
its note slot is empty. Criterion 5 is `console::kittest_tests::live_editing` (ticket 05).

The last criterion stays open in `13`, the human smoke test, as decided on 2026-10-13.
