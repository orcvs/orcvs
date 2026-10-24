# The Overview as one call

A refactor from the 2026-10-24 architecture review (candidate 3). `Console::ui` drives the Overview through three calls in a fixed order: `show_overview` before the Source panel, because it reads the menu bar's bottom from `root.available_rect_before_wrap()`, then `ShownOverview::outline_source_view` and `Overview::pan_source_view` after it, each passed the same geometry. Placement and fit, about fifteen lines inside `show_overview`, are tested only through whole console passes. This makes the Overview one call after the Source is presented, answering at most a Pan request, and puts placement and fit behind a pure function.

Nothing here changes what the Overview paints, where it sits, or how a press or drag on it Pans. No speedup is claimed.

## Decisions

1. **Blocked by cell-geometry 03.** That ticket moves the outline and the grab onto `ShownSourceView`; this orders calls that already take it.
2. **One call, after the Source panel.** `Console::ui` reads the menu bar's bottom once, right after `show_menu_bar`, and after `show_source_panel` calls `show_overview(root, below_menu, revision, &frame, appearance)`. It reads `self.source_view.shown()`, draws the picture, the marks and the outline inside its own `Area` closure, and answers `Option<CellPoint>`, the centre a press or drag asks for. `Console::ui` passes it to `SourceView::centre_on`; the Overview does not mutate `SourceView`. `outline_source_view`, `pan_source_view` and `ShownOverview` stop being part of the module's interface.
3. **Showing the Overview after the Source changes nothing a viewer sees.** The Overview is an egui `Area`, a Middle-order layer above the central panel, and egui hit-tests against the previous pass's layer rects, so the call order changes neither which layer takes the pointer nor what paints over what. The existing pinned-corner test and the Source's middle-drag tests are the check.
4. **Placement and fit are a pure function**, private to `overview`: `place_picture(window, below_menu, border, grid, pixels_per_point) -> Option<Rect>`, answering the picture's rect at the window's bottom-right corner at whole physical pixels per texel, or `None` where one pixel per texel does not fit.
5. **Tests:** `place_picture` tables cover whole pixels per texel, shrinking to fit, hiding below one pixel per texel, a fractional scale and the rounded corner; `the_overview_shrinks_to_whole_pixels_per_texel_that_fit_and_hides_below_one` becomes one. One new test drives the answered centre: a press on the picture answers `Some` and leaves `SourceView` untouched until `Console::ui` applies it. The outline, marks, grab and drag tests stay as wiring.
6. **The overlay gets its own ADR.** ADR 0080 says the Overview is docked and rejects a floating overlay; the Overview has floated over the window's bottom-right corner since it was pinned there, because a docked panel took 260 points or more of width from the Source. ADR 0081 records that and amends ADR 0080; `overview.rs`'s module doc stops saying "docked". It does not wait on the refactor.
7. **Out of scope:** settling the Pan (`.scratch/pan-settle/`), the Overview's colours, and any change to where the Overview sits or how it fits.

## Constraints

- ADR 0080: the Overview Pans through one pending request applied before the clamp, and never moves the Cursor or changes a Region.
- ADR 0045 and ADR 0047: that request is bounded by the same clamp and margin as every Pan.
- ADR 0040: the picture's colours are still decided as a value from a Render Frame and a Theme.
- Native and WASM both build; `place_picture` needs no `egui::Context`.
- No shipped function takes a parameter only a test populates.

## Tickets

- `issues/01`: show the Overview in one call. Blocked by cell-geometry/03.
- `issues/02`: record that the Overview floats over the window's corner.
