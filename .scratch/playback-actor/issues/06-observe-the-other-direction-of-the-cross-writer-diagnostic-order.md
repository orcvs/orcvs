# 06 — Observe the other direction of the cross-writer diagnostic order

**What to build:** A test that fails when a task-recorded diagnostic is drained *before* a
handle-recorded one that preceded it. Today only the opposite direction is observed.

`reports` is a clone of the same `mpsc` sender the task writes into (`orcvs/src/playback.rs:619`), so
one queue carries both writers and ordering across them is the property worth pinning. Two tests do
assert it — `diagnostics_drain_in_order_and_exactly_once` (`:2636`, whose doc at `:2622-2634`
explicitly names "two different writers") and `start_is_idempotent_and_draining_takes_the_diagnostics`
(`:2685`) — and both assert `[StartFailure, OutputFailure, ..]`.

The gap is which mutations they catch. Give the handle's reports their own
`Arc<Mutex<Vec<_>>>` drained **first**, and both tests stay green: the handle-side `StartFailure` is
recorded first in each of them, so a handle-first drain reproduces the asserted order by accident.
Drain that same separate buffer **last** and both go red. So the accurate claim is narrow: the
task-report-before-handle-report direction is unobserved. The guarantee as a whole is not untested.

Settling it needs a case where the task records first and the handle records second, and an
assertion that the drain returns them in that order — for example an output failure or an Overrun
from a live run, followed by a refused `start`, drained once.

**Status:** resolved

- [ ] A test records a diagnostic from the task first and from the handle second, and asserts the
      drain returns them in that order.
- [ ] It goes red under a mutation that gives the handle its own buffer drained before the channel,
      which neither existing test catches.
- [ ] `diagnostics_drain_in_order_and_exactly_once` keeps its existing direction; this adds to it
      rather than replacing it.

## Verification

`cargo fmt --all -- --check`, then `cargo clippy --package orcvs --all-targets --locked -- -D
warnings` and `PROPTEST_CASES=32 cargo nextest run --package orcvs --locked`.

## Comments

Filed from the `playback-actor` review ledger as **CR-05**, downgraded to minor by audit.

The original report said the cross-writer ordering guarantee was unobserved and that a mutation
breaking it left 70/70 passing. That was overstated in both halves: two tests assert the ordering,
one of them says in its own doc that cross-writer ordering is its subject, and a "drained last"
mutation does turn them red. Only the one direction described above escapes.

### Resolved

`a_task_recorded_diagnostic_drains_before_a_handle_recorded_one` records an
`OutputFailure` from the task — the run's first Tick refused by the device —
and then a `StartFailure` from the handle, and asserts one drain answers in
that order.

The mutation this ticket names was applied and behaves exactly as predicted:
with the handle's reports in their own `Arc<Mutex<Vec<_>>>` drained ahead of
the channel, `diagnostics_drain_in_order_and_exactly_once` and
`start_is_idempotent_and_draining_takes_the_diagnostics` both stay green and
only the new test goes red.
