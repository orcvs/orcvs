# 18 — Take the blocking hazards out of the Playback test harness

**What to build:** Bounds on the test-harness waits in `orcvs/src/playback.rs`, so that a harness
that goes wrong fails as a red test rather than as a watchdog kill.

**Unbounded waits that hold Tokio workers.** `BlockingOutputControl::wait_for_delivery`
(`:1506-1512`) is a `Condvar::wait_while` with no timeout. So is `BlockingOutputAdapter::submit`
(`:1558-1560`) — and that is the one that matters, because it blocks a Tokio worker *inside the
adapter* until `release_delivery` is called. An earlier report named only the first. `:2956-2958` is
an unbounded `yield_now` spin on an `AtomicBool`.

All of these sit in `#[tokio::test(flavor = "multi_thread", worker_threads = 2)]` tests that block
Tokio workers with std synchronisation. They work by work-stealing. If one ever misses — two workers,
both parked in std primitives — the failure mode is a watchdog kill with no failing assertion, which
is the hardest kind of red to read.

**Wall-clock budgets, separately.** `for _ in 0..1_000 { time::sleep(1ms) }` at `:2889` and `:2985`
are genuine ~1s budgets, on runtimes that are not `start_paused`. On the macOS merge tier under
nextest parallelism this is the tightest timing margin in the file, and its signature is "fails once,
passes on re-run" — the flake that gets re-run rather than read.

A timeout on the condvar waits, or moving the blocking adapter onto `spawn_blocking`, closes the
first set. The second set needs either a larger budget with the reason stated or a signal to wait on
instead of a poll.

**Status:** resolved

- [ ] Neither condvar wait can block forever; a harness that is never released fails the test.
- [ ] The blocking adapter no longer occupies a Tokio worker for the duration of a submission, or it
      is documented why work-stealing across two workers is sufficient and what would break it.
- [ ] The `yield_now` spin at `:2956-2958` is bounded.
- [ ] The two ~1s polling budgets either carry a stated margin for the macOS tier under parallelism,
      or wait on a signal instead of polling.
- [ ] The tests still assert what they assert now; this changes how they fail, not what they prove.

## Verification

`cargo fmt --all -- --check`, then `cargo clippy --package orcvs --all-targets --locked -- -D
warnings` and `PROPTEST_CASES=32 cargo nextest run --package orcvs --locked`. The condition being
guarded against is a scheduling one on a second platform under parallelism; that is the merge tier
and it is deferred to CI. A local run passing proves nothing about it.

## Comments

Filed from the `playback-actor` review ledger as **CR-19**, minor.

Minor because it is test-only. Filed because the failure it produces is not a failing assertion:
`docs/tooling.md` records what each tier covers, and neither tier reports "a worker deadlocked" as
anything but a timeout. `16` records that the same tier is where thirty new runtime-bearing console
tests also landed.

### Resolved

Every wait in the harness is now bounded by one named budget,
`HARNESS_TIMEOUT` (five seconds), whose doc says it is a bound on failure
rather than a schedule and why five seconds.

Both condvar waits use `wait_timeout_while` and assert they were not the ones
to time out. Three unbounded `yield_now` spins on `delivery_started` and four
`for _ in 0..1_000 { sleep(1ms) }` budgets are now one `wait_until` helper that
polls against a `std::time::Instant` deadline — std rather than the runtime
clock, so the budget cannot move if anything pauses time.

Verified the way the first acceptance asks: with `release_delivery` made a
no-op and the budget cut to 200ms, `dropping_the_final_handle_during_a_tick_completes_playback_safety`
and `stop_returns_without_waiting_for_a_tick_and_no_tick_follows_it` fail in
0.3s with "the delivery was never released", instead of hanging to a watchdog
kill.

The blocking adapter still occupies a Tokio worker, and that is now documented
rather than fixed: the seam is a synchronous trait method the engine's task
calls inline, so holding it is the only way to stage "the engine is mid-Tick" —
`spawn_blocking` would move a different call. What makes two workers enough,
and the two things that would break it — a third party wanting a worker at that
moment, or a second engine held here concurrently — are written beside it.
