# 01 — Lint the shipped crates for Windows

**What to build:** A pull-request gate that compiles the shipped crates for
`x86_64-pc-windows-msvc`, so a break behind `cfg(windows)` fails the pull request that wrote it
instead of reaching `main` unseen. No CI job runs on Windows today.

**Blocked by:** None — can start immediately.

**Status:** resolved

- [x] `rust-toolchain.toml` declares `x86_64-pc-windows-msvc`, so rustup installs it with the pinned toolchain.
- [x] `mise run check_windows` runs `cargo clippy --workspace --lib --bins --target x86_64-pc-windows-msvc --locked -- -D warnings`, and `check_pull_request` calls it.
- [x] `scripts/check-tooling-contract.sh` pins the task and its call, and the fixture suite proves removing the call is rejected.
- [x] `docs/tooling.md` records what the line covers and why it is scoped to the shipped crates.

## Comments

### Scope

The line is clippy, a check build: it neither links nor needs the MSVC linker or the Windows SDK,
so it runs on the Linux and macOS runners the tier already has. Measured in a cold Linux container
it took 49 s.

`--all-targets` fails. The test and bench targets pull in criterion, whose `alloca` build script
compiles C for the target with cc-rs and stops at `failed to find tool "lib.exe"`. Covering those
targets needs `cargo-xwin` or `clang-cl` plus the Windows SDK; that is a separate decision.

Default features only. `persistence` turns eframe's Windows storage path on; turning it off removes
Windows code rather than adding any.

### Follow-up

Running the tests on Windows is the next step and is not this ticket. Research, including the
runner and cross-compile-then-run options, is in the project's `research/windows-support.md`.
