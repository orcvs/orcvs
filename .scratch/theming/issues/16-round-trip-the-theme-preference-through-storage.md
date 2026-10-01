# 16 — Round-trip the theme preference through storage

**What to build:** A test that saves egui memory holding a non-default `ThemePreference` through the storage codec the shipped binary uses, restores it into a fresh context, constructs the console, and finds the preference intact. Residual of `02`.

**Blocked by:** None — can start immediately.

**Status:** resolved

**Tags:** release/v1

- [x] Under `persistence`, the test writes egui memory with a `Light` (or `Dark`) preference through `RonFileStorage` in an `IsolatedRonDir` (`console/src/persistence.rs`), reads it back into a new `egui::Context`, runs `Console::new`, and asserts the preference is unchanged.
- [x] The test builds its input below the shipped entry point, without a seam in shipped code.
- [x] `cargo nextest run --package console --locked` and the `--no-default-features` arm pass.

## Comments

**2026-09-23 — opened from a CodeRabbit review of the audit.** `02`'s "keeps it across a restart" line was ticked on `console_new_keeps_a_theme_preference_already_on_the_context` (`console/src/console.rs:3265`), which sets the preference on the context before `Console::new`. That proves `Console::new` does not overwrite a restored preference, but not that the preference survives eframe's save and restore of egui memory. `02` records the same limit and now leaves that line unticked, pointing here.

**2026-09-24 — joins `release/v1`.** The View menu's mode is shipped and persists only through eframe's egui-memory save, and `02`'s "keeps it across a restart" line points here. The definition of done's persistence line requires the save–restart–reload path to be proven. Blocks `v1-release/03`. `zoom-alignment/01` proves egui's zoom factor through the same path, so whichever lands first sets the restart-test pattern.

**2026-09-29 — audit at `cad296df`.** `console_new_keeps_a_theme_preference_already_on_the_context` moved with the console's tests to `console/src/console/tests.rs:1093`; it still sets the preference on the context directly. No storage test round-trips egui memory: the only `RonFileStorage` use in a test is the Source round-trip in `console/src/console/storage_tests.rs`. `RonFileStorage` and `IsolatedRonDir` are as described (`console/src/persistence.rs`). No criterion is met.

**2026-09-30 — the restart pattern exists.** `a_zoom_survives_a_save_and_a_restart` (`console/src/console/kittest_tests.rs:1355`, #182) round-trips egui memory through `RonFileStorage`/`IsolatedRonDir` and restores it before `start_console`. `Options::theme_preference` is serialized in the same `Memory`, so this test is that one with a `Light` preference asserted in place of the zoom. `console_new_keeps_a_theme_preference_already_on_the_context` is now at `console/src/console/tests.rs:951`. No criterion is met.

**2026-10-01 — resolved by `a_mode_survives_a_save_and_a_restart` (`console/src/console/kittest_tests.rs:1454`).** A harness console chooses Light through the top bar's mode button, then `save_on_exit` (`kittest_tests.rs:1351`) runs `App::save` and writes egui memory under eframe's key through `RonFileStorage` in an `IsolatedRonDir`, after asserting `App::persist_egui_memory`. `RonFileStorage::from_file` reads the file back, `restore_egui_memory` (`kittest_tests.rs:1371`) puts the memory into a new `egui::Context`, `Console::new` runs over that context with the storage in its `CreationContext`, and the test asserts `ThemePreference::Light` and a resolved `Theme::Light`. Light is chosen because neither start produces it unaided: the kittest harness opens on Dark and a fresh `Context` on System. The save and restore helpers are shared with `a_zoom_survives_a_save_and_a_restart`, which now calls them in place of its inline copy. Everything is test-only (`#[cfg(all(feature = "persistence", not(target_arch = "wasm32")))]`); no shipped code changed.

The restart uses a bare `Context` rather than `restarted_console`'s harness: `egui_kittest` 0.36.2's `Harness::from_builder` calls `ctx.set_theme(builder.theme)` after `build_eframe`'s closure returns, so any harness restart reads the builder's theme (Dark by default) instead of the restored one. Under a harness the test failed with `left: Dark, right: Light` for that reason alone.

Checked that the test exercises the path: with the `restore_egui_memory` call removed it fails (`left: System, right: Light`), and with the egui-memory write in `save_on_exit` removed both restart tests fail.

Commands, with `PROPTEST_CASES=32`: `cargo fmt --all -- --check`, `cargo clippy --package console --all-targets --locked -- -D warnings`, `cargo nextest run --package console --locked` (521 passed), and `cargo nextest run --workspace --tests --no-default-features --locked` (1472 passed, 1 skipped; the test compiles out) — all passed.
