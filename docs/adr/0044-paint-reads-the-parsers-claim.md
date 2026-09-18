# Paint reads the parser's claim

Status: proposed. Becomes accepted when both `.scratch/syntax-highlighting/issues/08` (carrying the claim) and `09` (painting from it) have landed. It refines [ADR 0040](0040-the-console-paints-from-a-value.md) on what a Render Frame Cell carries, and does not reopen how the console turns a Paint into Shapes. It reverses the per-Cell `bound` projection that `.scratch/syntax-highlighting/issues/04` introduced.

A Render Frame Cell carries the parser's claim on it, not scalar projections of that claim. `lang::PositionedEntry` records, for every slot the Parser reads, the Cells it claims, the Token its signature declared, and the Atom those Cells bound, or none. That covers operand slots that are blank or refused as well as ones that bind. `RenderCell::claim()` answers that record, and `None` means no entry claims the Cell. `RenderCell::bound()` and `LanguageMap::bound_at` are removed, and the Render Frame no longer copies its Token from `SourceRevision::token_at`.

**The projection was the defect, not a missing classification.** `ExpressionEntry::positioned()` is `pub(super)`, so the entry was visible only inside `orcvs::source`. Two scalars crossed that seam: `token_at`, and `bound_at` (added for issue 04). The entry was then rebuilt from them at six places. Three are in `orcvs`: `bound_at`'s Comment special case, the leftover `Token::Char` fallback in `SourceRevision::token_at`, and `RenderFrame::derive` packing `token` and `bound` onto the Cell. Three are in `console`: folding `None` into "bound" (`paint.rs`), the five-Token list naming which Tokens a refusal can reach (`style.rs`), and the second Token match that reads `bound` for `Function` alone (`style.rs`). A blank operand and a wrong one projected to the same pair, `(Some(Token::Number), Some(false))`. That is why Pending and Invalid could not be told apart without reading the Cells again.

**The claim is carried, not described.** The alternatives considered each minted a type describing what the entry already states: a per-Cell role enum, a role carrying its Token, and a `Binding { Pending, Valid, Invalid }` state. Each one was a lossy re-encoding of `token` and `atom`. The role enums had to name every combination of two independent facts, and `Binding` threw the Atom away. "Claim" is not new vocabulary: [ADR 0033](0033-partition-a-row-by-parse.md) has an Expression claim what its arity declares, and `CONTEXT.md` names the arity-determined claim.

**`parent` does not cross.** It indexes the owning Function in one Expression's entry order, and means nothing once entries are read without that Expression. The claim carries `cells`, `token` and `atom`.

**The paint decision stays in the console.** Choosing a colour for a declared Token, the tint rule, and the Cursor's precedence are the console deciding how it shows a fact ([ADR 0022](0022-keep-the-core-crate-free-of-the-ui-toolkit.md), [ADR 0042](0042-responsibility-is-not-freedom-from-the-toolkit.md)). This decision removes the rebuilding of the fact, not those choices. Foreground and tint become one decision over the claim. Pending is decided per slot, from whether any Cell in `claim.cells` holds written content, because a blank Cell beside written content belongs to an invalid slot. The claim does not answer that on its own, so the paint derivation reads it from the frame's Cell contents. The blank glyph a Cell with no content draws is only the character supplied for rendering. It is not how Pending is decided.

## Rejected alternatives

**Widen `ExpressionEntry::positioned()` and let the console read Language Map entries directly.** That would put `orcvs::source`'s internal Expression structure, including the Expression-local `parent`, into the console's interface. The Render Frame is where ADR 0040 pairs a Source revision with what the console draws, so the claim crosses there.

**Clone the claim into every Cell it spans.** A claim is stored once and shared by the Cells it covers. Cloning it would duplicate a value per Cell for a fact that belongs to the slot.

## Consequences

The `token_at` functions remain, and they are three distinct functions. `LanguageMap::token_at` is called by tick tests and the WASM test. `SourceRevision::token_at`, which adds the leftover `Char` fallback, is called by the console's persistence tests, and today by `RenderFrame::derive` until `09` removes `RenderCell::token()`. `orcvs/src/source/model.rs` has its own private helper of the same name. Only `RenderCell::token()` goes, once the paint reads the claim; its remaining callers are tests.

A Result destination is not part of the claim, because the Parser records no such fact. Deriving it is `.scratch/syntax-highlighting/issues/05` and `10`, and is not decided here.
