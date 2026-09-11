# 05 — Cull the Source Grid to the visible Position range

**What to build:** Iterate only the Positions the console is showing, so drawing cost follows the
viewport rather than the Source.

**Blocked by:** None. The geometry it rests on is settled; only its justification looks forward.

**Status:** ready-for-agent

- [ ] The Cell loop derives a Position range from the visible rectangle and the current transform,
      and iterates that range instead of walking every row of `frame.rows()` in full. The one
      remaining full walk is `source_dimensions` (`console/src/console.rs:104`), which reads the
      column count and is not the draw loop; leave it.
- [ ] The range is expanded by one Cell in every direction beyond what the viewport clips, and the
      clip rect discards the surplus. Culling to exactly the visible range is not enough, and the
      reason is ownership rather than tolerance: a Cell's left and top seam lines are drawn by the
      Cell one past the boundary (`console/src/console.rs:688-699`), so an exact range drops the
      trailing lines at the right and bottom edges. A Cell's border also spills roughly one physical
      pixel outside its own rectangle. The margin additionally gives the run logic below the
      neighbour state it needs.
- [ ] Background run coalescing still breaks at the same places. Runs are row-local, opened and
      flushed within one row (`console/src/console.rs:622`, `:730-734`), and a range-limited row must
      open its run at the first drawn Cell rather than carrying one in from off-screen. The failure
      this guards against is a one-Cell seam at the left edge of the viewport, visible only when
      zoomed in, which is exactly where no current test looks.
- [ ] Run endpoints still come from `GridViewport::cell_rect`, not from an accumulated sum, so a
      coalesced edge stays bit-identical under culling.
      `consecutive_cells_sharing_a_background_are_one_rectangle` keeps passing.
- [ ] Clicks inside the visible range select what they select today. Clicks outside it are already
      refused: the interaction rectangle is `grid.rect.intersect(clip)`
      (`console/src/console.rs:581-585`), so a click outside the console cannot reach the Grid. Check
      this holds rather than adding a second guard.
- [ ] Sector seam lines and the Cursor bloom are correct at the edges of the visible range,
      including when the Cursor is outside it.
- [ ] The visible range is a shipped pure function in `grid_viewport.rs`, alongside `grid_viewport`,
      `presented_grid` and `cell_at`, and is the draw loop's only source of Positions. Everything it
      needs is already `cell_at`'s inputs — `self.rect`, `self.cell_size`, `columns`, `rows` and the
      clip rectangle `show_source` already receives (`console/src/console.rs:560`) — so it is
      `cell_at` on the clip's two corners, plus the margin and a clamp. A test asserts on its real
      return value and derives the count from it. That is not a test-only seam: one shipped caller, a
      real return value, no branch a test alone reaches.
- [ ] A second test proves the draw loop honours the range, by comparing the shapes in `FullOutput`
      between a fully visible Grid and a zoom showing a fraction of it. Coalescing means the shape
      count is not proportional to the Position count, so assert a bound rather than an equality.
- [ ] Neither test introduces a counter the console increments for a test's benefit. If the range
      cannot be extracted as a pure function without distorting the module, say so here and drop the
      acceptance line rather than cutting the seam.
- [ ] The change claims structure, not speed: cost follows the viewport rather than the Source. No
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
