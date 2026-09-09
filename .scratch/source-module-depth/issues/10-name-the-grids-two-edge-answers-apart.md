# 10 — Name the Grid's two edge answers apart

**What to build:** Decide, and then spell, the difference between a Grid query that clamps at an
edge and one that reports the edge. `down(pos)` returns a `Position` and clamps in the bottom row;
`below(pos)` returns `Option<Position>` and answers `None` there. Both behaviours are wanted. The
names do not distinguish them, and only one axis has both.

**Blocked by:** None.

**Status:** ready-for-agent

- [ ] The clamping family and the reporting family are named so that a reader picks the one they
      meant without reading the doc comment.
- [ ] The decision covers whether the reporting form exists for all four directions or only where a
      caller needs it, and records which.
- [ ] Every call site moves in the same change; the crate is internal and unpublished, so a
      deprecation window buys nothing.
- [ ] `orcvs/src/grid.rs`'s `mod property` and `mod test` keep stating both behaviours, whatever
      they end up called.

## Comments

Raised from `property-testing/02` on 2026-09-09, which was told to cover both behaviours and to
raise a rename if writing the properties made the confusion concrete. It did, in three ways.

**The property has to say which is which, because the names cannot.**
`every_move_lands_on_a_cell_of_the_grid` states the bottom-row case as two assertions in one
branch — `below(pos)` is `None`, `down(pos)` is `pos` — with a comment naming both, because a
reader who met either line alone could not tell whether the clamp was the bug or the contract. The
existing example test
`test_grid_answers_the_row_below_and_stops_past_the_bottom_row` carries the same pairing for the
same reason. Two tests exist to hold apart a distinction a name could hold on its own.

**Only the vertical axis has the pair.** `up`, `left` and `right` clamp with no reporting
counterpart, so "one row up, or nothing" and "one column left, or nothing" are not askable. A
caller who needs them has to compare coordinates against the dimensions by hand, which is the
arithmetic `source-playback-engine/10` moved into the Grid in the first place.

**A tagged open issue needs the missing half.** `spatial-tick-planning/03` requires that "blocked
or out-of-Grid movement replaces the current Span with Bang", for all four directions and at row
edges. On the vertical axis that is `below`, and `Portal::ordinary_result`
(`orcvs/src/source/portal.rs:79`) already reads exactly that way: `None` from `below` *is* the
bottom-row diagnostic. On the horizontal axis there is nothing to read, and `left`/`right` answer a
silent clamp — the same Position back — which a mover cannot distinguish from a successful step of
zero Cells. Whatever this ticket decides about names, `spatial-tick-planning/03` will otherwise add
the horizontal answer under a third naming convention.

That is an ordering preference, not a dependency, and it is deliberately not written as one.
`spatial-tick-planning/03` does not list this ticket in its `Blocked by:` line and should not: `03`
can be built against today's names and renamed afterwards. `scripts/roadmap.ts` derives the graph
from `Blocked by:` alone, so the preference is invisible to the roadmap by design — whoever picks
up `03` first should read this ticket, and the cost of ignoring it is a third naming convention to
unpick later rather than blocked work.

Deliberately not decided here, because the shape of the answer is the decision:

- One family named for its purpose (`down` stays cursor movement) and the other for its relation
  (`below`, `above`, `left_of`, `right_of`).
- One family suffixed (`down_or_edge`, or `try_down` for the `Option`).
- `Option` everywhere, with the clamp left to the one caller that wants it — the Cursor, which
  CONTEXT.md already says "holds no dimensions and does no clamping of its own". This is the
  smallest surface and the largest call-site change.

No behaviour changes under any of them. `Grid::down` is `below(pos).unwrap_or(pos)`, so the two
already cannot disagree about where one row down is; this is about what a reader is told, not about
what the Grid does. Do not rename without settling the second checkbox: renaming the vertical pair
and leaving the horizontal axis clamp-only would make the asymmetry harder to see rather than
easier.
