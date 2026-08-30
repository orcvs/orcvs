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
