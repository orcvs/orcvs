# 02: Remove the Source View's own Zoom

**What to build:** Delete the Source View's own Zoom now that egui's zoom is the only one: its state, the zoom command recogniser, the stepped-zoom function, the zoom limits and the glyph scale quantised from them. The Source scene transform keeps its translation (Pan) and loses its independent scaling. Nothing a viewer sees changes, and no shipped path is left that only a test can reach. See `../spec.md`.

**Blocked by:** 01 — Command chords zoom the whole console through egui.

**Status:** ready-for-agent

**Tags:** release/v1

- [x] The Source View holds no zoom. The zoom command recogniser, the stepped-zoom function, the zoom limits and the glyph-scale quantisation are gone, and so are the unit tests that only tested them.
- [x] The Source scene transform carries translation only. Glyphs are laid out at the Source's own size, and egui's pixels-per-point does the enlargement.
- [x] Tests that varied the Source View Zoom are re-expressed over egui zoom factors or device scales, and still protect what they protected: fixed display-point Grid, Sector Seam and Cursor stroke widths, Sector Seams inside the clip, and the Cursor follow after a zoom.
- [x] The Diagnostics window no longer reports a Source View Zoom.
- [x] Code comments and rustdoc no longer describe a Grid or Source View zoom separate from egui's, and ADR 0038 and ADR 0040 point to the ADR `01` added where they discuss the Source View's scaling.
- [ ] `paint_derive` stays within `benches/floors.toml` on the pull request's benchmark job.
- [x] `cargo fmt --all -- --check`, `cargo clippy --package console --all-targets --locked -- -D warnings`, `cargo nextest run --package console --locked` and the `--no-default-features` arm pass. `mise run check_wasm` passes.

## Comments

**2026-09-29 — part landed with `01`.** The zoom command recogniser (`ZoomCommand`, `zoom_command`), `stepped_zoom`, the View menu's `requested_zoom` and their unit tests are deleted by `01`: once no input reached them they were dead code the clippy gate refuses. The Source View's `zoom` field, `MIN_ZOOM`/`MAX_ZOOM`, the Glyph scale quantisation, the Diagnostics readout and the Zoom-parameterised width tests remain here.

**2026-09-30 — implemented on `feat/egui-whole-ui-zoom`.** Every line but the benchmark one is met:

- `SourceView` holds a Pan and the `origin` the Source's top-left is presented at; its `zoom` field, `MIN_ZOOM`/`MAX_ZOOM`, `GLYPH_SCALE_STEP`, `glyph_scale`, `GridViewport::cell_scale` and `source_bounds` are gone, with the unit tests that only tested them. `presented_grid` takes that origin and snaps the Source's own `CELL_SIZE`; nothing takes a scale. `console::tests::a_glyph_is_laid_out_at_the_sources_own_size_at_every_device_scale` failed first (a 10.0625 point Glyph at 1.1 ppp) and now holds every Glyph at 11.5 points at device scales 1.0, 1.1, 1.5 and 2.0.
- The width, clip and follow tests vary the device scale or egui's zoom factor instead: `grid_and_sector_widths_stay_fixed_display_points_across_device_scales`, `zero_grid_and_sector_widths_hide_every_border_and_seam_at_every_device_scale`, `a_console_pass_strokes_at_its_own_theme_width_and_snaps_runs_to_the_device_scale` (at 1.1, where the snapped Cell is under 16 points), `a_panned_console_paints_every_sector_seam_inside_the_clip` (at 1.0 and 1.5), `the_draw_loop_paints_the_visible_range_rather_than_the_whole_source` (egui zoom 2.0), the snapped-Cell follow tests at 1.1, the two drag-Pan tests at a device scale of two, and `cursor_effects`' stroke tests over snapped Cell sides. `grid_viewport::tests::every_cell_is_a_whole_number_of_physical_pixels` sweeps egui's whole zoom range at three native scales.
- The Diagnostics window shows the visible Source region from the origin and no Source zoom; `the_view_menu_opens_the_diagnostics_window_a_viewer_asked_for` asserts the row is absent.
- ADR 0038 and ADR 0040 point to ADR 0058 where they discuss the Source's scale; ADR 0058's consequence now states the Source View holds no scale.

**Benchmark line: not verified locally.** `paint_derive` against `benches/floors.toml` is the pull request's benchmark job, which a local `mise run bench` cannot decide. This issue resolves once that job passes on the pull request.
