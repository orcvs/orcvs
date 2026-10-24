# 01 — Record the Overview

**What to build:** The Overview's vocabulary and standing decisions, before any code uses them. `CONTEXT.md` gains the **Overview** entry and its _Avoid_ list from `../spec.md`, and Pan's entry names the Overview as a way to Pan. A short ADR records that the Overview is a further Pan gesture (amending ADR 0045's list), that its picture is cached in `console` across Render Frames keyed by the Source's revision and the presented Theme rather than on the Language Map (ADR 0050, ADR 0052), why a retained texture is consistent with ADR 0040's one-frame GlyphTable, and that one new Theme key styles the outline. See `../spec.md` and `docs/research/egui-minimap.md`.

**Blocked by:** None — can start immediately.

**Status:** resolved

- [x] The ADR takes the next number in `docs/adr/`, follows the existing ADR shape (decision, rejected alternatives, consequences), and records each of the four decisions above with its reason.
- [x] Rejected alternatives cover: per-Cell Shapes or one Mesh (re-sent every Render Frame), a paint callback (backend-specific), a floating overlay (covers Cells), and caching on the Language Map.
- [x] `CONTEXT.md` gains **Overview** with _Avoid_: minimap, map, thumbnail, navigator, viewport, scrollbar; no second term is minted for the outline.
- [x] `node --test scripts/tests/roadmap.test.ts` and `node scripts/roadmap.ts > /dev/null` pass.

## Comments

### Shipped, 2026-10-23

ADR 0080, `docs/adr/0080-the-overview-pans-over-a-cached-picture.md`, records the four decisions and
the four rejected alternatives. ADR 0045 carries a dated amendment adding the Overview to its Pan
gestures. `CONTEXT.md` gains **Overview**, and **Pan** names it as a way to Pan. The outline's Theme
key is spelled when ticket 03 adds it to the schema.
