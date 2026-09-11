# 01 — Carry the Token to the Render Frame and retire the Glyph classifier

**What to build:** `RenderCell` carries `Option<Token>` instead of `Glyph`, and every copy of the
token vocabulary between the Parser and the painter is deleted.

**Blocked by:** None.

**Status:** ready-for-agent

- [ ] `RenderCell::glyph() -> Glyph` becomes a `Token`-valued accessor answering `Option<Token>`.
      `None` is what `Glyph::Space` meant: a Cell no Expression claims and no non-space byte fills.
- [ ] `orcvs::glyph::Glyph`, `From<Token> for Glyph`, `GlyphString::marker()`,
      `GlyphString::highlight()` and the blank spelling table in `GlyphString`'s `Display` are
      deleted. Whether `GlyphString` survives at all is `02`'s decision; this issue deletes only
      what the Glyph vocabulary carried.
- [ ] `LanguageMap`'s per-row `Vec<Option<Glyph>>` is deleted along with the two sites that fill it
      (`language_map.rs`, the `positioned()` loop and the trailing non-space-byte pass). ADR 0034
      deferred that array's existence — "neither individually allocated linked nodes nor a
      cell-indexed classification array is a settled requirement" — so this settles the deferral.
      The Token a Cell carries is answered from the Expression that claims it.
- [ ] `glyph_at`'s replacement answers the same Cells the non-space-byte pass reached. A Cell
      holding a character that no Expression claims answered `Glyph::Char`; it must answer
      `Token::Char`, or the unparsed text in a Grid loses its colour.
- [ ] `cell_visuals` takes the Token. `Activation`, `Atom` and `Sequence` reach it as themselves
      rather than folded into `Char`, and each is given a colour by `03`'s handover — until then
      they take the colour `Char` takes today, so this issue moves no pixel.
- [ ] Strict parity is asserted: for every Token that maps onto a Glyph today, the painted colour is
      unchanged. The existing `style.rs` tests that name hexes are the record, and they are updated
      to name Tokens without changing a value.
- [ ] `console`'s `BLANK_GLYPHS`, `blank_glyph_index` and `GlyphTable::blanks` are deleted, and
      `GlyphTable::character` collapses to the Cell's content. Their painted path is unreachable
      today — no Cell with `content() == None` carries a Token — which is why deleting them changes
      nothing on screen. Confirm that before deleting rather than trusting this line.
- [ ] `CONTEXT.md`'s **Glyph** entry is removed and no term replaces it. The Grid's background
      rulings — the sector seams and the Cursor bloom — are described where they are drawn.
- [ ] `PALETTE.marker` and `PALETTE.highlight` are deleted. `restyle-egui-console/02` names
      twenty-two tokens as the decided record; it goes to twenty, and that issue records why.
- [ ] The `orcvs` tests that assert Glyph classification (`source/tick.rs`, `source/model.rs`,
      `source/mod.rs`, `language_map.rs`, `render_frame.rs`, `app.rs`) assert Tokens, and none of
      them loses a case in the translation.

## Comments

Check `orcvs/src/app.rs`'s `rendered` test helper first. It builds a `GlyphString` from a Cell and
is the shape most of the `orcvs` Glyph assertions take, so what replaces it decides how large this
diff is.

The parity claim is the whole risk. `From<Token> for Glyph` is lossy in exactly one direction, so
every Glyph has a Token that produced it, but `Char` has four (`Char`, `Activation`, `Atom`,
`Sequence`). Those four must keep painting identically until `03` says otherwise, or this issue
ships a palette change it did not decide.
