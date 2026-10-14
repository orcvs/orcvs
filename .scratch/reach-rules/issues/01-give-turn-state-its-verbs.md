# 01 — Give Turn state its verbs

Status: resolved
Blocked by: None (can start immediately)

**What to build:**

A private `tick/execution/turns.rs` owns the `ComputationState` vector as `Turns`, with its fields private to it. Execution and its operand binding ask it rather than indexing states.

- `activate(root)` answers whether the Bang was delivered or missed. It replaces the two activation loops in `deliver_output` and `fire` and `misses_activation`.
- `suppress(subtree)` suppresses every computation in the subtree that has not attempted its Turn. It replaces the suppression loops in `deliver_output` and `lock_portal`.

The verbs push no Effects: execution words the missed-Bang diagnostic and keeps the roots a Selected write activated. No suppression, activation, lock or diagnostic changes. See `../spec.md`.

## Acceptance criteria

- [x] No module outside `turns.rs` reads or writes a `ComputationState` field.
- [x] The Bang activation rule and its miss check are written once.
- [x] Every existing Tick, Bang Function, write and Halt test passes unchanged.
- [x] Gates pass with `PROPTEST_CASES=32`: fmt, clippy and nextest on `orcvs` and `console`.

## Comments

2026-10-14 — Implemented. `tick/execution/turns.rs` owns `ComputationState` and its fields as `Turns`; execution, operand binding and the stated-answer fixture ask it. `activate(root)` answers two outcomes, `Delivered` or `Missed`, rather than three: execution keeps a Selected write's activated roots whenever the Bang was not missed, as it did, and an "already active" outcome would have had no caller that treats it apart from `Delivered`. `Execution::activate` words the missed-Bang diagnostic for `deliver_output` and `fire`. `suppress(subtree)` serves `deliver_output` and `lock_portal`. Every existing test passes unchanged.
