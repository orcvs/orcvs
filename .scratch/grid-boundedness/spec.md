# Decide what the Grid's edge means before the language depends on it

**Goal:** Settle whether the Source Grid's boundary is a permanent language concept or a convenience
of the bounded prototype, while both answers are still free.

## Why now

`ADR 0005` defers Source addressing precisely so Orcvs *may* become a sparse infinite canvas. The
console's surface is already described as more canvas than grid. Nothing has decided the question,
and two efforts about to land would answer it by accident.

An audit of how deeply the model assumes a bounded Grid found the execution path remarkably ready
for an unbounded one. `lang` has no Grid concept at all — no dependency on `orcvs`, no row or column
counts; Clock, Delay and Euclidean read only the absolute Tick. Tick planning is already
O(expressions) rather than O(cells), iterating `map.expressions()` and not Positions. And every edge
behaviour on the Tick path **refuses** rather than clamping: `Grid::position`, `Grid::below`,
`Grid::offset_in_row` and `Portal` all answer `None` at a boundary, while the clamping helpers
`up`, `down`, `left` and `right` are reached only by the Cursor. Unbinding the Grid would remove
refusals; it would not silently change what any existing Source means.

Two things are not like that, and both are about to stop being free.

## The first gate: what bounds a Reservation

`Reserved::Row` covers `start..start + (grid.cols() - output.x())` (`orcvs/src/source/tick.rs:142`).
That is not a bounds check. It builds the dependency edges that order Turns, and ADR 0036's own
rejected alternatives identify a row-wide reservation as something that "manufactures cycles between
Expressions that never touch". In an unbounded row, `the end of its row` names nothing and the edge
set is unbounded.

It is dormant today. `Answer::Sequence` carries `#[expect(dead_code)]` (`lang/src/atom.rs:373-383`)
because Range, Reverse, Concatenate and Replace are unbuilt, and the attribute is deliberately
`expect` rather than `allow` so "the first of them turns this attribute into the error that deletes
it". `Reserved::Row` is specified, coded, tested, and unreachable from any Source anyone can write.

`sequence-values/03` and `05` build those Functions. After they land, "reserves through the end of
that destination's row" is the meaning of real programs, and unbinding the row changes what
already-written Grids do.

## The second gate: whether leaving the Grid is a terminal state

Self-Banging and Directional Bang movement, Jump chains and Halt have no implementation at all —
`spatial-tick-planning/03`, `04` and `05` are `ready-for-agent`. Their boundary semantics exist only
as ADR prose and glossary text: a blocked or **out-of-Grid** move replaces its Span with `**`
(ADR 0006, ADR 0014, `CONTEXT.md`); an out-of-Grid Jump destination diagnoses and writes nothing
(ADR 0014, ADR 0020).

Hitting an edge is currently a Self-Banging Function's **only** terminal state. On an unbounded
canvas it never stops, allocating a Cell per Tick forever. That is a language design question with
no present answer, and implementing those three issues answers it by writing an edge-terminal
semantic into the language.

## What this is not

Not a renderer question. `source-grid-rendering` bakes in nothing: `RenderFrame`'s dense
`Vec<Vec<RenderCell>>` is a consequence of the Grid rather than a commitment about it, and that
effort's issue 05 culls the draw loop to a visible Position range, which is a prerequisite for an
unbounded Source rather than an obstacle to one. Build it either way.

Not a decision to go infinite. The answer may well be that the Grid stays bounded and both semantics
are permanent. That is a fine outcome and costs nothing to record.

## Known regardless of the answer

`CellIndex` is a scalar `y * cols + x`, and there is no Grid resize path at all: `Grid` is `Copy`
with private dimensions and no setters, and a new `Grid` mints a new `GridId` that panics every
outstanding Position — Cursor, Portal, Span, Expression root, Computation anchor.
`LanguageMap::rebuild` additionally asserts the Grid is the same one. So **even a bounded but
growable canvas is this same rearchitecture**, independent of infinity. Worth knowing before anyone
promises a resizable console.

Two stale numbers found while auditing, neither load-bearing: `orcvs/src/grid.rs:76-79` says the
default is 8 by 5 where the constants below it are 40 and 25, and `ADR 0005` calls the console's
default 64 by 64.
