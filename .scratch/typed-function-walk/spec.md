# Walk a schedule's Functions through types that hold its invariants

Status: resolved
Stacked on: `../output-portal-reservation/` (PR #229)

## Problem Statement

`orcvs/src/source/tick.rs`, `computations`, and the paint walk
`output_portal_reservations` each walk every Function of a schedule's inputs.
`computations` resolves each entry's parent through a per-Expression
`BTreeMap` from entry index to node index, and that lookup quietly treats an
entry whose parent is not a bound Function as a root, though the Parser never
produces one.

A `Computation` states its place in its Expression with two raw `usize`
fields, `parent: Option<usize>` and `owner: usize`, where a root's owner is
its own index. Nothing in the type stops a root naming a parent, an owner that
is not the parent's owner, or an entry index standing where a node index
belongs. `Operand::child` is a raw `usize` too.

## Solution

- `ScheduleInputs::functions(grid)` yields every Function a schedule computes
  as a `FunctionSite { anchor, function }`, in row, Source and then Parser
  preorder, and allocates nothing. The paint walk reads it.
- A `ComputationId` newtype names a computation in its schedule. A
  `Computation` states where it stands as `Place::Root` or
  `Place::Nested { parent, owner }`, and answers `parent()` and `owner()`.
  `Operand::child` is an `Option<ComputationId>`.
- `computations` resolves each parent through one reused `Vec` indexed by
  entry, and states the Parser invariant (a parent entry is a bound Function)
  as an assertion at that one place. A nested computation copies its owner
  from its parent.
- The schedule's other index spaces (`Claim`, dependency edges, ordering, the
  `active` table) stay `usize`; they meet a `ComputationId` through
  `ComputationId::index`.

Behaviour-preserving: every schedule, Tick Plan and highlight is unchanged.

## Decisions (Toby, 2026-10-16)

- Build after #229, stacked on its branch.
- The type-safe shape: newtype ids, nesting as an enum, the invariant checked
  once, an allocation-free Function walk, no `Step` stream callers must
  consume in order.

## Out of scope

- Converting every node index in `dependencies`, `ordering` and `execution`
  to `ComputationId`.
