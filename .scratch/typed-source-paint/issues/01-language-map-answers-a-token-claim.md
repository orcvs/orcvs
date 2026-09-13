# 01 — Language Map answers a Token claim

**What to build:** The Language Map answers the Token of the Expression that covers a Cell, or
nothing. It does not classify leftover Source bytes and it does not keep a per-Cell array.

**Blocked by:** None — can start immediately.

**Status:** resolved

- [x] The Language Map answers `token_at` for a Position: the Token of the Expression that covers
      that Cell, found by walking the row, or `None` when no Expression covers it.
- [x] An unclaimed Cell that still holds a non-space byte answers `None` here. Leftover `Char` is
      ticket `04`.
- [x] No Token array is stored. Classification is a reading of the claiming Expression.
- [x] A Position minted by another Grid is refused the same way today's per-Cell classification is.
- [x] The existing per-Cell Glyph answer remains so later tickets can migrate callers without a
      flag day. (Retired in `06` of this same pass.)
- [x] Nothing about what is drawn changes.

## Comments

Settled in the render-pipeline architecture review: this pass is the expand half of retiring Glyph.
The Map is the semantic view and stays free of Source bytes.
