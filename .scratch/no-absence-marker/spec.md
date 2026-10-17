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

The one Output with a one-Tick lifetime is the Bang Output, and the lifetime
belongs to the Output, not to an answer.

## Solution

1. **A pulse answers whether it fired.** Equality, Delay and Euclidean answer
   a pulse, not an `Atom`. Their Output is a Bang Output (from
   `BangAnswer::Only` in the Function table). It writes `**` when the pulse
   fired and two spaces when it did not, on every Turn, root or nested, as
   Orca's bang ports write `.`. The cleared Cells are the ones the Function
   already reserves for its Bang, so no dependency changes. A nested pulse
   returns the Cells its Output wrote. A quiet one therefore leaves its
   parent's operand unwritten, and the parent is invalid under ADR 0069, as
   it is today.
2. **A Copy answers the Cells it read.** A Copy still validates that its
   Input Portal holds one whole Language Unit or two blank Cells, and its
   answer is that pair. `orcvs` writes it and returns it verbatim. A copied
   `**` activates through its glyph, which is ADR 0071's rule, so nothing
   about Bang activation needs the copied pair parsed into an `Atom`.
3. **Every value Function answers a value.** `Atom::Empty` is removed, along
   with these parts that exist only for it:
   - `Rendered::Nothing`
   - the two `copies_language_unit()` arms and the `answers_only_bang()` arm
   - `Untyped::from_atom`'s refusal
   - the parser proptest guard
   - the `_` spelling
   - the test that states an Addition answering it

## Behaviour

Every Source behaves as it does on main, with one exception. A nested quiet
pulse currently diagnoses its parent with "nested computation … returned
nothing". After this change the parent diagnoses with the message for an
unwritten operand. The parent is invalid in both cases.

## Decisions

- On 2026-10-17 Toby reframed candidate 3. The underlying design is the
  problem, not the blank, and the type system should do the work. Toby
  approved writing this spec.
- The Absence Marker is removed from the language, not renamed. CONTEXT.md
  loses its entry, and the entries for Equality, Delay, Euclidean and Return
  are reworded.
- A new ADR records the decision. It supersedes ADR 0069's "The Absence
  Marker is an answer" clause and amends the bang-port clause of ADR 0071.

## Open for the build

- Whether the pulse is its own `Interpretation` variant or a `bool` carried
  by the Bang Output. Pick whichever leaves `orcvs` with no Function check.
- Whether a Copy's pair travels as `Interpretation::Cells` or as an
  `Encoding` built in `lang`. `Encoding` lives in `orcvs` today.

## Out of scope

- Track, Read and the other pass-through Functions. They already carry Cells.
- Candidate 5, MIDI selection as one Phase.
