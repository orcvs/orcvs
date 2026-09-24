# 03 — Retire the objc2 0.5 skips once AccessKit's macOS adapter moves to objc2 0.6

**What to build:** Once neither winit nor `accesskit_macos` requires `objc2` 0.5, the older `objc2`, `objc2-foundation`, `objc2-app-kit` and `block2` copies leave the macOS graph, and their skip entries come out of `deny.toml`.

Waiting on: an `accesskit_macos` release on `objc2` 0.6 that the eframe release from 02 brings in. As of 2026-09-25, `accesskit_macos` 0.27.0 still requires `objc2 ^0.5.1`.

**Blocked by:** 02 — Retire the winit 0.30 skips once eframe adopts winit 0.31.

**Status:** needs-triage

- [ ] The four `objc2`-family skip entries are removed.
- [ ] `cargo deny --locked --all-features check bans` reports no duplicate for those crates and no unused-skip warning.
