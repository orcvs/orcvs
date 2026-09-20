# The Language Map answers Source Paint per Cell

Status: proposed. If accepted, supersedes [ADR 0044](0044-paint-reads-the-parsers-claim.md). It refines [ADR 0040](0040-the-console-paints-from-a-value.md) on what a Render Frame Cell carries and preserves [ADR 0042](0042-responsibility-is-not-freedom-from-the-toolkit.md): `orcvs` answers language facts, while the console still chooses how colours represent them.

The Language Map answers, for every Cell, the distinctions Source Paint must draw. A Function spelling, Pending Operand, Valid Operand, Invalid Operand, Bang, Comment, and Unclaimed Cell are language facts derived once for a Source revision. The Output Portal remains the separately derived per-Cell fact established by ADR 0044's follow-up work. A Render Frame Cell carries these finished facts, as it already carries whether it is an Output Portal; the console maps them to foreground, background, and border colours without interpreting a Span.

A Language Unit remains language-shaped: it has an anchor Position and a Span of one or more Cells. Paint remains Cell-shaped: it decides one Cell's foreground, background, and border. The mapping between those shapes belongs in the Language Map because that is where every Span is established and the row's Source bytes are already in hand. In particular, Pending Operand asks whether any Cell in the claimed operand slot holds written content. The Language Map can answer that while walking the row, once per revision. The console otherwise has to recover it by walking the same Span once for every drawn Cell on every Render Frame.

This reverses ADR 0044's decision to carry the parser's shared claim into every covered Render Frame Cell. That ADR rejected a per-Cell answer as a lossy re-encoding because it would discard the Atom. Source Paint has never read the Atom: its only distinction is whether `claim.atom.is_some()`, and ADR 0044's own test helper gives every bound claim the same filler for exactly that reason. The discarded information is therefore outside Source Paint's question, not a loss in its answer. The parser's claim remains authoritative inside the Language Map; only the console boundary changes.

Cadence makes the boundary consequential. The Language Map is rebuilt when the Source revision changes, while Render Frames and Paint are rebuilt many times per second. ADR 0044 made `Paint::derive_with_colours` walk `claim.cells` per drawn Cell, then introduced a `HashMap<*const Claim, bool>` constructed empty on every derivation to memoise those walks. The benchmark after that change regressed `paint_derive/culled/16x16` from 2,165 ns to 12,506 ns (5.78x) and `paint_derive/fitted/256x256` from 629,884 ns to 3,232,981 ns (5.13x). Ablation attributes 77–79% of the added per-Cell cost to that map, and sampling attributes 25.7–29.0% of total derive time to its reserve-and-rehash path. The work is both repeated at the wrong cadence and cached at the wrong boundary.

## Rejected alternatives

**Keep the claim and make its cache cheaper.** A cheaper hasher recovered only 7.7–10.3 ns of the 16.2–18.3 ns cache cost, while a one-entry last-claim memo recovered 2.5–3.2 ns. Claims average about two Cells and change too frequently for memoisation to pay. Construction, growth, and the downstream interpretation remain even if hashing improves.

**Carry only whether an operand slot is written.** That removes the dominant cache cost but leaves the console interpreting the rest of a language-shaped claim per Cell. It moves one fact without moving the Span-to-Cell mapping that owns all Source Paint distinctions.

**Keep carrying the Atom because it may support future paint.** Source Paint's decided scope is the distinctions in `.scratch/syntax-highlighting/spec.md`. Carrying richer parser state across the boundary for an unnamed future presentation repeats ADR 0044's coupling without a present language or console requirement.

## Consequences

`RenderCell` no longer carries a shared claim. The claim's `Arc`, the console's Cell-range walk, its `slot_written` recovery, and the per-derive pointer-keyed `HashMap` are removed. The Language Map stores the per-Cell answers alongside its other revision-derived views, and Render Frame derivation copies the answer for each Cell as it does the Output Portal fact.

The console owns no second classifier. Its Source Paint decision becomes a match over the Language Map's answer plus the Output Portal fact, returning the Cell's foreground, background, and border. Cursor and other presentation precedence remain console concerns.

`CONTEXT.md` is unchanged. Language Map, Language Unit, Span, Cell, Render Frame, and Paint already name the two shapes and the boundary between them. This decision introduces no new language vocabulary.
