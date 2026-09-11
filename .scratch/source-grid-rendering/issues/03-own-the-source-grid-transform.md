# 03 — Own the Source Grid transform and retire the Scene

**What to build:** Replace `egui::Scene` with a scale and translation the console holds, applied
when Cell rectangles are computed, with Glyphs laid out at the zoom they are drawn at.

**Blocked by:** 02 — Paint the Source Grid instead of building a Cell button field.

**Status:** ready-for-agent

- [ ] The `Scene` container is gone from `show_source_scene` and the console holds the `TSTransform`
      that replaces it. `SourceView` carries it in place of the Scene-space `rect` it holds now, or
      records why it still holds a rect.
- [ ] Zoom, scroll-pan and the `zoom_range` clamp are kept, not reimplemented.
      `Scene::register_pan_and_zoom` is public, takes a `&mut TSTransform` the caller owns, and does
      not set a layer transform. Those three parts are layer-independent and survive standalone.
- [ ] The helper's drag-pan branch is disabled and replaced. It computes
      `to_global.translation += to_global.scaling * resp.drag_delta()` (`scene.rs:239`), and
      `Response::drag_delta` divides by the layer transform's scaling when one exists
      (`response.rs:455-465`). Inside `Scene::show` the two cancel and panning is 1:1 with the
      pointer; with no layer transform `layer_transform_from_global` returns `None`, nothing divides,
      and the multiply over-pans by the zoom factor. Configure `DragPanButtons::empty()` so the branch
      is dead and apply `translation += resp.drag_delta()` directly.
- [ ] A test covers panning at a scale other than 1.0. At the default fit the scale is exactly 1.0
      and the bug above is invisible, so a test at the default window size proves nothing here.
- [ ] The Source Grid layer is no longer transformed, so no `TextShape` in it reaches
      `Arc::make_mut`. This is the acceptance criterion the whole effort exists for. Confirm it by
      asserting the Source Grid's `LayerId` has no entry among the context's layer transforms:
      `make_mut` is unconditional given a transform, so the absence of the transform is the whole
      proof. Do not try to observe it in a profile — the clone happens in `GraphicLayers::drain`
      inside `end_pass`, not in `tessellate_shapes`, which is why a 705-star implementation has been
      paying it unnoticed.
- [ ] `Scene::show` also calls `set_sublayer` and `force_set_min_rect` around the transform; account
      for both when the container goes, rather than only the transform.
- [ ] Glyph `FontId` size is derived from the current scale so text is laid out at the size it is
      drawn at rather than resampled from the atlas. The scale that reaches the `FontId` is quantised
      so the galley cache hits across frames at a steady zoom, and the quantisation step is stated.
- [ ] The quantisation step is justified as an atlas budget, not only as a cache-hit rate. Every
      distinct size rasterises a fresh glyph set into the font atlas, and `subpixel_binning` is on by
      default, so a zoom sweep across N steps costs up to N x alphabet x 4 rasters. That is how
      `atlas.fill_ratio()` is driven past 0.8 and the whole `FontsImpl` is recreated
      (`epaint/src/text/fonts.rs:734-748`), which restarts glyph rasterisation mid-session. State the
      budget the chosen step implies.
- [ ] Cell geometry is snapped to whole physical pixels once the Cell size is derived from the zoom.
      An unsnapped fractional Cell size pixel-snaps differently row by row, so some rows are a pixel
      taller than others and the eye reads it as irregular spacing.
- [ ] The glyph table from issue 02 is built at the zoom-derived `FontId` size. Because that table
      is rebuilt every Render Frame, a `pixels_per_point` change needs no explicit invalidation —
      epaint's galley cache keys on `pixels_per_point` and re-lays out on its own. Do not reintroduce
      a retained table here to avoid the per-frame rebuild; issue 02 records why.
- [ ] Pan, zoom and their limits behave as they do today: middle-button drag pans, the zoom range is
      `MIN_ZOOM..=MAX_ZOOM` widened to admit the fitted scale, double click hands the view back to
      the fit, and any pan or zoom pins it. `console.rs:995`, `:1020` and `:1039` keep their
      semantics with their assertions restated against the owned transform — they read `view.rect`
      today, and the first acceptance line above removes it, so they cannot pass untouched.
- [ ] The fit is rebuilt rather than borrowed. `fit_to_rect_in_scene` is a private `fn`
      (`scene.rs:15`) and cannot be called. `grid_viewport` already holds the numbers.
- [ ] A degenerate transform is guarded. `Scene::show` resets a transform that is not valid
      (`scene.rs:151-152, 168-173`) and nothing replaces that. `grid_viewport` answers
      `cell_size == 0.0` for a zero-area console (`grid_viewport.rs:63-70`), and a `TSTransform` with
      `scaling == 0.0` makes `inverse()` divide by zero, so every click resolves to NaN. The console
      guards the transform, not just the zoom range as it does today at `console.rs:370-376`.
- [ ] `scene_zoom` (`console.rs:211-215`) and its test
      `diagnostics_derive_frame_rate_and_scene_zoom_from_view_state` (`console.rs:621`) follow. Under
      an owned transform the zoom is `transform.scaling` and that function collapses to a field read.
- [ ] Letterboxing and the square-Cell fit are unchanged, and `the_grid_fills_the_centred_viewport_and_the_letterboxing_holds_no_cell`
      (`console.rs:814`) and `the_default_window_presents_the_default_grid_at_its_own_scale`
      (`console.rs:875`) pass unchanged.
- [ ] Clicks still resolve to the same Position at every zoom and pan the click tests exercise,
      including the letterboxed and below-the-zoom-floor cases.
- [ ] The doc comment at `console.rs:346-352` no longer says the Scene is the one place the Source is
      scaled. Whatever replaces it is now that one place, and says so.
- [ ] `grid_viewport.rs` remains the single expression of the geometry. If the transform's arithmetic
      belongs there rather than in `console.rs`, move it there.

## Comments

Text sharpness at zoom is the visible change, and it is an improvement rather than a parity break:
today's glyphs are bilinearly resampled at any zoom away from 1.0. Say so in the change, because a
reviewer comparing captures will see a difference and the effort's spec claims strict parity.
Parity is claimed over the palette and the geometry, not over resampling artefacts.

The price is smaller than it first looks. `register_pan_and_zoom` keeps the pan and zoom input
handling; what actually becomes the console's is applying the transform when Cell rectangles are
computed. The fit-to-viewport reset was already console code. The ADR from issue 01 should say this
rather than overstating the cost.

Keep the two-widget split issue 02 establishes rather than collapsing to one `click_and_drag`
rectangle. One full-coverage drag-sensing rectangle would make a double click on a Cell also reset
the view, which is a silent behaviour change — `console.rs:1039` double-clicks the letterbox and
would still pass. It would also require an explicit inside-the-Grid guard that the Cell widgets
provide for free today, which
`the_grid_fills_the_centred_viewport_and_the_letterboxing_holds_no_cell` (`console.rs:814`) depends
on. A full-area `click_and_drag` response allocated first, plus a Grid-sized `click` response from
`Ui::interact` second, reproduces today's resolution exactly by registration order.

`peters/horizon` is an `egui` 0.36.1 terminal grid painting in screen coordinates with no per-cell
widgets. Read `crates/horizon-ui/src/terminal_widget/render.rs` before starting.
