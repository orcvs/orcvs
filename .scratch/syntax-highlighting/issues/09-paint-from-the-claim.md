# 09 — Paint from the claim

**What to build:** The console makes one decision per Cell: foreground and tint together, from the claim and whether the claimed slot holds written content, with Cursor precedence applied afterwards. This replaces the two overlapping Token matches and the `bound` value passed alongside them. Result destinations are not an input yet; `06` adds that input when it paints Results.

**Blocked by:** 08 — Carry the parser's claim on the Render Frame.

**Status:** resolved

**Tags:** release/v1

- [x] `style.rs` has one function computing foreground and tint. Its inputs are the claim (or none) and whether any Cell in `claim.cells` holds written content. The Cursor's own fill still beats the tint on its Cell. It takes no destination input; that arrives with `06`.
- [x] Paint derivation computes "written" for each claim from the Render Frame's own Cell contents over `claim.cells`, converting each index through the frame's `Grid`, once per claim rather than once per Cell. `Claim { cells, token, atom }` alone cannot tell Pending from Invalid, so the decision function takes the slot-content fact as an explicit input.
- [x] Rules are unchanged, now read from the claim. A bound claim paints its Token colour on its Token tint. An unbound operand claim keeps its declared Token tint and draws a Diagnostic glyph. An unbound Function claim draws a Diagnostic glyph and no tint. Comment, Bang and unclaimed Cells are not tinted. Visible output matches today's for every case pinned by `02`–`04`.
- [x] An unbound operand slot is Invalid if it holds any written content, and Pending if it is entirely blank. The slot's blank Cells follow that verdict: in `.+0` the blank Cell belongs to an Invalid slot.
- [x] The blank-glyph fallback `content().unwrap_or(' ')` (`paint.rs:255`) stays. It gives the renderer a character to draw, and it is not how Pending is decided.
- [x] Removed: the second Token match in `fill_tint_colour` (`style.rs:192-200`); the `bound().unwrap_or(true)` fold (`paint.rs:214`); the five-Token `matches!` list (`style.rs:106-111`); the `#[cfg(test)]` `cell_visuals` shim (`style.rs:60`); the adjacent positional `bound, selected, cursor_visible` bools; and `bound_is_read_only_for_the_tokens_a_refusal_can_reach`.
- [x] `RenderCell::token()` is removed once the paint reads `claim.token`. Its other callers are tests (`orcvs/src/app.rs`, `console/src/marks.rs`, `render_frame.rs`'s own tests, `paint.rs` tests), and they move to `claim()`. A leftover-content Cell has no claim, where `token()` answered `Token::Char` for it; no shipped test relied on that specific answer, so none needed the `claim().is_none()`-with-content-present rewrite the ticket anticipated — the rule is recorded on `style.rs::claim_paint`'s `Token::Char` arm (`unreachable!`) instead.
- [ ] `Paint::derive` stops serving as a test-only path that paints default settings. Not done: colour tests no longer lean on it, but it is kept because `console/benches/paint.rs` calls it (`cursor_effects::DEFAULT_REGION_COLOUR` is `pub(crate)`). Carried to `07`'s follow-ups.
- [x] Colour tests build the claim and the slot-content fact directly, with no egui `Context` and no running `Orcvs` (ADR 0040). They must cover an Invalid slot with a blank Cell (the `.+0` shape), an entirely blank Pending slot, an unbound Function claim, and an unclaimed Cell. Deriving claims from Source is tested in `08`, not here.
- [x] `console/src/theme.md` describes the rules in claim terms.

## Answer

`console/src/style.rs::claim_paint(claim: Option<&Claim>, written: bool, source_paint: SourcePaintSettings) -> (Color32, Option<Color32>)` is the one decision: foreground and tint together, from the claim and its slot's written fact alone. It delegates an Operand Token's shared rule to `operand_paint`, and the tint mix itself to `fill_tint_colour(colour, source_paint)` (now colour-in, not Token-and-bound-in). `cell_visuals_with_cursor_colour` composes `claim_paint`'s answer with Cursor precedence afterwards — the Cursor's own fill over the tint, and the border — exactly where it already lived.

`console/src/paint.rs::slot_written(frame: &RenderFrame, claim: &Claim) -> bool` answers the slot-content fact: any Cell of `claim.cells`, converted through `Grid::cell_index`/`Grid::position_at`, holding content. `Paint::derive_with_colours` calls it once per distinct claim via a `HashMap<*const Claim, bool>` cache keyed by the claim's own address (every Cell of one claim shares one `Arc`), not once per Cell.

`RenderCell::token()` and its backing field are gone from `orcvs/src/render_frame.rs`; `RenderFrame::derive` no longer calls `source.token_at(position)` for the per-Cell record. `orcvs::source::Atom` is re-exported (`orcvs/src/source/mod.rs`) so a caller building a `Claim` for a test can name its `atom` field's type without a direct `lang` dependency. Every migrated caller now reads `RenderCell::claim().map(|claim| claim.token)`.

Visible output is unchanged for every `02`–`04` case: a Pending operand's foreground now differs from an Invalid one's in principle (Token colour versus Diagnostic), but a Pending Cell is always blank and `paint.rs`'s blank-glyph fallback draws no character for it regardless of foreground, so nothing painted moves.

## Comments

The Token → colour mapping, the Cursor precedence and the tint rule are the console deciding how a fact is shown (ADR 0022, ADR 0042). They stay in `console`. What goes away is the console rebuilding the fact.

`SourcePaintSettings` keeps its ten fields and its encode/decode order. Restructuring it changes the stored format, which makes it a separate persistence change.

Gates: `cargo fmt --all -- --check`; clippy and nextest scoped to `console`.
