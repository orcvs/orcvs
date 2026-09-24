# 02: Capture the web console in CI

**What to build:** A job in the capture workflow that builds the web console, serves it, and captures it in headless Firefox. It covers the same wide and tall viewports, both built-in Themes, the same seeded Source and pinned settings, and adds the four images to the run's artifact and manifest. See `../spec.md`.

**Blocked by:** 01 — Capture the native console in CI.

**Status:** ready-for-agent

**Tags:** release/v1

- [ ] The capture workflow gains a WASM job that builds the web target with `trunk` at the dispatched SHA and serves it locally.
- [ ] Playwright drives headless Firefox to produce four images: {wide, tall} × {Okabe–Ito, Orcvs Light}, at egui zoom 1.0 with the mode set explicitly. Playwright's addition carries a recorded rationale.
- [ ] The seeded Source, mode and Theme reach the page through browser storage, the web build's shipped persistence path, using the same fixture as `01`.
- [ ] Before capturing, the job asserts that the canvas rendered and the pinned mode, Theme and Source were applied. A failure to apply them fails the job.
- [ ] The four images join the run's artifact, and the manifest records the browser and its version as the renderer.
- [ ] One real dispatched run against `main` produces all eight images in one artifact, and its run link is recorded here.
- [ ] `actionlint`, `zizmor --offline .github/workflows`, `bash scripts/check-tooling-contract.sh` and `bash scripts/tests/check-tooling-contract.sh` pass. `mise run check_wasm` passes.
