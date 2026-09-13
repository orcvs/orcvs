# 02 — Prototype-aligned console palette

**What to build:** One decided, named console palette that `restyle-egui-console/03` can falsify a capture against. `console/src/style.rs` already holds the single `ConsolePalette` struct and the `PALETTE` const that every rendering colour comes from, and `console/src/theme.md` is the decided record of the 21 token values in hex. This issue makes that record the decision rather than a description: the palette below is the one the captures must match, token by token. Where the shipped code is the only evidence for a choice, this issue says so and leaves the choice open for `03` to settle against the prototype.

The palette is a charcoal page over near-black Source, subtle one-pixel Cell grid lines, muted ordinary Glyphs, teal Functions, soft-red Bang and diagnostic states, restrained teal selection and Cursor treatment, and a calm blue Number colour distinct from Functions, Notes, and ordinary Characters. Cell backgrounds stay near-black: the only background changes are the meaningful states — selection and the Cursor field — and each of those is itself near-black. Nothing decorative is added on top: no gradients, no rounded tiles, no shadows, no animation.

**Blocked by:** None — every listed blocker is resolved.

**Status:** resolved

**Tags:** release/v1

- [x] Semantic colours are derived from one theme boundary rather than scattered rendering literals: the `ConsolePalette` struct and the `PALETTE` const in `console/src/style.rs`.
- [x] Token colour choice remains in `cell_visuals` while Grid rendering owns geometry.
- [x] The 21 palette tokens hold exactly these values, and `console/src/theme.md` states the same ones:
      page `#0B1112`; source `#070D0D`; grid line `rgba(29, 55, 49, 0.28)`; sector line
      `rgba(55, 101, 86, 0.43)`; ordinary `#A5B7B2`; comment `#7A8784`; function `#68E0B8`;
      bang and error `#FF7F87`; number `#83A6D8`; note `#AA91D6`; Cursor bloom core fill `#0A1E1A`
      and line `rgba(76, 190, 156, 0.59)`; inner fill `#091A17` and line `rgba(58, 148, 122, 0.49)`;
      middle fill `#081614` and line `rgba(43, 110, 92, 0.39)`; outer fill `#081211` and line
      `rgba(34, 78, 67, 0.32)`; selection fill `#0A2A22`; selection stroke while the caret is
      hidden `#52C3A3`; selection and Cursor stroke `#65E6BE`. `palette_tokens_match_the_decided_record`
      in `console/src/style.rs` pins each one as a `Color32`.
- [x] Numbers use a calm non-yellow colour distinct from Functions, Notes, and ordinary Characters. `#83A6D8` against `#68E0B8`, `#AA91D6` and `#A5B7B2` satisfies this.
- [x] Cursor and Token classifications retain their existing behaviour.
- [x] The console adds no gradients, no rounded tiles, no shadows, and no animation. `console/src/style.rs:137-149` already sets `CornerRadius::ZERO` on windows, menus, and every widget state, and `Shadow::NONE` on windows and popups; `console/src/style.rs:153` sets `animation_time: 0.0`. Any new rendering keeps all four true.
- [x] The sector registration marks are kept: partial 0.75-pixel `+` marks at each 8 × 8 sector corner
      (`SECTOR_LINE_WIDTH` at `console/src/console.rs:24`) with arm strengths
      `100, 72, 34, 13, 13, 34, 72, 100` (`SECTOR_SEAM_STRENGTHS` at `console/src/marks.rs:51`).
      They are the Marker spacing moved into geometry; removing them renders that spacing nowhere.
- [x] The Cursor focus matrix is proposed, not settled: four Cell-aligned square bands of width
      `1 : 1 : 2 : 3` at cumulative radii 1, 2, 4 and 7 Cells, with Position-hashed edge breakup.
      `console/src/theme.md` describes the shipped four-band reticle and states that a match to
      that record does not settle it. `restyle-egui-console/03` reports the capture against the
      prototype and states whether the four bands, their `1 : 1 : 2 : 3` widths, and the edge
      breakup are kept, retuned, or dropped.
- [x] `console/src/theme.md` is named in the issue trail as the decided record, so a later palette change is a documented change rather than a drift.

## Comments

### Rewritten to be falsifiable, 2026-09-04

The previous statement of this issue described the palette only in adjectives — "teal Functions", "soft-red Bang", "a calm Number colour". Nothing in it could fail. That is a problem because `restyle-egui-console/03` exists to check a capture against this decision, and an unfalsifiable decision cannot be checked. The hex values above come from `console/src/theme.md`, which already documents them, and from the `PALETTE` const in `console/src/style.rs`, which already holds them; naming them here turns a description of the shipped code into the record the captures are held to.

### Decision: the Cursor bloom and the sector marks stay

The old text forbade "decorative chrome". The shipped code draws a seven-Cell Cursor focus matrix and 0.75-pixel phosphor registration marks at every sector corner, so the issue and the code disagreed and one of them had to give.

The code is right and the prohibition was too broad. Both features carry information rather than decoration. `theme.md` states that the focus matrix "reads as an address reticle rather than radial light" — it answers where the Cursor is on a Grid with no other landmarks, and it does so through grid-line energy, not through background light, so it does not violate the near-black Cell rule. The sector marks "replace the historical `+` Marker Glyphs, leaving every empty Cell visually empty while preserving the configured Marker spacing as geometry" — they are the Marker spacing, moved off the Glyph layer, which is why an empty Cell now reads as empty.

The prohibition is therefore narrowed to what it was actually protecting: no gradients, no rounded tiles, no shadows, no animation. "Decorative chrome" is dropped as a term, because it was doing no work that those four do not do, and it made two shipped, load-bearing features look like violations.

Neither feature is animated: the edge breakup and the mark gaps derive from a hash of the absolute Grid Position, so they are stable per Cell and change only when the Cursor moves to sample a different boundary. That is the property that keeps them inside the no-animation rule, and `restyle-egui-console/03` should check it by capturing the same Cursor Position twice.

### Correction: this is a design ticket, not a regression spec, 2026-09-04

The rewrite above named all 22 tokens, which was the right fix and stays. But it then pinned every acceptance line to the shipped `PALETTE` const. This issue is `ready-for-agent` and its blocker `01` is unbuilt, so an issue whose every line already passes commissions no work — and it guarantees that `03`'s fourth line, "any remaining prototype differences are reported", reports none by construction. Four corrections:

**The two shipped features are split, because the evidence for them is not the same.** The sector marks are kept on code evidence rather than on `theme.md`'s wording. `orcvs/src/render_frame.rs:87-102` no longer emits `Glyph::Marker` on any path; `marker_spacing` is read at line 87 and spent on `sector_seam_strength` at lines 98 and 102, and `orcvs/src/render_frame.rs:250` asserts an empty Cell gets `Glyph::Space`. Both claims check out, as does the arm shape: `SECTOR_SEAM_STRENGTHS` at `orcvs/src/render_frame.rs:155` is `[100, 72, 34, 13]`, mirrored about the corner, and the test at `orcvs/src/render_frame.rs:378-394` pins offsets 0 and 7 to 100 with a fainter middle. So the marks are not an aesthetic preference; they are the only remaining rendering of the Marker spacing.

The Cursor focus matrix has no such backing. The sole evidence for it is `console/src/theme.md`, which documents the shipped code, so citing it to justify that code is circular. It is therefore recorded as proposed, and `03` must report the four-band reticle against the prototype instead of assuming it passes.

**The prohibition is restored as an acceptance line.** It survived only inside a prose sentence, where nothing checks it. It is now a checklist line naming what already satisfies it, so a later change that adds a gradient or an animation fails a line rather than contradicting a paragraph.

**"Transparent" is dropped from the Cell background rule.** No console background is transparent: every `cell_visuals` background at `console/src/style.rs:80-89` is an opaque `Color32` — `PALETTE.source`, `PALETTE.selection_fill`, or one of the four bloom fills. The rule is restated as near-black throughout, including in the meaningful states, which is what the values actually are.

**The `marker` token's status is stated, because the issue contradicted itself.** It asserted that the sector marks replaced the `+` Marker Glyphs while its own token table required `PALETTE.marker`. Both halves are true of the shipped code: the variant survives at `orcvs/src/glyph.rs:44`, still renders `"+"` at `orcvs/src/glyph.rs:72`, and `console/src/style.rs:76` must still map it because the `match` over `Glyph` has to be exhaustive. But nothing constructs it on a rendering path — `From<Token>` has no `Marker` arm, `orcvs/src/source/language_map.rs` never assigns it, and the only caller of `GlyphString::marker()` is the test at `orcvs/src/glyph.rs:107`. The variant is dead; the token is kept solely to keep the match compiling, and no capture can exercise it.

Removing the variant is real work and is not in this effort's scope: it touches `orcvs/src/glyph.rs`, the `pub fn marker()` constructor, both crates' tests, and the `ConsolePalette` field, and it is a defect in inherited code rather than a restyle decision. It belongs in the `pre-split-defects` effort as a new ticket, the next number after `15`. `pre-split-defects/13` already anticipated it: its comment notes the commented-out `"+" => true` terminator arm and says that if the Marker difference is a real open question it should be written as an issue with a title. It is, and it should be.

### Amended to the shipped 21-token record, 2026-09-13

`typed-source-paint/06` and `retired-glyph-vocabulary/04` deleted `marker` and `highlight` from `ConsolePalette`. This issue still named twenty-two tokens, omitted `comment` `#7A8784`, and listed those two retired fields. The decided list is the twenty-one fields `PALETTE` ships. The Marker/Highlight *token* lines are gone; the `theme.md` paragraph that explains why sector marks replaced the historical `+` Marker Glyphs stays. `palette_tokens_match_the_decided_record` pins each shipped `Color32`, so a later colour change fails unless the record moves with it. The Cursor focus matrix remains proposed: `03` reports the capture against the prototype and decides keep, retune, or drop.

## Answer

The decided dark palette is the twenty-one tokens `PALETTE` already ships. `console/src/theme.md` is the named record; `palette_tokens_match_the_decided_record` pins each `Color32` so a later colour change is a documented change. The list is page, source, grid_line, sector_line, ordinary, comment `#7A8784`, function, bang, number, note, the four bloom fill/line pairs, selection_fill, selection_stroke_rest, and selection_stroke. `marker` and `highlight` are not restored.

Sector registration marks stay: they are the Marker spacing moved into geometry.

The Cursor focus matrix stays proposed. `restyle-egui-console/03` still has to report the captured four-band `1 : 1 : 2 : 3` reticle against the prototype and state whether the bands, their widths, and the edge breakup are kept, retuned, or dropped. A match to `theme.md` does not settle that.
