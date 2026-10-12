# Derive a Tick's Dependencies in one module

Status: resolved

## Problem Statement

To understand the order a Tick takes its Turns in, a maintainer has to read three modules that call into each other:

- **Tick scheduling** (`tick.rs`) works out which roots activation reaches, then walks every computation for its edges. `producer_edges` is one 91-line function that holds seven rules: the lock subtree, the self exemption, lock cover, contact ordering, the mover exemption, literal consumers, and Bang roots. `reader_edges`, `input_edges` and `dynamic_writers` add the rest.
- **Ordering** (`tick/ordering.rs`) sorts those edges into Turns. It also reads the Lookup directly: Grid position for tie-breaks, each computation's Expression for cycle closure, and `writes` for input writers. When a root joins during a Tick, `Progress::activate` calls back into `tick::activate`, `producer_edges` and `reader_edges`. So the call graph runs tick → ordering → tick.
- **Execution** (`tick/execution.rs`) matches every `LockTarget` variant itself, and asks `written_over` which Functions and roots a write reached.

Ordering can be tested only by going through `plan`. A rule change such as bang-function ticket 08 means finding the rule inside `producer_edges` and checking every caller. Five open tickets go through this code: bang-function/08, observable-schedule/02, placement-semantics/01 and /04, and source-read-write/13.

## Solution

One module, `tick/dependencies.rs`, derives every Dependency (see `CONTEXT.md`) from a Lookup. It also answers the questions execution asks of the same reservations. Ordering schedules a plain graph and never reads a Lookup. Execution asks the dependency module instead of reading Lookup fields.

No Grid plays differently. No scheduling edge, activation, diagnostic or ADR changes.

## User Stories

1. As a maintainer changing a scheduling rule, I want every edge rule in one module, so that I change it in one place and can see its neighbours.
2. As a maintainer deciding bang-function ticket 08, I want the mover exemption to be one named rule with its own test, so that keeping or removing it is a one-rule change.
3. As a maintainer working on observable-schedule ticket 02, I want activation worked out in one place, so that recording why a root was activated has one home.
4. As a maintainer reading ordering, I want it to sort a plain graph, so that I can understand Grid tie-breaks, cycle closure and writers-first order without knowing what a computation is.
5. As a maintainer testing ordering, I want to build a graph by hand, so that a tie-break or cycle case needs no Grid.
6. As a maintainer testing an edge rule, I want a test that names the rule and shows its Grid, so that a regression points at the rule rather than at a Tick outcome.
7. As a maintainer reading execution, I want it to ask what a write or a lock reached, so that it stops depending on how the Lookup stores those facts.
8. As a maintainer, I want the tick → ordering → tick callback gone, so that the modules form a line I can read in one direction.
9. As a reviewer, I want every existing Tick, schedule-reuse and property test to pass unchanged, so that the refactor is shown to move no behaviour.
10. As a performer, I want every Grid to play exactly as before.

## Implementation Decisions

- **Scope.** The dependency module owns:
  - the edges: parent, lock, producer, reader and input;
  - the dynamic writers;
  - which roots activation reaches, before the Tick and when roots join;
  - what an admitted write reached;
  - what a lock targets.
  Building the Lookup and its reservations stays where it is, for architecture candidate 2 to take later.
- **Interface.** The module answers four questions:
  - `derive(lookup)`: the active roots, every edge and the dynamic writers, before the Tick. It replaces `active_roots`, the edge loop in `order_turns`, `dynamic_writers` and `input_edges`.
  - `join(lookup, active, roots)`: the roots activation now reaches, and their edges and dynamic writers, when roots join during a Tick.
  - `reached(lookup, write)`: the Functions and Bang roots an admitted write covers. It replaces execution's use of `written_over`.
  - `lock_target(index)`: what a lock reaches. Lock targets move out of `Lookup::new`, and the Lookup stops holding `locks`.

  `PortalRelationships`, `FunctionContact` and `LockTarget`'s construction become private to the module.
- **Ordering's input.** Ordering is given:
  - the node count;
  - the edges and the dynamic writers;
  - each node's Grid-order key;
  - each node's Expression, for cycle closure.

  It answers the order, which nodes a cycle stops, and which nodes each cycle holds. Tick scheduling words the cycle diagnostics, which today come from `diagnose_cycles` in ordering. Ordering's private `Dependencies` struct is renamed `Graph`.
- **Roots that join.** `take_turns` takes a one-method trait that answers `join` for the roots a Turn activated. Production implements it with the dependency module. Ordering's tests implement it with a fixed table, so the trait has two implementations and the seam is real.
- **Names.** "Dependency" is the glossary term. "Dependency derivation" describes the module in comments and is not a glossary term.
- **Sequencing.** This lands before bang-function ticket 08 and observable-schedule ticket 02. It ships as one pull request of three commits, one per ticket, each green.
- **Unchanged.**
  - ADR 0032 (dependency-ordered execution), ADR 0060 (the edge rules) and ADR 0036 (reservations) are kept, not reopened.
  - `ScheduleCache` and `DerivedRow::schedules_as` read the same inputs as before.

## Testing Decisions

- A good test here observes a Dependency, an order or a Tick outcome through an interface, never a private helper.
- **Ordering** gets unit tests on hand-built graphs, through the fixed-table side of the join trait:
  - Grid tie-breaks between independent nodes;
  - cycle closure stopping a whole Expression;
  - writers-first order;
  - rebuilding the order when a root joins mid-Tick.
- **The dependency module** gets one named test per producer rule, each on a small Grid, through `derive`:
  - the lock subtree;
  - the self exemption;
  - lock cover;
  - contact ordering;
  - the mover exemption;
  - literal consumers;
  - Bang roots.

  It also gets one test of `join` for a root activated during a Tick.
- **Behaviour is pinned by what already exists**, which must pass unchanged:
  - the Tick tests and the tick/* family suites;
  - the schedule-reuse tests and their property;
  - the placement and Bang Function tests;
  - the nested-expression property, with the known self-suppression failure (self-suppression/01) unchanged.
- **Prior art:**
  - the Source Bundle table test, which pins declared facts at the interface;
  - `schedule_reuse.rs`, which compares a shared schedule with a fresh one.
- **Gates:** clippy and nextest on `orcvs` and `console`, then the workspace runs and doctests, with `PROPTEST_CASES=32`.

## Out of Scope

- Deciding bang-function ticket 08 or observable-schedule ticket 02.
- Deriving reservations once for the Language Map and the Tick (architecture candidate 2).
- Giving a Turn's state verbs (architecture candidate 6). The duplicated activation loop in `fire` is left for it.
- The self-suppression diagnostic (self-suppression/01).
- Any change to which edges exist, to activation, or to diagnostics.

## Further Notes

The design was settled by grilling architecture candidate 3 on 2026-10-12. The Source Bundle change (source-bundle/01) already rewrote the bundle rules in `producer_edges` against named facts, which makes moving them mostly mechanical.
