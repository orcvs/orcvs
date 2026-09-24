# 01: Capture the native console in CI

**What to build:** A dispatch-only workflow that, given a SHA, renders the native console to images through the kittest harness with a software GPU driver. It covers wide and tall viewports in Okabe–Ito and Orcvs Light, showing one seeded Source that reaches every checklist state, with the mode, Theme and zoom pinned, and uploads the images with a manifest as one artifact. See `../spec.md`.

**Blocked by:** None (can start immediately).

**Status:** ready-for-agent

**Tags:** release/v1

- [ ] A new workflow runs on manual dispatch with a SHA input, and optionally on a release-candidate tag. It never runs on pull requests or merges, and `scripts/check-tooling-contract.sh` pins that.
- [ ] `egui_kittest`'s `snapshot` and `wgpu` features are enabled only for the capture job. The ordinary `console` build and every existing gate compile without them. The rationale in `console/Cargo.toml` and the egui skill's guide is updated, and `mise run audit_deps` passes.
- [ ] The job renders through Mesa's lavapipe on a Linux runner and produces four images: {wide, tall} × {Okabe–Ito, Orcvs Light}, at egui zoom 1.0 with the mode set explicitly.
- [ ] One seeded Source fixture reaches every checklist state: occupied and empty Cells, Function, Number, Note, Bang, Comment, Sequence, the fitted Output Portal, an invalid operand's diagnostic, a Region, Sector Seams and the Cursor. It enters the console through the shipped persistence path; no shipped code gains a capture-only branch.
- [ ] Before rendering, the job asserts that the seeded Source loaded, the pinned mode and Theme are the ones presented, zoom is 1.0, and every state is present. A missing state fails the job.
- [ ] The artifact holds the images and a manifest recording SHA, runner OS, renderer, viewport, mode, Theme, zoom and procedure for each image.
- [ ] One real dispatched run against `main` produces the artifact, and its run link is recorded here.
- [ ] `actionlint`, `zizmor --offline .github/workflows`, `bash scripts/check-tooling-contract.sh` and `bash scripts/tests/check-tooling-contract.sh` pass. `cargo clippy --package console --all-targets --locked -- -D warnings` and `cargo nextest run --package console --locked` pass without the capture feature.
