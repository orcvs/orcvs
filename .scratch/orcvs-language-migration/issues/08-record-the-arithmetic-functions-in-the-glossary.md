# 08 — Record the arithmetic Functions in the glossary

**What to build:** One `CONTEXT.md` glossary entry for the nine arithmetic Functions of the `.`
family, so the glossary states what the code already implements. This is documentation only; no
behaviour changes and no Rust changes.

**Blocked by:** 07 — Complete the numeric Function family.

**Status:** ready-for-agent

**Tags:** release/v1

- [ ] `CONTEXT.md` carries an entry naming all nine: Add `.+`, Subtract `.-`, Absolute Difference
      `.|`, Multiply `.x`, Divide `./`, Modulo `.%`, Minimum `.<`, Maximum `.>`, and Equality `.=`.
- [ ] The entry states the family rule the code holds: every member takes two Number operands and
      returns a value, general arithmetic wraps within the byte range, and a Note or any other Atom
      type diagnoses rather than coercing.
- [ ] The entry states the two behaviours that are not plain arithmetic: Divide and Modulo diagnose a
      zero divisor and produce no value, and Equality returns Bang for equal operands and no value
      for unequal ones.
- [ ] The entry sits between `Numeric Conversion Function` and `Sequence`, keeping the numeric
      Functions together, and carries an `_Avoid_` line in the style of its neighbours.
- [ ] Every spelling in the entry is checked against `lang/src/atom.rs`'s `define_functions!` block
      rather than against an ADR, so the glossary records what shipped.

## Comments

`CONTEXT.md` goes straight from `Numeric Conversion Function` to `Sequence`. The nine arithmetic
Functions have no entry at all, while `.v` and `.^` — the two conversions — do. So the glossary
describes the exception and omits the rule.

The Functions themselves are built and settled. `issues/07` is `resolved`, and
`lang/src/atom.rs:153-166` declares all nine with `[Number, Number]` signatures. This issue does not
reopen any of that; it closes the gap between what the code holds and what the glossary says it
holds.

That gap is release-relevant, which is why this carries the `release/v1` tag. The Definition of Done
requires that "User-facing documentation, `CONTEXT.md`, implemented behavior, and the shipped
inventory agree", and its Function-inventory line names all nine spellings. A reviewer checking that
line against the glossary today finds nothing to check it against.

Kept deliberately small: one entry, no renaming, no ADR change. ADR 0008 and ADR 0011 already hold
the decisions; this only states them where the vocabulary lives.
