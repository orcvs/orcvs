# A Function with nothing to write answers nothing, and a Clear writes two spaces

Status: accepted. Supersedes [ADR 0069](0069-an-unwritten-operand-is-invalid-input.md)'s clause "The Absence Marker is an answer". Amends [ADR 0071](0071-a-bang-travels-as-its-glyph.md)'s bang-port clause and [ADR 0072](0072-a-standalone-bang-is-a-function.md)'s overwrite clause, both of which name the Absence Marker; their behaviour stands. Settled in `.scratch/no-absence-marker/`.

The Absence Marker was a variant of the Atom type that no Source spells and no operand accepts. Its only producers were the three pulse Functions on a Turn they do not Bang and a Copy of two empty Cells, and every one of them wrote two spaces. The Source tick therefore had to ask which Function answered before it knew what the marker meant, and its "write nothing" reading was reachable only from a test.

Orca has no such value. A port is a Cell with no lifetime: a value operator whose operation yields nothing returns before writing and leaves the old glyph, a bang port writes `*` or `.` on every frame, and `.` is an ordinary glyph, so `T` and `X` copying an empty Cell write it (`operator.js` and `library.js` at `223e45c84d`, as ADR 0071 reads them).

## Decision

**A value Function answers a Language Unit or a Clear.** A Clear writes two spaces at the Output Portal, root or nested, and a nested one leaves the operand it stands in unwritten, so the parent is invalid under ADR 0069 and its diagnostic names the child that cleared it. Equality, Delay and Euclidean answer a Clear on every Turn they do not Bang, which erases the `**` they wrote last, as Orca's bang ports write `.`. A Copy, Read or Track answers a Clear for two empty Cells, which is the copy ADR 0069 already describes. The Cells a pulse clears are the ones it reserves for its Bang, so no dependency changes. A Clear is not an Atom: no operand holds it, and it writes no Language Unit, so no reach rule reads it as a Function or a Bang.

**"No output this Tick" is no answer.** A Function that takes no Turn, or is invalid, writes nothing and leaves its last write standing (ADR 0034, ADR 0069). Nothing marks it, and every Atom has a Source encoding.

## Consequences

- Every Source behaves as before, except the diagnostic on the parent of a nested Clear. The parent reads `nested computation at column 2, row 0 cleared its operand`, naming the child, where before a nested pulse that did not Bang gave "returned nothing" and a nested Copy of empty Cells gave `expected a number, found "  "`. Two spaces cannot say which Function wrote them, so the name comes from the Turn. The parent is invalid either way.
- The Source tick no longer asks whether a Function copies a Language Unit or answers only Bang, and those two Function predicates are removed.

## Considered options

- **A Bang Output declared in the Function table, with a pulse answering whether it fired.** A Copy of empty Cells would still need its own answer, so this gives two shapes for one effect, writing two spaces.
- **A Copy that answers the raw pair it read.** Bang activation and Function Replacement read the written Atom, so the Source tick would parse the pair a second time.
- **Keep the marker and document the special case.** The type would still hold a value that is not one, and the Source tick would still ask which Function answered.
