# 05 — Cover the Source Grid viewport geometry

**What to build:** Unit tests over the square, centred viewport geometry that
`restyle-egui-console/01` introduces, so the geometry decisions are checkable without capturing an
image.

**Blocked by:** 01 — Rename the shell crate to console; restyle-egui-console/01 — Square, centred Source Grid viewport.

**Status:** wontfix

- [ ] The geometry in `grid_viewport.rs` is reachable as pure functions over sizes and rectangles,
      in the way `source_bounds`, `top_right_source_view` and `scene_zoom` already are.
- [ ] Cell geometry is square at every viewport the tests exercise.
- [ ] A wide viewport centres the Source Grid and letterboxes horizontally, and the letterbox extents
      are asserted rather than described.
- [ ] A tall viewport centres it and letterboxes vertically.
- [ ] The Scene reaches a fit below the viewer's zoom floor, which is the second commit on
      `restyle-egui-console/01`.
- [ ] Degenerate viewports are covered: zero-sized, and one large enough for a single Cell.
- [ ] If the geometry cannot be expressed as pure functions without distorting the module, say so in
      this issue's comments rather than working around it. That finding is the trigger this effort's
      `spec.md` names for reconsidering Reference Renders.

## Comments

Status is `needs-triage` rather than `ready-for-agent` because `grid_viewport.rs` does not exist on
`main` yet. It is two commits ahead in `.worktrees/01-square-centred-source-grid`, adding 225 lines of
new module and 470 lines to the console. The acceptance lines above are written against what that
work is for, not against its current shape, and should be re-read against the landed module before
an agent starts.

This is the one part of the console where the case for image comparison is genuinely strong, and it
is strong only if the geometry turns out not to be unit-testable. Settle that here before spending
the wgpu dependency tree.

### Audit at cad296df — 2026-09-29

Wontfix: the premise was rejected by design. ADR 0045 refused the fit and re-centring, and ADR 0047
lists centring a smaller Grid among its rejected alternatives; the Grid rests at the top-left.
`console/src/grid_viewport.rs` has landed on `main`, and its geometry is already pure functions with
unit tests, including the degenerate case `a_viewport_with_no_area_answers_no_cell` (`:814`).
`scene_zoom` and `top_right_source_view` no longer exist, and the worktree the comment names is gone.
