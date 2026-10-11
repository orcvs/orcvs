# 04 — A Halt above a typed Bang misses it

Status: ready-for-agent
Blocked by: 01

**What to build:**

Pin D2, D3, D6, D7 and D15 from ticket 07. Halt is an ordinary Producer: its lock holds only if its Turn precedes its target's, and only a Bang that executed activates it. Orca departure: Orca's `H` above `*` runs first and locks it.

## Acceptance criteria

- [ ] `*!` above a typed `**` above Raw Play: Raw Play plays, the `**` is cleared, and "Bang reached a root that has taken its Turn" is diagnosed at the Halt; no cycle (E1).
- [ ] An independent activator that bangs: the Halt locks the `**`, which stands, Tick after Tick, while nothing plays (E2).
- [ ] The activator quiet, suppressed or itself locked: the Halt is inert and the `**` fires, with a missed Bang at the Halt (E2).
- [ ] A dynamic activator landing after the Halt's Turn: missed Bang at the Halt, the `**` fires.
- [ ] An activator that depends on the target: a same-Tick dependency cycle stopping every Expression on it (E3).
- [ ] A locked `**` under a Copy is carried and plays every Tick (E7).
- [ ] The Halt-over-`**` "not an Expression root" test (`tick.rs:2870-2895`) is flipped to a lock.
- [ ] `lock_covers`'s doc comment is restated for the case where the excluded producer is the Halt's only activator.
- [ ] CONTEXT.md's Halt Function entry records the missed Bang and the lock on a `**`.
