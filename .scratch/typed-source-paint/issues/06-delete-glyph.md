# 06 — Delete Glyph

**What to build:** The Glyph classifier is gone. The Language Map has no per-Cell Glyph row. The
glossary has no **Glyph** entry. The console no longer maps Marker or Highlight as Cell
classifications.

**Blocked by:** 05 — Render Frame and Paint carry Tokens.

**Status:** resolved

- [x] Glyph, the conversion from Token to Glyph, and GlyphString are deleted. No third vocabulary
      replaces them.
- [x] The Language Map's per-Cell Glyph row and the sites that filled it are deleted. `token_at`
      is the classification answer.
- [x] The **Glyph** entry is removed from `CONTEXT.md`. No term replaces it. Sector Seams and
      Cursor Bloom stay described where they are drawn.
- [x] Palette entries that existed only for Marker and Highlight are deleted.
- [x] `retired-glyph-vocabulary/03` is `wontfix` — this ticket absorbs it. Do not do both.
- [x] Every former Glyph assertion now asserts a Token, and no case is lost in the translation.
- [x] Nothing about what is drawn changes.

## Comments

This is the contract half. Tickets `02` and `03` of this effort unblock when this resolves.
