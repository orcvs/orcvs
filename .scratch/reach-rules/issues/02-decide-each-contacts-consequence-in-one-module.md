# 02 — Decide each Contact's Consequence in one module

Status: resolved
Blocked by: 01

**What to build:**

`tick/dependencies.rs` answers a `Consequence` for each Contact an admitted write or Reservation makes, and for each root a Bang aligns with, given a phase: `Schedule`, or `Turn` with the write's `Delivery` and a read-only view of the Turns. `Delivery` moves from execution into the dependency module.

- `producer_edges` turns "order after" into Dependencies and states no rule of its own.
- `deliver_output` and `fire` apply the Consequences through 01's verbs and word the diagnostics.
- The ordering defect stays a refused write with its diagnostic.
- The `PortalRelationships` doc stops saying the phases decide separately.

No Dependency, suppression, activation or diagnostic changes. See `../spec.md`.

## Acceptance criteria

- [x] Execution decides no reach rule: replacement, suppression, activation, the own-Expression diagnostic and the ordering defect each come from a Consequence.
- [x] A verdict table test in the dependency module has one row per rule in each phase that decides it.
- [x] One focused test hands the Turn phase a Reserved write onto an attempted computation.
- [x] Every existing Tick, schedule-reuse and property test passes unchanged.
- [x] Gates pass with `PROPTEST_CASES=32`: fmt, clippy and nextest on `orcvs` and `console`.

## Comments

2026-10-14 — Implemented. `tick/dependencies.rs` answers three questions during a Turn and one before it:

- `contact(lookup, producer, contacted, phase)` answers a `Contacted`: its `Consequence` (`Unchanged`, `OrderAfter`, `Replace` or `Suppress`), and whether a Selected write lands on its producer's own Expression.
- `bang_root(lookup, producer, root, phase)` answers `OrderAfter` before the Tick and `Activate` during it.
- `refusal(relationships, written, delivery, turns)` answers whether a write is refused whole, as a Function Replacement that changes a scheduled fact or as an ordering defect. Refusal stays a separate question because it is decided over every Contact before any of them changes.

`Phase::Turn` carries the `Delivery`, the written Atom and a `TurnFacts` view, which `Turns` implements and the dependency tests implement with stated facts. Each Contact is asked in turn while it is applied, because one Contact's suppression can cover a later one.

The schedule-only filters stay in `producer_edges`: a bundle that clears its own Span is exempt from its own Cells, and a lock's own cover. The lock cover moves in 03. `fire` asks `bang_root` with the Delivery of its Turn. The pre-Tick activation closure asks `bang_root` too, so which roots a Bang can reach is one rule. Every existing test passes unchanged, including the orcvs suite at the 256-case proptest default.
