# 26 — Share console test setup and stop polling with sleeps

**What to build:** Console tests build a Console through one helper and wait on Playback deterministically. Today about thirty tests in `console.rs` repeat the context, install and `Console::new(cc, ThemeRegistry::built_in(), Config::default())` sequence, alongside separate helpers (`running_console`, `running_console_with`, `console_under_os_appearance`, `console_with_settings_moved` in `kittest_tests.rs`; `storage_tests::console_over`). Six Playback-driven tests poll with real one-millisecond `tokio::time::sleep`, up to two thousand times. `console/tests/wasm.rs` waits on Playback with real `TimeoutFuture`s and is out of scope.

**Blocked by:** 12 — Move console.rs inline tests into sibling test modules.

**Status:** resolved

- [x] One test helper constructs a Console; no test repeats the construction sequence, and the existing helpers build on it or are merged into it.
- [x] No native console test sleeps on wall-clock time to wait for Playback; they run on paused time or await the Playback observation.
- [x] The test count is unchanged and every test passes.

## Comments

**2026-09-25 — audited against `origin/main` `199c3331`.** Still valid. `Console::new` now takes a third `Config` argument (5f35edc9). The polling bound is two thousand iterations, not one thousand. The existing helpers and the WASM scope are named.

**2026-09-25 — implementation (epic PR 15).** Delivered in orcvs/orcvs#158, stacked on #157. `console::tests::start_console` is the one helper that calls `Console::new`; `console_on` and `fresh_console` build on it for the `app_pass` tests, kittest's `console_harness` (behind `running_console`, the stored-revision harnesses and `configured_console_under_os_appearance`, which absorbs `console_with_settings_moved`'s setup) and `storage_tests::console_over` call it. The eight polling loops across six tests (PR 14 added `an_opened_console_repaints_as_soon_as_its_new_playback_publishes`) now await the Playback observation through `engine_reaches`, moved from `kittest_tests` into `tests` and bounded by `ENGINE_WAIT`. The native console test list is unchanged at 496; `console/tests/wasm.rs` (9 tests) is untouched.
