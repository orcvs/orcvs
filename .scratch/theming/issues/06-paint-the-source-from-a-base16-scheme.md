# 06 — Paint the Source from a base16 scheme

**What to build:** The `Theme` ADR 0053 decides — a `Scheme` of sixteen slots, one template mapping Orcvs Tokens onto them, and the named keys with their slot defaults — and the Source Grid painting from it, replacing `SourcePaintSettings`' ten resolved colours and `CursorEffectSettings`' colours. Built-in Themes only. Loading a scheme at runtime is `07`, custom Themes are `10`, and chrome reading the Theme is `03`.

**Blocked by:** 01 — Decide where Source colour authority lives; paint-cell-cost/04 — Prove Paint recovered without making Source revisions slower.

**Status:** ready-for-agent

**Tags:** release/v1

- [ ] `Scheme` holds sixteen `Color32` slots, `base00`–`base0F`, and nothing else. No Orcvs vocabulary on it — it is the interchange type, and every published base16 scheme is one.
- [ ] One `TEMPLATE` maps each Token to a slot, at the assignment ADR 0053 records (Sequence on `base0F`). It is a single table, not a method per role.
- [ ] A `Theme` is a `Scheme`, a declared appearance (dark or light), and the named keys ADR 0053 lists, at its exact spellings. Each key a Theme leaves unset resolves from its default slot, so a bare `Scheme` is a complete Theme. Diagnostic and Output Portal take a key per channel (`foreground`, `background`, `border`).
- [ ] Resolution runs one way: `Scheme` → `TEMPLATE` for Tokens, and explicit value or default slot for keys. There are no overrides. A fact's channel paints over the Token's on the same channel, with opacity compositing, and a transparent channel leaves the Token's showing.
- [ ] Okabe–Ito ships as a built-in dark Theme that sets every key explicitly, at the values `console/src/theme.md` maps, so the Source Grid's appearance does not change when this lands. The `style.rs` tests that pin today's glyph colours still pass, read through the scheme rather than from `SourcePaintSettings`.
- [ ] At least two further built-in Themes ship as bare published base16 schemes, so the template and the slot defaults are exercised by palettes they were not designed around.
- [ ] `style.rs::claim_paint` and `operand_paint` stop choosing which channel shows which fact. They answer facts; resolution produces the `CellVisuals`.
- [ ] `SourcePaintSettings`, `CursorEffectSettings`' colours and the `source_paint` storage key are gone. Settings hold the dark Theme's name and the light Theme's name, and no Theme values. The old key is left unread rather than migrated, per ADR 0053.
- [ ] `Theme → Source colours` and the colour controls of `Theme → Cursor effects` become a dark Theme picker and a light Theme picker, each listing only Themes of that appearance. No "Reset" remains.
- [ ] `console/src/theme.md` records the template and the shipped Themes.
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

**2026-09-21 — revised for ADR 0053.** This issue was written against ADR 0051's per-Token overrides keyed by scheme and a Source-only scope. ADR 0053 removes the overrides (a different look is a custom Theme, `10`), moves the Cursor Effect's colours into the Theme, and makes chrome read the same Theme (`03`, which now follows this issue). The performance lines stand unchanged: the Theme resolves once per frame into a flat lookup.
