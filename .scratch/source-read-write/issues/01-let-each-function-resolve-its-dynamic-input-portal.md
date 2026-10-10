# 01 — Let each Function resolve its dynamic Input Portal

Status: resolved

**What to build:**

A prefactor. Resolving a dynamic Input Portal (ADR 0067's amendment) currently assumes Track's selection. Let each Function with a dynamic Input Portal state how its operands select its Cell pair, so a second one can join by its definition alone. Track's behaviour does not change.

## Acceptance criteria

- [x] Resolving a dynamic Input Portal asks the Function how its operands select its pair, rather than assuming Track.
- [x] Track's existing tests pass unchanged, including the reused-versus-fresh schedule comparison.
- [x] No test-only input or seam is added to shipped code.
- [x] Comments and docs that call Track's the one dynamic Input Portal still read true, or are reworded to describe the rule rather than the single case.

## Comments

Resolved. `InputPortal::Dynamic` carries a `PairSelection`, the rule a Function's definition names for how its operands select its pair; `InputPortal::resolve` asks the rule for the pair's offset (Cells east of the last operand, rows south) and no longer assumes Track. Track names `PairSelection::IndexModuloCount`. Track's tests pass unchanged; the only test edits are the Function table sweep's expected `Dynamic(IndexModuloCount)` and the schedule-reuse helper's `Dynamic(_)` pattern. Because the rule is part of the declared Input Portal, a Function Replacement between two Functions with different selection rules is refused as a change to the Source write it declares.
