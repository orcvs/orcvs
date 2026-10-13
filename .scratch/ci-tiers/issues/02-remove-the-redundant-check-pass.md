# 02 — Remove the redundant check pass

**What to build:** Delete `cargo check --workspace --all-targets --locked` from the `check` task in
`mise.toml`. Clippy performs the same compilation and adds the lints, so the separate check pass
produces no signal that clippy does not already produce.

**Blocked by:** 01 — Restore the Linux build.

**Status:** resolved

- [x] The `check` task runs fmt, clippy, nextest, doctests, rustdoc, and `cargo deny`.
- [x] `cargo check` no longer appears in any mise task.
- [x] `mise run check` still fails on a clippy warning, a test failure, and a rustdoc warning.
- [x] `scripts/check-tooling-contract.sh` still passes.

## Comments

Do not expect this to save time. Measured cold, the check pass costs 48 seconds and clippy then
costs 1 second, because clippy reuses every dependency artefact and only re-checks the workspace
crates. Delete the step because it is redundant, not because it is a lever.

The 48 seconds moves to clippy rather than disappearing.

`mise run check` and `bash scripts/tests/check-tooling-contract.sh` passed before the PR. The
pull-request tier also passed twice in hosted CI.

**2026-10-13 — Audit against origin/main cf01e3c.** Criterion 2 had regressed: `test_persistence` ran `cargo check --package orcvs --lib --features persistence --locked`, justified by a reason that did not hold (other lines reach the feature). Decided 2026-10-13 by Toby: the line becomes `cargo clippy --package orcvs --lib --features persistence --locked -- -D warnings`. It is kept because it is the one compilation of `orcvs`'s shipped library with `persistence` and without the test-only features, which every `--all-targets` line unifies in through `orcvs`'s dev-dependency on itself. `mise.toml`, its pin in `scripts/check-tooling-contract.sh`, the meta-test and `docs/tooling.md` now say so. No mise task runs `cargo check` again.
