# Syntax highlighting scope

What Source Paint must distinguish. Not colours, not Cursor, not Sector Seams.

Shipped paint colours `Token` only. It does not tell pending from valid from invalid, and it does not tell a Result from an Operand.

Every role below already exists on the Language Map or the Render Frame. Do not mint a third classifier.

## Function

The two-Cell spelling. One Token.

- **Pending Operand** — the Function has claimed the Cells; they are still empty. Declared Token is visible with no letter. Not Pending Operand Encoding (spatially written characters waiting for the next evaluation).
- **Valid Operand** — an Operand Literal that binds as the declared Token: Number, Note, Char, Atom, or Sequence.
- **Invalid Operand** — claimed Cells whose content fails to bind, or an operand diagnostic from evaluation. The Token is still the declared one (`c4` in a Number slot is still Number).
- **Result** — the Output Portal write, one row south unless the Function names otherwise.
  - Scalar Atom
  - Sequence
  - Bang (`**`)
  - none — Terminal Output Function; Absence Marker

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
