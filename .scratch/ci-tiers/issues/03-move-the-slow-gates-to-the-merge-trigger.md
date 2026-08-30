# 03 — Move the slow gates to the merge trigger

**What to build:** Define the verification tiers as mise tasks and let the workflow jobs call them.
A pull request runs fmt, clippy, tests, and doctests. A push to main additionally runs the
persistence gate, the WASM gates, and `cargo deny`. Keep the three existing jobs. Do not divide the
Ubuntu job by check type.

**Blocked by:** 02 — Remove the redundant check pass.

**Status:** resolved

- [x] A pull-request tier task runs fmt, clippy, workspace tests, and doctests.
- [x] A merge tier task runs `test_persistence`, `check_wasm`, `test_wasm`, `cargo deny`, and rustdoc.
- [x] `mise run check` remains the aggregate that runs every tier locally.
- [x] The workflow jobs invoke the mise tasks and hold no verification commands of their own.
- [x] The merge tier runs on `push` to main and is a required check.
- [x] `docs/tooling.md` describes the tiers and stays accurate.

## Comments

A red main branch is a stop condition under this arrangement. The merge tier is required, not
advisory, because an advisory tier with one maintainer is a tier nobody fixes. CI was red for three
days before this effort found it.

The tiers must stay reproducible from a checkout. `docs/tooling.md` states that `mise.toml` defines
the commands used both locally and in CI. Tiers expressed only as workflow YAML would break that.

Keep `RUSTFLAGS` identical across every step. A different value invalidates the whole cache.
`RUSTDOCFLAGS` is safe, because it affects rustdoc units alone.

Accept the consequence: a WASM break is now found after merge, not before it.

PR 2 confirmed that `full-gate` and `macos` run the pull-request tier while `wasm` skips. Main now
requires the `full-gate`, `macos`, and `wasm` status contexts with strict branch protection. The
same contexts run the merge components after a push to `main`.
