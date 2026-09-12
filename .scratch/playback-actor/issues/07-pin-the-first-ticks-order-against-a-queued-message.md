# 07 — Pin the first Tick's order against a queued message

**What to build:** A test that a message already queued when a run begins is applied before that
run's immediate first Tick, not after it.

Two mechanisms enforce it and neither is observed. `next_playback_event` answers a first deadline
that is already reached without awaiting, by taking a queued message first (`orcvs/src/playback.rs:1212`),
and the `tokio::select!` below it is `biased;` (`:1219`, in the select at `:1218-1225`) so a message
wins a tie. Removing `biased;`, and separately making `due_on_arrival` return `Deadline`
unconditionally, were each reported to leave 70/70 tests passing. No test queues a `Retune`,
`Disconnect` or `select_midi_destination` before a run's first Tick.

It matters more than the mutation score suggests. A `Disconnect` or a destination change applied a
Tick late delivers that first Tick to the destination the user has just left, and nothing in the
system notices — no diagnostic, no state change, just a note on the wrong device.

**The caveat has to be recorded with the test.** Without `biased;`, `tokio::select!` randomises poll
order, so a single green run is weak evidence about a tie that only matters when both arms are
ready. Settling it means repeated runs, and even then one pass is not decisive. The `try_recv` path
at `:1212` is the deterministic half and is the one a test can pin outright.

**Status:** resolved

- [ ] A test queues a message before a run's immediate first Tick and asserts the message was applied
      to that Tick, not to the one after it.
- [ ] It goes red with `due_on_arrival` returning `Deadline` unconditionally.
- [ ] What the test does and does not prove about `biased;` is stated in its doc: a green run under a
      randomised poll order is evidence, not proof, and the number of repeats is part of the claim.

## Verification

`cargo fmt --all -- --check`, then `cargo clippy --package orcvs --all-targets --locked -- -D
warnings` and `PROPTEST_CASES=32 cargo nextest run --package orcvs --locked`. A tie between two ready
arms is exactly what the merge tier's multi-threaded and second-platform runs probe; those are
deferred to CI.

## Comments

Filed from the `playback-actor` review ledger as **CR-06** (major).

ADR 0041 states that a run "still executes its first Tick immediately" and ticket 04's Comments call
the immediate first Tick a rule the browser cares about. Neither says what happens to a message that
was already in the queue when that Tick came due, and that is the case with a user-visible wrong
answer.

### Resolved

`a_message_queued_before_a_run_begins_is_applied_to_its_first_tick` queues a
`Start` and a `Disconnect` before the task is polled at all, and asserts the
run's immediate first Tick delivered nothing — the disconnect was applied
first.

Red under the mutation this ticket names: with the `due_on_arrival` branch
returning `Deadline` unconditionally, the first Tick is delivered to an output
the queue had already closed behind it. Run against the whole suite, that
mutation is now caught by this test and by
`midi::tests::selecting_a_destination_after_disconnect_restores_output`.

The doc states what the test does and does not prove: it pins the
deterministic `try_recv` half only, and says nothing about the `biased;` tie,
where a randomised poll order makes a green run evidence rather than proof.
