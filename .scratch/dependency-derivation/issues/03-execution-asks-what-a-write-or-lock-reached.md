# 03 — Execution asks what a write or a lock reached

Status: resolved
Blocked by: 02

**What to build:**

Execution stops reading how the Lookup stores reservations and lock targets. It asks the dependency module:

- `reached(lookup, write)` answers the Functions and Bang roots an admitted write covers, where execution now calls `written_over` and reads `PortalRelationships`.
- `lock_target(index)` answers what a lock reaches, where `lock_portal` now matches `lookup.locks`.

Lock targets move out of `Lookup::new`, so the Lookup stops holding `locks`. `PortalRelationships`, `FunctionContact` and the construction of `LockTarget` become private to the dependency module. No suppression, activation, lock or diagnostic changes. See `../spec.md`.

## Acceptance criteria

- [x] `tick/execution.rs` reads no Lookup field and names neither `PortalRelationships` nor a `LockTarget` constructor.
- [x] The Lookup holds no `locks`.
- [x] Every existing Tick, Bang Function, write and Halt test passes unchanged.
- [x] Gates pass with `PROPTEST_CASES=32`: fmt, clippy and nextest on `orcvs` and `console`, then the workspace clippy, nextest and doctests once before the pull request.

## Comments

2026-10-12 — Implemented. `tick/dependencies.rs` holds `PortalRelationships`, `FunctionContact`, `reserved_at`, `reservations` and `reached`, and `Locks`, built from the Lookup and the Map when the schedule is ordered and kept on the Schedule beside it. `Locks::lock_target` answers a two-variant `Lock`: the locked subtree, or a site something other than a root occupies. An empty site and one outside the Grid answer nothing, as both were no-ops already. `FunctionContact` exposes its facts through methods, so only the module builds one. `Lookup::new` no longer takes the Map. Every existing test passes unchanged; the dependency tests build their locks beside their Lookup.
