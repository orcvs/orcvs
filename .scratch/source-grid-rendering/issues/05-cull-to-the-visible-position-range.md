# 05 — Cull the Source Grid to the visible Position range

**What to build:** Iterate only the Positions the console is showing, so drawing cost follows the
viewport rather than the Source.

**Blocked by:** None. The geometry it rests on is settled; only its justification looks forward.

**Status:** resolved

- [x] The Cell loop derives a Position range from the visible rectangle and the current transform,
      and iterates that range instead of walking every row of `frame.rows()` in full. The one
      remaining full walk is `source_dimensions` (`console/src/console.rs:104`), which reads the
      column count and is not the draw loop; leave it.
- [x] The range is expanded by one Cell in every direction beyond what the viewport clips, and the
      clip rect discards the surplus. Culling to exactly the visible range is not enough, and the
      reason is ownership rather than tolerance: a Cell's left and top seam lines are drawn by the
      Cell one past the boundary (`console/src/console.rs:688-699`), so an exact range drops the
      trailing lines at the right and bottom edges. A Cell's border also spills roughly one physical
      pixel outside its own rectangle. The margin additionally gives the run logic below the
      neighbour state it needs.
- [x] Background run coalescing still breaks at the same places. Runs are row-local, opened and
      flushed within one row (`console/src/console.rs:622`, `:730-734`), and a range-limited row must
      open its run at the first drawn Cell rather than carrying one in from off-screen. The failure
      this guards against is a one-Cell seam at the left edge of the viewport, visible only when
      zoomed in, which is exactly where no current test looks.
- [x] Run endpoints still come from `GridViewport::cell_rect`, not from an accumulated sum, so a
      coalesced edge stays bit-identical under culling.
      `consecutive_cells_sharing_a_background_are_one_rectangle` keeps passing.
- [x] Clicks inside the visible range select what they select today. Clicks outside it are already
      refused: the interaction rectangle is `grid.rect.intersect(clip)`
      (`console/src/console.rs:581-585`), so a click outside the console cannot reach the Grid. Check
      this holds rather than adding a second guard.
- [x] Sector seam lines and the Cursor bloom are correct at the edges of the visible range,
      including when the Cursor is outside it.
- [x] The visible range is a shipped pure function in `grid_viewport.rs`, alongside `grid_viewport`,
      `presented_grid` and `cell_at`, and is the draw loop's only source of Positions. Everything it
      needs is already `cell_at`'s inputs — `self.rect`, `self.cell_size`, `columns`, `rows` and the
      clip rectangle `show_source` already receives (`console/src/console.rs:560`) — so it is
      `cell_at` on the clip's two corners, plus the margin and a clamp. A test asserts on its real
      return value and derives the count from it. That is not a test-only seam: one shipped caller, a
      real return value, no branch a test alone reaches.
- [x] A second test proves the draw loop honours the range, by comparing the shapes in `FullOutput`
      between a fully visible Grid and a zoom showing a fraction of it. Coalescing means the shape
      count is not proportional to the Position count, so assert a bound rather than an equality.
- [x] Neither test introduces a counter the console increments for a test's benefit. If the range
      cannot be extracted as a pure function without distorting the module, say so here and drop the
      acceptance line rather than cutting the seam.
- [x] The change claims structure, not speed: cost follows the viewport rather than the Source. No
      frame time is asserted and no benchmark is added to `console`.

## Comments

**Why this is worth doing is resize, not the infinite canvas.** An earlier draft rested the whole
case on `ADR 0005` — a renderer whose cost scales with the total Cell count cannot serve a sparse
infinite canvas. That canvas is now a v2 concern, and an issue justified by a deferred ADR inherits
the deferral. The live justification is a resizable Grid: the draw loop is built for a thousand
Cells, and a 200 by 200 Grid is forty thousand. Culling is what stands between resize and a console
that stops being usable at the sizes resize exists to offer.

**It does not wait for resize.** Resize is gated on the `GridId` rearchitecture that
`.scratch/grid-boundedness/issues/01` decides — `Grid` is `Copy` with private dimensions and no
setters, and minting a new one panics every outstanding Position. None of that reaches this issue,
which reads `frame.rows()` and geometry and does not care how the Grid got its dimensions. So this
lands on its own and removes one thing from resize's path.

**Culling the draw loop does not make total cost follow the viewport, and this issue does not claim
it does.** `RenderFrame::derive` (`orcvs/src/render_frame.rs:74-112`) walks every Cell every Render
Frame with nothing cached, allocates a `Vec` per row, and pays `cursor_bloom`'s Chebyshev arithmetic
and `cell_hash` for every Cell rather than only those inside the radius — `classify_cursor_bloom`
returns `None` for distant Cells only after the hash is computed. `SourceCommander::read_revision`
additionally clones the whole Source string per call (`orcvs/src/source/mod.rs:159-166`). That cost
is an `orcvs` concern with its own effort and its own benchmark
(`orcvs/benches/source.rs:283-295` already measures `render_frame()` at 16, 32 and 64 square). Apply
culling to the draw loop and **not** to `RenderFrame::derive`: everything the draw loop reads is
already local to its Cell, so a Cursor outside the visible range still blooms correctly on visible
Cells with no special handling. If a later effort culls derivation, this issue's one-Cell margin
becomes load-bearing rather than cosmetic.

**Be accurate about what culling buys.** epaint already discards off-screen shapes at tessellation
(`coarse_tessellation_culling`, default on, `epaint-0.36.1/src/tessellator.rs:679`), so this saves
Cell iteration and shape construction, not tessellation of shapes nobody sees. What it does reach is
the part coalescing could not: a default frame emits roughly 1,100 to 1,230 shapes, of which about a
thousand are per-Cell border strokes, one per Cell, untouched by issue 04. Culling is the only lever
that reduces those.

**Nothing benchmarks the console draw loop.** There is no `console/benches/`, and `console/tests/`
holds only `wasm.rs`. Do not add a harness for this — `hxy-view` recorded that its equivalent win did
not appear in an `egui_kittest` harness at all, because such a harness never drives the tessellator.
Assert against the Render Frame, which is this effort's stated rule, and claim structure.

**Every surveyed painter-path implementation culls**, including a hex editor at far smaller per-Cell
cost than this console. The one project that culls nothing draws on the GPU in a single call, and
culls strictly on the one screen where it does use the painter per Cell.

---

Resolved. `GridViewport::visible_positions` answers a column range and a row range, and the Cell
loop iterates those and nothing else. On the default Grid at the zoom limit the console shows 345 of
1,000 Positions and emits 413 shapes against 1,217 — a third of the iteration and a third of the
Shape construction, which is the whole of what this change claims.

**The range function came out as a pure function with no distortion**, so that acceptance line
stands rather than being dropped. It is `cell_at` on the two corners of `self.rect.intersect(clip)`,
plus the margin and the clamp, and `cell_at` already refuses a corner outside the Grid — which is
where the missed clip, the empty Grid and the degenerate viewport are all refused, rather than in
three guards of its own. `VisiblePositions::count` is what the tests derive a Cell count from.

**The draw loop slices rather than filters**, and that is what makes the run row-local under
culling: `row.get(visible.columns.clone())` gives the inner loop a slice that starts at the first
drawn Cell, so there is no off-screen run to carry in. A loop that walked the whole row and skipped
the Cells outside the range would be the version the issue warns about.

**Four mutation checks, and one of them is a finding rather than a confirmation.**

1. Dropping the one-Cell margin (`first..last + 1` instead of `first - 1..last + 2`) fails
   `the_visible_range_is_the_shown_positions_and_one_cell_more_each_way` with
   `VisiblePositions { columns: 4..12, rows: 4..12 }` against `3..13`; with that exact-range
   assertion also removed it still fails on the derived property, at
   `the Cell at 4,4 is shown, so the range has no margin there`.
2. Never flushing the run at the end of a drawn row fails
   `a_zoomed_row_fills_every_cell_the_viewport_shows_and_no_other` at
   `the shown Cell Position { x: 14, y: 14 } was filled wrongly: left: None, right: Some(#08_12_11_FF)`,
   and fails `consecutive_cells_sharing_a_background_are_one_rectangle` at `left: 24, right: 25`.
3. Drawing one Cell short of the range on the left fails the same zoomed test at
   `the shown Cell Position { x: 14, y: 13 } was filled wrongly`, along with four older tests.
4. Culling rows but not columns fails `the_draw_loop_paints_the_visible_range_rather_than_the_whole_source`
   at `the zoomed console drew Positions the viewport does not reach: left: 600, right: 345`.

**The finding: the row-locality of the run is not observable in a Render Frame, and was not
observable before this change either.** Hoisting `let mut run` out of the row loop — so a run
survives into the row below — passes all 76 tests, this issue's new ones included. The reason is
arithmetic rather than a gap in the fixtures. A leaked run widens to
`Rect::from_min_max(covered.min, rect.max)` where `covered.min` is in row *N* at the run's own start
column and `rect.max` is in row *N+1* at the **first drawn** column. The first drawn column is the
smallest column either row can hold, so `max.x` is at most `min.x` plus one Cell: the rectangle is
either inverted, and `Rect::contains` answers false for every point in it, or exactly one Cell wide
at the first drawn column — which under culling is a margin Cell the clip discards, and without
culling is column zero. Either way nothing a viewer sees moves. The end-of-row flush *is*
observable, and check 2 above pins it. So the acceptance line about runs is satisfied by
construction and by check 2; the "carried in from off-screen" half of it has no reachable failure to
guard against, and a later reader should not add a test claiming to catch one.

**The zoomed fixture is deliberately small and asserts its own preconditions.** A 400 by 300 console
at the zoom limit is narrower than the Cursor's fifteen-Cell bloom, so every drawn row both starts
and ends inside a filled region and a run has somewhere to leak from. The Cursor sits below the
window rather than inside it, so the viewport holds filled and unfilled Cells at once; with it
inside, every shown Cell was filled and the "asks for nothing" half of the test held vacuously — the
first attempt failed on `48 filled and 0 unfilled shown Cells`. The test asserts the filled-at-both-
edges and mixture preconditions from the Render Frame, so it cannot go quietly vacuous if the bloom
moves.

**Clicks were checked rather than guarded again, and the check is now structural.** The interaction
rectangle is still `grid.rect.intersect(clip)` and nothing was added beside it. A pointer inside
that rectangle is inside the clip, and `visible_positions` is `cell_at` on that same rectangle's
corners, so the Cell a click resolves to is always inside the drawn range — culling cannot make a
click land on a Cell that was never drawn. The existing click tests, at the fit and at a zoom, pass
unchanged.

**The tests set `SourceView` directly rather than driving twenty wheel events.** `pinned_at` writes
`to_global` and `adjusted`, which is the state `register_pan_and_zoom` would leave behind and which
nothing in `show_source` or `show_source_scene` exists to serve. No parameter, field or branch was
added to shipped code for a test, and no counter was added for one either.

`RenderFrame::derive` is untouched, `source_dimensions` still walks `frame.rows()` for the column
count, and nothing under `orcvs/` was changed.

---

Reviewed. Three independent reviews ran against `main`, their findings were verified against the
code, and the fixes are below. Two tests were added and the console suite is 78.

**The margin's stated rationale was backwards, and the correction is substantive rather than
editorial.** Three comments said "a Cell's left and top sector seams are drawn by the Cell one past
them". A Cell draws the seams on its *own* left and top edges (`console/src/console.rs:729-740`), so
it is a Cell's *right and bottom* seams that belong to the Cell one past it. The conclusion — one
more Cell on the far side — was right, but the reason as written argued for a margin on the near
side, where none is needed.

Correcting it exposed the larger claim. **The margin reaches no painted Shape strictly inside the
clip.** A margin Cell's left edge *is* the right boundary of what is shown, never inside it, so the
seam it would draw is on screen only where the clip's edge falls exactly on that Cell boundary; the
feathering is sub-pixel. Removing the margin outright — `first..last + 1` — changes no painted
output and fails only `the_visible_range_is_the_shown_positions_and_one_cell_more_each_way`, which
is a range assertion. So the acceptance line about trailing seams is satisfied by the margin
existing, not by a seam a test can catch it rescuing, and the acceptance text's own version of the
rationale carries the same inversion. The margin is kept: the boundary-aligned case and the
sub-pixel spill are real, and the clip is what the surplus is handed to.

**`a_zoomed_console_paints_every_sector_seam_inside_the_clip` is the seam criterion, finally
asserted.** The pre-existing seam test runs on a default `SourceView` — the fit, every Position
drawn — so nothing pinned seams under culling. The new test asserts, from the Render Frame at a zoom
that culls on all four sides, that every seam strictly inside the clip is painted. It sweeps two
pans, because which seams land strictly inside is a property of the pan: at one pan the nearest seam
sat six columns from the edge and a column-side error went unseen. It bites on every cull that drops
a shown Cell — two shown columns on the near side, one on the far, the same for rows, and columns
culled to nothing. It is provably blind to a cull short by exactly one, which removes only the
margin, for the reason above.

**A clip sharing one edge with the Grid was ranging over Positions it shows nothing of.**
`Rect::intersect` answers a zero-area rectangle where two rectangles touch, and `Rect::contains` is
inclusive, so both corners landed inside the Grid and `cell_at` accepted them: a Grid at x in
[10, 810] with a clip whose left edge sat at x = 810 answered `columns: 30..32`, building the Shapes
for 64 entirely off-screen Positions. `visible_positions` now refuses an intersection with no area —
the same question the tests' `overlaps` helper asks of a single Cell, asked of the whole Grid — and
`a_clip_that_only_touches_the_grid_ranges_over_nothing` covers all four edges. This also subsumes
the inverted rectangle that `intersect` answers when the two do not meet.

**The draw loop indexes its slices rather than defaulting them.** `get(..).unwrap_or_default()`
turned a broken invariant into a row that silently went unpainted: in a release build, where
`source_dimensions`' rectangularity check is compiled out, a ragged Frame would have dropped a whole
row that the pre-culling loop drew in full. Both ranges are clamped to the dimensions read from the
same Frame, so the invariant is proven and indexing is what states it.

Also corrected: `show_source`'s doc still claimed every Cell is painted; `VisiblePositions`' doc
claimed to hold only the Positions the console shows, which the margin contradicts; and
`render-frame-derivation/spec.md` struck the Source clone from its list on local benchmark figures
without the caveat issue 01 attaches to them.
