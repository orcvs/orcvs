# 05 — Render Frame and Paint carry Tokens

**What to build:** The Render Frame snapshots the revision's Token per Cell. Paint colours from that
Token. Every Cell that already had a Glyph colour keeps that colour. Atom, Sequence, and Activation
still paint as Char until ticket `03` gives them colours of their own.

**Blocked by:** 04 — Source revision composes leftover Char.

**Status:** resolved

- [x] A Render Cell answers `token()` as `Option<Token>`. `None` is a Cell no Expression claims and
      no content fills.
- [x] The Frame stores the revision's composition. It does not ask the Language Map for leftover
      `Char`.
- [x] Paint and the palette match take the Token. For every Token that maps onto a Glyph today, the
      painted colour is unchanged.
- [x] Atom, Sequence, and Activation take the colour Char takes today. Ticket `03` is what changes
      that.
- [x] Console blank spellings stay and are keyed on `Option<Token>`. Ticket `02` decides whether
      those placeholders survive.
- [x] Glyph remains in the crate until ticket `06`. Callers that still read it can; the Frame and
      Paint no longer need it. (Retired in `06` of this same pass.)
- [x] Nothing about what is drawn changes.

## Comments

Parity is the risk. `From<Token> for Glyph` is lossy: Char has four Tokens behind it. Those four
must keep painting identically until `03`.
