# 07 — Complete the numeric Function family

**What to build:** Add Absolute Difference `.|`, Modulo `.%`, Minimum `.<`, Maximum `.>`, and
Equality `.=` with the Number-only contracts from ADR 0011. Equality produces Bang for equal
operands and no value for unequal operands.

**Blocked by:** 01 — Move arithmetic onto the `.` family; 02 — Free Bang spelling; 03 — Wrap general arithmetic over bytes; lang-foundations/06.

**Status:** resolved

**Tags:** release/v1

- [x] Every Function parses and round-trips through its behavior-first spelling.
- [x] Absolute Difference is symmetric and cannot underflow.
- [x] Modulo diagnoses a zero divisor and produces no result.
- [x] Minimum and Maximum return one Number.
- [x] Equality returns Bang only for equal Numbers and otherwise returns no value.
- [x] Notes and other Atom types diagnose rather than coercing.
- [x] Sequence broadcasting is left to the Sequence-values effort.

## Downstream integration

- [ ] Equality's pervasive Sequence broadcasting — the vacuous all-equal cases and the
      incompatible-length diagnostic ADR 0011 defers to ADR 0007 — is owned by
      `.scratch/sequence-values/issues/02-broadcast-atomic-functions-over-sequences.md`.

## Comments

This completes ADR 0011 without pulling first-class Sequence implementation into the migration.

Resolved at the parser, interpreter, and Source seams. The five Functions join the canonical
`define_functions!` table, so the parser, the Language Map, and `Function::ALL` recognize them by
their definition alone; `.<` and `.>` are pinned against the `<<` and `>>` Activation spellings the
parser tests first. Absolute Difference, Modulo, Minimum, Maximum, and Equality each carry
exhaustive coverage over all 65,536 operand pairs, with the oracle computed independently of the
implementation. Equality answers a pulse rather than a truth value: `Atom::Bang` for equal Numbers,
and `Atom::Empty` — already the Interpreter's "no result write" signal — otherwise, so the unequal
case leaves no Cell the next Tick would read back as an operand. Modulo carries its own
`ModuloByZero` diagnostic rather than reusing Division's, so the Source learns which Function it
wrote.

Exhaustive Function-level arithmetic laws belong to
`.scratch/property-testing/issues/05-exhaustive-arithmetic-and-note-conversion.md`.
