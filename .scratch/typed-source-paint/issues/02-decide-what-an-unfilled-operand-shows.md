# 02 — Decide what an unfilled operand Cell shows

**What to build:** A decision, recorded where the vocabulary lives, about whether an operand Cell a
Function declares but nothing fills shows a character or is painted by its declared type.

**Blocked by:** 06 — Delete Glyph.

**Status:** resolved

- [x] The question is answered for all of `Number`, `Note`, `Char`, `Atom` and `Sequence` operands,
      not only the two that `GlyphString` currently spells.
- [x] Whichever way it goes, the console blank table's remaining reason to exist is stated or the
      table is deleted. Ticket `06` already retired GlyphString.
- [x] If operand slots are painted rather than spelled, `03`'s handover carries the declared-type
      colours; if they are spelled, the spelling table returns with a producer that reaches it.

## Comments

`GlyphString` spells `h` for an unfilled Number slot, `n` for a Note and `F` for a Function. That is
the same terminal inheritance `Marker` and `Highlight` were: standing a character in because a
terminal has only characters to stand anything in with. A console that paints can colour the empty
Cell in its declared type and spell nothing.

The reason this is a decision and not a consequence: those spellings read as deliberate
operand-placeholder design rather than leftovers — Orca shows operand hints, and an unfilled slot
you can see the shape of is genuinely useful while writing. Retiring them is a judgement about what
the console should look like, which is why `06` deletes the *dead* Glyph vocabulary and leaves this
      alone.

Note that these spellings are reached today, which is what makes this a choice between keeping a
working feature and removing one. `LanguageMap` gives every Cell an Expression's claim covers that
claim's classification whether or not the Cell holds a byte — `language_map.rs:1584` states the rule
— so an Addition's reserved but unfilled operand Cells answer `Number` with no content and spell
`h`. `paint.rs:633` asserts exactly that. The producer this decision would need already exists:
keeping the placeholders costs nothing to build, and retiring them is a deletion with a visible
effect on screen, not the removal of dead vocabulary.

**Resolved by `syntax-highlighting/03`, painted rather than spelled.** An empty claimed operand Cell
shows only the Fill tint `syntax-highlighting/02` already paints for its declared Token
(`console/src/style.rs::fill_tint_colour`) — no letter. `03` deletes `console/src/paint.rs`'s blank
spelling table (`BLANK_TOKENS`, `blank_token_index`, `blank_character`, `CellCharacters`) and the
three tests that pinned it, and a Cell's shown character becomes `cell.content().unwrap_or(' ')`
with no Token lookup at all. `Token::Char` is answered vacuously: no Function signature ever declares
a Char operand (`lang/src/atom.rs`'s `operand_token!` has no `Char` arm, and `lang/src/stack.rs`'s
`check_token` marks that arm `unreachable!`), so there is no empty *claimed* Char Cell to paint
either way — every `Token::Char` the Render Frame carries is the Leftover Char fallback, which was
already excluded from the tint by `syntax-highlighting/02`. A slot cut off at the row edge carries
its declared Token the same way an ordinary unfilled slot does — `LanguageMap::token_at` already
reads it from the Parser's own record of the Cells the row's tail held
(`lang::Parser::take_token`'s error path), so no second per-Cell classifier was needed beside the
Language Map. See `syntax-highlighting/03` for the full change and its tests.
