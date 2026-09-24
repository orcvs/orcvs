# 01 — Fix the Grid at 256 by 256

**What to build:** Every Grid is 256 columns by 256 rows, per ADR 0054. A fresh console, File > New, the Function reference and a restored Source all stand on that one shape.

**Blocked by:** None (can start immediately).

**Status:** ready-for-agent

**Tags:** release/v1

- [ ] `Grid` has one shape; no caller states dimensions, and `DEFAULT_COL_COUNT`/`DEFAULT_ROW_COUNT` give way to the one shape.
- [ ] The Function reference loads onto the 256 by 256 Grid rather than a Grid rounded up from its text.
- [ ] A stored Source whose Grid is not 256 by 256 is refused through the existing persistence path — set aside under `REFUSED_KEY`, noticed, replaced by an empty Grid — and not migrated.
- [ ] A fresh console opens on the Grid's top-left corner; ADR 0047's margin and Pan reach are unchanged.
- [ ] A benchmark covers each path that walks every Cell (Language Map derivation, Source snapshot, the stored value) at 256 by 256, so CI's comparison reports what the larger Grid costs.
- [ ] Tests that built their own small Grids either keep a test-only constructor beside the shipped one or move to the fixed shape; no shipped function takes a dimension only a test supplies.
- [ ] Scoped gates for `orcvs` and `console` pass, and the no-default-features arm of persistence compiles.
