# 01 — Declare a Function's Output in lang

Status: open
Blocked by: None (can start immediately)

**What to build:**

`lang` declares `Output` beside `InputPortal`, and `Function::output()` answers it. lang's own callers match on it: the interpreter's pre-dispatch returns, `PairSelection` resolution's tests, and `DECLARED_CHANGES`'s Write row, which compares `output()` and `input_portal()`. See `../spec.md`.

## Acceptance criteria

- [ ] `Function::output()` returns `Output` for every Function, with no `None` variant.
- [ ] The Portal sweep over `Function::ALL` asserts each Function's `Output`.
- [ ] `DECLARED_CHANGES`'s Write row compares `output()` and `input_portal()` only.
- [ ] Gates pass with `PROPTEST_CASES=32`: fmt, clippy and nextest on `lang` and `orcvs`.
