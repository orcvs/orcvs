# 02 — Concentrate the clock loop

**What to build:** One clock loop per target instead of four, with the deadline rule as
a single shared function rather than two implementations that must agree.

**Blocked by:** `playback-clock/01` — the behaviour change lands first, so this one is a
refactor with no behaviour to argue about.

**Status:** ready-for-agent

- [ ] `start` and `retune` no longer each carry a clock loop; they differ in the first
      deadline they supply and in whether they begin a run.
- [ ] The deadline rule is computed in one place for both targets. The native clock no
      longer delegates it to `tokio::time::Interval`'s missed-tick machinery, so the
      only remaining per-target difference is how to wait until an instant.
- [ ] Cancellation, the `ClockRunGuard`, the `Weak` upgrade, and Tick delivery are
      written once.
- [ ] `start`'s immediate first Tick and `retune`'s anchoring to the last executed Tick
      both survive, with tests naming them.
- [ ] No behaviour changes. Effort 01's tests pass unaltered.

## Comments

Two architecture reviews reached this from opposite directions. The 2026-09-09 review
("Give the Playback clock a real seam") wanted a `Clock` trait with tokio, browser and
manual adapters. The 2026-09-10 review ("Concentrate Playback clock execution") wanted
no trait at all, and treated the two targets' timing as behaviour to preserve.

Effort 01 settles that: there was no target-specific behaviour to preserve, which
removes the reason the second review rated this second-wave work. What the reviews both
missed is that the thing worth sharing is the deadline rule, not the loop scaffolding —
the scaffolding is only how the rule came to be written twice.

The manual test adapter the first review proposed is out of scope; see the spec.
