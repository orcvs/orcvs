# 02: Remove the Source View's own Zoom

**What to build:** Delete the Source View's own Zoom now that egui's zoom is the only one: its state, the zoom command recogniser, the stepped-zoom function, the zoom limits and the glyph scale quantised from them. The Source scene transform keeps its translation (Pan) and loses its independent scaling. Nothing a viewer sees changes, and no shipped path is left that only a test can reach. See `../spec.md`.

**Blocked by:** 01 — Command chords zoom the whole console through egui.

**Status:** ready-for-agent

**Tags:** release/v1

- [ ] The Source View holds no zoom. The zoom command recogniser, the stepped-zoom function, the zoom limits and the glyph-scale quantisation are gone, and so are the unit tests that only tested them.
- [ ] The Source scene transform carries translation only. Glyphs are laid out at the Source's own size, and egui's pixels-per-point does the enlargement.
- [ ] Tests that varied the Source View Zoom are re-expressed over egui zoom factors or device scales, and still protect what they protected: fixed display-point Grid, Sector Seam and Cursor stroke widths, Sector Seams inside the clip, and the Cursor follow after a zoom.
- [ ] The Diagnostics window no longer reports a Source View Zoom.
- [ ] Code comments and rustdoc no longer describe a Grid or Source View zoom separate from egui's, and ADR 0038 and ADR 0040 point to the ADR `01` added where they discuss the Source View's scaling.
- [ ] `paint_derive` stays within `benches/floors.toml` on the pull request's benchmark job.
- [ ] `cargo fmt --all -- --check`, `cargo clippy --package console --all-targets --locked -- -D warnings`, `cargo nextest run --package console --locked` and the `--no-default-features` arm pass. `mise run check_wasm` passes.
