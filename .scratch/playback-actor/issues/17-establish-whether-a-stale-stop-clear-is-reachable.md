# 17 — Establish whether a stale stop clear is reachable

**What to decide:** Whether `stop_requested` can be cleared out from under a request that is still in
flight, and if it can, whether the resulting Tick matters. Filed as **unconfirmed**: the mechanism is
confirmed, a constructible interleaving is not.

The mechanism. `stop_requested.store(false, Ordering::Release)` at `orcvs/src/playback.rs:1271` runs
in the `Stop` arm on *any* `Stop` message, not specifically on the one whose handle raised the flag,
and the flag is one `Arc<AtomicBool>` shared by every clone of the handle (`:580`). So a `Stop` that
arrives and clears the flag can clear a request a different `stop()` raised and whose own message is
still queued behind it — and a Tick that the flag would have declined then executes.

Why it is smaller than it first looks. The queue mitigations are real: `biased;` at `:1219` and the
`try_recv` at `:1212` both take a queued `Stop` before any deadline, so a `Stop` sitting in the queue
is never overtaken by a Tick. What neither covers is the window inside the Deadline arm itself —
between the `stop_requested.load` at `:1281` and `execute_tick` at `:1292`. That gap, not a second
handle, is what makes the sequence reachable at all. And ADR 0040 already concedes the same window
from the other side: "a Tick already in flight continues past the return". The incremental risk over
what the ADR admits is therefore small.

Why it is still worth recording. The stale clear does change behaviour: a Tick the flag would
otherwise have declined gets executed, and no diagnostic marks it. That is a different fact from the
one ADR 0040 concedes, which is about a Tick already running.

Why nobody reviewing the console could construct it. The console is single-threaded through
`Orcvs::event_handler`; there is one handle and the calls are serialised. A second concurrent caller
is possible through the public API — `PlaybackEngine` is `Clone` — but no in-tree caller does it.

**Status:** needs-triage

- [ ] Whether an interleaving exists that clears a live request is established, by construction or by
      argument that none exists.
- [ ] If one exists, the decision is recorded: tie the clear to the request that raised it (a
      generation, or a per-request flag), or accept it as within what ADR 0040 already concedes and
      say so in the ADR.
- [ ] If none exists, what forbids it is written beside `:1271`, because the code does not currently
      say that the `Stop` it is clearing for is the `Stop` that raised the flag.

## Verification

`cargo fmt --all -- --check`, then `cargo clippy --package orcvs --all-targets --locked -- -D
warnings` and `PROPTEST_CASES=32 cargo nextest run --package orcvs --locked`. Anything staged here is
a concurrency test; the multi-threaded and second-platform runs that would expose it are deferred to
CI.

## Comments

Filed from the `playback-actor` review ledger as **CR-18**, downgraded to minor by audit and recorded
as suspected rather than confirmed.

The downgrade has two parts, both in the text above: the original report attributed the reachability
to a second handle, when the load-to-execute gap is what actually opens it; and ADR 0040 already
admits the same window for a Tick in flight, so the new exposure is narrower than first stated. It
should be worked as an investigation, not as a fix.

### Established, and it survives the gate

The mechanism this ticket describes is narrowed but not gone.

`stop_requested` is now `TickGate`, and `clear_stop` is a conditional
compare-exchange from `STOPPING` rather than an unconditional store. That
closes the window the ticket says made the sequence reachable at all — the gap
between the load and `execute_tick` inside the Deadline arm — because admission
is now a single compare-exchange and a Tick falls on one side of a request or
the other.

What the compare-exchange does not carry is *whose* request it clears. It
checks that a request is standing, not that this `Stop` is the message the
standing one raised. So the interleaving is still constructible: handle A calls
`stop`, a `start` is queued behind it, handle B calls `stop` and raises the gate
again, A's `Stop` is applied and clears B's request, the `start` begins a run,
and a deadline is admitted before B's `Stop` arrives. A Tick executes after
`B.stop()` returned.

It stays out of reach in-tree for the reason the ticket gives: the console is
single-threaded through `Orcvs::event_handler`, with one handle and serialised
calls. Nothing else clones a `PlaybackEngine`.

So the decision the second acceptance asks for is still open, and it is a real
one: tying the clear to the request that raised it needs request identity — a
token or generation on the gate — which is machinery ADR 0040 deleted from the
clock and would be reintroducing here. Left `needs-triage` deliberately rather
than closed, because that trade is not mine to make.

Verified against `orcvs/src/playback/gate.rs` and the `Stop` arm at
`orcvs/src/playback.rs`, not inferred from the flag it replaced.
