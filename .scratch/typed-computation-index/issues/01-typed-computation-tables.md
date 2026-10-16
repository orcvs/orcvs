# 01 — Index every per-computation table by `ComputationId`

Status: resolved

**What to build:**

`PerComputation<T>` and `ComputationRange` in `tick.rs`, and the conversion
of `Lookup`, `dependencies`, `execution`, `ordering` and the schedule tests
to take and return `ComputationId` wherever they mean a computation. See
`../spec.md`.

## Acceptance criteria

- [x] A Cell index or ordering key passed where a computation belongs fails
  to compile.
- [x] The typed vector's indexing, `ids()` order and range slicing are tested.
- [x] The existing suite passes unchanged in behaviour.
- [x] Gates pass with `PROPTEST_CASES=32`.

## Comments

2026-10-16 — `PerComputation<T>` (indexed by `ComputationId`, with `ids()`, `iter_enumerated()` and `into_vec()`) and `ComputationRange` (a subtree or Expression as ids) replace every bare-`usize` computation table in `tick`, `dependencies`, `execution`, `ordering` and `schedule_reuse`. Ordering indexes the same computation list, so it is converted through; its keys stay Cell indices. `Occupancy` keeps a bare index outside the schedule, converted by `ComputationId::index` at its two calls. `a_table_per_computation_is_read_and_written_by_computation_id` pins the contract.
