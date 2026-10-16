# 02 — Type a computation's place in its Expression

Status: resolved
Blocked by: 01

**What to build:**

`ComputationId`, `Place { Root, Nested { parent, owner } }`, and
`Computation::parent()`/`owner()`; `Operand::child: Option<ComputationId>`.
`computations` replaces its `BTreeMap` with one reused `Vec` and asserts that
a parent entry is a bound Function. Callers in `dependencies`, `execution`,
`schedule_reuse` and `Lookup` read the accessors. See `../spec.md`.

## Acceptance criteria

- [x] No `Computation` can name a parent while standing as a root.
- [x] A nested computation's owner is its root, tested on a two-level nest.
- [x] Gates pass with `PROPTEST_CASES=32`.

## Comments

2026-10-16 — `Computation` holds `id` and `place`; `parent()`, `owner()` and `is_root()` replace the raw fields, and `Operand::child` is an `Option<ComputationId>`. `computations` resolves parents through one reused `Vec<Option<ComputationId>>` and `expect`s a bound Function at every parent entry, where the `BTreeMap` lookup made a broken parent a root. `Lookup::new` debug-asserts each id is its index. `a_nested_computation_is_owned_by_its_root_and_answers_its_parent` covers a three-level nest. Workspace clippy, nextest (1716), doctests and rustdoc pass with `PROPTEST_CASES=32`.
