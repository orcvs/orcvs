# 02 — Retire the winit 0.30 skips once eframe adopts winit 0.31

**What to build:** Once the console's eframe runs on winit 0.31, the duplicates that only winit 0.30 kept are gone from the graph, and their skip entries come out of `deny.toml`. winit 0.31's Wayland stack (`smithay-client-toolkit 0.21`, `calloop 0.14`) requires `thiserror 2`, and its macOS backend uses the `objc2-core-*` 0.3 crates in place of `core-graphics`.

Waiting on: an eframe release on winit 0.31. As of 2026-09-25, winit 0.31 is at `v0.31.0-beta.3` and egui has no tracking issue for adopting it.

**Blocked by:** 01 — Skip the duplicates winit 0.30 and AccessKit's macOS adapter keep in the graph.

**Status:** needs-triage

- [ ] The console depends on an eframe release built on winit 0.31.
- [ ] The nine winit-only skip entries are removed.
- [ ] `cargo deny --locked --all-features check bans` reports no duplicate for those crates and no unused-skip warning.
