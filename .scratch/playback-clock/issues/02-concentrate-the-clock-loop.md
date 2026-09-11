# 02 — Concentrate the clock loop

**What to build:** One clock loop per target instead of four, with the deadline rule as
a single shared function rather than two implementations that must agree.

**Blocked by:** `playback-clock/01` — the behaviour change lands first, so this one is a
refactor with no behaviour to argue about.

**Status:** resolved

- [x] `start` and `retune` no longer each carry a clock loop; they differ in the first
      deadline they supply and in whether they begin a run.
- [x] The deadline rule is computed in one place for both targets. The native clock no
      longer delegates it to `tokio::time::Interval`'s missed-tick machinery, so the
      only remaining per-target difference is how to wait until an instant.
      Landed with `playback-clock/01`: holding one rule on both targets forced it.
- [x] Cancellation, the `ClockRunGuard`, the `Weak` upgrade, and Tick delivery are
      written once.
- [x] `start`'s immediate first Tick and `retune`'s anchoring to the last executed Tick
      both survive, with tests naming them.
- [x] No behaviour changes beyond the elapsed-browser-wait correction below.
      Effort 01's tests pass with the agreed zero-delay test replacement.

## Comments

Two architecture reviews reached this from opposite directions. The 2026-09-09 review
("Give the Playback clock a real seam") wanted a `Clock` trait with tokio, browser and
manual adapters. The 2026-09-10 review ("Concentrate Playback clock execution") wanted
no trait at all, and treated the two targets' timing as behaviour to preserve.

Effort 01 settles that: there was no target-specific behaviour to preserve, which
removes the reason the second review rated this second-wave work. What the reviews both
missed is that the thing worth sharing is the deadline rule, not the loop scaffolding —
the scaffolding is only how the rule came to be written twice.

The manual test adapter the first review proposed is out of scope; see the spec.

One box arrived already ticked. Effort 01 could not make both targets hold one rule
without abandoning `tokio::time::Interval`, so the shared `next_scheduled_at` and the
`sleep_until` loops landed there. What remains here is the scaffolding: four loop
bodies, four copies of cancellation, the `ClockRunGuard`, the `Weak` upgrade and Tick
delivery.

Concentrating those loops is also what would let one test cover both targets. Today
nothing in the workspace compiles the browser loops, so the rule is shared by
construction — both call `next_scheduled_at` — rather than by a test that would fail if
they stopped. See the note on effort 01.

Three corrections from the review of effort 01, which found two deadline defects and
fixed both by hand where this effort would have made one of them unwritable.

The loop shapes are not a decision to preserve. Browser `start` is execute-then-sleep
and the other three loops are sleep-then-execute, and nothing in ADR 0037, `CONTEXT.md`
or any comment says why. `28e22e5` hand-ported the native `time::interval_at` loop,
whose first `tick()` returns immediately, and reproduced that by moving the sleep to the
bottom; `bcc95c7` then wrote browser `retune` in the native shape, because its first
deadline is a period away rather than zero. The shape follows mechanically from whether
the first deadline is `ZERO`, so the browser already holds both. It is the same
unrecorded transcription artefact ADR 0037 was written about. What is real underneath it
is one platform fact: `tokio::time::sleep_until` on an elapsed deadline is `Ready` on
first poll, while `TimeoutFuture::new(0)` costs a `setTimeout` hop of one to four
milliseconds — enough for `is_overrun` to decline the first Tick of a browser run at the
one-millisecond end of `Bpm`. State the wait as "a deadline already reached costs no
wait" and both targets hold the sleep-then-execute shape with no semantic change. That
narrows the always-schedule-a-timer rule `0ab1a4c` introduced, so
`zero_wasm_delay_still_schedules_a_browser_timer` has to be replaced by one pinning
sub-millisecond rounding. The criterion above that asks for effort 01's tests to pass
unaltered has that one exception.

The seam is two free functions and a spawner value, not a trait. A `sleep_until` per
target, mirroring how `ClockInstant` is already one alias per target, plus a spawner
obtained before the lock is taken — native `retune` proves the runtime present before it
cancels the previous clock and bumps the generation, and a spawn that failed after that
would leave a playing engine with no clock. `Send` is not a constraint: both loops
already live in `impl<A: OutputAdapter + Send + 'static>`, and a shared `async fn` takes
its auto-traits per instantiation, so the native one satisfies `tokio::spawn` and the
browser one does not have to.

The note above about nothing compiling the browser loops is stale, and it understates
the real gap. `check_wasm` runs `cargo clippy --workspace --all-targets --target
wasm32-unknown-unknown` on every pull request, so both loops type-check, and
`console/tests/wasm.rs` drives `PlaybackEngine::start` in headless Firefox in the merge
tier. What is missing is narrower and worse: no test on any target asserts a browser
deadline, and browser `retune` — where effort 01's second defect lived — is executed by
nothing at all, because it is `pub(crate)` and `wasm.rs` never calls it. This effort
should add a deadline assertion to `console/tests/wasm.rs` and a path that drives `retune`
there. Concentrating the loops closes the divergence; it does not close the sleep
primitive itself, where `wasm_timeout_millis` rounding, `setTimeout` clamping and
background-tab throttling stay browser-only and native-untestable.

## Implementation and verification — 2026-09-11

`start` and `retune` now share `run_clock`, which owns cancellation, the
`ClockRunGuard`, the `Weak` upgrade, Tick delivery and advancement of the Tick
Grid. The per-target `sleep_until` functions and `ClockSpawner` hold only the
platform differences. Native runtime acquisition still precedes retiring the
previous clock, and retune keeps the current run's absolute Tick. An elapsed
browser deadline returns without scheduling a timer.

The 60 Playback tests passed before and after the refactor. The renamed rounding
test pins waits just below, at, and just above a millisecond. Two browser tests
were added at the existing public interfaces: `PlaybackEngine::start` must execute
before a browser timer, and `Orcvs::set_bpm` must retain its anchored deadline and
resume on the same grid after a stall. All 11 headless Firefox tests passed.
The browser run was local to answer the platform-specific waiting question, not
to duplicate a merge-tier gate. The test-only JavaScript bindings pass numeric
timestamps through wasm-bindgen and introduce no handwritten unsafe code.

### Standards review

No actionable documented-standard breaches or worthwhile introduced smells.
Cancellation and stale-generation protection remain inside the Playback Engine;
no mutex guard spans an await. Native spawning retains `Send + 'static`, while
the browser spawner accepts its platform future. No shipped test-only input,
public-interface change, dependency, feature, or atomic change was introduced.

### Spec review

No missing, partial, unrequested, or incorrectly implemented requirements found.
The shared loop, pre-acquired spawner, platform waiting seam, browser elapsed-wait
correction, existing lifecycle tests and added browser coverage match this ticket.
Both reviews compared the implementation against branch base `68e5eba`.

### Commands

Rust compilation used the worktree's own `target/`, with
`CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0` to keep
build artefacts small, and `PROPTEST_CASES=32` as required for local verification.

- `cargo fmt --all -- --check` — passed.
- `git diff --check` — passed.
- `cargo nextest run --package orcvs --locked -E 'test(playback::tests::)'` — passed
  before and after consolidation: 60 tests in each run.
- `wasm-pack test --headless --firefox console --test wasm --locked` — passed: 11 tests.
- `cargo clippy --workspace --all-targets --locked -- -D warnings` — passed.
- `cargo nextest run --workspace --locked` — passed: 631 tests.
- `cargo test --workspace --doc --locked` — passed: 13 doctests, including five
  compile-fail examples.
- `node --test scripts/tests/roadmap.test.ts` — passed: 10 tests.
- `node scripts/roadmap.ts > /dev/null` — passed.
- `mise run check_wasm` — passed: WASM Clippy over all targets and Trunk builds
  with persistence enabled and disabled.

The first native attempt could not run until the new worktree was trusted by
`mise trust`; the subsequent sandboxed attempt was refused by the configured
compiler cache. Rerunning outside the sandbox passed. No test failed in those
setup attempts because neither reached test execution.

### Remaining verification and risks

The combined `mise run check`, `mise run check_merge`, `mise run bench`, other CI
feature combinations, and 256-case proptest are deferred to CI. Workspace gates
cover the changed `orcvs` crate and its dependent `console` crate. No performance
improvement is claimed. The change's risks are lifecycle concurrency and platform
waiting, covered by existing native tests and the browser tests; the browser
anchoring assertion allows 150 ms of dispatch jitter.

## Review — 2026-09-11

Three reviewers ran against `68e5eba`. CodeRabbit returned no findings; the
standards/spec pass and the built-in review converged independently on one
defect, and each reported it from its own side.

The elapsed-deadline correction was applied to every iteration, not only to the
first Tick the Comments above argued it for. Since `observed_at` is sampled
before the Tick executes, a Tick costing more than its period leaves the next
deadline already behind the clock, so the browser wait was skipped again and
again and the loop ran Tick after Tick without returning to the event loop. The
burst ends at the first Overrun — a declined Tick is cheap and re-anchors the
grid ahead of the clock — so this froze the page for `period / (cost - period)`
Ticks rather than forever, but every one of those Ticks was a frame not drawn
and an input not dispatched. Both browser loops on `main` awaited their timer
unconditionally, which is what the deleted zero-delay test pinned.

The shortcut now belongs to `run_clock`, which spares the run's first deadline
only, and `sleep_until` waits on every deadline it is given. `retune`'s anchored
first deadline is usually ahead of its epoch and waits like any other.

`console/tests/wasm.rs` gained `web_clock_yields_to_the_event_loop_between_ticks`,
which drives a 100 ms grid through an adapter that spends 115 ms of the browser
thread per submission and asserts the page gets a turn. It reported 6 Ticks
against the consolidated loop before the fix and 1 after.

`web_start_executes_its_first_tick_before_a_browser_timer` was decided by about
a millisecond, racing the clock's `setTimeout(1)` against the test's own
`setTimeout(0)`. The ordering held — gloo registers the test's timer before the
`spawn_local` microtask registers the clock's — but a ten-second period draws
the same distinction with no race and no reliance on that ordering.

ADR 0037 still said no test in the workspace compiled the browser loops, which
this effort made false in both halves. Its shared-rule paragraph now names the
one loop and the browser tests, and says they are merge-tier only.

Rejected: the rationale comments deleted from `retune` (they survive verbatim on
`begin_run` and `first_retuned_tick_at`, where the reasoning is computed); the
placement and prose of this report (`## Resolution` after `## Comments` is the
form every resolved issue under `.scratch/verification-gaps/` takes); the
`tokio::select!` branch order against an already-cancelled token (`execute_tick`
declines on `!playing` or a stale generation); and the `ClockSpawner` indirection,
the `Option<ClockInstant>` first deadline and doc-comment length, none of which
carry a failure.

### Commands

- `cargo fmt --all -- --check` — passed.
- `cargo clippy --workspace --all-targets --locked -- -D warnings` — passed.
- `cargo nextest run --workspace --locked` — passed: 631 tests.
- `cargo nextest run --workspace --tests --no-default-features --locked` — passed:
  617 tests, the arm that proves `persistence` still compiles out.
- `cargo test --workspace --doc --locked` — passed: 13 doctests.
- `mise run check_wasm` — passed.
- `wasm-pack test --headless --firefox console --test wasm --locked` — 11 passed
  with the new test red at 6 Ticks, then 12 passed. Run to answer the
  browser-waiting question the fix turns on, not to stand in for a merge gate.

`mise run check`, `mise run check_merge`, `mise run bench` and the 256-case
proptest stay deferred to CI.

## Rebase — 2026-09-11

Rebased onto `main` after the `shell` crate was renamed to `console`. The only
conflict was the import block in `console/tests/wasm.rs`, where this branch's
`std::sync` imports met the renamed `console::web_startup` import. Every path
this branch newly writes now names `console`; ADR 0037's Overrun paragraph keeps
`shell/src/diagnostics.rs` because it is dated text this branch does not touch,
which is how the rename issue left ADRs 0022 and 0037.

- `cargo fmt --all -- --check` — passed.
- `cargo clippy --package orcvs --all-targets --locked -- -D warnings` — passed.
- `cargo clippy --package console --all-targets --locked -- -D warnings` — passed.
- `cargo nextest run --package orcvs --package console --locked` — passed: 410 tests.
- `mise run check_wasm` — passed, which is what compiles `console/tests/wasm.rs`
  after the conflict resolution.

`mise run test_wasm` was not rerun; the resolution changed imports only and
`check_wasm` compiles the target. The browser suite is merge-tier and deferred
to CI.
