# 01 — Lint the shipped crates for Windows

**What to build:** A pull-request gate that compiles the shipped crates for
`x86_64-pc-windows-msvc`, so a break behind `cfg(windows)` fails the pull request that wrote it
instead of reaching `main` unseen. No CI job runs on Windows today.

**Blocked by:** None — can start immediately.

**Status:** resolved

- [x] `rust-toolchain.toml` declares `x86_64-pc-windows-msvc`, so rustup installs it with the pinned toolchain.
- [x] `mise run check_windows` runs `cargo clippy --workspace --lib --bins --target x86_64-pc-windows-msvc --locked -- -D warnings` with default features and with `--no-default-features`, and `check_pull_request` calls it.
- [x] `scripts/check-tooling-contract.sh` pins the task, its call and the toolchain target, and the fixture suite proves removing the call or the target is rejected.
- [x] `docs/tooling.md` records what the line covers and why it is scoped to the shipped crates.

## Comments

### Scope

The line is clippy, a check build: it neither links nor needs the MSVC linker or the Windows SDK,
so it runs on the Linux and macOS runners the tier already has. Measured in a cold Linux container
it took 49 s.

`--all-targets` fails. The test and bench targets pull in criterion, whose `alloca` build script
compiles C for the target with cc-rs and stops at `failed to find tool "lib.exe"`. Covering those
targets needs `cargo-xwin` or `clang-cl` plus the Windows SDK; that is a separate decision.

Both feature sets, as everywhere else in the tier: the feature-off line took 44 s cold.

Measured on Linux only. The macOS runner runs the same line in the merge tier; the build scripts
that run depend on the target, not the host, so the result should match.

### Follow-up

Running the tests on Windows is the next step and is not this ticket. Research, including the
runner and cross-compile-then-run options, is in `docs/research/windows-support.md`.
