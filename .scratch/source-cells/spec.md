# Source Cells

A refactor from the 2026-10-24 architecture review. No module owns how the Source View's coordinate frames relate, so the Overview, Diagnostics, `source_view` and the tests each convert between them by hand: `padded_source` is called 12 times and `SOURCE_MARGIN_CELLS` is read 29 times across `overview.rs`, `source_view.rs` and `panel.rs`, and the tests rebuild the same mappings in `unclipped_outline`, `overview_cells`, `drawn_cell`, `padded_size`, `padded_at`, `padded_point` and `source_view_on_overview`. This collapses five frames into two and puts every question asked of the presented Source View behind one value.

Nothing here changes what the console paints or how any Pan behaves, except the Diagnostics readout of the visible region (decision 8). No speedup is claimed.

## The frames

Today there are five: `SourceView.pan` (points, from the padded Source's corner relative to the console's top-left), `SourceView.origin` (the screen point of the Grid's corner), the visible region (`visible_source_region`, the console in Grid-corner points), `requested_centre` (padded Cells) and the Overview's texels (padded Cells scaled to the picture).

After this there are two: **screen points**, egui's, and **Source Cells** (see `CONTEXT.md`): fractional Cells from the Grid's top-left, whole at every Position. The padded Source is an extent in Source Cells, `[-2, columns + 2] × [-2, rows + 2]`, not a frame.

## Decisions

1. **Two frames.** The interface speaks screen points and Source Cells only. Padded points, Source points and padded Cells disappear. `pan` stays private state in points inside `SourceView`: wheel and drag deltas arrive in points, and the zoom and resize follow compares sizes in points.
2. **Source Cells is a `CONTEXT.md` term**, added beside Source View.
3. **`ShownSourceView` answers five things**: `screen()` (the Source Cells ↔ screen points map, exactly Paint's corner), `shown()` (the visible region in Source Cells, unclipped), `outline()` (clipped to the extent, `None` where nothing can Pan), `console()` and `extent()`. It is built from the `GridViewport` Paint drew, the console rect and the Grid. The Overview's pointer handling and painting stay in `overview`.
4. **`source_view` uses the same map internally.** `cursor_cell`, `followed_cell`, the requested centre and `clamp_pan`'s bound go through the map and the extent. `outline()` and the clamp share one private slack predicate, so an outline is shown exactly when the view can Pan. The clamp stays in points, and the order follow → requested centre → clamp is unchanged.
5. **One path to the value.** `show_source_scene` stores it on `SourceView` in place of `pub(super) origin`; `Console::ui` and the tests both read `source_view.shown()`; `show_source_panel` returns no geometry. It is not an `Option`: `SourceView::new` holds a degenerate value whose `outline()` is `None` until the first frame.
6. **The margin is private to `source_view`.** `panel::source_reserve` calls a `source_view` function for the height of one Row between its margins.
7. **`ShownSourceView` replaces `ShownSource`**, after the console's `Shown*` convention. `SourceView` keeps its name.
8. **The Diagnostics visible-region readout is in Source Cells**, read from the snapped corner. Today it reads Source points from the unsnapped `origin`, which is not the corner Paint draws from.
9. **Each test family keeps an independent oracle.** The outline, grab and marks families each keep one test whose expectation is worked by hand from texel counts; the rest may go through the map.
10. **Source Cells are newtypes with private fields.** A Source Cell point and rect convert to and from egui types only through a map, so a screen rect cannot be passed where Source Cells are meant.
11. **Two maps, two implementations.** The Source View's map is a multiply-add, `corner + cells × cell`, built from `GridViewport` and bit-exact with Paint, in `grid_viewport.rs` beside `presented_grid`. The Overview's is `emath::RectTransform::from_to(extent, picture)` wrapped to take and return Source Cells; it is not bit-exact and need not be. `GridViewport` and Paint are unchanged. The map is a value, not a seam, and has no adapter. Converting a screen point to Source Cells is for continuous geometry (the outline, the grab) and never chooses a Position: that stays with `GridViewport::cell_at` and `nearest_cell`.
12. **Out of scope:** delta conversions and Position helpers on the map, per-axis panning, a generic or sealed conversion trait, a map type private to the Overview, the Overview's colours (review candidate 2), the Overview as one call (review candidate 3), separating the Pan settle from egui input (review candidate 4), and amending ADR 0080's "docked" against the overlay fcc9ffce shipped.

## Constraints

- ADR 0038: the console owns the Source Grid transform and `presented_grid` is its one expression; the Source View's map is built from it.
- ADR 0045 and ADR 0047: one clamp bounds every Pan by the margin.
- ADR 0080: the Overview Pans through one pending request applied before the clamp, and never moves the Cursor.
- ADR 0022: `orcvs` gains no API.
- Native and WASM both build; nothing needs an `egui::Context` to build or query the value.
- No shipped function takes a parameter only a test populates.
- No new ADR: the change is internal to `console` and reversible, and its constraints are already recorded.

## Tickets

- `issues/01`: the Source Cells newtypes and the Source View map.
- `issues/02`: `ShownSourceView`, and `source_view` through it. Blocked by 01.
- `issues/03`: the Overview over the extent. Blocked by 02.
- `issues/04`: Diagnostics and the test helpers. Blocked by 02.
