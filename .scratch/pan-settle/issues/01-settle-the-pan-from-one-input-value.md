# 01 — Settle the Pan from one input value

**What to build:** `PanInput`, `Settled` and `SourceView::settle` in `console/src/console/source_view.rs`, per the spec's decisions 2–5. `show_source_scene` reads egui into a `PanInput`, calls `settle`, sets the cursor icon from `hand` and requests a repaint from `repaint`, then presents and handles the Region drag as it does now. The order gesture deltas → follow or edge scroll → requested centre → clamp is unchanged.

Tests that convert to `settle` tests, keeping their names and assertions: `a_fresh_source_view_does_not_pan_to_the_cursor_on_its_first_frame`, `a_cursor_move_that_would_leave_it_outside_the_view_pans_to_show_it`, `a_cursor_follow_to_an_edge_cell_shows_the_margin_past_it`, `a_pan_with_no_cursor_move_and_no_zoom_is_not_pulled_back_to_the_cursor`, `the_follow_pan_is_still_bounded_by_the_grids_edges`, `a_resize_that_would_open_a_gap_settles_the_source_view_back_inside`, `a_resize_that_would_leave_the_cursor_outside_the_view_pans_to_show_it`, `a_resize_after_a_pan_away_from_the_cursor_keeps_the_pan`, `a_zoom_that_would_open_a_gap_settles_the_source_view_back_inside`, `a_zoom_that_would_leave_the_cursor_outside_the_view_pans_to_show_it`, `releasing_a_drag_past_the_edge_scrolls_no_more_than_a_cell`. A listed test that turns out to assert more than the Pan stays a `console_pass` test, with the reason in this ticket's Comments.

Tests that stay as wiring: `wheel_pans_a_larger_source_to_its_edges_and_a_smaller_one_nowhere`, `a_middle_drag_pans_by_the_pointer_and_a_later_click_selects_the_cell_under_it`, `alt_held_with_a_primary_drag_pans_by_exactly_what_the_pointer_moved`, `a_drag_past_the_edge_scrolls_faster_the_further_out_and_stops_at_the_grid`, `a_browser_zoom_that_would_leave_the_cursor_outside_the_view_pans_to_show_it`, `a_new_cell_side_that_would_leave_the_cursor_outside_the_view_pans_to_show_it`, `a_far_edge_pan_leaves_no_gap_past_a_snapped_grid`, `a_cursor_follow_shows_the_whole_snapped_cursor_cell`, the Region and Paint tests, and the kittest Overview-centre and grab-hand tests.

**Blocked by:** cell-geometry/02

**Status:** resolved

- [x] `show_source_scene` reads egui into a `PanInput` and decides no Pan rule itself; `follow_cursor`, `overshoot` and `edge_scroll` are called only from `settle`.
- [x] `settle` tables pin the order: a follow and a requested centre on one frame settle at the centre; a requested centre past an edge settles at the clamp; Alt pressed during a Region drag edge-scrolls rather than Pans; a resize follows a Cursor shown in full and leaves one Panned partly out of view.
- [x] Edge scroll is tested for its pace and its one-Cell cap, and `repaint` is true exactly when the pointer is past the edge during a Region drag.
- [x] Every listed test keeps its name and what it asserts; every other existing test passes unchanged.
- [x] `cargo fmt --all -- --check`, `cargo clippy --package console --all-targets --locked -- -D warnings` and `cargo nextest run --package console --locked` pass, and `mise run check_wasm` passes.

## Comments

### Implemented, 2026-10-24

- **`Settled` answers the hand, not `can_pan`.** The spec's decisions 2 and 4 named `can_pan`, and now name `hand`. Which
  hand the pointer shows depends on whether a drag Panned and on Alt outside a Region drag, which are
  Pan rules; deciding it from `can_pan` in `show_source_scene` would restate them there. So `settle`
  answers `hand: Option<CursorIcon>` from `can_pan` and those rules, and `show_source_scene` only sets
  it, as decision 4 keeps the icon there.
- **The `settle` tests live in `source_view.rs`'s own `tests` module**, not `console/src/console/tests.rs`.
  A Region drag in progress is the private `region_drag` field, which `show_source_scene` sets from the
  painted `GridViewport`; only a test inside the module can set it without a seam in shipped code.
- **Eight of the eleven listed tests converted.** Three stay `console_pass` tests. The two zoom tests,
  `a_zoom_that_would_open_a_gap_settles_the_source_view_back_inside` and
  `a_zoom_that_would_leave_the_cursor_outside_the_view_pans_to_show_it`, also assert egui's zoom factor
  and the Cell the pass presented. `releasing_a_drag_past_the_edge_scrolls_no_more_than_a_cell` guards
  the release frame itself, which `show_source_scene` answers by spanning no further while `settle`
  still paces it; only an egui pass plays that frame. Its `settle` half is
  `the_frames_after_a_paced_cursor_move_do_not_follow_it_in_full`.
- **`Response::mark_changed` is no longer called on the Pan response.** It only sets a flag on the
  `Response` value `allocate_exact_size` returned, and nothing reads that value's `changed()`; egui does
  not read it back after the frame.
- `ShownSourceView::outline` and `settle`'s Pan offer share one predicate, `pannable`, over the same
  `slack` the clamp uses.
