# 06 — Paint the Source from a base16 scheme

**What to build:** A `Scheme` of sixteen slots, one template mapping Orcvs Tokens onto them, and a resolution chain the Source Grid paints from — replacing `SourcePaintSettings`' ten resolved colours. Compiled-in schemes only; loading one at runtime is `07`.

**Blocked by:** 01 — Decide where Source colour authority lives; paint-cell-cost/04 — Prove Paint recovered without making Source revisions slower.

**Status:** ready-for-agent

**Tags:** release/v1

- [ ] `Scheme` holds sixteen `Color32` slots, `base00`–`base0F`, and nothing else. No Orcvs vocabulary on it — it is the interchange type, and every published base16 scheme is one.
- [ ] One `TEMPLATE` maps each Token to a slot, at the assignment ADR 0051 records. It is a single table, not a method per role.
- [ ] Affordance colours — Diagnostic, Output Portal, Region, selection, Cursor, and the Fill tint strength — are named keys with their own defaults, not slots. A `Scheme` supplies no value for any of them.
- [ ] Resolution runs one way: `Scheme` → `TEMPLATE` → resolved Token colour → per-Token override → final. An override is `Option<Color32>`; `None` resolves from the scheme, so a scheme change or a template change reaches every Token the viewer has not pinned.
- [ ] Okabe–Ito ships as a `Scheme`, so the Source Grid's appearance does not change when this lands. The `style.rs` tests that pin today's glyph colours still pass, read through the scheme rather than from `SourcePaintSettings`.
- [ ] At least two further schemes ship, chosen from published base16 schemes, so the template is exercised by palettes it was not designed around.
- [ ] `style.rs::claim_paint` and `operand_paint` stop choosing which channel shows which fact. They answer facts; resolution produces the `CellVisuals`.
- [ ] `SourcePaintSettings` and the `source_paint` storage key are gone. Storage holds the chosen scheme and the overrides, keyed by scheme. The old key is left unread rather than migrated, per ADR 0051.
- [ ] `Theme → Source colours` becomes a scheme picker plus per-Token overrides. "Reset" clears overrides to `None` rather than writing defaults in.
- [ ] `console/src/theme.md` records the template and the shipped schemes.
- [ ] The fact-to-channel mapping resolves **once per frame**, into a flat lookup the per-Cell body indexes. It is never walked, matched or hashed per Cell. `paint-cell-cost` measured the per-Cell body at 24 ns/Cell after `syntax-highlighting/08`–`10`, against 4.3 before, and 77–79% of that was one `HashMap` lookup in this loop. A mapping consulted per Cell would spend the recovery `paint-cell-cost/03` bought.
- [ ] The Fill tint is precomputed per resolved scheme rather than mixed per Cell. `fill_tint_colour`'s `lerp_to_gamma` measures 1.0–1.3 ns/Cell today, and resolving from a scheme gives a natural place to hoist it.
- [ ] `paint_derive` still holds `benchmarks/07`'s recorded floor. The comparison lives in the action, so it is read on the pull request.
- [ ] `cargo nextest run --package console --locked` and the `--no-default-features` arm both pass.

## Comments

**Why the template is one table.** Ten repeated colour-picker blocks became one `SOURCE_COLOURS` table in `syntax-highlighting/07` for the same reason. The mapping is data; expressing it as ten accessors is what made the previous shape hard to change.

**The contrast rule does not come with this.** `style.rs` currently pins each colour's measured ratio against `#000000` and which side of Comment it reads on. Those are facts about Okabe–Ito, and an arbitrary scheme will break them. `08` replaces them with a validator. Until it lands, keep the existing assertions passing against the Okabe–Ito scheme and do not extend them to the new schemes — asserting a second palette against a first palette's measurements is the thing `08` exists to stop.

**Interaction with `paint-cell-cost`.** That effort exists because `syntax-highlighting/08`–`10` made every drawn Cell about 5.5 times more expensive, and ADR 0052 removed the `slot_written` map from the per-Cell loop: each shared Claim answers whether its slot is written as the Render Frame builds it, and `RenderCell::source_paint` reads that answer with no lookup. `02`'s move of all finished Paint facts onto the Language Map was rejected (ADR 0050); do not rebuild on it. This issue rewrites the same loop. The ordering is recorded as a blocker rather than left to chance: `paint-cell-cost/03` lands first, `benchmarks/07` pins the recovered figure, and this issue is then built against a floor that will fail if resolution creeps back into the per-Cell body.

The two are compatible in kind. ADR 0051 says facts belong to the Frame and the theme maps them to channels, which is the boundary ADR 0052 implements. What must not happen is the mapping itself becoming a per-Cell cost.

Precomputing the role tints remains this ticket's concern: `SourcePaintSettings` does not survive this issue, but the optimisation does, as a precompute per resolved scheme.
