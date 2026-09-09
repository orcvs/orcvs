# 10 — Own Tick-local execution transitions

**What to build:** Give Tick-local execution one module that owns its changing computation state,
working Source, and pending Effects. The outer planner obtains a dependency schedule and asks for
its execution; it does not coordinate activation, suppression, typed answers, and spatial writes.

**Blocked by:** 09.

**Status:** resolved

- [x] Execution state is private to the execution module; the caller receives a Tick Plan.
- [x] Turn admission, syntax propagation, and operand consumption read the same state.
- [x] A successful typed answer survives rejected spatial delivery, and a child's admitted write
      survives its parent's failure.
- [x] Replacement and suppression update later Turns together with working Source and Effects.
- [x] A spatial write reaching an attempted Turn rejects all writes and Play Commands for the
      Tick, preserves diagnostics, and does not execute later Turns.
- [x] Existing behavior tests pass unchanged, with a focused test for defensive execution rejection.

## Comments

### Confirmed design, 2026-09-09

The architecture discussion confirmed that execution owns the transition rules together with
working Source and pending Effects. Merely grouping six vectors would leave callers coordinating
the same rules. The implementation is a private `tick::execution` module whose one entry takes
the schedule and Source inputs and returns the publishable Tick Plan.

Lookup retains the immutable original computations established by the Parser. Scheduling still
decides dependency order; Portal still admits complete writes; the Evaluator still answers typed
Function evaluation. The new module interprets neither spellings nor scheduling relationships
independently of those owners. This builds on issue 09's Lookup ownership change, PR #41.

One computation's changing facts live together, but remain independent. A Turn is attempted before
syntax and evaluation checks, so even a syntax-blocked or failed Turn prevents a later spatial
writer reaching it. Activation is independent of outcome. A successful typed answer is retained
before spatial delivery, so a delivery diagnostic does not erase it. These are not exclusive
states of one lifecycle enum.

The existing Source/Tick tests remain the test surface. The new regression supplies a deliberately
broken order at the internal execution seam, because the scheduler cannot legitimately produce
one: successful, failed, and syntax-blocked targets all force rejection when a later writer reaches
them. The fixture first proves that its valid order produces writes and a Play Command. The broken
order must discard both, retain ordered diagnostics, and stop before the remaining Turn. This test
passed against the previous execution loop before the refactor.

ADRs 0032 and 0034 remain in force. No language semantics, public interface, unsafe, concurrency,
dependencies, features, or platform-specific code change. No performance improvement is claimed;
benchmark comparisons and the full platform/feature matrix remain CI's responsibility.

### Verification, 2026-09-09

- `RUSTC_WRAPPER= PROPTEST_CASES=32 cargo nextest run --package orcvs --locked -E 'test(a_late_spatial_write_rejects_the_tick_even_when_the_earlier_turn_failed)'`
  — the new regression passed against the old execution loop before its extraction.
- `cargo fmt --all -- --check` — passed.
- `RUSTC_WRAPPER= cargo clippy --package orcvs --package shell --all-targets --locked -- -D warnings`
  — passed after the refactor.
- `RUSTC_WRAPPER= PROPTEST_CASES=32 cargo nextest run --package orcvs --package shell --locked`
  — failed during build preparation with `No space left on device`.
- `RUSTC_WRAPPER= CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 cargo clippy --package orcvs --package shell --all-targets --locked -- -D warnings`
  — passed after clearing this checkout's generated target directory.
- `RUSTC_WRAPPER= CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 PROPTEST_CASES=32 cargo nextest run --package orcvs --package shell --locked`
  — all 335 tests passed, including the new three-case regression and every existing test unchanged.
- `node --test scripts/tests/roadmap.test.ts` — all 10 tests passed.
- `node scripts/roadmap.ts > /dev/null` — passed.
- `git diff --check` — passed; complete diff and the new execution module reviewed.

The compiler cache remains disabled because of the previously observed sccache permission error.
Debug symbols and incremental compilation were disabled for the final Rust gates to fit local
disk space; debug assertions and the declared toolchain remain unchanged. No repository build
settings were edited.

Not run: persistence, Linux, and WASM combinations — deferred to CI; no persistence, feature, or
platform-specific code changed. `mise run check`, `mise run check_merge`, `mise run test_wasm`,
`mise run bench`, and proptest's 256-case default — deferred to CI.

### Pre-PR verification, 2026-09-09

- `RUSTC_WRAPPER= CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 cargo clippy --workspace --all-targets --locked -- -D warnings`
  — passed.
- `RUSTC_WRAPPER= CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 PROPTEST_CASES=32 cargo nextest run --workspace --locked`
  — all 524 tests passed.
- `RUSTC_WRAPPER= CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 cargo test --workspace --doc --locked`
  — all 9 doctests passed.

The pull request is stacked on PR #41's published Lookup ownership change. The build settings
above retain the disk-space workaround from the implementation checks.
