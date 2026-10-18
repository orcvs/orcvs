# A Play whose note a nested Function cleared rests

Status: accepted. Amends [ADR 0073](0073-a-function-with-nothing-to-write-answers-nothing.md)'s clause that a nested Clear leaves its parent invalid, for the note operand of Raw Play `!>`, Timed Play `!~` and Monophonic Play `!%`; every other decision in ADR 0073 stands. Settled in `.scratch/play-rest/`.

Under ADR 0073 a nested Function that answers a Clear leaves the operand it stands in unwritten, so its parent is invalid and is diagnosed as `nested computation at column X, row Y cleared its operand`. A tracker whose voices read their notes from a pattern of Cells, a Read or Track in each Play's note slot, rests a voice on every empty step, and every resting voice was diagnosed every Tick: seven of the eight Plays in the Learn Orca tracker port. The sound was already right, since an invalid Play sends nothing. Orca's `:` with no note plays nothing and says nothing.

## Decision

**A Clear in a Play's note operand is a rest.** The Play's Turn is taken, sends no Play Command and raises no diagnostic. The Function table declares the operand with a `rest: note` clause on each of the three rows, so the rule is a fact of the definition, which the Source tick reads when it decodes a Turn's operands; it does not ask which Function it is running.

**Only the note, and only a Clear.** A Clear in a Play's channel, velocity or length, or in any operand of any other Function, leaves the parent invalid as ADR 0073 says. Note to Number `.v` reads a Note too, and still refuses a Clear: it answers a value rather than sounding one. A rest also needs every other operand to decode, so a Play whose note and length were both cleared is diagnosed for its length. A nested Function that is invalid gives no Clear and no rest, so a half-typed Note a Read selects is diagnosed at the Read, and the Play names it as having returned nothing. An unwritten or partly written note slot with no nested Function is invalid input under ADR 0069, as before.

## Consequences

- Source that rests a Play on a cleared note is quiet where it was diagnosed every Tick. Nothing else changes: the nested Function still writes its Clear through its own Output Portal.
- A rest does not evaluate the Play, so a channel or velocity outside its MIDI domain is diagnosed only on a Turn that has a note to play.
- A rest sends no Play Command, so a Monophonic Play that rests leaves its channel's voice to its own Note Off rather than replacing it with silence, as a velocity or length of `00` would.

## Considered options

- **Diagnose rests as before.** The diagnostic reports correct Source on every Tick, and a performer learns to ignore the panel that should tell them what is wrong.
- **Rest on any cleared operand of a Play.** A cleared channel, velocity or length is not a musical rest; it is Source that cannot say what to play, which is what ADR 0073's diagnostic exists for.
- **Treat a Clear in any Note operand as a rest.** Note to Number has no silence to fall back on, so it would answer nothing and say nothing about why.
