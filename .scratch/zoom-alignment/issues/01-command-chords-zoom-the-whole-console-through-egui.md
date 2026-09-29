# 01: Command chords zoom the whole console through egui

**What to build:** Command `+` (or `=`) and command `-` step egui's whole-UI zoom, and command `0` resets it, exactly as in any egui app. The Source Grid grows with the menus, the Panel and the Diagnostics because its Cells are sized in points. View → Zoom In, Zoom Out and Reset offer the same actions with their chords shown. The chosen zoom survives a restart in a persistence build. The Source View's own Zoom no longer responds to any input: it stays at 1.0 until `02` deletes it. See `../spec.md`.

**Blocked by:** None (can start immediately).

**Status:** ready-for-agent

**Tags:** release/v1

- [x] The console no longer turns egui's keyboard zoom off. Command `+`, command `=` and command `-` step egui's zoom factor by egui's own step within egui's own range, and command `0` returns it to 1.0.
- [x] No command chord changes the Source View's Zoom or Pan, and no chord writes to the Source. Bare `+`, `-`, `=` and `0` still write Glyphs.
- [x] The View menu offers egui's Zoom In, Zoom Out and Reset buttons, with their chords shown, beside the existing mode and Theme pickers.
- [x] A zoomed console paints square, whole-physical-pixel Cells, and Grid lines, Sector Seams and Cursor strokes keep their Theme's display-point widths.
- [x] A zoom that would open a gap past the Grid's edge settles the Source View back inside the margin, and the Cursor follow keeps the Cursor in view.
- [x] In a persistence build a zoom factor is restored after a save and restart. A build without persistence starts at 1.0.
- [ ] The web build is checked in the headless browser suite: command `+` changes egui's zoom factor, and whether the browser also zooms the page is recorded. If both zoom, the double zoom is removed.
- [x] A new ADR supersedes ADR 0045's zoom section. It records egui's whole-UI zoom as the console's only zoom, and command-shift withdrawn because it collides with egui's logical match of command `+`. ADR 0045's Status line points to it.
- [x] CONTEXT.md's **Zoom** is redefined as egui's whole-UI zoom.
- [x] `source-view/12` is marked superseded by this issue.
- [x] `command_zoom_chords_change_the_source_view_and_never_the_source` becomes a test that the chords change egui's zoom factor and never the Source. The kittest harness over the running console is the seam.
- [x] `cargo fmt --all -- --check`, `cargo clippy --package console --all-targets --locked -- -D warnings`, `cargo nextest run --package console --locked` and the `--no-default-features` arm pass. `mise run check_wasm` passes.

## Comments

**2026-09-29 — implemented on `feat/egui-whole-ui-zoom`.** Every line but the web one is met, each by a test through the shipped console:

- Chords, range, reset and bare characters: `console::kittest_tests::command_zoom_chords_change_egui_zoom_and_never_the_source` (the renamed test) drives command `=`, `+`, `-` and `0` through the kittest harness and asserts egui's zoom factor steps by a tenth within 0.2..=5.0, the Source View's Zoom stays 1.0, its Pan does not move and no Cell is written; bare `+`, `-`, `=` and `0` write Glyphs.
- View menu: `the_view_menu_zooms_egui_as_its_chords_do` finds egui's own Zoom In, Zoom Out and Reset Zoom above Diagnostics, each showing egui's chord, and clicks them through egui's zoom factor.
- Painting: `a_zoomed_console_paints_whole_pixel_cells_at_the_themes_widths` zooms to 1.3 by chord and asserts square Cells a whole number of physical pixels at a whole-pixel corner, and the painted Grid line, Sector Seam and Cursor strokes at the Theme's point widths.
- Bounds and follow: `console::tests::a_zoom_that_would_open_a_gap_settles_the_source_view_back_inside` and `a_zoom_that_would_leave_the_cursor_outside_the_view_pans_to_show_it` over egui's zoom factor, and `arrow_keys_that_move_the_cursor_out_of_view_pan_the_source_view_to_follow_it` at egui zoom 2.0. The Source View follows the Cursor when egui's zoom factor changes from the last frame's.
- Persistence: `a_zoom_survives_a_save_and_a_restart` (persistence, native) writes egui memory through the native RON codec under eframe's `egui` key, restarts over it and finds zoom 1.3. Without persistence nothing restores egui memory, and the chords test, which runs in the `--no-default-features` arm, opens at 1.0.
- ADR 0058 supersedes ADR 0045's zoom section; ADR 0045's Status line and `CONTEXT.md`'s **Zoom** point to egui's zoom.

`ZoomCommand`, `zoom_command`, `stepped_zoom` and the View menu's `requested_zoom` are deleted here rather than in `02`: with no input reaching them they are dead code, which the clippy gate refuses. Their unit tests, and the Source View chord-step and chord-limit tests, went with them. The Source View's `zoom` field, `MIN_ZOOM`/`MAX_ZOOM` and the Glyph scale remain for `02`.

**Web line: unmet, and its premise does not hold.** Read from `eframe-0.36.2`, not observed in a browser: eframe's web runner sets `zoom_with_keyboard = false` and `zoom_factor = 1.0` before it builds the app (`src/web/app_runner.rs`, `AppRunner::new`), and `should_prevent_default_for_key` (`src/web/events.rs`) does not prevent the browser's default for command `+`, `=`, `-` or `0`. So on the web command `+` zooms the browser's page, which eframe follows as the native `pixels_per_point`, and does **not** change egui's zoom factor. There is no double zoom, and the outcome that avoids one is eframe's default, which the console now leaves alone. ADR 0058 records the browser's zoom as the web's one zoom. `console/tests/wasm.rs`'s `zoom::the_zoom_chords_leave_the_zoom_to_the_browser` holds the console's half: built over the options eframe's web runner sets, the console leaves keyboard zoom off, and the four chords leave egui's zoom factor at 1.0. `mise run check_wasm` compiles it; running it is the merge tier's headless Firefox run, which has not happened yet. What no automated test can observe is the browser's own page zoom: a synthetic key event does not trigger it. Whether the web View menu should keep egui's zoom buttons, which do change egui's factor on top of the browser's zoom and show no chord there, is open.

