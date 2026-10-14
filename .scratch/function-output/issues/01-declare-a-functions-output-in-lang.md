# 01 — Declare a Function's Output in lang

Status: resolved
Blocked by: None (can start immediately)

**What to build:**

`lang` declares `Output` beside `InputPortal`, and `Function::output()` answers it. lang's own callers match on it: the interpreter's pre-dispatch returns, `PairSelection` resolution's tests, and `DECLARED_CHANGES`'s Write row, which compares `output()` and `input_portal()`. See `../spec.md`.

## Acceptance criteria

- [x] `Function::output()` returns `Output` for every Function, with no `None` variant.
- [x] The Portal sweep over `Function::ALL` asserts each Function's `Output`.
- [x] `DECLARED_CHANGES`'s Write row compares `output()` and `input_portal()` only.
- [x] Gates pass with `PROPTEST_CASES=32`: fmt, clippy and nextest on `lang` and `orcvs`.

## Comments

2026-10-14 — Implemented test-first: `every_function_does_one_thing_through_its_output_portal` failed to compile before `Output` existed and passes against it. `Function::output()` reads the kind column for every effect and the selection rule or static offset for a value answer. The interpreter's pre-dispatch returns, `PairSelection`'s resolution tests (through a `selection` helper) and `DECLARED_CHANGES`'s Write row match on it. The old Portal sweep now asserts Input Portals only, because the new test owns each Output. The narrow predicates stay until ticket 02 moves `orcvs` off them.
