# Cycle through editable note Cells

Status: ready-for-agent

## Problem Statement

A performer needs to edit a row of musical Notes directly in Source, cycle
through it with a Clock, and hear each selected Note when triggered. Constructing
an invisible Sequence can play pitches but does not provide that interaction.
Empty pairs must retain their timing, and edits and same-Tick spatial writes
must reach the Play in the Tick they happen. An empty pair copied straight into
a Play's note slot triggers nothing; a value Function between Track and the
Play that is invalid writes nothing, so the next Bang replays its last answer
(ADR 0069).

## Solution

Track `@t index count` reads one pair of the ordinary Source Cells east of its
last operand each Tick, through an Input Portal it reads exactly as a Jump does
(ADR 0067). A Clock drives the index, and a trigger activates Timed Play
receiving what Track reads. Occupied pairs are editable two-Cell Notes; Track
copies an empty pair's Cells as it copies any others. The Source File contains
the complete pattern and works with ordinary console editing, Open and Save.
Nested Track uses the same receiving-operand interpretation as spatial delivery.

## User Stories

1. As a performer, I want each Note visible in Source, so that I can edit the
   pattern where I hear it being played.
2. As a performer, I want Clock-driven selection, so that the pattern repeats
   without a hidden note list.
3. As a performer, I want an explicit count, so that empty pairs keep their
   position in the cycle.
4. As a performer, I want an index to wrap at that count, so that different
   index sources can drive the same row of Notes.
5. As a performer, I want the first Clock Tick and its rate to be predictable,
   so that a new Playback run starts the pattern consistently.
6. As a performer, I want playback to continue beyond Tick 255, so that long
   runs retain their rhythm.
7. As a performer, I want clearing a pair that Track copies straight into
   Timed Play's note slot to prevent a new note trigger, so that I can leave a
   step silent without moving later pairs.
8. As a performer, I want existing Timed Play note lifetimes to survive an
   empty pair, so that clearing a pair does not unexpectedly stop a sustained
   note.
9. As a performer, I want a complete edit to affect the next eligible Tick,
   so that Live Editing feels immediate.
10. As a performer, I want malformed intermediate edits to diagnose without
    spurious notes, so that typing a replacement Note is recoverable.
11. As a composer, I want a same-Tick write to the index to affect selection,
    so that spatially connected Functions compose in dependency order.
12. As a composer, I want a same-Tick write to the selected pair to reach its
    consumer, so that the sound reflects current Source rather than a parse-time
    copy.
13. As a composer, I want partial and competing writes to the pair to follow
    ordinary Cell-wise ordering, so that Track does not introduce a second write
    rule.
14. As a composer, I want a same-Tick write to the count to affect selection,
    so that the count is an operand like the index.
15. As a composer, I want nested results decoded by their receiving operand,
    so that nesting and spatial composition agree about the same characters.
16. As a composer, I want nested feedback Functions to write their own Output
    Portals, so that their state advances across Ticks.
17. As a composer, I want a Function whose operand is unwritten to be
    invalid and not execute, and its parent with it, so that an empty slot
    never produces a value and its Output Portal is not updated.
18. As a performer, I want copied and pasted empty pairs to stay in place,
    so that editing preserves the pattern's rhythm.
19. As a performer, I want Open, Save and reopen to preserve the pattern,
    so that the Source File is sufficient to reproduce it.
20. As a performer, I want Source Paint to show malformed data Track reads at
    its receiving operand, so that I can locate mistakes.
21. As a composer, I want cycles to publish no partial Tick effects,
    so that an invalid dependency does not produce a partly played pattern.
22. As a performer, I want a working Source File and an alignment guide,
    so that I can load the tracker and understand how to change it.
23. As a composer, I want several Play roots activated together to play a chord,
    so that removing Sequence broadcasting retains a visible chord workflow.

## Implementation Decisions

1. Implement the accepted Return and Sequence-retirement decisions in ADRs
   0061 and 0063, the unwritten-operand decision in ADR 0069, and Track as
   ADR 0067 decides it, which supersedes ADR 0063's List clauses. Keep the
   general spelling sweep in a separate follow-up.
2. Keep the existing Source Tick interface as the principal test seam. Playback
   consumes ordered Play Commands and does not parse or select Track's pairs.
   No new external interface or adapter is required.
3. Track is an ordinary value Function in the Function table with Number
   operands `index` and `count`, each literal, nested or written by a Portal.
   It claims only its operands; the Cells east of it are ordinary Source that
   the Parser parses, paints and edits as any other. The Parser has no
   Track-specific rule.
4. At its Turn Track reads the pair `index % count` pairs east of its last
   operand through the Portal read Jump uses, and answers what it reads as a
   Jump does. A `count` of `00` diagnoses at the Turn; a pair past the row
   edge diagnoses.
5. Order Track's read by ADR 0032's rule, completed at its Turn: once `index`
   and `count` settle, every writer of the selected pair that has not taken
   its Turn goes first. Nothing is reserved for Track before the Tick, and do
   not cache the pair's characters as the value to be consumed for that Tick.
6. Partial, competing, absent and failed suppliers use the existing surviving
   Source rules. Cycles preserve atomic Tick publication. A reused schedule
   gives the same result as a freshly built one for Sources holding Track.
7. Return and Pending Operand Encoding use the same receiving operand's
   literal interpretation. Keep printable encoding and destination-fit
   decisions local to their existing modules rather than adding a Track parser
   or duplicating decoding at MIDI output.
8. A nested value Function still writes its own Output Portal and reserves the
   Cells it may reach. A refused spatial destination does not erase a valid
   Return. Effect Functions and Functions unable to answer one two-Cell Return
   are refused by the Parser when nested.
9. An inline operand slot whose Cells are all empty is invalid input
   (ADR 0069), handled as a partly written slot is: the Function does not
   evaluate, writes nothing, returns nothing and is diagnosed, and a nested
   invalid Function fails its parent. Its Output Portal keeps the Cells it
   held. An unwritten operand does not give the Absence Marker a Source
   encoding.
10. Track reading an empty pair copies its Cells as a Jump does: it clears its
    Output Portal and, nested, returns the empty Cells, so its parent is
    invalid (ADR 0069). Copying empty Cells is a successful evaluation.
11. A Bang reaching a Timed Play whose note slot is empty finds it invalid,
    so it emits no new Play Command. It does not cancel notes whose Timed
    Play lifetimes are still running; their existing Note Off scheduling
    remains.
12. Retire the Sequence value, its Functions, pervasive extension and
    variable-width Reservations as specified by ADR 0063. Preserve ordinary
    scalar behavior and the declared distinction between value and effect
    Functions. Keep each implementation change and its reference documentation
    in agreement.
13. Reuse Grid, Cursor, Region, Source Paint and file workflows. Track has no
    hidden persistent state, dedicated editor or independent Playback clock.
14. Record the nested-absence clarifications alongside their
    implementing work in the relevant ADRs and domain glossary, keeping one
    authoritative language contract.

## Testing Decisions

- Assert observable Source Cells, ordered Play Commands and diagnostics through
  the existing Source Tick interface. Avoid exposing implementation-only seams
  or adding test-only inputs to shipped code.
- Use the existing nested computation, rejected Portal, current-Tick supplier,
  feedback, Bang activation and atomic-cycle tests as prior art.
- Compare identical characters delivered through a Return and a spatial write,
  including Number/Note contextual decoding and malformed encodings. Check
  independent child writes when a parent fails and valid Return delivery when
  the child's spatial destination is refused.
- Exercise unwritten operands: a root or nested Function with an unwritten
  slot is invalid and does not execute, and its parent is invalid in turn.
  It writes nothing, so its Output Portal keeps the Cells it held. A partly
  written slot or one the row edge cuts short still diagnoses. Contrast it
  with Absence Marker results, which diagnose a nested parent. Test that a
  Banged Play whose note slot is unwritten emits no Play Command, including
  when Track empties that slot directly, by copying an empty pair to an Output
  Portal that is the slot, or through nesting, as the note operand returning
  the empty Cells. Test that a Banged Play reading the Output Portal of an
  invalid value root plays the Cells that Portal still holds. Nested Track copies an empty pair's Cells as it
  copies any others, which leaves its parent invalid.
- Drive both index and selected-pair writers before and after Track in Grid
  order, and east of and below it. Check the resulting Note in the same Tick,
  partial and competing writes, absent or failed suppliers, a writer of an
  unselected pair that is not ordered against Track, and a writer of the
  selected pair that waits on Track, which forms a diagnosed cycle.
- Write count during a Tick, including `00`, and check selection that Tick.
- Cover nested `index` and `count`, row-edge arithmetic, a count of one,
  all-empty pairs, and the Jump reading rules for what Track reads: `**`
  relays a Bang, a Function spelling answers that Function, and a partial pair,
  a Comment or Cells straddling two Language Units diagnose.
- Compare reused scheduling with fresh scheduling for Sources holding Track.
- Run Clock-driven selection from Tick zero, holding each index for the rate,
  wrapping at count and continuing beyond Tick 255.
- Load the exact shipped Source File and verify two complete loops, including
  empty pairs, through Source/Tick and Playback. Assert Note Off timing
  independently from the absence of a new trigger at an empty pair.
- Use real console input and file workflows for edit, clear, restore, copy,
  paste, Save and reopen. Follow the egui skill for console tests. Playback
  tests retain existing output adapters; do not introduce one for Track.
- Record an audible MIDI smoke test with device, channel and BPM. This is
  delivery evidence and is not replaced by a silent automated run.
- Follow repository-scoped implementation gates and use 32 local proptest
  cases. This planning update requires roadmap tests, roadmap generation and
  complete diff review; broader platform and performance gates remain in CI.

## Out of Scope

- The general Function spelling sweep and related console presentation aids.
- Addressed or vertical Track reads, and other Functions that select Cells.
- A dedicated tracker editor or moving playhead.
- Changes to MIDI encoding, Timed Play lifetimes or Playback scheduling.
- New dependencies, unsafe code, feature combinations or performance claims.

## Further Notes

This is active pre-release language design, with no public compatibility
contract. Sequence retirement, Return and Track are implemented (tickets 01,
02 and 04); console verification and the shipped example remain. Ticket 03
delivered unwritten operands as ADR 0066's pending state, and ADR 0069
superseded it and the pending clauses of ticket 04.
`.scratch/pending-presentation/issues/01-treat-an-unwritten-operand-as-invalid-input.md`
implemented the invalid-input behaviour Decisions 9–11 describe, and its
Comments name the tests that cover it.

The selected `@t` spelling follows existing lowercase second-Cell spellings
such as Control Change `!c` and Pitch Bend `!b`; no family-wide rename is needed
for Track. The general spelling rule and its directional-glyph exception belong to the
separate spelling effort.

The worked tracker places Delay above a Bang aligned with Timed Play, Clock
above Track's index, and Track above Timed Play's Note operand. Its eight pairs
are C4, D4, E4, empty, G4, C5, empty and E4. Clock's first write selects pair zero
in the same Tick even if the stored initial index names another pair. The guide
must give exact columns and explain that the four-Tick note length can sustain
notes through later empty pairs.

Delivery numbers the six deliverables in dependency order: Sequence
retirement (01), Return (02), unwritten operands (03), Track (04), console
verification (05), and the shipped example (06). Unwritten operands depend on
Return; Track depends on both Return and unwritten operands. Sequence retirement
in 01 removes the existing alternative value form, not an opportunity to
introduce a new abstraction. If implementation exceeds one fresh context,
split its migration before claiming it rather than leaving a half-retired
language on the main branch.

| Ticket | Deliverable | Blocked by |
| --- | --- | --- |
| 01 | Sequence retirement and scalar behavior | None |
| 02 | Return and nested Output Portal writes | 01 |
| 03 | Unwritten operands (delivered as pending under ADR 0066, which ADR 0069 superseded) | 02 |
| 04 | Track reads a Portal as Jump does | 02, 03 |
| 05 | Real console editing and file workflows | 04 |
| 06 | Shipped example and acceptance evidence | 03, 05 |
