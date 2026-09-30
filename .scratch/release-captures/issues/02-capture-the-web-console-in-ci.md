# 02: Capture the web console in CI

**What to build:** A job in the capture workflow that builds the web console, serves it, and captures it in headless Firefox. It covers the same wide and tall viewports, both built-in Themes, the same seeded Source and pinned settings, and adds the four images to the run's artifact and manifest. See `../spec.md`.

The native job from `01` uploads its four images and `manifest.json` as a native-only artifact. One artifact holding all eight images and one manifest comes from a final `collect` job that needs both capture jobs, downloads their artifacts, merges their manifests, and uploads the combined artifact. The native job is unchanged, and the two captures run in parallel.

**Blocked by:** 01 — Capture the native console in CI.

**Status:** ready-for-agent

**Tags:** release/v1

- [ ] The capture workflow gains a WASM job that builds the web target with `trunk` at the dispatched SHA and serves it locally.
- [ ] Playwright drives headless Firefox to produce four images: {wide, tall} × {Okabe–Ito, Orcvs Light}, at egui zoom 1.0 with the mode set explicitly. Playwright's addition carries a recorded rationale.
- [ ] The seeded Source and mode reach the page through browser storage, the web build's shipped persistence path, using the same fixture as `01`. The web reads no `config.toml`, so the Theme is the default selection for the pinned mode: Okabe–Ito for Dark, Orcvs Light for Light.
- [ ] Before capturing, the job asserts that the canvas rendered and the pinned mode, Theme and Source were applied. A failure to apply them fails the job.
- [ ] The web job uploads its four images and manifest entries as its own artifact, recording the browser and its version as the renderer. A `collect` job that needs both capture jobs merges the two manifests and uploads one artifact holding all eight images.
- [ ] One real dispatched run against `main` produces all eight images in one artifact, and its run link is recorded here.
- [ ] `actionlint`, `zizmor --offline .github/workflows`, `bash scripts/check-tooling-contract.sh` and `bash scripts/tests/check-tooling-contract.sh` pass. `mise run check_wasm` passes.

## Comments

**2026-09-30 — audit: the artifact layout is still undecided.** The spec asks for one artifact per run holding every image and a manifest, but does not choose between a final job that downloads the native and web outputs and merges their manifests, and the web job writing into the native job's artifact. That choice must be made and recorded in the implementing pull request (and in this ticket). A `collect` job needs no change to `01`: the native job already uploads its images and `manifest.json` as `release-captures-native-<sha>` (`.github/workflows/release-captures.yml`, the `upload-artifact` step), which a later job can download.

**2026-10-01 — decided: a `collect` job.** The web job does not write into the native artifact. It uploads its own, and a final job that needs `native` and the web job downloads both, merges their manifests into one `manifest.json`, and uploads the single artifact the spec asks for. This leaves `01` unchanged, keeps the two captures parallel so a web failure does not hide the native images, and puts the manifest merge in one place. Writing into the native artifact would serialise the jobs and still need the same merge, since the web job would have to re-upload the whole set.
