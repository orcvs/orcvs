# 08 — Run each tier once per change

**What to build:** A merged change pays for the pull-request tier once, not once on its pull request and again in the merge queue, so a stack of pull requests does not wait on each check twice.

**Blocked by:** None — can start immediately.

**Status:** resolved

- [x] Decide where the merge-only gates (macOS, the browser suite, full-count proptest, `cargo deny`) run if the queue stops re-running the pull-request tier: in the queue alone, on push to `main`, or on a schedule.
- [x] The workflow and `scripts/check-ci-results.sh` match that decision, and the tooling contract pins it.
- [x] The repository's merge settings match the workflow (a required merge queue needs a `merge_group` run that reports `ci`).

## Comments

Today `test.yml` runs `check_pull_request` on the pull request, then again in the merge queue on
Linux and macOS, plus the merge tier. For a single maintainer merging a stack, the second full run
is the compounding delay. The decision needs the repository owner because the merge queue is a
repository setting, not a file.

### Decision (2026-10-24)

Toby kept the merge queue and chose to have it run the merge-only gates alone. In the queue, the
Linux job skips `check_pull_request` and runs the native merge tier, and the WASM job skips
`check_wasm` and runs the browser suite. macOS still runs `check_pull_request` there, so the
queued candidate gets one full native pass. `ci` is unchanged: every job still reports. No setting
changes.
