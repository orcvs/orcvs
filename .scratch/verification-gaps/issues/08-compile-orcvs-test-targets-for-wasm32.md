# 08 — Compile orcvs's test targets for wasm32

**What to build:** Every crate's test targets type-check for the browser target, not shell's alone. `orcvs` builds a Tokio runtime in a test with no target guard, so its test targets cannot compile for `wasm32-unknown-unknown` at all, and issue 04 had to name shell explicitly to work around it.

**Blocked by:** 04 — Compile the WASM test targets before a merge.

**Status:** in-review

- [x] `orcvs`'s native-only tests are guarded so its test targets compile for the browser target.
- [x] The WASM tier compiles the workspace's test targets rather than one package's.
- [x] The contract pins the widened invocation.

## Comments

`orcvs` already carries browser-target test support that no gate has ever compiled — an import guarded on the browser target, inside a test module nothing builds for it. Dead code under every gate today, and the kind that looks like coverage.

The consequence is narrow but real: mid-playback tempo change has no browser coverage in any form, while starting, stopping and observing playback get indirect coverage through the browser suite, which drives them through `orcvs`.

Worth deciding rather than assuming: `orcvs`'s unit tests are native by intent, and guarding them buys type-checking for test code nobody plans to run in a browser. The gain is that the WASM tier stops naming one package for a reason that has nothing to do with that package.

## Resolution

One guard, not a family of them. `failed_tempo_retune_keeps_existing_playback_running` in
`orcvs/src/app.rs` stages a retune failure by hand-building a `tokio::runtime::Runtime`, which is the
multi-threaded builder only the `[target.'cfg(not(target_arch = "wasm32"))'.dependencies]` table
pulls in. It carries `#[cfg(not(target_arch = "wasm32"))]`; every other test in the module, including
the `#[tokio::test(start_paused = true)]` one the macro builds on the current-thread runtime,
compiles for the browser target unchanged. The native suite still runs 199 tests, verified by
re-running with the file stashed rather than by arithmetic.

The browser-target support this ticket predicted would compile for the first time did compile
cleanly: `#[cfg(target_arch = "wasm32")] use tokio::time;` in `orcvs/src/playback.rs`, standing in for
what `use super::*` supplies on native, and genuinely used at `time::advance` in an ungated test.

`check_wasm` then collapsed to one line. `--all-targets` expands to `--lib --bins --tests --benches
--examples`, so the workspace form strictly covers both commands it replaced, and adds `orcvs`'s and
`lang`'s test targets and `orcvs`'s criterion bench, which nothing had ever checked for the browser
target.
