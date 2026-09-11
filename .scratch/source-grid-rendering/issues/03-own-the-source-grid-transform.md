# 03 — Own the Source Grid transform and retire the Scene

**What to build:** Replace `egui::Scene` with a scale and translation the console holds, applied
when Cell rectangles are computed, with Glyphs laid out at the zoom they are drawn at.

**Blocked by:** 02 — Paint the Source Grid instead of building a Cell button field.

**Status:** resolved

- [x] The `Scene` container is gone from `show_source_scene` and the console holds the `TSTransform`
      that replaces it. `SourceView` carries it in place of the Scene-space `rect` it holds now, or
      records why it still holds a rect.
- [x] Zoom, scroll-pan and the `zoom_range` clamp are kept, not reimplemented.
      `Scene::register_pan_and_zoom` is public, takes a `&mut TSTransform` the caller owns, and does
      not set a layer transform. Those three parts are layer-independent and survive standalone.
- [x] The helper's drag-pan branch is disabled and replaced. It computes
      `to_global.translation += to_global.scaling * resp.drag_delta()` (`scene.rs:239`), and
      `Response::drag_delta` divides by the layer transform's scaling when one exists
      (`response.rs:455-465`). Inside `Scene::show` the two cancel and panning is 1:1 with the
      pointer; with no layer transform `layer_transform_from_global` returns `None`, nothing divides,
      and the multiply over-pans by the zoom factor. Configure `DragPanButtons::empty()` so the branch
      is dead and apply `translation += resp.drag_delta()` directly.
- [x] A test covers panning at a scale other than 1.0. At the default fit the scale is exactly 1.0
      and the bug above is invisible, so a test at the default window size proves nothing here.
- [x] The Source Grid layer is no longer transformed, so no `TextShape` in it reaches
      `Arc::make_mut`. This is the acceptance criterion the whole effort exists for. Confirm it by
      asserting the Source Grid's `LayerId` has no entry among the context's layer transforms:
      `make_mut` is unconditional given a transform, so the absence of the transform is the whole
      proof. Do not try to observe it in a profile — the clone happens in `GraphicLayers::drain`
      inside `end_pass`, not in `tessellate_shapes`, which is why a 705-star implementation has been
      paying it unnoticed.
- [x] `Scene::show` also calls `set_sublayer` and `force_set_min_rect` around the transform; account
      for both when the container goes, rather than only the transform.
- [x] Glyph `FontId` size is derived from the current scale so text is laid out at the size it is
      drawn at rather than resampled from the atlas. The scale that reaches the `FontId` is quantised
      so the galley cache hits across frames at a steady zoom, and the quantisation step is stated.
- [x] The quantisation step is justified as an atlas budget, not only as a cache-hit rate. Every
      distinct size rasterises a fresh glyph set into the font atlas, and `subpixel_binning` is on by
      default, so a zoom sweep across N steps costs up to N x alphabet x 4 rasters. That is how
      `atlas.fill_ratio()` is driven past 0.8 and the whole `FontsImpl` is recreated
      (`epaint/src/text/fonts.rs:734-748`), which restarts glyph rasterisation mid-session. State the
      budget the chosen step implies.
- [x] Cell geometry is snapped to whole physical pixels once the Cell size is derived from the zoom.
      An unsnapped fractional Cell size pixel-snaps differently row by row, so some rows are a pixel
      taller than others and the eye reads it as irregular spacing.
- [x] The glyph table from issue 02 is built at the zoom-derived `FontId` size. Because that table
      is rebuilt every Render Frame, a `pixels_per_point` change needs no explicit invalidation —
      epaint's galley cache keys on `pixels_per_point` and re-lays out on its own. Do not reintroduce
      a retained table here to avoid the per-frame rebuild; issue 02 records why.
- [x] Pan, zoom and their limits behave as they do today: middle-button drag pans, the zoom range is
      `MIN_ZOOM..=MAX_ZOOM` widened to admit the fitted scale, double click hands the view back to
      the fit, and any pan or zoom pins it. `console.rs:995`, `:1020` and `:1039` keep their
      semantics with their assertions restated against the owned transform — they read `view.rect`
      today, and the first acceptance line above removes it, so they cannot pass untouched.
- [x] The fit is rebuilt rather than borrowed. `fit_to_rect_in_scene` is a private `fn`
      (`scene.rs:15`) and cannot be called. `grid_viewport` already holds the numbers.
- [x] A degenerate transform is guarded. `Scene::show` resets a transform that is not valid
      (`scene.rs:151-152, 168-173`) and nothing replaces that. `grid_viewport` answers
      `cell_size == 0.0` for a zero-area console (`grid_viewport.rs:63-70`), and a `TSTransform` with
      `scaling == 0.0` makes `inverse()` divide by zero, so every click resolves to NaN. The console
      guards the transform, not just the zoom range as it does today at `console.rs:370-376`.
- [x] `scene_zoom` (`console.rs:211-215`) and its test
      `diagnostics_derive_frame_rate_and_scene_zoom_from_view_state` (`console.rs:621`) follow. Under
      an owned transform the zoom is `transform.scaling` and that function collapses to a field read.
- [x] Letterboxing and the square-Cell fit are unchanged, and `the_grid_fills_the_centred_viewport_and_the_letterboxing_holds_no_cell`
      (`console.rs:814`) and `the_default_window_presents_the_default_grid_at_its_own_scale`
      (`console.rs:875`) pass unchanged.
- [x] Clicks still resolve to the same Position at every zoom and pan the click tests exercise,
      including the letterboxed and below-the-zoom-floor cases.
- [x] The doc comment at `console.rs:346-352` no longer says the Scene is the one place the Source is
      scaled. Whatever replaces it is now that one place, and says so.
- [x] `grid_viewport.rs` remains the single expression of the geometry. If the transform's arithmetic
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

---

Resolved. `egui::Scene` is no longer a container here. `SourceView` carries a `TSTransform` — the
scale and translation the console owns — and `grid_viewport::presented_grid` applies it to produce
the `GridViewport` every Cell rectangle, every Glyph position and every click resolves through.
`Scene` survives only as the configuration object `register_pan_and_zoom` hangs off: it is built
with the zoom range and `DragPanButtons::empty()`, and `.show()` is never called.

Six things a later reader needs.

**The over-pan is real, and the test proves it rather than assuming it.** With the branch left
enabled and the console's own `translation += drag_delta()` removed,
`a_middle_drag_pans_by_the_pointer_and_not_by_the_pointer_times_the_zoom` fails with
`[80.0 48.0]` against `[40.0 24.0]` — exactly the zoom factor of two the console is sized to fit at.
The same check was run against the layer-transform assertion: adding one `set_transform_layer` call
back into `show_source_scene` fails `no_layer_carrying_the_source_grid_is_transformed`. Neither test
passes vacuously.

**Strokes take the scale explicitly now.** The Scene's layer transform scaled the Grid line and the
sector seam along with everything else. Painting in presented points does not, so
`GRID_LINE_WIDTH` and `SECTOR_LINE_WIDTH` are multiplied by `grid.cell_size / CELL_SIZE` at the
point of use. That is why `a_cell_border_is_one_grid_line_wide_whatever_the_cell_is_doing` passes
with its assertion untouched, which is the parity signal worth having.

**The Cell snap floors rather than rounds, and the Grid is re-centred afterwards.** Rounding to the
nearest pixel lets a Grid of `columns` Cells exceed the fit by half a pixel per column — twenty
points at the default forty — and the surplus is *clipped*, not letterboxed. Flooring spends the
same error where the letterboxing already is. Re-centring is load-bearing rather than cosmetic: a
Grid anchored at its snapped corner leaves a zero-width letterbox on one side, and
`the_grid_fills_the_centred_viewport_and_the_letterboxing_holds_no_cell` clicks the midpoint of
that letterbox at its 300x300 shape.

**`show_source_scene` returns the *presented* Grid, not the fit.** It used to return
`grid_viewport(..)`'s answer, which under a Scene was also what was drawn. Under an owned transform
the drawn Grid is the pinned or zoomed one, and the click tests read that return value to find a
Cell, so returning the fit would make every test that pans or zooms assert against a Grid that was
never painted.

**The interaction rectangle is clipped to the console explicitly.** `Ui::interact` bounds a widget
by the `Ui`'s clip rect, not by the console area, and `Scene::show` used to set that clip rect
itself (`scene.rs:209`). A zoomed-in Grid reaches past the console on every side and shares the
background layer with the menu bar above it, where a later-registered click widget wins the tie. The
Grid states its own bound instead.

**The quantisation step is an eighth and the budget is stated in the code.** Fifteen distinct scales
across `MIN_ZOOM..=MAX_ZOOM`, so at most `15 x 94 x 4` rasters for a full sweep — roughly two
megapixels of a 2048-square atlas at one device pixel per point, inside the 0.8 fill ratio that
would otherwise recreate the whole `FontsImpl`. At two device pixels per point a full sweep would
pass it; the honest statement is that the step bounds the sweep rather than eliminating it. Both
zoom limits and the Source's own scale are exact multiples of the step, so the default window lays
its Glyphs out at exactly `DEFAULT_FONT_SIZE`.

`scene_zoom` is gone: under an owned transform the diagnostics read `to_global.scaling`, and the
visible Source region is `to_global.inverse() * console`. Its test kept the frame-rate half and
restated the zoom half against the transform and the degenerate-transform guard that makes reading
the field safe.
