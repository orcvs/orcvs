# 14 — Give both deadline computations the same overflow answer

**What to build:** `TickClock::deadline()` (`orcvs/src/playback.rs:1157-1159`) computes
`self.epoch + self.scheduled_at` with an unchecked add. `TickTiming::deadline()` (`:169-173`) computes
the same shape with `checked_add(..).unwrap_or_else(ClockInstant::now)` and a documented reason for
the fallback at `:165-167`.

Two computations of the same kind of value, two different answers to the same overflow, and only one
of them explains itself. Cosmetic today — reaching it needs a `scheduled_at` no run can accumulate —
but the asymmetry is what outlives the reason: a reader who finds the checked one assumes the rule,
and a reader who finds the unchecked one assumes there isn't one.

Either `TickClock::deadline` takes the same fallback, or it carries a comment saying why its input
cannot overflow where `TickTiming`'s can.

**Status:** resolved

- [ ] Both deadline computations answer overflow the same way, or the difference is explained where
      the unchecked one lives.
- [ ] The existing documented reason at `orcvs/src/playback.rs:165-167` still reads correctly after
      the change.

## Verification

`cargo fmt --all -- --check`, then `cargo clippy --package orcvs --all-targets --locked -- -D
warnings` and `PROPTEST_CASES=32 cargo nextest run --package orcvs --locked`.

## Comments

Filed from the `playback-actor` review ledger as **CR-15**, minor and cosmetic. Recorded as a small
consistency debt rather than a defect; nothing in the tree can reach the overflow.

### Resolved

Both are checked now, and the difference in what they do with an overflow is
explained where the difference is made. `TickClock::deadline` returns
`Option<ClockInstant>` and the loop treats `None` as `PlaybackEvent::Unschedulable`,
ending the run with a diagnostic; its doc says why it cannot take
`TickTiming::deadline`'s fallback — a deadline of now is one the loop reaches
immediately, executes, and recomputes the same unrepresentable offset from
again. `TickTiming::deadline`'s documented reason at its own site is unchanged
and still reads correctly: the two agree the addition is checked and differ on
what an overflow means, because one is reporting a Tick and the other is
waiting for one.

`start` and `retune` also refuse an unschedulable period up front, as
`PlaybackStartError::UnschedulableTickPeriod`, so the loop's arm is the second
line rather than the first.
