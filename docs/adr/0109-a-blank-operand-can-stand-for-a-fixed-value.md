# A blank operand can stand for a fixed value

Status: accepted. Settled with Toby on 2026-11-13 in `.scratch/musical-terminology/` (ticket 12).
It takes up the default [ADR 0108](0108-a-plays-note-is-optional-by-type.md) left as "a separate
decision", beside that ADR's `Optional<O>`.

ADR 0108 made a slot that may hold no value a property of its operand type: `Optional<O>` admits
a blank slot and binds it to "no value". It set aside a default for an empty operand: nothing a
Play plays has a default note. Monophonic Play's `legato` (ticket 13) is the first operand that
does: blank means "retrigger", the same as `00`, so Source written before the operand existed, and
a musician who never wants legato, leaves it blank and gets what `00` gives. The Function body
should not have to tell a blank slot from a written `00`, any more than it tells a Portal write
from a literal.

## Decision

**The operand adapter `Defaulted<O, BLANK>` gives a blank slot a fixed Number.** `O` is an operand
read as a Number, and `BLANK` is a Number fixed in the Function table where the operand is
declared. A written value is read and bound exactly as `O` reads and binds it. A slot that holds no
value, whether its Cells are blank or a nested Function answered a Clear, binds as if `BLANK` were
written there, through `O`'s own bind, so the Function body receives the same value for both.

**A Function table row states the value.** A row declares the operand as
`role: Defaulted<Number, 0x00>`: the table's row grammar takes a literal as an operand type's second
parameter, beside the operand type the first one names.

**Admission still comes from the signature alone.** `Defaulted` answers that it admits a blank
slot through the same `Operand::ABSENT` that `Optional` answers through, so the Parser, the
Language Map and the Tick need no change: the Parser reads a blank slot as absent where the
signature admits one, the Language Map and the Tick carry the absent operand without interpreting
it, and binding alone decides what it means.

**A fixed value lies in its operand's domain before it ships.** `O`'s bind is not `const`, so the
build cannot check `BLANK` against it. A `lang` test binds the blank of every operand every
Function declares and fails on one that `O` refuses, with the diagnostic a written value would
get. `ABSENT` therefore keeps answering the bound value, not a `Result`, and the bind it runs
asserts that proven invariant.

**Neither adapter wraps one that admits a blank slot.** `Optional<Defaulted<..>>` and
`Defaulted<Optional<..>>` would each give one blank two meanings, so both `ABSENT`s assert at
compile time that their inner operand requires a value.

**Only a Number has a fixed value.** `BLANK` is a byte. A default Note, Interval Set or Voicing
would need its own literal form in the table, and no operand asks for one.

## Consequences

- Making an operand default to a value is a change to its declared type in its Function's row, as
  making one optional is: no change to the Parser, the Language Map or the Tick.
- A Source cannot tell a blank `Defaulted` slot from one holding `BLANK` by what the Function does;
  Source Paint still shows the blank Cells.
- `lang`'s `operand` tests cover a blank slot binding its fixed value, a written slot binding what
  is written, and a written value refused as `O` refuses it. A fixed value outside `O`'s domain
  fails the table test as a written one would be refused, and each stacking of the two adapters
  fails to compile.

## Considered options

- **`Optional<O>` with the default applied in the Function body.** Each body would carry its own
  copy of what blank means, which ADR 0108 rejected for the Rest.
- **A default as a third admission beside "required" and "optional" in the Function table.** It
  would put the rule outside the operand's type, so the table and the binder could disagree.
- **`ABSENT` answering a `Result`, so binding refuses a fixed value outside the domain.** The
  fault would surface only when a Source left the slot blank, on every Tick, blamed on blank Cells
  the musician wrote correctly; the table test finds it before it ships.
- **A fixed value that skips `O`'s bind.** It would need a second, unchecked way to build `O`'s
  bound value, and a value out of `O`'s domain would reach the body.
