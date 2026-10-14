# 02 — Decide each Contact's Consequence in one module

Status: ready-for-agent
Blocked by: 01

**What to build:**

`tick/dependencies.rs` answers a `Consequence` for each Contact an admitted write or Reservation makes, and for each root a Bang aligns with, given a phase: `Schedule`, or `Turn` with the write's `Delivery` and a read-only view of the Turns. `Delivery` moves from execution into the dependency module.

- `producer_edges` turns "order after" into Dependencies and states no rule of its own.
- `deliver_output` and `fire` apply the Consequences through 01's verbs and word the diagnostics.
- The ordering defect stays a refused write with its diagnostic.
- The `PortalRelationships` doc stops saying the phases decide separately.

No Dependency, suppression, activation or diagnostic changes. See `../spec.md`.

## Acceptance criteria

- [ ] Execution decides no reach rule: replacement, suppression, activation, the own-Expression diagnostic and the ordering defect each come from a Consequence.
- [ ] A verdict table test in the dependency module has one row per rule in each phase that decides it.
- [ ] One focused test hands the Turn phase a Reserved write onto an attempted computation.
- [ ] Every existing Tick, schedule-reuse and property test passes unchanged.
- [ ] Gates pass with `PROPTEST_CASES=32`: fmt, clippy and nextest on `orcvs` and `console`.
