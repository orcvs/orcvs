# 04 — A Halt above a typed Bang misses it

Status: resolved
Blocked by: 01

**What to build:**

Pin D2, D3, D6, D7 and D15 from ticket 07. Halt is an ordinary Producer: its lock holds only if its Turn precedes its target's, and only a Bang that executed activates it. Orca departure: Orca's `H` above `*` runs first and locks it.

## Acceptance criteria

- [x] `*!` above a typed `**` above Raw Play: Raw Play plays, the `**` is cleared, and "Bang reached a root that has taken its Turn" is diagnosed at the Halt; no cycle (E1).
- [x] An independent activator that bangs: the Halt locks the `**`, which stands, Tick after Tick, while nothing plays (E2).
- [x] The activator quiet, suppressed or itself locked: the Halt is inert and the `**` fires, with a missed Bang at the Halt (E2).
- [x] A dynamic activator landing after the Halt's Turn: missed Bang at the Halt, the `**` fires.
- [x] An activator that depends on the target: a same-Tick dependency cycle stopping every Expression on it (E3).
- [x] A locked `**` under a Copy is carried and plays every Tick (E7).
- [x] The Halt-over-`**` "not an Expression root" test (`tick.rs:2870-2895`) is flipped to a lock.
- [x] `lock_covers`'s doc comment is restated for the case where the excluded producer is the Halt's only activator.
- [x] CONTEXT.md's Halt Function entry records the missed Bang and the lock on a `**`.

## Comments

2026-10-11 — Pinned in `orcvs/src/source/tick/bang_function.rs`; no code change beyond ticket 01. E1, E2, E3 and E7 are ticket 01's tests. Added: an activator suppressed by an overwrite, and one locked by another Halt, each leave the Halt inert, so the `**` fires and its Bang is missed at the Halt. A dynamic activator, a Write nested in a Raw Play the `**` activates, lands after the Halt's Turn and is missed as well. The Halt-over-`**` test became `an_active_halt_locks_a_bang_function` in ticket 01, and `lock_covers`'s doc comment states the case where the excluded producer is the Halt's only activator. The CONTEXT.md criterion landed with the documentation.

