# A Play's note is optional by type

Status: accepted. Supersedes [ADR 0075](0075-a-play-with-a-cleared-note-rests.md),
[ADR 0104](0104-a-blank-note-or-root-is-a-rest-however-it-got-blank.md) and
[ADR 0107](0107-a-blank-note-is-invalid-input-however-it-got-blank.md). Amends
[ADR 0069](0069-an-unwritten-operand-is-invalid-input.md)'s rule that an unwritten slot is invalid
input, and [ADR 0073](0073-a-function-with-nothing-to-write-answers-nothing.md)'s rule that a nested
Clear leaves its parent invalid, by operand type: both hold for every operand whose type requires a
value, and neither holds for an optional one. Settled in `.scratch/optional-play-note/`.
[ADR 0109](0109-a-blank-operand-can-stand-for-a-fixed-value.md) takes up the default for an empty
operand this ADR set aside: `Defaulted<O, BLANK>` binds a blank slot as a fixed Number.

A tracker or melody pattern leaves most of its steps empty, and a performer reads an empty step as
"nothing plays here", as Orca's `:` with no note does. ADR 0075 and ADR 0104 made a blank note a
Rest: an exception that the Function table, the Parser, the Language Map and the Tick each had to
spot on their own, and whose copies drifted apart. ADR 0107 removed the Rest, which made every empty
step of a pattern a diagnostic on every Tick it is reached. Those diagnostics report no mistake, hide
real ones among them, and each allocates its message on the Tick path.

"A blank note plays nothing" is part of what a Play's note is, not an exception to the rules for
operands. Nothing in the program said so.

## Decision

**An operand type answers whether its slot may hold no value.** The operand adapter `Optional<O>`
admits a slot that holds no value, for any operand type `O`. It is read as `O` is read, and a value
that is present binds as `O` binds it, so a value `O` refuses is refused with `O`'s diagnostic. A
slot that holds no value, whether its Cells are blank or a nested Function answered a Clear, binds
to "no value" instead of being refused. Every other operand type refuses such a slot with the
diagnostic it always gave: the same message and anchor for a blank slot, and `nested computation at
column X, row Y cleared its operand` for a nested Clear.

**A Play declares its note `Optional<Note>`.** This covers Raw Play `!>`, Timed Play `!~` and
Monophonic Play `!%`. A Play that receives no note sends no Play Command and raises no diagnostic.
That holds whether the Source was written blank, a Portal write emptied the slot, or a nested
Function cleared it. A half-typed or mistyped note is still diagnosed, exactly as a plain Note is.

**Admission comes from the signature alone.** The Function table derives each slot's admission
from the operand type it declares, so there is no separate list and no `rest:` clause. The Parser
reads a blank slot as absent only where the signature admits one, and diagnoses a blank plain
operand before any Tick, as before. The Parser, the Language Map and the Tick carry an absent
operand without interpreting it, and only binding decides what it means.

**Every other operand still binds.** A Play's channel, velocity and length bind before its body
runs, so a blank channel or an out-of-range velocity is diagnosed on a blank step too. This
reverses ADR 0104's "a Rest checks no operand's range". The Play keeps its Turn and its place in
Tick order, and as [ADR 0065](0065-an-error-never-stops-the-performance.md) has it, an absence or an
error reaches no other Expression.

## Consequences

- An empty step in a pattern is quiet and clean: the shipped Melody, Tracker, Sequencer and Knobs
  examples raise no diagnostic for it.
- Reading a Play with a blank note, and taking its Turn, allocate nothing: binding no value and
  returning no command build no message and no collection. `lang/tests/allocation.rs` measures the
  parse and the Turn. The Tick's decode of a Turn's operands costs a blank-note Play what it costs
  any Function, and that cost is not measured here.
- Making another operand optional is a change to its declared type, and needs no change to the
  Parser, the Language Map or the Tick.
- A Chord Play's root stays a plain Note. An empty pattern row also blanks the Interval Set, Voicing
  and inversion, and whether those become optional too is its own decision.

## Considered options

- **Keep a blank note invalid (ADR 0107).** It needs no exception, but it diagnoses every empty step
  of a correct pattern on every Tick.
- **A Rest, or each Function deciding what its own empty slots mean.** Both place the rule outside
  the operand's type, so each module that meets an empty slot needs its own copy of it.
- **A default value for an empty operand.** It would sit at the same place as `Optional<O>` and is a
  separate decision; nothing a Play plays has a default note.
