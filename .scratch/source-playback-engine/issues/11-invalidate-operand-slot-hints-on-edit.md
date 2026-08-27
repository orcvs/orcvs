# 11 — Invalidate operand-slot hints on edit

**What to build:** Make a Cell's glyph classification a function of the Source's current contents. An Expression paints operand-slot hints onto empty Cells beyond its own extent, but an edit landing on one of those Cells finds no Expression to invalidate, so the hint is neither cleared nor restored and the display drifts from the Source.

**Blocked by:** None (can start immediately).

**Status:** resolved

- [x] Editing a Cell carrying an operand-slot hint invalidates and reparses the Expression that painted it.
- [x] A Source built incrementally through edits has the same glyph classification, Cell for Cell, as a Source rebuilt from its own snapshot.
- [x] A Cell holding a character never renders as empty because its glyph was cleared.
- [x] The change set an edit returns reports exactly the Cells whose content or classification differ from the previous revision.
- [x] Operand-slot hints stop at the row edge: a Function near the end of a row neither paints hints into the next row nor clears a neighbouring row's glyphs when its own Cells are emptied.
- [x] Behaviour tests cover a Function at a row edge, not only one at the last Cells of the Grid.

## Notes

Found by review of the Source edit work, not yet triaged by a human.

Reproduction on a single-row Source: type `++`, which paints four operand-slot hints. Set a character on the last hint Cell, then delete it. The hint is gone permanently and the Function shows three operand slots instead of four. A randomized comparison of incremental state against a rebuild from `snapshot()` diverges on hundreds of Cells across 300 seeds.

Fix direction: either confine hint painting to the Expression's own extent, or record the hint extent per Expression so an edit can find the Expression that owns a hint Cell.

## Comments

**2026-08-26 — operand-slot hints bleed across the row edge (review)**

Found by code review of the branch that implemented issue 10; the defect is pre-existing and belongs here.

`Source::set_glyphs` (`console/src/source/source.rs`, lines 367-376) truncates a hint run only at the end of the glyph store — the whole Grid — with `if pos >= self.glyphs.len() { break; }`. The end of a *row* is never consulted, so a hint run started near a row edge continues into the row below.

Concrete repro on a 10 x 6 Grid: write `++` at indices 8 and 9. The completed Function asks for four operand-slot Cells, and they are painted as `Number` at indices 10, 11, 12 and 13 — which are columns 0-3 of row 1, Cells the Expression cannot reach and does not own.

`Source::unset_glyphs` (lines 383-397) has the mirror problem: it walks `start..self.glyphs.len()` clearing every contiguous glyph, so a walk that reaches the last Cell of a row keeps going into the next one. Since it is what `unparse_around` uses to decide how far to invalidate, emptying a Cell near a row edge can clear the glyphs of a neighbouring row's Expression.

Missing coverage: the only test in this area is `test_set_near_grid_end_truncates_operand_hints` (line 562), and it writes `++` at indices 58 and 59 — the last two Cells of the Grid, where the store's own length happens to stop the run. Nothing exercises a Function at a row edge in the middle of the Grid, which is where the truncation is actually absent, and nothing exercises the clearing walk crossing a row edge at all.

## Answer

Source edits now rebuild their complete derived state from the accepted contents. Parsing clears each occupied Expression extent before painting its own glyphs, then Source classifies any remaining occupied Cells as `Char`; App only renders the classification Source provides. Operand hints remain confined to their row. Regression coverage verifies hint overwrite/delete restoration, exact change sets, equality with a snapshot rebuild, visible occupied Cells, and invalidation at a middle row edge without disturbing the neighbouring row.
