# A Claim answers whether its slot is written

Status: proposed. Supersedes [ADR 0050](0050-the-language-map-answers-source-paint-per-cell.md) and refines [ADR 0044](0044-paint-reads-the-parsers-claim.md). It preserves [ADR 0040](0040-the-console-paints-from-a-value.md): `orcvs` carries the language facts a frame needs, and the console chooses how colours represent them. It becomes accepted only when [paint-cell-cost/04](../../.scratch/paint-cell-cost/issues/04-prove-paint-recovered-without-making-source-revisions-slower.md) accepts its benchmark comparison.

The parser's shared Claim carries one more fact: `written`, whether any Cell it covers holds content in the Source revision it was read with. The Render Frame already builds one `Arc<Claim>` per positioned entry for every frame (`LanguageMap::claims_by_cell`). `SourceRevision` passes that function its own Cell bytes, and each Claim reads its own Cells once, as it is built. Every Cell sharing the Claim reads the same answer. `RenderCell::source_paint` combines `token`, `atom` and `written` into the Function, Pending Operand, Valid Operand, Invalid Operand, Bang, Comment, and Unclaimed distinctions. Paint performs no pointer-keyed lookup and no per-Cell Span walk.

The Language Map still retains only parser Claims. It does not keep Source bytes, a whole-Grid Source Paint view, or a fitted Output Portal highlight. Output Portal fitting stays on `SourceRevision`, where syntax-highlighting/12 merged it. No revision rebuild computes `written`: a Claim exists only while a Render Frame is derived, and a Claim built from one revision's bytes lasts no longer than that revision.

## Why this seam

The answer belongs where the Claim is made. That is the only place that holds the Claim's Cell range and the revision's bytes together, before anything shares the Claim. There, the answer costs one contiguous byte scan over the Claim's own Cells. The scan stops at the first written byte, so a written Claim usually costs one byte read; only a blank or blank-leading slot reads further. Any later place re-derives the Claim's identity, which costs a pointer-keyed map in Paint. Or it walks the Claims a second time, which costs a grid-sized side table in the Render Frame.

## Rejected alternatives

**Cache finished Paint facts on the Language Map.** This is ADR 0050. It moved work from the draw path to revision rebuilds, and a Source can revise often enough for that transfer to be unacceptable. `paint-cell-cost/02` keeps the implementation and review history.

**Keep the pointer-keyed Paint cache.** `paint-cell-cost/01` measured the cache's reserve-and-rehash path as most of the added per-Cell cost. Claims cover few Cells, so neither a cheaper hasher nor a last-Claim memo recovers enough of it.

**Answer `written` in a Render Frame side table.** This was the first `03` implementation, `ba99abc`. After building the Claims, the Render Frame allocated a grid-sized `Vec<bool>`. It collected a second `Vec` of unique Claims, scanned each Claim's Cells through Position conversions, and walked each Claim's range again to copy the answer. Paint recovered, but `source_render_frame` regressed 1.05–1.20x on the authoritative runner, and `paint-cell-cost/04` rejected it. A local ablation assigned most of the added cost to the extra vectors and the redistribution pass, and the rest to the scan.

**Derive written state independently for every Cell.** Every Cell in a Claim asks the same Span-shaped question. Repeating the Span walk per Cell was the original performance defect.

## Consequences

`Claim` gains a public `written: bool`, so it is no longer purely the parser's record. It now also records one fact about the revision the Claim was read against. Nothing outside `LanguageMap::claims_by_cell` constructs a `source::Claim`; `tick.rs` has an unrelated private `Claim` of its own.

`LanguageMap::claims_by_cell` takes the revision's bytes and asserts that their length matches the Grid. `SourceRevision::claims_by_cell` is the one caller. It pairs the Map with the bytes it was derived from, which is the same reason `SourceRevision` owns the fitted Output Portal.

A space counts as blank, as it does for `SourceRevision::content_at` and the Output Portal fit.

The fitted Output Portal's first-blank-Cell and one-Cell-gutter behaviour is unchanged. No new language vocabulary or public compatibility promise is introduced.
