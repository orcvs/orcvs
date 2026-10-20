# 04 — The selection outline owns its edges

**What to build:** The Cursor's Cell, when no Region spans more than one Cell, and the perimeter of a Region that does, are the highest-ranked edges. They show one outline in `cursor.border` at `cursor.border.width`, or `region.border` at `region.border.width`, centred on its edges and identical on all four sides, with no Grid line or neighbour's border beside it. The Cursor's own Cell inside a spanning Region ranks next, with the border the Paint gives it. Nothing is stroked inside a Cell any more. See `../spec.md` and ADR 0078.

**Blocked by:** 03 — Draw every border edge once.

**Status:** ready-for-agent

- [ ] Built test-first at the Paint: the selection outline outranks every other fact on its edges, for a single Cursor and for a spanning Region, and the Cursor's Cell inside a spanning Region ranks below it and above Portals.
- [ ] A width of zero for the Cursor or Region border withdraws the outline and the edges fall to the next fact.
- [ ] A Shape-level test shows the Cursor's four sides are the same width and colour and that no Grid line piece lies beside them.
- [ ] If the Cursor Effect is still present, its animated fragments draw after the Glyphs and its own still frame is no longer stroked; the effect's living area is unchanged.
- [ ] The "a Region hides the Cursor's Cell border" test and the Cursor-border tests are rewritten to the edge rule.
- [ ] `.scratch/source-cell-size/spec.md`'s "The Cursor outline" decision and its ticket 02 are reworded so the outline is the highest-ranked edge, not a stroke inside the Cell. If that effort's tickets have already shipped the inside stroke, this ticket replaces it.
- [ ] The console gate and the `--no-default-features` arm pass; `mise run check_wasm` passes.
