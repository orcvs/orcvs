# 01 — Restore the Linux build

**What to build:** Add the `x11` and `wayland` features to the `eframe` dependency, and record the
reason in the manifest. `default-features = false` removes the windowing backend, so winit fails to
compile on Linux. Both features are needed: `x11` alone leaves a native Wayland session without a
backend.

**Status:** resolved

- [x] `eframe` declares `x11` and `wayland` alongside `glow`.
- [x] A comment in the manifest states why the features are explicit.
- [x] `cargo check --workspace --all-targets --locked` passes on Linux.
- [x] One CI run completes green and populates the `Swatinem/rust-cache` entries.
- [x] The macOS and WASM jobs are unaffected.

## Comments

The failing run is 33070410805. It ends at `error: The platform you're compiling for is not
supported by winit`, raised from `winit` 0.30.13. The `libasound2-dev` step in the workflow serves
`midir`, not winit, so it does not address this.

Nothing else in this effort can be measured until this run is green. The cache has never been
populated, so the first green run also establishes the warm baseline that issue 04 needs.

PR 2 run 33312928171 passed on Linux and macOS. The Linux job completed in 15m44s and saved a
432,143,224-byte cache entry. The unchanged WASM gates passed locally through `mise run check`.
