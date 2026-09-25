# 11 — Settle Playback fairness, including the first Tick

**What to decide:** Record the fairness policy for both branches of `next_playback_event` in `orcvs/src/playback.rs`. Sleeping deadlines already receive priority after `MESSAGES_BEFORE_A_DEADLINE = 64` messages. The immediate first Tick takes the separate `due_on_arrival` branch, which always tries another queued message and never applies that budget. Sustained arrivals can therefore postpone the first Tick indefinitely while observation says Playing, without reaching the Overrun check.

This is a sustained-producer risk, not a demonstrated failure under ordinary console interaction. The existing fairness test establishes eventual delivery in its tested run; it does not establish a bounded number of messages before the first Tick under a continuously nonempty queue. The resolved playback-actor/07 test establishes a separate requirement: a destination change or Disconnect already queued when the run begins must precede its first Tick.

**Blocked by:** None — policy decided below; implemented with source-audit/27 in one PR.

**Status:** resolved

Related: source-audit/27 bounds command admission and defines overload behavior. A memory bound alone does not prevent a producer from replenishing the queue forever. source-audit/02 preserves the stop-admission guarantee when deadline fairness overtakes a queued Stop.

**Sources of truth:** `next_playback_event`, `TickClock::due_on_arrival`, `BACKLOGS_BEFORE_A_DEADLINE` and `run_engine` in `orcvs/src/playback.rs`; playback-actor/07's first-Tick ordering regression.

- [x] Record the policy separately for the immediate first Tick and sleeping deadlines. State whether continuous arrivals may defer the first Tick indefinitely; if so, specify the supported producer constraint and how overload is observable.
- [x] If progress is required under sustained arrivals, define which queued messages must precede the first Tick and how later arrivals stop extending that set forever. Preserve playback-actor/07's destination-change and Disconnect ordering guarantee.
- [x] A deterministic test keeps the queue nonempty across more than the fairness budget before the first Tick and checks the chosen policy. It distinguishes initial backlog from subsequent arrivals and does not depend on a wall-clock delivery count.
- [x] Sleeping deadlines retain their tested progress guarantee, and a deadline taking priority over a queued Stop still encounters closed Tick admission.
- [x] State what the chosen message budget guarantees about progress and what it does not guarantee about wall-clock rate. Any claim that a different budget improves rate has a reproducible measurement.

## Verification

`cargo fmt --all -- --check`, then `cargo clippy --package orcvs --all-targets --locked -- -D
warnings` and `PROPTEST_CASES=32 cargo nextest run --package orcvs --locked`.

## Comments

Filed from the `playback-actor` review ledger as **CR-12**, minor.

Minor because it cannot happen with the producers that exist. Filed because the thing missing is not
a fix but a record: ADR 0041 made the loop the single place Ticks come from, and `biased;` is the one
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

**2026-09-25 — acceptance-criteria review against `199c3331`.** Reframed the active scope around the fairness budget already shipped and the immediate-first-Tick exception it excludes. The historical measurements above are not evidence of bounded first-Tick progress. Linked command-queue capacity to source-audit/27 while keeping scheduling policy here.

**2026-09-25 — triage: first-Tick policy decided.** Implement with source-audit/27 in one PR (epic PR 4).

- The immediate first Tick applies only the backlog that existed when the run began, then takes its turn. Messages arriving after that point cannot defer it. The run's start marks the boundary, for example with a count of queued messages or a generation mark; the implementation records which it uses.
- The boundary preserves playback-actor/07: a destination change or Disconnect already pending when the run begins precedes the first Tick. With source-audit/27's slots, a pending destination is always applied first.
- After the first Tick, sleeping deadlines keep `MESSAGES_BEFORE_A_DEADLINE = 64`, which guarantees a deadline gets its turn within 64 messages. It guarantees nothing about wall-clock rate, and no other budget value is claimed without a reproducible measurement.
- A deadline that takes priority over a queued Stop still meets closed admission, because Stop closes the gate before it enqueues.
- The test keeps the queue nonempty for more than 64 messages before the first Tick, separates the initial backlog from later arrivals, and asserts the first Tick lands after exactly the initial backlog, without counting deliveries over a wall-clock window.

**2026-09-25 — implementation.** Implemented with source-audit/27 in orcvs/orcvs#148: the first Tick follows exactly the backlog taken with its Start; policy in ADR 0056. Resolve on merge.

**2026-09-25 — resolved.** Merged in orcvs/orcvs#148 (`67d28248`). The fairness budget is now `BACKLOGS_BEFORE_A_DEADLINE` (formerly `MESSAGES_BEFORE_A_DEADLINE`) and counts coalesced backlogs rather than messages, so the beyond-budget first-Tick test is bounded by slots, as its doc comment states.
