# 02 — Keep Expressions horizontal and diagnosable

**What to build:** Let users freely create complete, incomplete, and invalid Expressions during Live Editing while keeping Expression grouping spatially correct and diagnostics current for the visible Source revision.

**Blocked by:** 01 — Make Source edits synchronous and consistent; 10 — Source derives no grid arithmetic.

**Status:** resolved

- [x] An Expression is confined to one row and never joins the last occupied Cell of one row to the first occupied Cell of the next.
- [x] Incomplete and invalid Expressions preserve their Cells and receive observable diagnostics instead of rejecting intermediate edits.
- [x] Editing an Expression recomputes its classification and diagnostics in the same Source transition.
- [x] Fixing or removing the cause of a diagnostic removes that diagnostic from the resulting Source revision.
- [x] Behavior tests cover joining, splitting, replacing, and deleting Expressions through Source edits, including rectangular grids and row edges.
- [x] Tests no longer require public access to Expression range maintenance merely to verify Source behavior.
- [x] Every Tick test that currently records a wrapped Expression as expected-for-now asserts the row-confined outcome instead, so no test still encodes the wrap.

## Answer

Expression grouping is confined to each Grid row. Source edits preserve incomplete and invalid content while synchronously rebuilding classification and range-addressed diagnostics for the visible revision; joining, splitting, replacing, deleting, rectangular grids, and row edges are covered through Source behavior tests.

## Comments

**2026-08-26 — root cause and repro for the row wrap (review)**

Found by code review of the branch that implemented issue 10; the defect is pre-existing and belongs here, not to that branch.

`ExpressionMap::set_inner` (`console/src/source/expression_map.rs`, lines 144-198) joins a Cell to its left and right neighbour by index alone — `idx - 1` and `idx + 1`, with no knowledge of where a row ends. Every branch that appends, prepends, joins or splits therefore treats the last Cell of a row and the first Cell of the next as adjacent.

Concrete repro on a 10 x 6 Grid: write a character at index 9, then one at index 10. The second edit takes the (Some(left), None) branch and appends index 10 to the Expression that starts at index 9, producing a single Expression spanning (9, 0) and (0, 1) — exactly the join criterion 1 forbids.

The fix needs the Grid: `set_inner` has to ask whether a neighbouring index is in the same row rather than deriving it. That dependency is satisfied now — the Grid exists and issues 08, 09 and 10 have landed, so this ticket is unblocked on that count.

One existing test currently asserts the wrap as expected-for-now: `test_result_that_cannot_fit_before_the_row_edge_is_discarded` in `console/src/source/source.rs` (lines 721-736, with the `issue 02` note at line 729) writes `++0102` at index 9, so the Expression spans rows 0 and 1, and asserts row 2 stays empty. Once an Expression is confined to one row that write becomes `+` at index 9 plus a separate `+0102` Expression at indices 10-14, which does have a result to commit into row 2. The assertions invert when this ticket lands.

**2026-08-26 — a second test now encodes the wrap (review)**

The comment above names one test recording a wrapped Expression as expected-for-now. There are two. `test_result_reaching_the_last_column_exactly_is_committed` was added to `console/src/source/source.rs` on the same branch, before the comment above was written and not caught by it. It writes `++0102` at index 8 of a 10 x 6 Grid, so the Expression spans Cells 8 to 13 and crosses the row edge exactly as `test_result_that_cannot_fit_before_the_row_edge_is_discarded` does, and carries the same `issue 02` note.

What it is for is the row edge's other side: a two-Cell result whose last Cell is the last column of its row, which must be committed rather than discarded. It needs a wrapping Expression only because that is the only way to put a result in that Cell today. When this ticket lands, the write splits into `++` at Cells 8 and 9 and a bare `0102` at Cells 10 to 13 — an incomplete Function with no operands and a Number with no Function, neither of which commits anything — so row 1 becomes ten spaces and the assertion inverts. The exact-fit boundary then needs a setup that does not depend on a wrap, or it stops covering anything.

Criterion 7 is reworded from "The Tick test that currently records" to "Every Tick test that currently records", for the same reason: the singular was accurate when written and now understates the work.

**2026-08-26 — implemented (agent)**

`ExpressionMap` now owns the Source's Grid and considers neighbours only within the edited Cell's row. Parser failures are retained as ordered, range-addressed Source diagnostics while the tolerant parse continues to classify and preserve incomplete or invalid user content; accepted edits synchronously replace or remove stale diagnostics. Operand-slot glyph hints and their invalidation walk also stop at the Expression's row edge.

Behavior tests at the Source seam cover the row boundary and current diagnostic lifecycle alongside the existing join, split, replacement, and deletion coverage. Both Tick setups that depended on a wrapped `++0102` now assert the row-confined outcome; the misleading exact-fit claim was removed because no complete computing Expression can start in the last two columns without itself wrapping, and a separate assertion pins row-edge operand hints. Strict parsing now also diagnoses trailing content after an otherwise valid Expression prefix.
