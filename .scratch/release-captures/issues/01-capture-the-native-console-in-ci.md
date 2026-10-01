# 01: Capture the native console in CI

**What to build:** A dispatch-only workflow that, given a SHA, renders the native console to images through the kittest harness with a software GPU driver. It covers wide and tall viewports in Okabe–Ito and Orcvs Light, showing one seeded Source that reaches every checklist state, with the mode, Theme and zoom pinned, and uploads the images with a manifest as one artifact. See `../spec.md`.

**Blocked by:** None (can start immediately).

**Status:** ready-for-human

**Tags:** release/v1

- [x] A new workflow runs on manual dispatch with a SHA input. It never runs on pull requests or merges, and `scripts/check-tooling-contract.sh` pins that. (The optional release-candidate tag trigger, user story 11, moved to `03`.)
- [x] `egui_kittest`'s `snapshot` and `wgpu` features are enabled only through the `release-capture` feature. The ordinary `console` build and every pull-request gate compile without them; the merge tier compiles them, without rendering, through `mise run check_release_capture`, called once from `check_merge_native`, and only `capture_native` runs the capture. The rationale in `console/Cargo.toml` and the egui skill's guide is updated, and `mise run audit_deps` passes.
- [ ] The job renders through Mesa's lavapipe on a Linux runner and produces four images: {wide, tall} × {Okabe–Ito, Orcvs Light}, at egui zoom 1.0 with the mode set explicitly.
- [x] One seeded Source fixture reaches every checklist state: occupied and empty Cells, Function, Number, Note, Bang, Comment, Sequence, the fitted Output Portal, an invalid operand's diagnostic, a Region, Sector Seams and the Cursor. It enters the console through the shipped persistence path; no shipped code gains a capture-only branch.
- [x] Before rendering, the job asserts that the seeded Source loaded, the pinned mode and Theme are the ones presented, zoom is 1.0, and every state is present. A missing state fails the job.
- [x] The artifact holds the images and a manifest recording SHA, runner OS, renderer, viewport, mode, Theme, zoom and procedure for each image.
- [ ] One real dispatched run against `main` produces the artifact, and its run link is recorded here.
- [x] `actionlint`, `zizmor --offline .github/workflows`, `bash scripts/check-tooling-contract.sh` and `bash scripts/tests/check-tooling-contract.sh` pass. `cargo clippy --package console --all-targets --locked -- -D warnings` and `cargo nextest run --package console --locked` pass without the capture feature.

## Comments

**2026-09-29 — implemented; the dispatched run waits on merge.** `.github/workflows/release-captures.yml` runs `mise run capture_native` on `workflow_dispatch` with a required full-SHA input, and nothing else. The release-candidate tag trigger is left out: the repository has no candidate tag convention to match yet, and adding a `push: tags:` trigger later means widening the contract's dispatch-only rule deliberately. `scripts/check-tooling-contract.sh` pins that any workflow running the task or naming the feature has `workflow_dispatch` as its only trigger, that `--features release-capture` appears on exactly one `mise.toml` line and no task calls the capture task; `scripts/tests/check-tooling-contract.sh` covers each with a named rejection.

The capture feature is `console/release-capture` (`persistence`, `egui_kittest/snapshot`, `egui_kittest/wgpu`), rationale in `console/Cargo.toml`, `docs/tooling.md` and the egui skill's guide. `deny.toml` gains `pollster@0.4` and `rustc-hash@1` skips and an MPL-2.0 exception for `colored@2` alone (behind `dify`, `snapshot`'s differ); `mise run audit_deps` passes.

The test is `console::kittest_tests::capture`. The fixture is `console/tests/fixtures/release-capture.orcvs`; it enters as `orcvs_source` in an `app.ron` written with the native RON codec, beside egui memory holding the mode and `zoom_factor` 1.0 (restored into the `Context` as eframe's native runner does), with the Theme selections from a `config.toml` read by `Config::read`. Viewports are 1200 × 700 and 700 × 1200 points at `pixels_per_point` 2. A local run on macOS (Metal, not lavapipe) produced all four images and a valid manifest, and the console's clippy and nextest gates pass without the feature (491 tests); the lavapipe criterion stays open until the job has run on the Linux runner, which is the same dispatched run the next criterion asks for.

What is left is human: once this is on `main`, dispatch **Release captures** with `main`'s head SHA, record the run link here, and tick the lavapipe and dispatched-run criteria.

**2026-09-29 — review follow-ups.** The tag trigger moves to `03-capture-a-release-candidate-tag.md`. The capture is now compiled, without rendering, by `mise run check_release_capture` from `check_merge_native`, so a change to a console test helper it calls fails in the merge queue rather than at dispatch. `release-capture` is therefore named on two `mise.toml` lines, the capture task and the compile task, and the contract pins both lines, the compile task's single caller, and every spelling cargo accepts for the feature. Pull-request gates still compile without it. The capture now also asserts, before the harness applies its theme, that the console started under the mode and zoom its stored egui memory pins.

**2026-09-30 — audit: no dispatched run yet.** `gh run list --repo orcvs/orcvs --workflow release-captures.yml` returns `[]`: #184 merged the workflow to `main`, but nobody has dispatched it. The lavapipe four-image criterion and the dispatched-run criterion both wait on that one dispatch, so Status stays `ready-for-human`. The first two criteria were reworded to match the code: the tag trigger is `03`'s, and `mise.toml`'s `check_merge_native` runs `check_release_capture`, so the merge tier compiles the feature (without rendering) while pull-request gates do not.

### Issue audit against d3fd1b27 — 2026-10-01

Still no dispatched run (`gh run list --workflow release-captures.yml` is empty). The ticked
criteria hold on main. The only remaining step is a human dispatch with main's head SHA; 02 and 03
wait on it.
