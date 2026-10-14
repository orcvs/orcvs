# 02 — Match every consumer on Output

Status: resolved
Blocked by: 01

**What to build:**

`orcvs` matches on `Function::output()` wherever it asked a narrow predicate: `PortalAccess::resolve`, `output_portal_reservation`, `tick/dependencies.rs`, `tick/execution.rs` and the tests. The predicates `performs_terminal_output`, `source_effect`, `locks_root`, `output_portal` and `dynamic_output_portal` are then removed from `Function`. CONTEXT.md's Output Portal entry says a Terminal Output Function's Portal is never addressed. See `../spec.md`.

## Acceptance criteria

- [x] No `pub` narrow Output predicate remains on `Function`.
- [x] `PortalAccess::resolve` decides a Function's output with one match.
- [x] Every existing Tick, Halt, Bang, Source-write and schedule-reuse test passes.
- [x] Gates pass with `PROPTEST_CASES=32`: workspace clippy, nextest, doctests, rustdoc and the roadmap checks.

## Comments

2026-10-14 — Implemented. `PortalAccess::resolve` decides a computation's output with one match on `Function::output()`, and `output_portal_reservation` reserves only for `Output::Static`, so Halt needs no exclusion of its own. Dependencies, execution, schedule reuse and the Source-write exclusion test match on `Output`. `performs_terminal_output`, `source_effect`, `locks_root`, `output_portal` and `dynamic_output_portal` are gone from `Function`; the selection rule is a private `selection`. `no_function_declares_an_output_portal_it_does_not_answer_through` is removed because a total `Output` makes its case unrepresentable. CONTEXT.md's Output Portal entry says a Terminal Output Function's Portal is never addressed.
