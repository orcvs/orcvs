# A Comment closes at the next `||`, and its text is data a Read can select

Status: accepted. Supersedes [ADR 0035](0035-spell-the-comment-as-a-parser-unit.md)'s clause that a Comment claims every remaining Cell of its row; every other decision in ADR 0035 stands.

Orca's `#` toggles a comment, so one row holds several, and Source after a closing `#` runs. Its `Q` and `T` read glyphs inside a comment, which is how the Learn Orca tracker keeps its pattern of Notes framed in `#..#` beside the operators that read them. ADR 0035's Comment claimed the rest of its row, so an Orcvs tracker could frame nothing, and a Read selecting a pair under a Comment refused it as part of another Language Unit.

## Decision

**A Comment claims from its `||` introducer through the next `||` on its row, which closes it.** With no closer it claims the rest of the row, as before. The closer is the leftmost `||` after the introducer, so `|||` is unclosed, `||||` is an empty closed Comment, and a longer run of pipes closes at its first two: `|| x |||` closes after `x ` and leaves a `|` that is incomplete Source. A closed Comment is one Language Unit like any other, so the row walk resumes after it and the Source there parses. The closer is found by search because a Comment's text is never parsed; the introducer is still recognized only where an Expression could start. The Parser records the text's Cells with the claim, and the Language Unit carries them, so whether a Comment is closed is decided once and every reader asks the unit rather than its Cells.

**A Comment's text is data.** The Cells between its delimiters belong to the Comment, which is never scheduled and reports nothing, so a Read, Track or Copy that selects a pair of them reads it as it reads a pair no Language Unit covers: the Interpreter decodes it, and an undecodable pair diagnoses at the reader. A pair touching either delimiter is refused as partial input, as a pair straddling any unit is.

## Consequences

- Source that held several `||` on one row reads differently: the second closes the first. The Function reference's six group headers on row 0 are each written `|| … ||`.
- A Comment's text Cells are ordinary Cells to every Function; its two delimiters are the Comment's occupied Cells. A Halt aimed at its text locks nothing and does not diagnose, while one aimed at a delimiter is aimed at no Expression root. A mover enters blank text and is refused by glyphs there without contacting a Language Unit, and a Write into the text writes its Cells. A pair straddling a delimiter meets part of the Comment. The Comment's Span is still its whole claim, so paint, Tokens and scheduling are unchanged.
- A Source effect that overwrites a closer obscures those Cells, so the Comment no longer covers them; what remains of its text stays readable until the next Snapshot re-parses the row.

## Considered options

- **Close at the row's end only, and read through it.** It frames nothing on the left of an operator, so the tracker's pattern still could not share a row with Source.
- **A distinct closer spelling.** Every doubled glyph is spoken for or refused in ADR 0035, and one delimiter that toggles is Orca's own model.
