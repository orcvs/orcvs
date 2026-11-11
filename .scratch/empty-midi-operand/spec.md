# A MIDI output with an empty operand sends nothing

Status: ready-for-agent

Settled in design on 2026-11-11. ADR 0106 records the decision, which supersedes ADR 0075 and
ADR 0104 and amends ADR 0069 for the MIDI outputs. Where `.scratch/chords-and-pitch/spec.md`
describes a Rest, this spec replaces it.

## Problem Statement

A performer writing a pattern leaves Cells empty where nothing should sound, exactly as in Orca.
In Orcvs, whether that empty step is silent or diagnosed has depended on how the Cells became
empty and on which operand they sit in. A nested Clear rested, a note left empty in Source was
first diagnosed and later rested, and a Chord Play's empty root excused some empty operands
beside it but not others, with refusals held back until the root was known.

The language has no concept of how a Cell came to be empty. A Function is handed what its Cells
hold. Each of those exceptions had to be taught separately to the Parser, the Language Map and
the Tick, so the same rule lived in several places and could drift.

## Solution

A MIDI output whose operand Cells are empty sends nothing and reports nothing. This holds for
Raw Play `!>`, Timed Play `!~`, Monophonic Play `!%`, Raw Chord Play `!:`, Timed Chord Play `!#`,
Control Change `!c` and Pitch Bend `!b`, and for every operand each one declares. This is how
Orca's `:`, `%` and `!` behave: they check each required port's glyph and return when it is empty.

Empty means both Cells of the operand are empty. The output looks only at the Cells, so the
Source, a Portal write and a nested Function's Return all give the same answer. A half-written
operand, with one Cell empty beside a written one, is still an error. Every other Function still
treats an empty operand as invalid input. Empty is not a new value or type.

## User Stories

1. As a performer, I want a Play whose note Cells are empty to send nothing, so that an empty pattern step is silent.
2. As a performer, I want that silent Play to raise no diagnostic, so that intended silence does not fill the diagnostics.
3. As a performer, I want a Play to behave the same whether I left its Cells empty, a Portal emptied them, or a nested Function wrote empty Cells into them, so that layout never decides what I hear or see.
4. As a performer, I want a Play with an empty channel to send nothing, so that it behaves as Orca's `:` does.
5. As a performer, I want a Play with an empty velocity to send nothing, so that one rule covers every operand.
6. As a performer, I want a Timed or Monophonic Play with an empty length to send nothing, so that one rule covers every operand.
7. As a performer, I want a Chord Play with an empty root to send nothing, so that an empty step in a chord progression is silent.
8. As a performer, I want a Chord Play with an empty Interval Set, Voicing or inversion to send nothing, so that a Chord Play follows the same rule as a Play.
9. As a performer, I want a wholly empty Chord Play row to send nothing, so that an empty pattern row is silent with no special case.
10. As a performer, I want a Control Change with an empty channel, controller or value to send nothing, so that every MIDI output follows one rule.
11. As a performer, I want a Pitch Bend with an empty channel, LSB or MSB to send nothing, so that every MIDI output follows one rule.
12. As a performer, I want a half-written operand, such as `C ` in a note, to stay an error on every MIDI output, so that half-typed input is reported while I edit live.
13. As a performer, I want a half-written operand to be reported even when another operand of the same output is empty, so that a typo is never hidden by an empty step.
14. As a performer, I want an operand that does not decode, such as a Number where a Note belongs, to stay an error beside an empty operand, so that wrong input is never silent.
15. As a performer, I want an Arithmetic, Pitch or other value Function with an empty operand to stay an error, so that this change does not widen beyond MIDI outputs.
16. As a performer, I want a MIDI output with empty operands to be scheduled like any other root, so that it starts sounding as soon as its Cells are written.
17. As a performer, I want an empty MIDI output still to take its Turn when a Bang reaches it, so that nothing about activation changes.
18. As a performer, I want a Monophonic Play that sends nothing to leave its channel's voice sounding to its own Note Off, so that an empty step does not cut a held note short.
19. As a performer, I want an empty operand's channel or velocity range never to be checked, so that a silent output reports nothing about values it never used.
20. As a performer, I want a nested Read or Track of two empty Cells in any MIDI output operand to make that output send nothing, so that pattern-driven voices rest on empty steps.
21. As a performer, I want a nested Function that is itself invalid to leave the MIDI output invalid, so that an error in the child is still reported.
22. As a performer, I want the Melody example to play its empty steps silently with no diagnostics, so that the shipped example stays clean.
23. As a performer, I want the shipped tracker examples to keep sounding exactly as they do now, so that this change does not move any music.
24. As a performer, I want Source Paint to show an empty operand in its declared type's colour, so that empty Cells look the same as they do today.
25. As a reader of the Function reference, I want one short paragraph saying a MIDI output with an empty operand sends nothing, so that the rule is easy to learn.
26. As a reader of `CONTEXT.md`, I want Rest and Clear to describe this behaviour rather than a mechanism, so that the glossary matches the code.
27. As a maintainer, I want the rule to read the existing Terminal Output declaration, so that the Function table gains no new clause.
28. As a maintainer, I want the `rest:` clause and its `ignoring blanks` variant removed from the Function table, so that the table states only facts the language still has.
29. As a maintainer, I want the Parser's rest-specific bookkeeping removed, so that it treats empty operands of MIDI outputs by one rule with nothing held back.
30. As a maintainer, I want the Tick's operand decoding to decide an empty MIDI operand without depending on order, so that nothing has to be deferred until the whole signature is read.
31. As a maintainer, I want the Parser and the Tick to agree on every case by construction of one simple rule, so that the Language Map and a Tick never disagree about whether a row sends.
32. As a maintainer, I want a resting row still to allocate nothing while it is parsed and executed, so that the allocation guarantees hold.
33. As a maintainer, I want ADR 0075 and ADR 0104 marked superseded by ADR 0106, so that the decision record says which rule is current.

## Implementation Decisions

- **The rule reads the Terminal Output declaration.** Every Terminal Output Function that declares typed operands is a MIDI output; UDP and OSC take a Payload instead. The rule applies to an operand of any Function whose output is Terminal, so no new fact enters the Function table.
- **The Function table loses the rest clause.** The `rest:` role, its `ignoring blanks` marker, `rest_operand()` and `rest_ignores_blanks()` are removed, along with the macro arms that support them and the test pinning which Functions declare one.
- **The Parser has one rule for empty operands.** Two empty Cells in an operand of a Terminal Output Function are recorded with no Atom and raise no refusal. Any other operand that fails to decode refuses the Expression as it does today. The rest-specific bookkeeping goes: the blank-operand classification, the written-rest tracking, the held-back refusal and the withdrawal of excused entries. Strict parsing still refuses an empty operand, because it yields no value.
- **An Expression with only empty MIDI operands unbound is executable.** It has no error, so the Language Map admits it as a root and schedules it. The per-entry marker that tells the Tick an unbound entry blocks nothing stays, but it records "empty Cells in a Terminal Output operand" rather than a rest role, and is set by that one rule.
- **The Tick decides at the Turn, in any order.** When a Terminal Output Function's Turn decodes its operands, an operand that fails to decode for any reason other than being two empty Cells is an error at once. An operand whose Cells are empty, whether read from Source or carried by a nested Function's Return, is noted. After every operand is read, a noted empty operand makes the Turn send nothing and report nothing. A nested Function that answered a Clear has written two empty Cells, so its Return is treated exactly as empty Cells.
- **No evaluation happens on an empty Turn.** The Play body is not called, so no range is checked and no Play Command is built, exactly as a Rest behaves today.
- **Chord Play needs nothing of its own.** It follows the same rule as every other MIDI output.
- **Records.** ADR 0106 is added. ADR 0075 and ADR 0104, with its amendment, are marked superseded. ADR 0069 is noted as amended for MIDI outputs. The Rest, Clear, Return and Pending entries in `CONTEXT.md` are rewritten to state the rule. The chords-and-pitch spec's Rest stories point here. The Function reference gets one paragraph on the rule.
- **Delivery.** One pull request stacked on the top of the chord and pitch stack, which removes the layer the stack added for Chord Play rests as well as the Rest code already on `main`.

## Testing Decisions

- A good test states a Source layout and asserts what a Tick sends and what is diagnosed. It never asserts which internal flag or entry carried the decision.
- **One seam: the Source Tick fixtures.** Source rows go in, and the Tick Plan's Play Commands and the Language Map's diagnostics come out. This crosses the Parser, the Language Map, scheduling and the Turn together, so the rule is checked once, end to end. The existing helpers that bang a single row and read back its commands and diagnostics are the prior art, along with the two-Tick helper for a Portal that empties an operand.
- **A table over every MIDI output and every operand.** For each of the seven outputs and each declared operand, that operand empty sends nothing with no diagnostic. Half-written is diagnosed. A nested Read of two empty Cells sends nothing. A Portal write that empties the operand sends nothing on that Tick.
- **Cases beside an empty operand.** A half-written or wrongly typed operand next to an empty one is diagnosed. A nested invalid child is diagnosed.
- **Value Functions are unchanged.** One case each for an Arithmetic and a Pitch Function with an empty operand, still diagnosed.
- **Playback.** The existing Monophonic Play test that a silent Turn leaves the voice to its own Note Off stays.
- **Allocation.** The existing resting-row allocation fixtures stay and gain an empty Chord Play row.
- **Removed.** Tests that pin the rest clause, excused operands, held-back refusals, or a route-by-route difference are deleted, because the behaviour they describe no longer exists. Parser property tests keep the invariant that an Expression with no error has every entry bound except empty Terminal Output operands.

## Out of Scope

- Default values for an empty velocity or length, as Orca's `f` and `1`. Recorded as an open question in ADR 0106; adding them later only makes a silent output sound.
- Any change to how value Functions treat an empty operand (ADR 0069 stands for them).
- Reading an empty operand as zero, which ADR 0069 rejected.
- UDP, OSC and Application Command, which take no typed operands.
- How Source Paint shows an empty operand.

## Further Notes

- Orca reference: `desktop/sources/scripts/core/library.js` on `hundredrabbits/Orca` `master`, where `:` checks channel, octave and note (lines 555-557), `%` does the same (lines 650-652), and `!` checks channel and knob (lines 588-589), each returning when the glyph is `.`.
- ADR 0062, since superseded, held the same model ("an inline input portal whose Cells are all empty… a Terminal Output Function with a blank operand emits no command"). ADR 0106 restores that model for MIDI outputs only.
