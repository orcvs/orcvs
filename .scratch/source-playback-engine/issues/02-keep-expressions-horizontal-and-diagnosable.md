# 02 — Keep Expressions horizontal and diagnosable

**What to build:** Let users freely create complete, incomplete, and invalid Expressions during Live Editing while keeping Expression grouping spatially correct and diagnostics current for the visible Source revision.

**Blocked by:** 01 — Make Source edits synchronous and consistent; 10 — Source derives no grid arithmetic.

**Status:** ready-for-agent

- [ ] An Expression is confined to one row and never joins the last occupied Cell of one row to the first occupied Cell of the next.
- [ ] Incomplete and invalid Expressions preserve their Cells and receive observable diagnostics instead of rejecting intermediate edits.
- [ ] Editing an Expression recomputes its classification and diagnostics in the same Source transition.
- [ ] Fixing or removing the cause of a diagnostic removes that diagnostic from the resulting Source revision.
- [ ] Behavior tests cover joining, splitting, replacing, and deleting Expressions through Source edits, including rectangular grids and row edges.
- [ ] Tests no longer require public access to Expression range maintenance merely to verify Source behavior.
- [ ] The Tick test that currently records a wrapped Expression as expected-for-now asserts the row-confined outcome instead, so no test still encodes the wrap.

## Comments

**2026-08-26 — root cause and repro for the row wrap (review)**

Found by code review of the branch that implemented issue 10; the defect is pre-existing and belongs here, not to that branch.

`ExpressionMap::set_inner` (`console/src/source/expression_map.rs`, lines 144-198) joins a Cell to its left and right neighbour by index alone — `idx - 1` and `idx + 1`, with no knowledge of where a row ends. Every branch that appends, prepends, joins or splits therefore treats the last Cell of a row and the first Cell of the next as adjacent.

Concrete repro on a 10 x 6 Grid: write a character at index 9, then one at index 10. The second edit takes the (Some(left), None) branch and appends index 10 to the Expression that starts at index 9, producing a single Expression spanning (9, 0) and (0, 1) — exactly the join criterion 1 forbids.

The fix needs the Grid: `set_inner` has to ask whether a neighbouring index is in the same row rather than deriving it. That dependency is satisfied now — the Grid exists and issues 08, 09 and 10 have landed, so this ticket is unblocked on that count.

One existing test currently asserts the wrap as expected-for-now: `test_result_that_cannot_fit_before_the_row_edge_is_discarded` in `console/src/source/source.rs` (lines 721-736, with the `issue 02` note at line 729) writes `++0102` at index 9, so the Expression spans rows 0 and 1, and asserts row 2 stays empty. Once an Expression is confined to one row that write becomes `+` at index 9 plus a separate `+0102` Expression at indices 10-14, which does have a result to commit into row 2. The assertions invert when this ticket lands.
