# 14 — Decide the marker block size

**What to build:** Settle how wide the cursor's marker block is, and make the predicate and its test agree with the answer. The block is currently one Cell wider and one Cell taller than the marker spacing, so it reaches into the neighbouring block.

**Blocked by:** None (can start immediately).

**Status:** needs-triage

- [ ] The intended block size is recorded: either the spacing itself, or the spacing plus its closing marker Cell.
- [ ] The predicate matches that decision, and the test asserts the decided size rather than the current behaviour.
- [ ] Highlight dots no longer appear in a Cell belonging to the next marker block, if that is the decision.

## Notes

Found by review, not yet triaged by a human — hence `needs-triage` rather than `ready-for-agent`. This is a question about intent before it is a defect.

With a spacing of 8 and the cursor at (8, 8), the block spans x and y from 8 through 16 inclusive: nine Cells, not eight. Cell 16 begins the next marker block. The effect is visual only — one extra column and row of highlight dots — and where those Cells also carry a marker, the marker wins because it is tested first.

The behaviour predates this work. Ticket 07 moved the predicate verbatim and kept its test intact, so the test asserts the nine-Cell span; whichever way this is decided, that test changes with it.

## Comments

**2026-08-26 — the Source's shape no longer derives from the marker spacing (review)**

Recorded here because it changes what this decision can cost. On the issue 10 branch, `DEFAULT_COL_COUNT` and `DEFAULT_ROW_COUNT` moved from `console/src/opts.rs` to `console/src/grid.rs`, and in the move stopped being `2 * (DEFAULT_MARKER_SPACING as usize)` and became plain literal `16`s. Both values are unchanged — the spacing is 8.0 — so nothing renders differently. What is gone is the derivation: a Source's shape was a function of a visual constant and is not any more.

That matters to this ticket in one direction. Whichever way the block size is settled, and whether or not settling it moves the marker spacing too, the default Source stays 16 x 16. Before the move, changing `DEFAULT_MARKER_SPACING` would have silently resized the Source along with the block, so any answer here that touched the spacing carried a Source resize with it. It no longer does, and the two questions can be decided independently.
