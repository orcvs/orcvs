# 01 — Give Turn state its verbs

Status: ready-for-agent
Blocked by: None (can start immediately)

**What to build:**

A private `tick/execution/turns.rs` owns the `ComputationState` vector as `Turns`, with its fields private to it. Execution and its operand binding ask it rather than indexing states.

- `activate(root)` answers whether the root was newly activated, already active, or missed the Bang. It replaces the two activation loops in `deliver_output` and `fire` and `misses_activation`.
- `suppress(subtree)` suppresses every computation in the subtree that has not attempted its Turn. It replaces the suppression loops in `deliver_output` and `lock_portal`.

The verbs push no Effects: execution words the missed-Bang diagnostic and keeps the roots a Selected write activated. No suppression, activation, lock or diagnostic changes. See `../spec.md`.

## Acceptance criteria

- [ ] No module outside `turns.rs` reads or writes a `ComputationState` field.
- [ ] The Bang activation rule and its miss check are written once.
- [ ] Every existing Tick, Bang Function, write and Halt test passes unchanged.
- [ ] Gates pass with `PROPTEST_CASES=32`: fmt, clippy and nextest on `orcvs` and `console`.
