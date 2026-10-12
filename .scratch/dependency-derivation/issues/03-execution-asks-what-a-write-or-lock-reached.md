# 03 — Execution asks what a write or a lock reached

Status: ready-for-agent
Blocked by: 02

**What to build:**

Execution stops reading how the Lookup stores reservations and lock targets. It asks the dependency module:

- `reached(lookup, write)` answers the Functions and Bang roots an admitted write covers, where execution now calls `written_over` and reads `PortalRelationships`.
- `lock_target(index)` answers what a lock reaches, where `lock_portal` now matches `lookup.locks`.

Lock targets move out of `Lookup::new`, so the Lookup stops holding `locks`. `PortalRelationships`, `FunctionContact` and the construction of `LockTarget` become private to the dependency module. No suppression, activation, lock or diagnostic changes. See `../spec.md`.

## Acceptance criteria

- [ ] `tick/execution.rs` reads no Lookup field and names neither `PortalRelationships` nor a `LockTarget` constructor.
- [ ] The Lookup holds no `locks`.
- [ ] Every existing Tick, Bang Function, write and Halt test passes unchanged.
- [ ] Gates pass with `PROPTEST_CASES=32`: fmt, clippy and nextest on `orcvs` and `console`, then the workspace clippy, nextest and doctests once before the pull request.

## Comments
