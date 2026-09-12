# 05 — Close the abort window in the engine state's destructor

**What to build:** `Drop for PlaybackInner` (`orcvs/src/playback.rs:829-839`) can turn a recoverable
adapter panic into a process abort, because on exactly the path it was written for it calls back
into the adapter that panicked while that panic's unwind is still in progress.

The destructor's own doc names its reason as "a panic unwinding out of an adapter". On that path it
runs *during* the unwind and `:837` calls `self.stop()` — `PlaybackInner::stop` (`:672-685`) reaches
`send_safety_reset()` at `:676`, which calls `self.adapter.safety_reset()` at `:703`. `OutputAdapter`
(`:69-91`) is a public trait with no no-panic contract, so a panic there is a panic in a destructor
during unwinding: the process aborts, instead of tokio catching the task panic and the engine
reporting `ClockFailure`.

It is reachable. `run_engine` owns `inner` by value (`:1238`), so a panic out of `inner.execute_tick`
(`:1292`) unwinds through the drop of `inner` before tokio's task-level catch ever sees it.

The suite cannot see it either. `PanickingOutputAdapter` (`:1541-1548`) panics in `submit` but
returns `Ok(())` from `safety_reset`, so `clock_failure_remains_observable_after_output_panics`
(`:2944`) exercises only the half that cannot abort.

**This is pre-existing, not introduced by this branch, and should be prioritised on that basis.**
At `25c8de8`, `impl Drop for ClockRunGuard` (`orcvs/src/playback.rs:1202-1218`) did the same thing:
it reported the identical `"Playback clock terminated unexpectedly"` message and then called
`inner.stop()` into `adapter.safety_reset()`. ADR 0041 moved the destructor and deleted the guard;
it did not open this window.

A `std::thread::panicking()` check, or a `catch_unwind` around the safety action, closes it. Neither
is free: skipping the safety action during an unwind leaves a note sounding, which is the outcome
the destructor exists to prevent, so whichever is chosen needs its reason written beside it.

**Status:** resolved

- [ ] A panic out of `OutputAdapter::safety_reset`, reached through the drop of `PlaybackInner`
      during an unwind started by the same adapter, does not abort the process.
- [ ] A test stages it. Today's `PanickingOutputAdapter` cannot: its `safety_reset` returns `Ok(())`.
- [ ] `clock_failure_remains_observable_after_output_panics` still holds — the `ClockFailure` report
      at `:834` happens before the stop at `:837`, so closing this window must not cost the report.
- [ ] The choice between suppressing the safety action and catching its panic is recorded beside the
      code, because the trade is audible: a suppressed safety action leaves a note sounding.

## Verification

`cargo fmt --all -- --check`, then `cargo clippy --package <crate> --all-targets --locked -- -D
warnings` and `PROPTEST_CASES=32 cargo nextest run --package <crate> --locked` for `orcvs` and
`console`. A test that stages a double panic is a concurrency test; the multi-threaded runs on the
merge tier are the ones that matter and are deferred to CI.

## Comments

Filed from the `playback-actor` review ledger as **CR-02** (major).

The original report presented this as a defect of this branch. It is not, and the issue above says
so: `25c8de8`'s `ClockRunGuard` destructor had the same shape. What ADR 0041 changed is where the
destructor lives, not whether it re-enters the adapter mid-unwind.

Same module, same class of hazard, noted rather than filed: `lock_recover` (`:93-97`) exists for the
poisoned-lock case, and `InMemoryOutputAdapter` does not use it at `:294`, `:298`, `:302`, `:308` or
`:317`. That is a test-only adapter, so the failure mode there is a failed test rather than a failed
application, which is why it is a note and not its own ticket.

### Resolved

`Drop for PlaybackInner` now attempts the safety action inside
`std::panic::catch_unwind(AssertUnwindSafe(..))` and reports
`ClockFailure { message: "Playback output could not be silenced" }` when the
attempt panics. The choice — attempt and contain, rather than suppress — is
recorded beside the code, along with why `AssertUnwindSafe` is sound for state
being dropped.

`DoublyPanickingOutputAdapter` stages it: it panics on `submit` and again on
`safety_reset`, which is the device a panicking backend actually presents.
Before the fix the test aborted the process with `SIGABRT` — "panic in a
destructor during cleanup" — so this was demonstrated rather than reasoned
about. `clock_failure_remains_observable_after_output_panics` still holds.
