# 11 — Invalidate operand-slot hints on edit

**What to build:** Make a Cell's glyph classification a function of the Source's current contents. An Expression paints operand-slot hints onto empty Cells beyond its own extent, but an edit landing on one of those Cells finds no Expression to invalidate, so the hint is neither cleared nor restored and the display drifts from the Source.

**Blocked by:** None (can start immediately).

**Status:** ready-for-agent

- [ ] Editing a Cell carrying an operand-slot hint invalidates and reparses the Expression that painted it.
- [ ] A Source built incrementally through edits has the same glyph classification, Cell for Cell, as a Source rebuilt from its own snapshot.
- [ ] A Cell holding a character never renders as empty because its glyph was cleared.
- [ ] The change set an edit returns reports exactly the Cells whose content or classification differ from the previous revision.

## Notes

Found by review of the Source edit work, not yet triaged by a human.

Reproduction on a single-row Source: type `++`, which paints four operand-slot hints. Set a character on the last hint Cell, then delete it. The hint is gone permanently and the Function shows three operand slots instead of four. A randomized comparison of incremental state against a rebuild from `snapshot()` diverges on hundreds of Cells across 300 seeds.

Fix direction: either confine hint painting to the Expression's own extent, or record the hint extent per Expression so an edit can find the Expression that owns a hint Cell.
