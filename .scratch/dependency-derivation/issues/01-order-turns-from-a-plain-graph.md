# 01 — Order Turns from a plain graph

Status: ready-for-agent
Blocked by: None (can start immediately)

**What to build:**

Ordering sorts a graph it is handed and stops reading the Lookup. It is given:

- the node count;
- the edges and the dynamic writers;
- each node's Grid-order key;
- each node's Expression.

It answers the order, the stopped nodes, and the nodes each cycle holds. Tick scheduling words the cycle diagnostics.

When roots join during a Tick, `take_turns` asks a one-method join trait for their edges and writers, instead of calling back into `tick::activate`, `producer_edges` and `reader_edges`. Until ticket 02, the trait's production side wraps those existing functions.

Ordering's private `Dependencies` struct is renamed `Graph`. No Turn order, stop or diagnostic changes. See `../spec.md`.

## Acceptance criteria

- [ ] `tick/ordering.rs` no longer names `Lookup` and calls nothing in `tick.rs`.
- [ ] `take_turns` takes the join trait. Production implements it, and ordering's tests implement it with a fixed table.
- [ ] Cycle diagnostics are worded in Tick scheduling from what ordering answers, with the same messages, Positions and order as before.
- [ ] Ordering has unit tests on hand-built graphs covering Grid tie-breaks, cycle closure by Expression, writers-first order, and rebuilding the order when a root joins.
- [ ] Every existing Tick, schedule-reuse and property test passes unchanged.
- [ ] Gates pass with `PROPTEST_CASES=32`: fmt, clippy and nextest on `orcvs` and `console`.

## Comments
