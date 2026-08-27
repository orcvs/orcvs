# 17 — Measure marker spacing in whole Cells

**What to build:** Give marker spacing a type that matches what it measures. It is a count of Cells, held as a float, and the two places that use it disagree about what a fractional value means — one honours the fraction, the other truncates it, and a value below one Cell aborts the render.

**Blocked by:** None (can start immediately).

**Status:** resolved

- [x] Marker spacing is a whole number of Cells, validated once where it is configured rather than asserted on every Cell of every render frame.
- [x] Marker placement and the cursor's marker block derive from the same value, and cannot disagree.
- [x] No assertion runs in the per-Cell render path.
- [x] Rendering at the default spacing is unchanged.

## Answer

`MarkerSpacing` is a positive whole-Cell value backed by `NonZeroUsize`. Marker placement and marker-block geometry consume that same value without narrowing it, including at `usize::MAX`, and the render path contains no spacing assertion. Regression tests cover validation, exact one/two-Cell placement, the default block behavior, and the largest accepted spacing.

## Notes

Found by review, not yet triaged by a human.

Marker placement tests the raw float (`x as f32 % marker_spacing == 0.0`), while the block predicate truncates to an integer. At the default of 8 they agree exactly; at 8.5 markers land on a 8.5-Cell rhythm while blocks stay on 8. A spacing between 0 and 1 truncates to zero and trips the assert guarding the division — which sits inside the predicate, so it runs per Cell per render frame and takes the window down rather than degrading.

The field is public and mutable, so all of this is reachable without changing any code. Typing it as a count of Cells removes the disagreement, the assert, and the divide-by-zero together.

Related to issue 14, which is about the size of the block rather than the units it is measured in. Both are marker-block geometry and could be taken together.

## Comments

**2026-08-26 — one consumer of the spacing is gone (review)**

On the issue 10 branch, `DEFAULT_COL_COUNT` and `DEFAULT_ROW_COUNT` moved from `console/src/opts.rs` to `console/src/grid.rs` and stopped being `2 * (DEFAULT_MARKER_SPACING as usize)`. They are plain literal `16`s now. Both values are unchanged, so criterion 4 — rendering at the default spacing is unchanged — is not disturbed by the move.

It narrows this ticket. The two places that used the spacing and disagreed about a fractional value were marker placement and the block predicate; there was a third, quieter one, which took `DEFAULT_MARKER_SPACING as usize` at compile time and made the default Source's shape a function of it. A spacing that was not a whole number, or one below a single Cell — the exact two cases this ticket is about — resized the Source as well as breaking the rendering. That path no longer exists. Typing this field as a count of Cells is now a change to the console's visuals only, and the Grid's extent is not a thing the change has to preserve.
