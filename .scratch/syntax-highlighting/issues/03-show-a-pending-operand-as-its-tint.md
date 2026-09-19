# 03 — Show a Pending Operand as its tint, with no letter

**What to build:** An operand a Function has claimed but nothing fills shows its declared Token only as the background tint. No placeholder letter is drawn.

**Blocked by:** 02 — Tint Functions and Operands by their Token colour.

**Status:** resolved

- [x] An empty claimed operand Cell of every Token (Number, Note, Char, Atom, Sequence) shows the Token tint and no character.
- [x] The placeholder spellings (`h`, `n`, `c`, `F`, `*`) and the blank-character table that produces them are deleted, with the test that pins them.
- [x] An operand slot cut off by the end of its row also shows its declared Token tint, for each of the slot's Cells that exist in the Grid. Today that slot records no Cells and carries no Token; establish where that fact belongs in the Language Map rather than adding a per-Cell classifier beside it.
- [x] `typed-source-paint/02` is resolved by this ticket in the "painted, not spelled" direction, with a pointer to this ticket as its answer.
- [x] Focused tests cover an empty Number and Note slot, and a slot cut off at the row edge.

## Comments

Pending Operand is distinct from Pending Operand Encoding: this is only the claimed-and-empty case.

**Deleting the placeholder table was the whole implementation for boxes 1 and 2.** `console/src/
paint.rs::Paint::derive` built a `CellCharacters` table once per Render Frame and read `cell.content()
.unwrap_or_else(|| blanks[blank_token_index(cell.token())])` for the character each Cell shows. The
Token-driven tint `syntax-highlighting/02` added already paints the declared-type background under
that placeholder letter — the letter was the only thing left standing between "tint" and "tint, no
letter". Deleting `BLANK_TOKENS`, `blank_token_index`, `blank_character` and `CellCharacters`, and
reading `cell.content().unwrap_or(' ')` in their place, is the whole change: a Cell with nothing
written now answers the space it always would have if no signature had claimed it, and
`console.rs`'s `place_glyphs` already skips drawing a Glyph for the space, so no drawing-side change
was needed at all. The three pinning tests (`a_blank_cell_shows_what_its_token_spells`,
`each_cell_shows_the_character_the_table_answers`, `a_cell_answers_its_character_with_no_context`)
are deleted; `console::tests::a_glyph_is_painted_for_every_cell_that_shows_one_and_no_other` is kept
but tightened from "more than the two written Cells show something" to "exactly the two written
Cells show something", since the deleted table was the only other source of a shown character.

**`Token::Char` is excluded vacuously, as `syntax-highlighting/02`'s implementer already
established.** No Function signature declares a `Char` operand — `lang/src/atom.rs`'s
`operand_token!` macro has no `Char` arm (a signature naming one would fail to compile), and
`lang/src/stack.rs`'s `check_token` marks that arm `unreachable!` with "no operand type declares a
Token the Parser mints only as a label". So there is no empty *claimed* Char Cell for box 1 to cover:
every `Token::Char` the Render Frame carries is `SourceRevision::token_at`'s leftover-content
fallback, and a Leftover Char is never empty — content is what makes it Char in the first place. The
box is satisfied because the case it names does not occur, not because a Char operand was invented to
exercise it.

**The row-truncated slot needed no `lang` or `LanguageMap` change at all — only a test proving the
fact ticket 02's implementer believed was missing.** Traced and confirmed empirically (`.+01` written
into a 5-wide Grid, second Number operand needs columns 4-5): `lang::Parser::take_token`'s error path
(`lang/src/parser.rs`) already records a `PositionedEntry` for the truncated operand whose `cells`
range is exactly the Cells the row's tail held — `next_token` fails, but the explicit `self.source =
""` line still advances `self.consumed()` past those Cells before `add_positioned` is called with
`Some(token)`, so the range is `4..5`, not empty. `LanguageMap::token_at` walks
`expression.positioned()` and finds that entry, so `token_at(4, 0)` already answered
`Some(Token::Number)` before this ticket touched anything — proven with a throwaway probe test before
writing the real one. The `cells.is_empty()` skip in `name_units` (`orcvs/src/source/
language_map.rs`) only discards a *zero*-Cell entry, which happens exactly when zero Cells of the
slot exist in the Grid — and then there is no Position to query in the first place, so nothing is
lost. `RenderFrame::derive` and `console/src/paint.rs`'s `fill_tint_colour` both already read `token()`
with no Cell-count assumption, so the fact reaches the tint unchanged through every layer. What was
added: `orcvs/tests/language_map.rs::truncated_operand_owns_the_available_row_tail` gained a
`token_at` assertion on the truncated Cell, a sibling test
(`a_row_truncated_operand_with_an_empty_tail_cell_still_carries_its_token`) covers the Pending case
where that Cell is a space rather than a written character,
`orcvs/src/render_frame.rs::a_row_truncated_operand_cell_carries_its_declared_token` pins the same
fact at the Render Frame, and `console/src/paint.rs::a_row_truncated_operand_cell_carries_its_
declared_token_tint_and_no_character` pins it at the paint layer, tint and blank character together.

**For `syntax-highlighting/04`.** A row-truncated Pending slot and an ordinary Pending slot are
represented identically end to end — `Some(declared Token)`, `content() == None` — with nothing
marking that the slot's full width does not fit the Grid. When `04` teaches the Render Frame an
"operand bound no Atom" fact (for the Diagnostic-coloured Invalid Operand glyph and the lone `|`),
that fact is orthogonal to this one and should stay that way: it belongs beside `Token` on the same
per-entry record `PositionedEntry` already is (`atom: Option<Atom>` is already carried there and
already distinguishes a bound entry from an unbound one at the `lang` layer — `LanguageMap::token_at`
just does not expose it yet), not as a special case of row truncation. A row-truncated slot that
never bound anything and an in-bounds slot that failed to decode (`c4` in a Number slot) both want
`atom: None` on their `PositionedEntry`; row truncation only additionally has a narrower `cells`
range. Plumbing `atom.is_some()` through `LanguageMap`/`RenderCell` the same way `token_at` already
plumbs `token` is the natural next seam, and it will apply to a truncated slot's existing Cells with
no extra branch, since `04` reads the same entries `token_at` reads today.
