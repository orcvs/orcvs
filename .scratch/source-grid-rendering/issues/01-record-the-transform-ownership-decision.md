# 01 — Record the transform ownership decision as an ADR

**What to build:** An ADR stating that the console owns the Source Grid's pan and zoom transform
rather than delegating it to `egui::Scene`, and why.

**Blocked by:** console-testing/01 — Rename the shell crate to console.

**Status:** resolved

- [x] `docs/adr/0038-the-console-owns-the-source-grid-transform.md` exists. Note `0036` is already
      used twice in `docs/adr/`; `0038` is the next unused number, not the next after the highest file.
- [x] It states the decision: the console holds its own scale and translation, applies it when it
      computes Cell rectangles, and derives the `FontId` size from it so Glyphs are laid out at the
      zoom they are drawn at.
- [x] It records the mechanism that forces the choice, with citations: a transformed layer reaches
      every `TextShape` through `Arc::make_mut` (`epaint/src/shapes/text_shape.rs`), epaint's
      `GalleyCache` holds an `Arc` to every galley it hands out, so the clone is unconditional and
      per-Cell per-Render-Frame.
- [x] It records the two consequences that are not about cost: bilinear resampling of glyph quads
      against a fixed-resolution atlas, and `Glyph::pos` left untransformed by an acknowledged
      upstream `TODO`.
- [x] It records the alternative that was live and rejected: keep `Scene` and emit Glyph quads into
      a hand-built `Mesh` whose refcount is one, copying `RowVisuals::mesh` vertices and normalising
      their UVs by `font_image_size()` — which removes the clone but not the resampling, and buys
      that with a dependency on epaint's texel-space UV convention. No egui terminal renderer surveyed
      does this; every one goes through `Shape::text` or `Shape::galley`.
- [x] It says what this costs, accurately and no more. `Scene::register_pan_and_zoom`
      (`egui-0.36.1/src/containers/scene.rs:229`) is public, takes a `&mut TSTransform` the caller
      owns, and does the drag-pan, zoom-at-pointer, `zoom_range` clamping and `mark_changed` without
      setting a layer transform. So this decision does not give up pan and zoom input handling. It
      gives up the layer transform alone, and takes on applying the transform to Cell rectangles.
      The fit-to-viewport reset is already console code (`console.rs:389-393`).
- [x] It records `ctx.set_zoom_factor` as the second live alternative. `landaire/hxy` zooms this way
      and gets natively-laid-out text at every zoom with no transform to own, no `FontId` size to
      derive, no quantisation and no atlas pressure. Its cost is that it zooms the whole console,
      menu bar included, rather than the Source Grid alone. That is the trade-off to state.
- [x] It states the survey honestly rather than claiming a precedent. No surveyed egui terminal
      renderer leaves the transformed layer. `peters/horizon` is on this exact pin and does call
      `ctx.set_transform_layer` (`app/panels.rs:604-614`, `app/view.rs:189-193`), paying the galley
      clone without having noticed — its profiling spans cover shape building and `tessellate_shapes`,
      and the clone happens in `GraphicLayers::drain` inside `end_pass`. An unnoticed cost in a
      705-star implementation is a better argument for writing this down than a precedent would be.
- [x] It records that per-Cell glyph layout is a deliberate divergence from `horizon`, which batches
      contiguous same-foreground Cells into one string using a ligature-bearing font and therefore
      shifts columns on `->`, `==` and `!=`. Without that note a later reader will "optimise" the
      per-Cell loop back into runs.
- [x] It does not reopen ADR 0005. The Grid stays fixed; this decision only makes an infinite one
      reachable later.
- [ ] The doc comment on `show_source_scene` (`console.rs:346-352`), which currently argues that the
      Scene is the one place the Source is scaled, is updated or removed by issue 03 rather than
      left contradicting the ADR. **Not this issue's work** — issue 03 owns that edit and carries
      the same acceptance line; left unticked deliberately. The ADR was written so it does not
      depend on that comment changing first.

## Comments

This clears the ADR bar on all three counts. It is hard to reverse — pan, zoom and hit-testing all
move with it. It is surprising without context, because delegating pan and zoom to the toolkit
container built for exactly that is the obvious choice and the reason not to is buried in
`Arc::make_mut` semantics three crates down. And it is a real trade-off: `Scene` genuinely works, and
the alternative above is genuinely available.

Written first so issues 02 and 03 implement a recorded decision rather than the ADR describing what
was already built.
