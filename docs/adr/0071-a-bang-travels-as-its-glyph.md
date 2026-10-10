# A Bang travels as its glyph, as Orca's `*` does

Status: accepted. Supersedes [ADR 0014](0014-spatial-functions-preserve-performative-behaviour.md)'s relay clause for a Jump's Bang, now a Copy's. Amends [ADR 0032](0032-schedule-tick-execution-by-dependency.md)'s manual-Bang and absence clauses and [ADR 0070](0070-read-write-and-copy-are-separate-families.md)'s amendment on a Write's `**` value. [ADR 0019](0019-map-orca-capabilities-onto-orcvs-language-families.md) holds Orcvs to Orca's performative capabilities while changing its encoding and scheduling convention; this decision restores three Bang behaviours that departed from Orca without a recorded reason.

Orca's behaviour, read from `hundredrabbits/Orca` at `223e45c84d` (`desktop/sources/scripts/core/orca.js`, `operator.js`, `library.js`):

- A Bang is the glyph `*`. Each frame an operator runs when it is passive or when one of its four orthogonal neighbours holds `*` at its turn. The `*` operator's only action is to erase itself at its own turn on the following frame.
- An operator with a bang port writes `*` when it bangs and `.` when it does not, so its output Cell is cleared on every frame it is quiet.
- J, Y, X, O, T, P, Q and G read a Cell's glyph and write it with `orca.write`, which overwrites whatever stands at the destination. A `*` they carry is written as `*`, replaces an operator or data at the destination, and bangs that Cell's neighbours. Nothing activates the operator it replaced.
- A `*` typed into the grid bangs its neighbours on the next frame, then erases itself.
- Orca records nothing about who wrote a `*`. A produced `*` reaches each neighbour once only because of row-major turn order. A `*` in another operator's input is locked, so it never erases itself, and bangs its neighbours every frame. Once that operator is removed, the `*` behaves as a typed one: it bangs the neighbours whose turn comes first, then erases itself.

## Decision

**A Function that passes a pair through carries `**` as it carries any other pair.** A Copy, Read, Track, Write or Push whose answer is a Bang writes `**` over its complete destination, whatever stands there, and that write is its Bang output: the roots aligned with the destination are activated during the Tick, in dependency order, as for every Function that answers Bang. A root whose anchor the `**` covers is overwritten, not activated, and the Cells it leaves behind are ordinary Source. An out-of-Grid destination diagnoses as any out-of-Grid write does. ADR 0014's relay clause, which activated a root at the destination without changing it and refused a Bang onto an occupied non-root, is withdrawn: it had no counterpart in Orca's `J`.

**A Function whose answers are only Bang and the Absence Marker writes its answer through its Output Portal on every Turn.** Equality `.=`, Delay `~*` and Euclidean `~%` write `**` when they bang and clear their Output Portal's two Cells when they do not, as Orca's bang ports write `.`. This holds for a nested Function as for a root: a nested one that answers the Absence Marker clears its own Output Portal and still returns nothing to its parent, which is then invalid as before. The Cells written are the ones the Function already reserves for its Bang, so no dependency is added. Every other Function keeps ADR 0032's rule that ordinary absence performs no write.

**A `**` that the previous Tick did not write fires once.** A standalone `**` present in Source at the start of a Tick, other than the display of a Bang the previous Tick produced, is a Bang at its position: it activates its aligned roots during that Tick and is cleared, as Orca's typed `*` is. The display of a Bang the previous Tick produced is cleared before any Turn without activating anything, because dependency order already delivered that Bang to every aligned root in its own Tick; firing it again would activate them twice. Source distinguishes the two by remembering which Cells the previous Tick wrote as Bang display. A `**` in a typed operand remains invalid syntax and activates nothing.

## Consequences

- `@$0001**` above a Raw Play at row 2 still plays it; aimed at the Raw Play's anchor, it overwrites the Function's spelling, and the next Tick clears the display and leaves the operands as Source.
- A chain such as `.=` into `=v` into `=v` still carries a Bang down a column in one Tick, each Copy writing `**` and activating the roots aligned with its own destination.
- Anything placed in the Output Portal of Equality, Delay or Euclidean is cleared on every Tick that Function does not bang.
- A performer can fire a root by typing `**` beside it.
- Source remembers Bang display for one Tick only. A `**` left in an operand by an earlier Tick is not cleared, and fires once as typed if an edit later makes it standalone, as Orca's `*` does once freed from a locked input.
- Bang ordering, same-Tick delivery regardless of Position, Bang display cleanup, Halt, chain heads, and the clearing of an empty Copy input are unchanged.

## Considered options

- **Keep the relay for Copies and give Writes Orca's overwrite.** Two Bang rules for Functions that all pass a pair through, and the Copy rule still has no counterpart in Orca.
- **Drop the Bang display at the end of its Tick, so every `**` at the start of a Tick was typed.** No record is needed, but a performer never sees a Bang, which Orca always shows.
- **Treat every standalone `**` at the start of a Tick as a Bang.** Every aligned root of a produced Bang would activate in its own Tick and again in the next.
