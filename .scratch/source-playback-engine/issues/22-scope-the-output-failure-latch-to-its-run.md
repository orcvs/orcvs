# 22 — Scope the output failure latch to its run

**What to fix:** `PlaybackInner::record_output_failure` (`orcvs/src/playback.rs:734-740`) suppresses a
diagnostic that equals the last one it recorded, and `last_output_failure` is never cleared by
`begin_run` or `stop`. A device that fails the same way in two consecutive runs is reported once. The
second run reports nothing at all.

**Status:** resolved

- [x] A run that begins after a failed run reports that failure again if it recurs.
- [x] Within one run, a failure that repeats every Tick is still reported once — the de-duplication the latch exists for is kept.
- [x] The reset is stated where the other run-scoped inputs are reset, so the next run-scoped input added cannot be reset on one target and forgotten on another.
- [x] A regression test drives two runs against an adapter that fails identically in both, and asserts two `OutputFailure` diagnostics rather than one.

## Verification

`cargo fmt --all -- --check`, `cargo clippy --package <crate> --all-targets --locked -- -D warnings`,
and `PROPTEST_CASES=32 cargo nextest run --package <crate> --locked` for `orcvs` and `console`.

## Answer

`begin_run` clears `last_output_failure` beside `self.owned.clear()`, so the latch is scoped to the
run that recorded it. The comment at the reset states what the latch means — one report per run, not
one per Tick and not one per adapter lifetime — rather than leaving it to be inferred from where the
assignment sits.

**`stop` does not clear it.** The safety action `stop` sends is the last submission of the run that
recorded the latch, so a refusal repeating that run's failure is the same device fault, which is what
the latch exists to suppress; clearing before sending it would report one fault twice. A stopped
engine submits nothing, so a latch left standing past `stop` suppresses nothing until the next
`begin_run` clears it — which means `begin_run` alone is sufficient, and keeping it the single reset
site is what its docstring asks for: a run-scoped input reset in two places is one that can be
forgotten in a third.

## Comments

`record_output_failure` at `orcvs/src/playback.rs:734`:

```rust
fn record_output_failure(&mut self, error: OutputAdapterError) {
    if self.last_output_failure.as_ref() != Some(&error) {
        self.diagnostics
            .push(PlaybackDiagnostic::OutputFailure(error.clone()));
        self.last_output_failure = Some(error);
    }
}
```

The de-duplication is right and worth keeping: `execute_tick` calls this on every refused submission,
and a disconnected device would otherwise push one diagnostic per Tick for as long as Playback runs.
What is wrong is its scope. The latch is cleared in exactly two places — `orcvs/src/playback.rs:831`
when a submission succeeds, and `:960` when a destination is selected — and both are about the
adapter, not about the run.

The reproduction, entirely within one destination selection:

1. A run begins. A Tick submits, the adapter refuses with error `E`. `OutputFailure(E)` is pushed and the latch becomes `Some(E)`.
2. The run stops. `stop` sends the safety action, which fails with `E` as well; it is de-duplicated, so nothing is pushed. The latch still holds `Some(E)`.
3. A second run begins. `begin_run` bumps the generation, clears `last_tick_at`, resets the absolute Tick to zero and clears `owned` — and leaves the latch standing.
4. Every Tick of the second run submits and is refused with `E`. Every one is de-duplicated. The run produces no `OutputFailure` at all.

`PlaybackEngine::observe` drains the diagnostics with `std::mem::take` at `orcvs/src/playback.rs:868`,
so the first run's diagnostic has already been consumed and shown by the time the second run starts.
Nothing anywhere is still reporting the failure the user is now looking at.

`begin_run`'s own docstring at `orcvs/src/playback.rs:742-758` states the principle this violates:
every clock enters a run through that one place so that "a run-scoped input added here cannot be
reset on one target and forgotten on another". `last_output_failure` is a run-scoped input that was
never added there. The fix is to clear it in `begin_run` beside `self.owned.clear()`, with a comment
saying what the latch is for — one report per run, not one report per adapter lifetime — so the
distinction it draws is stated rather than inferred from where the assignment happens to sit.

Whether `stop` should clear it too is the question worth deciding rather than assuming. Clearing in
`begin_run` alone is enough for the defect above and keeps the latch's meaning simple: it is scoped
to the run that recorded it. Clearing in `stop` as well would mean the safety action's own failure
during `stop` is reported even when it repeats the failure that just happened mid-run, which is a
second report of one device fault rather than a new one.

The defect is not reachable from the `console` UI without a device that fails, so it needs a fake
adapter. `orcvs/src/playback.rs` already has the ones the surrounding tests use; the regression test
should drive two full runs through `begin_run` rather than calling `record_output_failure` twice, so
that it fails if the reset is put somewhere that is not on the path a run actually takes.

**2026-09-11 — fixed test-first (agent)**

`playback::tests::a_run_that_begins_after_a_failed_run_reports_the_failure_again` drives two runs
through `begin_run` (via `activate_for_test`, which calls it exactly as `start` does), two refused
Ticks each, with `stop` between them. The adapter is a new test-only `RefusingOutputAdapter` that
refuses every submission and every safety action with the same error, since
`InMemoryOutputAdapter::fail_next_submission` arms only one refusal and the latch de-duplicates a
failure that keeps happening. The expected two diagnostics pin both halves at once: one per Tick
would be four, one per adapter lifetime would be one, and a reset placed in `stop` ahead of the
safety action would be three.

Before the fix, with the test in place and `begin_run` unchanged:

```text
thread 'playback::tests::a_run_that_begins_after_a_failed_run_reports_the_failure_again' panicked at orcvs/src/playback.rs:3213:9:
assertion `left == right` failed
  left: [OutputFailure(OutputAdapterError { message: "device lost" })]
 right: [OutputFailure(OutputAdapterError { message: "device lost" }), OutputFailure(OutputAdapterError { message: "device lost" })]
```

One diagnostic for two runs of a device that never stopped failing — the defect exactly as filed.
The single added line in `begin_run` turns it green.

Verification, all passing:

```sh
cargo fmt --all -- --check
cargo clippy --package orcvs --all-targets --locked -- -D warnings
cargo clippy --package console --all-targets --locked -- -D warnings
PROPTEST_CASES=32 cargo nextest run --package orcvs --locked
PROPTEST_CASES=32 cargo nextest run --package console --locked
node --test scripts/tests/roadmap.test.ts
node scripts/roadmap.ts > /dev/null
```

The last two because this change edits `.scratch/`. The whole-workspace and merge-tier gates are
deferred to CI.
