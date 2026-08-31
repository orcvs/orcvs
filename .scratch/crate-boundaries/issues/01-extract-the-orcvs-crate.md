# 01 — Extract the `orcvs` crate

**What to build:** Create the `orcvs` crate and move every module that holds no reference to the
toolkit: `source/`, `grid.rs`, `glyph.rs`, `cursor.rs`, `render_frame.rs`, `playback.rs`, `midi.rs`,
`native_midi.rs`, and `opts.rs`. Rename the remaining crate to `shell`. Leave `app.rs` in `shell`
for now; issue 02 frees it.

**Status:** resolved

- [x] `orcvs/Cargo.toml` declares no dependency on `egui`, `eframe`, or `winit`, directly or transitively.
- [x] `shell` depends on `orcvs`, and `orcvs` depends on `lang`.
- [x] `console.rs`, `style.rs`, `main.rs`, and `web_startup.rs` stay in `shell`.
- [x] The `persistence` feature still works and still gates `serde` and `eframe/persistence` correctly.
- [x] The WASM and native target configurations move with the modules that need them.
- [x] Every existing test still passes, in its original module.
- [x] `cargo tree --package orcvs` shows no toolkit crate.

## Comments

The move is mechanical. It changes no logic. Keep it that way: a behaviour change hidden inside a
file move is very hard to review.

Visibility needs care. `LanguageMap` and its `Range` are `pub(super)` inside `source/`, and
`RenderFrameConfig`, `CursorBloom`, and several `RenderCell` accessors are `pub(crate)`. Those
markers keep their meaning inside `orcvs`, so the inline tests that reach them keep working. Do not
widen any of them to make the move easier.

`playback.rs` brings `tokio` and `midi.rs` brings `midir`. Both are light next to `eframe`. Do not
split them out yet; ADR 0022 defers that until a measurement asks for it.

Resolved on `feat/extract-orcvs-crate` by commits `fb44d1a` and `9848f9a`. Review follow-up
`a022b2d` also completed the dependent input-boundary and running-instance moves from issues 02 and
03 without introducing a toolkit dependency into `orcvs`.

Verification passed for workspace formatting, checking, clippy, native/default tests, persistence,
dependency policy, WASM compilation, and the five Firefox WASM tests. `cargo tree --package orcvs
--locked` contains no `egui`, `eframe`, or `winit` dependency.
