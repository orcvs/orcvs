# 03 — Decide a lock's Consequence in the same module

Status: ready-for-agent
Blocked by: 02

**What to build:**

`lock_verdict(locker, phase)` answers a lock's Consequence for the subtree it targets, with 02's phase and Consequence types. It absorbs `lock_covers`, which scheduling asks, and the attempted check in `lock_portal`, which execution asks. `Locks` and `Lock` stay as they are. No lock, suppression or diagnostic changes. See `../spec.md`.

## Acceptance criteria

- [ ] `lock_portal` decides no reach rule: an applied lock, a missed lock and the ordering defect each come from the Consequence.
- [ ] The verdict table gains rows for lock cover and a lock that finds its root attempted, in each delivery.
- [ ] Every existing Halt and Tick test passes unchanged.
- [ ] Gates pass with `PROPTEST_CASES=32`, then the workspace clippy, nextest and doctests once before the pull request.
