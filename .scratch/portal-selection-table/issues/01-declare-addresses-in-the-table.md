# 01 — Declare each Read's and Write's address in its table row

Status: resolved

**What to build:**

Rows declare `distance:` or `position:`; the macro generates `DistanceRule`,
`PositionRule` and `Carried`. `PairSelection` carries a rule instead of a
`Function`, and `read.rs` loses both `unreachable!` arms. See `../spec.md`.

## Acceptance criteria

- [x] No `PairSelection` variant carries a `Function`.
- [x] `read.rs` has no `unreachable!`.
- [x] `carried!` is gone; a row with a `value: Untyped` role carries it.
- [x] A sweep over `Function::ALL` checks every declared rule binds its own
  row's operands.
- [x] Gates pass with `PROPTEST_CASES=32`.

## Comments

2026-10-17 — Rows declare `distance:` or `position:`; `DistanceRule` and `PositionRule` bind each row's own operand struct through exhaustive matches, and `read.rs` keeps only `read` and `bound`. `carried!` munches each row's roles for `value: Untyped`. `every_declared_distance_or_position_binds_its_own_functions_operands` sweeps `Function::ALL`.
