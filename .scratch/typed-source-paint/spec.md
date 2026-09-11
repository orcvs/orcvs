# Paint the Source from its types

**Goal:** Retire the last spelling classifier between the typed expression structure and the paint,
so the console colours a Cell from what the Parser decided it is rather than from a lossy copy of
that decision.

## Why

ADR 0034 made the Parser's typed output the structure rendering uses and retired the independent
spelling classifier. One survived: `orcvs::glyph::Glyph`, which the Render Frame carries per Cell
and `cell_visuals` matches on. It is a copy of `lang::Token` with the type-carrying variants removed.

```
Token:  Activation  Bang  Char  Comment  Function  Note  Number  Atom  Sequence
Glyph:              Bang  Char  Comment  Function  Note  Number              Highlight  Marker  Space
```

Six names identical. `Space` is the absence of a Token. `Marker` and `Highlight` are Orca's
terminal inheritance — where the Grid's rulings are characters — and Orcvs draws both as geometry,
so nothing has produced either since. `Activation`, `Atom` and `Sequence` are what
`From<Token> for Glyph` folds into `Char`, with a comment in `orcvs/src/glyph.rs` recording the
loss.

So the classifier's entire contribution over `Option<Token>` is discarding type information — the
information ADR 0034 exists to carry. A generic Atom operand, a Sequence operand and a literal
character reach the painter indistinguishable.

Two further facts are computed and dropped before paint, and both make the console misleading rather
than merely coarse:

- **A parse error is invisible.** `ExpressionEntry::diagnostic` holds one with a Span, and
  `cell_visuals(glyph, cursor_bloom, selected, cursor_visible)` takes no diagnostic. A Cell inside a
  failed parse paints exactly like a valid one. The console's `diagnostics.rs` reports *Playback*
  failures only; language diagnostics and `lexical_diagnostics` reach no paint path at all.
- **Inert Source looks live.** `analysis.is_complete()` decides whether an Expression executes and
  becomes `ExpressionEntry::root`. An incomplete Function is painted like one that will run on the
  next Tick. In a livecoding console that is the single most useful thing a colour could say.

## Rules

**Delete, do not add.** The failure mode this effort exists to avoid is minting a third vocabulary
that restates `Token` and `LanguageUnitKind`. Every fact this effort paints from must already be
carried by the typed structure; if a fact seems to be missing, establish where it belongs in that
structure rather than introducing a per-Cell enum beside it.

**Per-Expression facts stay on the Expression.** A diagnostic and an executability decision cover a
Span. Copying them onto every Cell of that Span is the same duplication at a finer granularity.
`ExpressionEntry` already holds `span`, `diagnostic` and `root`.

**The palette does not move here.** `restyle-egui-console/02` pinned twenty-two tokens as the
decided record and five issues settled them. This effort exposes classifications that currently have
no colour; assigning those colours is that effort's work, not this one's, and `03` states the
handover rather than choosing hexes.

**Parity for everything already painted.** A Cell whose Token maps onto a Glyph today keeps its
current colour exactly. The visible changes are additive: Atom and Sequence stop impersonating
Char, and diagnostics and inexecutability become visible at all.

## Prior art

This repository's own arc. ADR 0024 recorded that the Language Map holds spellings rather than Atom
types. ADR 0034 revised that separation and retired the classifier it justified. `Glyph` is the
residue — the last consumer of the spelling-only view — which is why the amendment to 0034 rather
than a new ADR is the right record.

## Deliberately not in scope

**No new diagnostic model.** `Diagnostic` and `lexical_diagnostics` exist with Spans. This effort
routes them to the paint; it does not change what the Parser reports or when.

**No change to parsing, partitioning or execution.** ADR 0033 continues to determine the initial
parse from Source. Nothing here runs at Tick time.

**No operand-placeholder decision by default.** `02` decides it explicitly. Deleting `Glyph` alone
would settle it by accident, which is the sequencing trap this spec exists to avoid.

## What this deletes

- `orcvs::glyph::Glyph`, `GlyphString`'s blank spelling table, `GlyphString::marker()`,
  `::highlight()`, and `From<Token> for Glyph`.
- The `LanguageMap` row's `Vec<Option<Glyph>>` and the two sites that fill it.
- `console`'s `BLANK_GLYPHS`, `blank_glyph_index` and `GlyphTable::blanks`, whose painted path is
  unreachable today: no Cell with no content carries a Token, so every blank resolves to a space and
  paints nothing.
- `CONTEXT.md`'s **Glyph** entry, and `PALETTE.marker` and `PALETTE.highlight` with it.

## Vocabulary

`CONTEXT.md` gains no term. It loses **Glyph**, and the Grid's background rulings are described
where they are drawn rather than as a Cell classification. **Cursor**, **Render Frame**, **Grid**
and **Source** are unchanged.

## Sequencing

`01` is the deletion and carries the risk; `02` and `03` are independent of each other and both
depend on it. `03` hands off to `restyle-egui-console` rather than choosing colours.
