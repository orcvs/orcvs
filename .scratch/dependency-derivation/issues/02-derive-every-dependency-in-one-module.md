# 02 — Derive every Dependency in one module

Status: ready-for-agent
Blocked by: 01

**What to build:**

`tick/dependencies.rs` derives every Dependency from a Lookup, and takes the activation work with it:

- `derive(lookup)` answers the active roots, every edge and the dynamic writers. It replaces `active_roots`, the edge loop in `order_turns`, `dynamic_writers` and `input_edges`.
- `join(lookup, active, roots)` answers the roots activation now reaches, and their edges and writers. It becomes the production side of ordering's join trait.

`producer_edges`, `reader_edges`, `activate`, `lock_covers` and `locked_subtree` move into the module and become private to it. Each producer rule keeps its present-tense comment. No edge changes. See `../spec.md`.

## Acceptance criteria

- [ ] `tick.rs` builds the schedule from `derive` and holds no edge or activation rule.
- [ ] The join trait's production side is `join`.
- [ ] The dependency module has one named test per producer rule, each through `derive` on a small Grid: lock subtree, self exemption, lock cover, contact ordering, mover exemption, literal consumers, Bang roots. It also has one test of `join`.
- [ ] Every existing Tick, schedule-reuse and property test passes unchanged.
- [ ] Gates pass with `PROPTEST_CASES=32`: fmt, clippy and nextest on `orcvs` and `console`.

## Comments
