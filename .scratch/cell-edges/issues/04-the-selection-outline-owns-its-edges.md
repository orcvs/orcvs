# 04 — The selection outline owns its edges

**What to build:** The Cursor's Cell, when no Region spans more than one Cell, and the perimeter of a Region that does, are the highest-ranked edges. They show one outline in `cursor.border` at `cursor.border.width`, or `region.border` at `region.border.width`, centred on its edges and identical on all four sides, with no Grid line or neighbour's border beside it. The Cursor's own Cell inside a spanning Region ranks next, with the border the Paint gives it. Nothing is stroked inside a Cell any more. See `../spec.md` and ADR 0078.

**Blocked by:** 03 — Draw every border edge once.

**Status:** resolved

- [x] Built test-first at the Paint: the selection outline outranks every other fact on its edges, for a single Cursor and for a spanning Region, and the Cursor's Cell inside a spanning Region ranks below it and above Portals.
- [x] A width of zero for the Cursor or Region border withdraws the outline and the edges fall to the next fact.
- [x] A Shape-level test shows the Cursor's four sides are the same width and colour and that no Grid line piece lies beside them.
- [x] If the Cursor Effect is still present, its animated fragments draw after the Glyphs and its own still frame is no longer stroked; the effect's living area is unchanged.
- [x] The "a Region hides the Cursor's Cell border" test and the Cursor-border tests are rewritten to the edge rule.
- [x] `.scratch/source-cell-size/spec.md`'s "The Cursor outline" decision and its ticket 02 are reworded so the outline is the highest-ranked edge, not a stroke inside the Cell. If that effort's tickets have already shipped the inside stroke, this ticket replaces it.
- [x] The console gate and the `--no-default-features` arm pass; `mise run check_wasm` passes.

## Comments

- The outline is a value on the Paint: the Cursor's Cell, or a spanning Region, as Grid columns and rows, with `cursor.border` at `cursor.border.width` or `region.border` at `region.border.width`. `Paint::edges` gives it every edge of that perimeter outright, and `None` at width zero lets those edges fall to the facts beside them.
- The Cursor's own Cell inside a spanning Region keeps the border its fact gives it, ranked `RegionCursor`, between Output Portal and the outline. An unclaimed Cell has no border, so there it shows the Grid line.
- The single-Cell Cursor no longer reads `selection.border`, `.rest` or `cell.selection.border.width`. In the running console the Cursor Effect's frame already replaced that stroke whenever it was built, so nothing visible is lost. The key is still accepted and nothing reads it; ticket 07 asks whether to retire it. `RenderFrame::cursor_visible` has no console reader any more either.
- At amount zero the Cursor Effect builds no Shape of its own, since the edges draw the still outline. At a positive amount its fragments still draw after the Glyphs, and its living area is unchanged.
- `#259`'s spec ("The Cursor outline") and its ticket 02 were reworded on that PR's branch.
- Shape tests that pinned the Grid lattice now leave out the Cursor's outline box (`assert_drawn_once_around`). `the_cursor_outline_is_one_box_the_same_on_all_four_sides` asserts the box itself at 1×, 1.5× and 2×.
