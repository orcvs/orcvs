# 04 — Source revision composes leftover Char

**What to build:** The Source revision answers a Cell's Token by composing the Language Map's claim
with leftover content: claim, else `Char` when the Cell has content, else nothing. The same Cells
today's Glyph answer reached, including unparsed non-space bytes.

**Blocked by:** 01 — Language Map answers a Token claim.

**Status:** resolved

- [x] The Source revision is the composition: Language Map claim, else `Token::Char` when the Cell
      has content, else `None`.
- [x] The Language Map still does not read Source bytes.
- [x] Every Cell that today's Glyph answer classified as `Char` because no Expression claimed it
      answers `Token::Char` here.
- [x] A Cell no Expression claims and no content fills answers `None`.
- [x] The existing Glyph answer remains. Nothing about what is drawn changes. (Retired in `06`
      of this same pass.)

## Comments

The leftover `Char` rule needs the Source byte. The revision already holds the Map and the bytes;
that is the seam, not a Token array on the Map.
