# Release captures: render the candidate to images in CI

**Status:** ready-for-agent

**Tags:** release/v1

## Problem Statement

The release needs visual evidence that the exact candidate renders correctly: native and WASM, wide and tall, in both shipped built-in Themes. The tests prove every colour, contrast ratio and Cell geometry as values, but nothing proves what reaches the screen. Today that evidence is manual. Someone runs the candidate, arranges each state by hand, screenshots it and records the metadata. Once Orcvs Light ships and the default mode follows the OS, that means eight hand-made captures per candidate, with nothing stopping two reviewers from capturing different Themes. The only light captures that exist came from a temporarily patched build and are not tied to any candidate.

## Solution

A separate, dispatch-only CI workflow renders the candidate to images. Given a SHA, it captures the console natively (through the same harness the console's interaction tests use, rendered by a software GPU driver) and in the browser (the web build, served and captured in headless Firefox). It covers both built-in Themes at wide and tall viewports, with the mode, Theme, zoom and seeded Source pinned, so every run shows the same states. The images and their metadata are uploaded as one artifact per run. A human reviewer checks them against the visual checklist and signs off. One manual native macOS capture per Theme remains, as the real-GPU spot-check the software renderer cannot provide. The workflow is never part of the pull-request or merge tiers and never a required status.

## User Stories

1. As a release reviewer, I want every capture the release needs produced from one workflow run against the candidate SHA, so that no capture comes from a different build.
2. As a release reviewer, I want captures in both Okabe–Ito and Orcvs Light, so that every shipped palette is seen on the candidate.
3. As a release reviewer, I want native and WASM captures at wide and tall viewports, so that both targets and both aspect ratios are covered.
4. As a release reviewer, I want each capture to name its SHA, platform, renderer, viewport, mode, Theme, zoom and procedure, so that I know exactly what I am judging.
5. As a release reviewer, I want every capture to show the same seeded Source, so that the checklist's states are all present and comparable across Themes and targets.
6. As a release reviewer, I want the seeded Source to reach each checklist state (occupied and empty Cells, Function, Number, Note, Bang, Comment, Sequence, the fitted Output Portal, an invalid operand's diagnostic, a Region, Sector Seams, and the Cursor), so that one image per viewport covers the whole checklist.
7. As a release reviewer, I want captures taken with the mode set explicitly to Dark or Light, so that the machine's OS appearance cannot change what is captured.
8. As a release reviewer, I want captures taken at egui zoom 1.0, so that Cell sizes are comparable across runs.
9. As a release reviewer, I want the images downloadable as one artifact per run, so that I can review and archive them beside the candidate's other evidence.
10. As a release manager, I want to start the workflow by hand with a SHA, so that I can capture exactly the commit I nominate.
11. As a release manager, I want the workflow to be able to run on a release-candidate tag, so that capturing a tagged candidate needs no extra step.
12. As a contributor, I want pull requests and merges to pay nothing for captures, so that ordinary CI stays inside its budget.
13. As a contributor, I want the ordinary `console` build and its gates unchanged, so that the renderer dependencies used for captures never reach them.
14. As a maintainer, I want the capture renderer's dependencies confined to the capture job, with a recorded rationale, so that the dependency policy stays honest.
15. As a maintainer, I want the seeded Source and pinned settings to reach the console through its shipped persistence path, so that no seam is cut into shipped code for captures.
16. As a maintainer, I want the workflow's manual dispatch proven by a real run before the release relies on it, so that it cannot fail the way manual benchmark runs did.
17. As a maintainer, I want the capture job to report which states it reached, so that a capture missing a state fails loudly rather than producing an incomplete image.
18. As a release reviewer, I want one manual macOS native capture per Theme recorded beside the CI captures, so that real-GPU font rasterisation and Retina scaling are also seen.
19. As a release reviewer, I want the visual checklist to describe the shipped console (top-left rest with a two-Cell margin, not centring), so that I judge what ships.
20. As a Theme maintainer, I want the captures to need no committed baseline images, so that a Theme change never becomes an image-update chore.

## Implementation Decisions

- **A new workflow**, separate from `test.yml` and `bench.yml`. Triggers: manual dispatch with a SHA input, and optionally a release-candidate tag push. It is never a required status and never runs on pull requests or merges.
- **Native capture** uses `egui_kittest`'s `snapshot` and `wgpu` features, enabled only for the capture job (a dedicated test target or feature that no other gate turns on), and rendered through Mesa's lavapipe software Vulkan driver on a Linux runner. The ordinary `console` dev graph keeps them off, and the rationale in `console/Cargo.toml` and the egui skill's guide is updated to say where they are on and why. The dependency addition goes through the repository's dependency-change process (`mise run audit_deps`).
- **WASM capture** builds the web target with `trunk`, serves it locally in the job, and captures it with Playwright driving headless Firefox. Playwright is a new Node dev dependency with a recorded rationale.
- **One seeded Source**, a fixture reaching every checklist state, is used by both jobs. It enters the console through the shipped persistence path: the stored Source key natively, and browser storage on the web. Mode, Theme and zoom are pinned the same way, through persisted egui memory and the stored Theme identities, never through a capture-only branch in shipped code.
- **Matrix:** {native, WASM} × {wide, tall} × {Okabe–Ito, Orcvs Light}, eight images, at egui zoom 1.0. Wide and tall sizes are fixed in logical points and recorded.
- **Output:** one artifact per run holding the images and a manifest recording SHA, runner OS, renderer (lavapipe or the browser's), viewport, mode, Theme, zoom and procedure for each image.
- **No baseline comparison.** The workflow produces images for human review and never diffs them against committed images. Snapshot regression testing can be added later as an Improvement.
- **The macOS spot-check stays manual:** one native capture per Theme on real hardware, recorded in restyle-egui-console/03 with the same metadata.
- **Release wiring:** both tickets block restyle-egui-console/03, not v1-release/03, so capture tooling never delays cutting the candidate.

## Testing Decisions

- **A good test here proves the capture is of the right thing**, not what the pixels are: the seeded Source loaded, the mode and Theme pinned, zoom at 1.0, and every checklist state present in the frame that was rendered. The native job asserts these through the harness's widget tree and the console's own state before it renders. The WASM job asserts the canvas rendered and the pinned settings applied before it captures.
- **Seam:** the kittest harness over the running console, the same one the console's interaction tests use, with the renderer added. On the web the seam is the served page, with Playwright as the driver.
- **Prior art:** the console's kittest tests (the `running_console` fixture), the storage tests that restore Theme references, and `.scratch/theming/evidence/README.md`, which records how the manual Orcvs Light captures were made and what they covered.
- **Workflow checks:** `actionlint`, `zizmor --offline .github/workflows` and `bash scripts/check-tooling-contract.sh`, extended to pin that the capture workflow never triggers on pull requests or merges.
- **Proof of dispatch:** one real dispatched run against `main` produces the artifact before the release relies on it.

## Out of Scope

- Committed baseline images and automated visual regression diffs.
- Capturing loaded custom Themes: they are not decided palettes.
- Real-GPU capture in CI (a macOS or GPU runner).
- Making captures a pull-request or merge gate.

## Further Notes

- This effort follows the release decision to capture both built-in Themes. `v1-release/definition-of-done.md` and `restyle-egui-console/03` are amended to match.
- `zoom-alignment` changes zoom to egui's whole-UI zoom. Captures pin egui zoom 1.0 either way, so the two efforts are independent.
