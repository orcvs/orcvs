# A MIDI output with an empty operand sends nothing

Status: proposed. Supersedes [ADR 0075](0075-a-play-with-a-cleared-note-rests.md) and
[ADR 0104](0104-a-blank-note-or-root-is-a-rest-however-it-got-blank.md), and amends
[ADR 0069](0069-an-unwritten-operand-is-invalid-input.md) for the Functions named below.

Orca's MIDI operators check the glyph in each required port before doing anything. If the Cell is
empty, the operator returns: it sends nothing and reports nothing (`library.js`, `:` lines 555-557,
`%` lines 650-652, `!` lines 588-589). Orca does not know or care how the Cell became empty.

Orcvs instead treated an empty operand as invalid input (ADR 0069) and then added exceptions for
particular ways a note could become empty (ADR 0075 for a nested Clear, ADR 0104 for Source and
Portal writes, and excused operands beside a Chord Play's root). Each exception had to be taught
to the Parser, the Language Map and the Tick separately.

## Decision

**A MIDI output whose operand Cells are empty sends nothing and reports nothing.** This applies to
Raw Play `!>`, Timed Play `!~`, Monophonic Play `!%`, Raw Chord Play `!:`, Timed Chord Play `!#`,
Control Change `!c` and Pitch Bend `!b`, and to every operand each declares. A Bang still activates
it, and its Turn is still taken.

**Empty means the two Cells are empty.** The output looks at the Cells, as Orca looks at the
glyph. Whatever left them empty (the Source, a Portal write, or a nested Function's Return) makes
no difference, because none of that is visible to the output.

**A half-written operand is still an error.** One empty Cell beside a written one is malformed
input, as ADR 0069 says. Orca has no equivalent, because its operands are one Cell wide.

**Nothing else changes.** Every other Function still treats an empty operand as invalid input
under ADR 0069. Empty is not a new value or type.

## Differences from Orca

- Orca's velocity and length fall back to defaults (`f`, `1`) when empty. Here they are ordinary
  operands, so an empty one sends nothing like the others. Defaults stay an open question: adding
  them later only makes a silent Play sound, so it breaks nothing.
- Orca reads an empty port on every other operator as `0`. Orcvs keeps ADR 0069's refusal of that.

## Consequences

- The `rest:` clause, its `ignoring blanks` variant, and the parser, Language Map and Tick code
  that implements them are removed. Every Terminal Output Function that declares operands is a
  MIDI output (UDP and OSC take a Payload instead), so the rule reads the existing Terminal Output
  declaration and needs no new fact in the Function table.
- A MIDI output with empty operands is a root of the Language Map, and is scheduled like any other.
- A Monophonic Play that sends nothing leaves its channel's voice to its own Note Off.
- The Rest entry in `CONTEXT.md` becomes a description of this behaviour, not a mechanism.
