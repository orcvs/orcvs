# 01 — Show the Overview in one call

**What to build:** Per the spec's decisions 2–5, in `console/src/console/overview.rs` and `console/src/console.rs`. `Console::ui` reads the menu bar's bottom after `show_menu_bar` and calls `show_overview` once, after `show_source_panel`; `show_overview` answers `Option<CellPoint>` and `Console::ui` passes it to `SourceView::centre_on`. Outlining, the grab and the marks happen inside the `Area` closure, reading `self.source_view.shown()`. Placement and fit move into a private `place_picture(window, below_menu, border, grid, pixels_per_point) -> Option<Rect>`.

**Blocked by:** cell-geometry/03

**Status:** ready-for-agent

- [ ] `Console::ui` makes one Overview call, after the Source panel; `outline_source_view`, `pan_source_view` and `ShownOverview` are private to `overview` or gone.
- [ ] `show_overview` does not take or mutate `SourceView`; a test presses the picture and sees `Some` answered with `SourceView`'s Pan unchanged until `centre_on` is applied.
- [ ] `place_picture` has table tests for whole pixels per texel, shrinking to fit, hiding below one, a fractional scale and the rounded corner; the shrink-and-hide console-pass test is one of them, keeping its name and what it asserts.
- [ ] `the_overview_starts_pinned_over_the_bottom_right_corner_and_the_view_menu_toggles_it`, the Source's middle-drag tests and every other Overview test pass unchanged in what they assert.
- [ ] `cargo fmt --all -- --check`, `cargo clippy --package console --all-targets --locked -- -D warnings` and `cargo nextest run --package console --locked` pass, and `mise run check_wasm` passes.
