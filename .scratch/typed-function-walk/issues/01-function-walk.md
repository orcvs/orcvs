# 01 — One allocation-free walk over a schedule's Functions

Status: resolved

**What to build:**

`ScheduleInputs::functions(grid)` and `ScheduledExpression::functions(grid)`
yield each Function a schedule computes as a `FunctionSite`. The paint walk
`tick::output_portal_reservations` reads it; `ScheduledEntry::function`
becomes the only place an entry's anchor is derived. See `../spec.md`.

## Acceptance criteria

- [x] The walk yields root and nested Functions in Parser preorder and skips
  Expressions with no leading Function.
- [x] The allocation-bound highlight test still passes.
- [x] Gates pass with `PROPTEST_CASES=32`.

## Comments

2026-10-16 — `ScheduleInputs::functions` and `ScheduledExpression::functions` yield `FunctionSite`s; `output_portal_reservations` reads them. `the_function_walk_yields_what_the_schedule_computes_in_preorder` checks the order, the skipped bare Number, and that the walk and `computations` hold the same Functions. The allocation-bound highlight test passes unchanged.
