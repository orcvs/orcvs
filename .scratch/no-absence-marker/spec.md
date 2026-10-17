# No Absence Marker: an answer is a value, and "no output" is no answer

Status: ready-for-agent

## Problem Statement

`Atom::Empty`, the Absence Marker, is a variant of the value type that is not
a value. No Source spells it and no operand accepts it, and it stands in for
two different facts:

- **A pulse that did not fire.** Equality `.=`, Delay `~*` and Euclidean `~%`
  answer it on a quiet Turn (`stack.rs` `predicate`, `functions/tick.rs`
  `pulse`).
- **A Copy of two blank Cells.** `functions/copy.rs` answers it for `"  "`.
  ADR 0069 already says that copying empty Cells is a successful evaluation
  that moves those Cells, so this is a value that has been mislabelled as
  absence.

Since the type can't tell the two apart, `orcvs` asks which Function answered
before it decides what "nothing" means:

- `tick/execution.rs`, in the `Rendered::Nothing` arm: it clears the Output
  Portal only when `copies_language_unit() || answers_only_bang()`, and
  otherwise writes nothing.
- `tick/execution/operands.rs` `decode`: a Copy's nothing is returned as
  `EMPTY_PAIR`, and every other nothing is "returned nothing".

Guards also keep the fake value out of every place a value goes:

- `operand.rs` `Untyped::from_atom`
- the proptest in `parser.rs` that checks the parser never yields it
- `Encoding::render`'s `Rendered::Nothing`
- `Display` spelling it `_`

The "write nothing" branch is reached only by tests that state an Addition
answering `Atom::Empty`. Every shipped producer of `Atom::Empty` is a pulse
or a Copy, and both of those write. So that branch is a test-only input in
shipped code, which CLAUDE.md forbids.

## Orca, and what persists

Orca's ports are ordinary Cells and have no lifetime. A value operator whose
operation yields nothing returns before writing, and the old glyph stays
(`operator.js` `output`). A bang port writes `*` or `.` on every frame
(ADR 0071, read at `223e45c84d`). `.` is an ordinary glyph, so `T` and `X`
copying an empty Cell write `.` (the amendment to ADR 0062).

Orcvs already matches this. An Output write stays in Source until something
overwrites it (ADR 0003). A Function that takes no Turn, or is invalid, writes
nothing and leaves its last write standing (ADR 0069). "No output this Tick"
therefore needs no marker: it is the absence of a Turn or of a valid
evaluation, and the Turn state already records that (`Turns::result` is an
`Option`).

The one Output whose content lasts one Tick belongs to a pulse. It writes `**`
or two spaces on every Turn, and the two spaces are a write like any other,
not an absence.

## Solution

Each of the two facts gets its own type, and neither of them is an `Atom`:

- **A Language Unit to write** is `Interpretation::Cell(Atom)`. This is
  unchanged, and every `Atom` now has a Source encoding.
- **Two blank Cells to write** is a new `Interpretation::Clear`. It has no
  `Atom` and no Function attached. `orcvs` writes the two spaces at the Output
  Portal, and for a nested Function it returns them. The admitted write's
  `written` is `None`, which every reach rule already treats exactly as it
  treats `Some(Atom::Empty)`. They only match `Some(Atom::Function(_))` and
  `Some(Atom::Bang)`.

1. **A pulse that doesn't Bang answers `Clear`.** Equality, Delay and
   Euclidean answer `Cell(Atom::Bang)` or `Clear`, as Orca's bang ports write
   `*` or `.` every frame. The clear erases the last `**`, which a standalone
   `**` left in Source would otherwise fire again (ADR 0072). The cleared
   Cells are the ones the Function already reserves for its Bang, so no
   dependency changes. A nested pulse that doesn't Bang returns two spaces.
   Its parent's operand is then unwritten, and the parent is diagnosed under
   ADR 0069. Toby confirmed this on 2026-10-17.
2. **A Copy of two blank Cells answers `Clear`.** Every other Copy still
   answers the Language Unit it read. A copied `**` keeps activating through
   `Cell(Atom::Bang)`.
3. **Remove `Atom::Empty`.** These go with it, since they exist only for it:
   - `Rendered::Nothing`; `Encoding::render` answers an `Encoding` or a
     render error
   - the `copies_language_unit() || answers_only_bang()` arm in
     `tick/execution.rs`
   - the `copies_language_unit()` arm in `tick/execution/operands.rs`
   - `Untyped::from_atom`'s refusal
   - the parser proptest guard
   - the `_` spelling
   - the tests that state an answer of `Atom::Empty`

   `Turns::result` stores the Turn's answer, a unit or a clear, so a returning
   child is decoded from its Cells alone.

## Behaviour

Every Source behaves as it does on main, with one exception. A nested quiet
pulse currently diagnoses its parent with "nested computation … returned
nothing". After this change the parent diagnoses with the message for an
unwritten operand. The parent is invalid in both cases.

## Decisions

- On 2026-10-17 Toby confirmed that a nested pulse that does not Bang leaves its parent invalid, diagnosed as an unwritten operand.

- On 2026-10-17 Toby reframed candidate 3. The underlying design is the
  problem, not the blank, and the type system should do the work. Toby
  approved writing this spec.
- The Absence Marker is removed from the language, not renamed. CONTEXT.md
  loses its entry, and the entries for Equality, Delay, Euclidean and Return
  are reworded.
- A new ADR records the decision. It supersedes ADR 0069's "The Absence
  Marker is an answer" clause and amends the bang-port clause of ADR 0071.

## Rejected

- **A Bang Output declared in the Function table.** It would need a pulse
  type plus a separate Cells answer for Copy: two new shapes for one effect,
  which is writing two spaces.
- **A Copy that answers the raw pair it read.** Bang activation and Function
  replacement read the written `Atom`, so `orcvs` would have to parse the pair
  a second time.

## Out of scope

- Track, Read and the other pass-through Functions. They already carry Cells.
- Candidate 5, MIDI selection as one Phase.
