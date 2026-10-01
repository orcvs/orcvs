# 02 — Retire the winit 0.30 skips once eframe adopts winit 0.31

**What to build:** Once the console's eframe runs on winit 0.31, the duplicates that only winit 0.30 kept are gone from the graph, and their skip entries come out of `deny.toml`. winit 0.31's Wayland stack (`smithay-client-toolkit 0.21`, `calloop 0.14`) requires `thiserror 2`, and its macOS backend uses the `objc2-core-*` 0.3 crates in place of `core-graphics`.

Waiting on: an eframe release on winit 0.31. As of 2026-09-25, winit 0.31 is at `v0.31.0-beta.3` and egui has no tracking issue for adopting it.

**Blocked by:** 01

**Status:** needs-triage

- [ ] The console depends on an eframe release built on winit 0.31.
- [ ] The winit-only skip entries are removed: the nine from 01 and `windows-sys` 0.59 from 05.
- [ ] `cargo deny --locked --all-features check bans` reports no duplicate for those crates and no unused-skip warning.

## Comments

### Independent implementation audit — 2026-09-29

Still open: `Cargo.lock` resolves winit 0.30.13 and `deny.toml:36-51` retains the skips.
Shortened the blocker to its issue reference: `scripts/roadmap.ts:101` interpreted `0.30`
in the old blocker title as nonexistent issue `dependency-duplicates/30`. The real issue 01
is resolved; the upstream prerequisite remains in the body, not as a fictitious local blocker.

### Issue audit against d3fd1b27 — 2026-10-01

Unchanged. winit 0.30.13 locked; eframe 0.36.2 is still the latest release and winit 0.31 is
still `0.31.0-beta.3`. The skips are now `deny.toml:43-51`, plus `windows-sys@0.59` at `:58`.
