# 04 — Read Diagnostics and the tests through the shown Source View

**What to build:** `show_diagnostics` in `console/src/console/diagnostics_panel.rs` takes the `ShownSourceView` in place of `(origin, console, cell_size)`. Its visible-region readout shows `shown()` in Cells, from the snapped corner, and its Cell readout reads the map's Cell size. `visible_source_region` is deleted.

The test helpers that rebuild the Source View's geometry ask the shown Source View instead: `drawn_cell`, `padded_size`, `padded_at`, `padded_point`, `source_view_centre`, `source_view_on_overview` and `presented_source` in `kittest_tests.rs`, and `unclipped_outline`, `overview_cells` and the margin constant in `tests.rs`. Tests that read `source_view.origin` read `source_view.shown()`.

**Blocked by:** 02

**Status:** resolved

- [x] `visible_source_region` is gone, and the readout's tests expect Cells.
- [x] No test helper restates the margin or the padded Source; each one left is a call on `ShownSourceView` or the map, or is deleted.
- [x] No test reads `SOURCE_MARGIN_CELLS`.
- [x] The independent oracles 03 keeps stay independent.
- [x] `cargo fmt --all -- --check`, `cargo clippy --package console --all-targets --locked -- -D warnings` and `cargo nextest run --package console --locked` pass.
