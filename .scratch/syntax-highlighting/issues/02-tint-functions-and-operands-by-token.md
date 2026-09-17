# 02 — Tint Functions and Operands by their Token colour

**What to build:** Every Cell of a Function or an Operand gets a background tint of its Token colour, at a fill strength the user sets in Theme → Source colours, so an Expression reads as a run of typed units.

**Blocked by:** 01 — Expose Source colours as Theme settings.

**Status:** resolved

- [x] A Function Cell, nested Functions included, is tinted with the Function colour. An Operand Cell is tinted with its declared Token colour: Number, Note, Char, Atom or Sequence.
- [x] The tint is the Token colour mixed over the Source background at the Fill tint strength.
- [x] Theme → Source colours exposes Fill tint as a percentage control, default 16%, range 0–100. 0% paints no tint. Reset restores 16% with the colours; persistence stores it with them.
- [x] Comment, Bang, empty unclaimed Cells and Leftover Chars are not tinted.
- [x] The glyph keeps its Token colour on the tint.
- [x] On the Cursor Cell, the Cursor's own fill wins over the tint.
- [x] Adjacent tinted Cells of the same colour paint as one run rather than per Cell.
- [x] Focused paint tests cover a tinted Function, a tinted Operand of each Token, 0%, and the Cursor Cell.

## Comments

Prototype: `?variant=A&palette=okabe` in `console/prototypes/syntax-highlighting/source-paint-prototype.html`, where the tint is 16%. Not present in this checkout — the colours it fixed were read from `console/src/theme.md`'s already-committed record (`syntax-highlighting/01`) rather than from the file itself.

`Token::Char` is named among the "declared Token colour" list above, but no Operand Cell ever carries it: `lang/src/atom.rs`'s `operand_token!` macro never mints `Token::Char` for a Function signature, and `lang/src/stack.rs`'s `check_token` marks the arm `unreachable!` — "no operand type declares a Token the Parser mints only as a label". Every Cell this ticket's paint code sees `Some(Token::Char)` on is `SourceRevision::token_at`'s leftover-content fallback (`orcvs/src/source/mod.rs`), i.e. the Leftover Char role the third box above requires left untinted. `console/src/style.rs::fill_tint_colour` therefore excludes `Token::Char` from the tint match rather than including it — including it would have tinted every Leftover Char, since that fallback is the only source of that Token in this codebase. Tokens actually tinted as Operand Cells: Number, Note, Atom, Sequence.

Classification of Function versus Operand reads the existing `RenderCell::token()` (`Option<orcvs::source::Token>`) alone, with no new per-Cell fact: `Token::Function` is a Function Cell (nested included, since a nested Function's own two-Cell spelling carries `Token::Function` the same as a root one); `Token::Number | Token::Note | Token::Atom | Token::Sequence` is an Operand Cell tinted with that Token's own colour, whether the operand is Pending, Valid, or Invalid — the Parser labels a claimed operand slot with its signature's declared Token whether or not the content there binds (`lang/src/parser.rs::take_language_unit`), so Pending and Invalid Operand Cells tint exactly like a Valid one. `Token::Bang`, `Token::Comment`, and `None` are excluded, alongside `Token::Char` as above.

The mix uses `egui::Color32::lerp_to_gamma` (`ecolor-0.36.1`), already part of the pinned egui stack: a per-channel gamma-space lerp from the Source background toward the Token colour by the Fill tint percentage. `0%` short-circuits to `None` rather than painting `Some(source_background)`, so a Cell with nothing to tint and a Cell tinted at 0% answer identically and neither costs `background_runs` a Cell it has to walk.
