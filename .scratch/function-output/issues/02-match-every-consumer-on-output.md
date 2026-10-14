# 02 — Match every consumer on Output

Status: open
Blocked by: 01

**What to build:**

`orcvs` matches on `Function::output()` wherever it asked a narrow predicate: `PortalAccess::resolve`, `output_portal_reservation`, `tick/dependencies.rs`, `tick/execution.rs` and the tests. The predicates `performs_terminal_output`, `source_effect`, `locks_root`, `output_portal` and `dynamic_output_portal` are then removed from `Function`. CONTEXT.md's Output Portal entry says a Terminal Output Function's Portal is never addressed. See `../spec.md`.

## Acceptance criteria

- [ ] No `pub` narrow Output predicate remains on `Function`.
- [ ] `PortalAccess::resolve` decides a Function's output with one match.
- [ ] Every existing Tick, Halt, Bang, Source-write and schedule-reuse test passes.
- [ ] Gates pass with `PROPTEST_CASES=32`: workspace clippy, nextest, doctests, rustdoc and the roadmap checks.
