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
