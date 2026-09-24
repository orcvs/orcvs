# 01: Command chords zoom the whole console through egui

**What to build:** Command `+` (or `=`) and command `-` step egui's whole-UI zoom, and command `0` resets it, exactly as in any egui app. The Source Grid grows with the menus, the Panel and the Diagnostics because its Cells are sized in points. View → Zoom In, Zoom Out and Reset offer the same actions with their chords shown. The chosen zoom survives a restart in a persistence build. The Source View's own Zoom no longer responds to any input: it stays at 1.0 until `02` deletes it. See `../spec.md`.

**Blocked by:** None (can start immediately).

**Status:** ready-for-agent

**Tags:** release/v1

- [ ] The console no longer turns egui's keyboard zoom off. Command `+`, command `=` and command `-` step egui's zoom factor by egui's own step within egui's own range, and command `0` returns it to 1.0.
- [ ] No command chord changes the Source View's Zoom or Pan, and no chord writes to the Source. Bare `+`, `-`, `=` and `0` still write Glyphs.
- [ ] The View menu offers egui's Zoom In, Zoom Out and Reset buttons, with their chords shown, beside the existing mode and Theme pickers.
- [ ] A zoomed console paints square, whole-physical-pixel Cells, and Grid lines, Sector Seams and Cursor strokes keep their Theme's display-point widths.
- [ ] A zoom that would open a gap past the Grid's edge settles the Source View back inside the margin, and the Cursor follow keeps the Cursor in view.
- [ ] In a persistence build a zoom factor is restored after a save and restart. A build without persistence starts at 1.0.
- [ ] The web build is checked in the headless browser suite: command `+` changes egui's zoom factor, and whether the browser also zooms the page is recorded. If both zoom, the double zoom is removed.
- [ ] A new ADR supersedes ADR 0045's zoom section. It records egui's whole-UI zoom as the console's only zoom, and command-shift withdrawn because it collides with egui's logical match of command `+`. ADR 0045's Status line points to it.
- [ ] CONTEXT.md's **Zoom** is redefined as egui's whole-UI zoom.
- [x] `source-view/12` is marked superseded by this issue.
- [ ] `command_zoom_chords_change_the_source_view_and_never_the_source` becomes a test that the chords change egui's zoom factor and never the Source. The kittest harness over the running console is the seam.
- [ ] `cargo fmt --all -- --check`, `cargo clippy --package console --all-targets --locked -- -D warnings`, `cargo nextest run --package console --locked` and the `--no-default-features` arm pass. `mise run check_wasm` passes.
