# 05 — Cull the Source Grid to the visible Position range

**What to build:** Iterate only the Positions the console is showing, so drawing cost follows the
viewport rather than the Source.

**Blocked by:** 03 — Own the Source Grid transform and retire the Scene.

**Status:** needs-triage

- [ ] The Cell loop derives a Position range from the visible rectangle and the current transform,
      and iterates that range. `frame.rows()` is no longer walked in full.
- [ ] The range is expanded by one Cell in every direction beyond what the viewport clips, and the
      clip rect discards the surplus. Culling to exactly the visible range is not enough: a Cell's
      border spills roughly one physical pixel outside its own rectangle, and the Grid line and
      sector seam at a Cell's right and bottom boundary are owned by the Cell one past it, so the
      trailing line at the right and bottom edges would be dropped. The margin also gives issue 04's
      run logic the neighbour state it needs to decide where a run breaks.
- [ ] Clicks outside the visible range select nothing, and clicks inside it select what they select
      today.
- [ ] Sector seam lines and the Cursor bloom are correct at the edges of the visible range, including
      when the Cursor is outside it.
- [ ] The visible range is a shipped pure function in `grid_viewport.rs`, alongside `grid_viewport`
      and `scene_view`, and is the draw loop's only source of Positions. A test asserts on its real
      return value and derives the count from it. That is not a test-only seam: one shipped caller, a
      real return value, no branch a test alone reaches. The module's own doc comment already claims
      this shape — the geometry lives there so a test can ask about it without a window.
- [ ] A second test proves the draw loop honours the range, by comparing the shapes in
      `FullOutput` between a fully visible Grid and a zoom showing a fraction of it. With issue 04's
      coalescing the shape count is not proportional to the Position count, so assert a bound rather
      than an equality.
- [ ] Neither test introduces a counter the console increments for a test's benefit. If the range
      cannot be extracted as a pure function without distorting the module, say so here and drop the
      acceptance line rather than cutting the seam.

## Comments

`needs-triage` rather than `ready-for-agent`, but less marginal than an earlier draft of this issue
claimed. At the default 40x25 Grid this changes nothing a user can see. Its long-run value is that
`ADR 0005` defers Source addressing for an infinite canvas, and a renderer whose cost scales with
the total Cell count cannot serve one. Its near-term value is that every surveyed painter-path
implementation culls, including a hex editor at far smaller per-Cell cost than this console, and the
one project that culls nothing does so because it draws on the GPU in a single call — the same
project culls strictly on the one screen where it does use the painter per Cell.

Be accurate about what culling buys. epaint already discards off-screen shapes at tessellation
(`coarse_tessellation_culling`, on by default), so this saves Cell iteration and shape construction,
not tessellation of shapes nobody sees. With issue 02's per-shape costs that is still the majority of
the work.

Everything the draw loop reads is already local to its Cell: `cursor_bloom` is computed per Cell in
`RenderFrame::derive` and stored on the `RenderCell`, and the sector strengths depend only on the
Cell's own Position, spacing and hash. So a Cursor outside the visible range still shows its bloom on
visible Cells with no special handling — provided culling is applied to the draw loop over
`frame.rows()` and not to `RenderFrame::derive`. If a later effort culls derivation, that margin
becomes load-bearing rather than cosmetic.
