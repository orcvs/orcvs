# 17 — Measure marker spacing in whole Cells

**What to build:** Give marker spacing a type that matches what it measures. It is a count of Cells, held as a float, and the two places that use it disagree about what a fractional value means — one honours the fraction, the other truncates it, and a value below one Cell aborts the render.

**Blocked by:** None (can start immediately).

**Status:** needs-triage

- [ ] Marker spacing is a whole number of Cells, validated once where it is configured rather than asserted on every Cell of every frame.
- [ ] Marker placement and the cursor's marker block derive from the same value, and cannot disagree.
- [ ] No assertion runs in the per-Cell render path.
- [ ] Rendering at the default spacing is unchanged.

## Notes

Found by review, not yet triaged by a human.

Marker placement tests the raw float (`x as f32 % marker_spacing == 0.0`), while the block predicate truncates to an integer. At the default of 8 they agree exactly; at 8.5 markers land on a 8.5-Cell rhythm while blocks stay on 8. A spacing between 0 and 1 truncates to zero and trips the assert guarding the division — which sits inside the predicate, so it runs per Cell per frame and takes the window down rather than degrading.

The field is public and mutable, so all of this is reachable without changing any code. Typing it as a count of Cells removes the disagreement, the assert, and the divide-by-zero together.

Related to issue 14, which is about the size of the block rather than the units it is measured in. Both are marker-block geometry and could be taken together.
