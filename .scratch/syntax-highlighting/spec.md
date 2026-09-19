# Syntax highlighting scope

What Source Paint must distinguish. Not colours, not Cursor, not Sector Seams.

Shipped paint colours `Token` only. It does not tell pending from valid from invalid, and it does not tell an Output Portal from an Operand.

Every role below is read from the Language Map or the Render Frame. The Output Portal highlight is the one derived fact: each root Function's declared Output Portal placed by its anchor (`05`), which interprets no spelling. Do not mint a third classifier.

## Function

The two-Cell spelling. One Token.

- **Pending Operand** — the Function has claimed the Cells; they are still empty. Declared Token is visible with no letter. Not Pending Operand Encoding (spatially written characters waiting for the next evaluation).
- **Valid Operand** — an Operand Literal that binds as the declared Token: Number, Note, Char, Atom, or Sequence.
- **Invalid Operand** — claimed Cells whose content fails to bind, or an operand diagnostic from evaluation. The Token is still the declared one (`c4` in a Number slot is still Number).
- **Output Portal** — the Cells a root Function that answers a value writes to: its Reservation from its Output Portal, one row south unless the Function names otherwise. Highlighted from the current revision, before any Tick, whether empty or written.
  - Scalar Atom
  - Sequence
  - Bang (`**`)
  - none — Terminal Output Function; Halt; Source-writing Function; Absence Marker (plans no write)

A nested Function is highlighted as a Function. The parent consumes its typed result, not an Operand Literal.

## Bang

A standalone `**` Expression. Not a Function. A `**` rejected in a typed operand is Invalid Operand.

## Comment

- Introducer `||`
- Body the introducer claims
- A lone `|` is incomplete Source, not a Comment

## Unclaimed

- Empty Cell, no Token
- Leftover Char — letters no Expression claimed

## Overlays

Facts on the Expression, not on the Cell:

- will execute (`root`) vs will not
- diagnostic coverage (Expression or lexical)
