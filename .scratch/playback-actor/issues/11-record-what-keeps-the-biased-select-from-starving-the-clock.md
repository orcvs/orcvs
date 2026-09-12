# 11 — Record what keeps the biased select from starving the clock

**What to decide:** The `biased;` in `next_playback_event` (`orcvs/src/playback.rs:1218-1225`) makes
the message arm win every tie. Under sustained message load the deadline arm never gets polled:
Ticks stop, and because the deadline is never *reached*, `is_overrun` never runs — so playback halts
with no diagnostic at all. Decide whether that is a hazard to remove or a constraint to record.

It is latent, not live. The only producer is the console, which sends on button clicks and slider
commits; a human cannot generate messages faster than a Tick period. But nothing in the tree records
that as the reason the path is unreachable, so the constraint is invisible to the next author who
adds a producer — a MIDI clock input, a script driving the engine, an automation lane — and the
failure mode it opens is the worst kind: silent, with the state still published as `Playing`.

The two candidate answers are different in cost. Recording the constraint is a comment at `:1218`
saying what `biased;` buys, what it costs, and which property of the producers keeps the cost
unreachable. Removing the hazard means a fairness budget or a periodic unbiased poll, which trades
away the tie-break that `07` is about to pin.

**Status:** needs-triage

**Sources of truth:** `orcvs/src/playback.rs:1218-1225` (the select), `orcvs/src/playback.rs:1212`
(`due_on_arrival`, the other half of the message-first rule), ticket 04's Comments (why `biased` was
chosen).

- [ ] The decision is recorded, either as a comment at the select or as this ticket's answer.
- [ ] If `biased;` stays, what makes the starvation unreachable is written down as a constraint on
      producers rather than left as a property of today's only producer.
- [ ] If it goes, `07`'s guarantee about a queued message and the first Tick is preserved.

## Verification

`cargo fmt --all -- --check`, then `cargo clippy --package orcvs --all-targets --locked -- -D
warnings` and `PROPTEST_CASES=32 cargo nextest run --package orcvs --locked`.

## Comments

Filed from the `playback-actor` review ledger as **CR-12**, minor.

Minor because it cannot happen with the producers that exist. Filed because the thing missing is not
a fix but a record: ADR 0040 made the loop the single place Ticks come from, and `biased;` is the one
line that can stop them without saying so.

Related: `07` pins the ordering `biased;` exists to provide, so the two should be read together.

### The branch has taken one of the two answers without the triage

`MESSAGES_BEFORE_A_DEADLINE = 64` and a `messages_since_tick` counter now flip
the `tokio::select!` bias once the queue has had its share, so this ticket's
second candidate — remove the hazard with a fairness budget — is what shipped.
That was decided inside a review fix rather than here, and the number was never
this ticket's to skip.

What is measured. Six threads flooding idempotent `start` calls at a
one-millisecond period, over 200ms in which ~200 Ticks are due: 18 delivered
unbounded, 59 at a limit of 64. So the bound restores progress and does not
restore the rate. A sweep of 16, 8 and 4 was started and abandoned; nothing
below 64 has a measurement behind it, and 64 is the only value the code's own
comment argues for.

`a_run_keeps_ticking_while_messages_keep_arriving` originally sampled the
delivery count after a fixed 200ms window, and failed about one run in twelve
on a loaded machine — the bound says a deadline gets its turn once the queue
has had its share, and says nothing about when, which a window silently asserts.
It now waits for the first delivery instead, against the harness budget. The
property it states is the one the code guarantees; the rate is this ticket's.

`07` is resolved, so the constraint that answer had to preserve — a message
already queued is applied before the run's first Tick — is now observed rather
than assumed.

