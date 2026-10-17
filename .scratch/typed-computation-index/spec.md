# Name a schedule's computations by type in every table it keeps

Status: resolved
Stacked on: `../typed-function-walk/` (PR #230)

## Problem Statement

`orcvs/src/source/tick.rs` names a computation by `ComputationId`, but every
table the schedule keeps beyond `Computation` itself indexes computations by
bare `usize`: the `active` table, `Lookup::nodes`, `Claim::node`, dependency
edges, `Stall`, `Lookup::descendants` and `root_at`, ordering's `Nodes` and
`Graph`, and execution's `Turns`. The same code holds Cell indices and
ordering keys as `usize` too, so a Cell index passed where a computation
belongs compiles and reads the wrong row.

## Solution

- `PerComputation<T>` holds one value per computation and is indexed by
  `ComputationId` alone; `ids()` and `iter_enumerated()` yield ids in Parser
  preorder. `ComputationRange` is a contiguous run of computations, a subtree
  or an Expression, iterated as ids and indexing a `PerComputation` as a
  slice. Both are written by hand; no dependency is added.
- `Lookup::nodes()` answers a `PerComputation<Computation>`; `Claim`,
  `descendants` and `root_at` speak `ComputationId` and `ComputationRange`.
- `dependencies`, `execution` and its submodules, `ordering` and the schedule
  tests take and return `ComputationId` wherever they mean a computation.
- `ordering`'s nodes are positions in the same computation list, so ordering
  is converted through: `Nodes`, `Joined`, `Join`, `Stall`, `Order`, `Turn`,
  `Progress` and `Graph`. A node's ordering key stays `usize`, a Cell index,
  and is now the only `usize` in the key pair.
- `Occupancy` and `Portal::occupancy` sit outside the schedule and keep a
  bare index; `ComputationId::index` converts at those two calls.
- The states `plan` returns cross into `model` as a `Vec`, in Parser
  preorder.

Behaviour-preserving: `ComputationId` orders as its index does, so every
schedule, Tick Plan and highlight is unchanged.

## Decisions (Toby, 2026-10-16)

- Build after #230, stacked on its branch.
- A hand-written typed vector, not the `index_vec` crate.

## Out of scope

- Typing Cell indices and ordering keys.
- Typing `Occupancy::Root`, whose payload no shipped caller reads.
