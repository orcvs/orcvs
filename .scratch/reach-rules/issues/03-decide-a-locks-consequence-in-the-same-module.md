# 03 — Decide a lock's Consequence in the same module

Status: resolved
Blocked by: 02

**What to build:**

`lock_verdict(locker, phase)` answers a lock's Consequence for the subtree it targets, with 02's phase and Consequence types. It absorbs `lock_covers`, which scheduling asks, and the attempted check in `lock_portal`, which execution asks. `Locks` and `Lock` stay as they are. No lock, suppression or diagnostic changes. See `../spec.md`.

## Acceptance criteria

- [x] `lock_portal` decides no reach rule: an applied lock, a missed lock and the ordering defect each come from the Consequence.
- [x] The verdict table gains rows for lock cover and a lock that finds its root attempted, in each delivery.
- [x] Every existing Halt and Tick test passes unchanged.
- [x] Gates pass with `PROPTEST_CASES=32`, then the workspace clippy, nextest and doctests once before the pull request.

## Comments

2026-10-14 — Implemented. `lock_verdict(locks, locker, phase)` answers a `Locked`. Before the Tick it answers `OrderAfter` for the locked subtree, or `Unchanged`. During a Turn it answers `Lock`, `Missed`, `OrderingDefect` or `NotARoot`. `producer_edges` takes the lock's edges from it, and `lock_covers` asks it too, so the lock rule is stated once. `lock_portal` applies the answer and words the diagnostics. `Phase::Turn` carries the written Atom as an `Option`, because a lock writes nothing. Every existing test passes unchanged, including the orcvs suite at the 256-case proptest default.
