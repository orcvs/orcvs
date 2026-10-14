# Decide each Contact's Consequence in one module

Status: ready-for-agent

## Problem Statement

Every rule about what a producer's Portal reaches has two halves, and nothing makes them agree:

- **Scheduling** (`tick/dependencies.rs`, `producer_edges`) states each rule as Dependencies: the self-span exemption for a bundle that clears its own Span, emission's no-contact rule (ADR 0060 rule 3b), the mover exemption for an intrinsically active root, `lock_covers`, and Bang activation.
- **Execution** (`tick/execution.rs`) states the same rules again as runtime decisions: Function Replacement at an anchor, subtree suppression, the own-Expression diagnostic, the executed-computation guard, a missed Bang in `deliver_output` and `fire`, and the attempted check in `lock_portal`.

Both read the facts from `PortalRelationships`, and the `PortalRelationships` doc says the phases "decide separately". Dependency derivation shared the facts; the policy is still written twice.

The two most recent fixes, 5e465fe and 65f4634 (a dynamic write or replacement on its own Expression), were exactly a gap between the halves, and only a property test through `Source` found them. The Bang activation loop is written twice in execution (`deliver_output` and `fire`), each with its own miss check, and `ComputationState`'s fields are written from about twenty places.

## Solution

`tick/dependencies.rs` answers one `Consequence` for each Contact a Reservation, write or lock makes, and for each root a Bang aligns with, given the phase that asks. Scheduling turns "order after" into Dependencies. Execution applies the rest through a small set of Turn verbs and words the diagnostics.

No Grid plays differently. No Dependency, activation, diagnostic or ADR changes.

## User Stories

1. As a maintainer changing a reach rule, I want its scheduling and execution consequences side by side, so that I cannot change one and forget the other.
2. As a maintainer adding a Source Bundle kind or a Function family, I want to add one arm, not one in each phase.
3. As a maintainer testing a reach rule, I want a table row naming the rule, its Grid and its Consequence in each phase, so that a regression points at the rule and needs no property shrink.
4. As a maintainer reading execution, I want Turn state changed only through named verbs, so that "an attempted computation is never suppressed later" and "a Bang that reaches a taken root misses it" each hold in one place.
5. As a maintainer working on observable-schedule ticket 02, I want activation delivered through one verb, so that recording why a root was activated has one home.
6. As a reviewer, I want every existing Tick, schedule-reuse and property test to pass unchanged, so that the refactor is shown to move no behaviour.
7. As a performer, I want every Grid to play exactly as before.

## Implementation Decisions

- **Turn verbs (01).** A private `tick/execution/turns.rs` owns the `ComputationState` vector as `Turns`. Its fields are private to it; `execution.rs` and `operands.rs` ask it. Two verbs carry the duplicated rules:
  - `activate(root)` answers whether the Bang was delivered or missed because the root has taken its Turn without the activation it needed. An intrinsically active or already activated root is never missed.
  - `suppress(subtree)` suppresses every computation in the subtree that has not attempted its Turn.

  The verbs return outcomes and push no Effects. Execution words the diagnostics and keeps the roots a Selected write activated. They take no producer: observable-schedule/02 decides provenance.
- **Phase.** A Consequence is asked for in a phase:
  - `Schedule`: before the Tick. Nothing has a Turn, and the only Consequence is to order after. This keeps the glossary's line that a Reservation orders Turns and decides nothing else.
  - `Turn`: during the Tick, with the write's `Delivery` (Reserved, Joined or Selected) and a read-only view of the Turns. `Delivery` and `feeds_back` move from execution into the dependency module, because how a write's place was decided is what it may reach.
- **Consequence.** For writes (02) the Consequence of one Contact or aligned root is one of: order after, nothing, replace the Function, suppress the subtree, activate, miss the Bang, apply as feedback with the own-Expression diagnostic, or refuse as an ordering defect. The exact variants are settled while building 02; the rule is that each variant names what execution does, not which rule produced it.
- **Locks (03).** `lock_verdict(locker, phase)` is a sibling entry. It answers the same phase and Consequence types for a lock's subtree and absorbs `lock_covers` and `lock_portal`'s attempted check. `Locks` and `Lock` stay as they are.
- **Ordering defect.** A Reserved write reaching an attempted computation stays a refused write with a diagnostic. ADR 0028 rules out a panic inside Tick planning.
- **Names.** **Contact** joins the glossary: a Function whose Cells a Reservation or an admitted write covers. A Bang reaching an aligned root is activation, not Contact. `Consequence` names the answer; "reach" stays a verb.
- **Sequencing.** One pull request, one commit per ticket, each green. 01 lands first; 02 builds on its verbs.
- **Unchanged.** ADR 0028, 0032, 0034, 0060 and 0067 are kept, not reopened. `ScheduleCache` and `DerivedRow::schedules_as` read the same inputs as before. `ordering.rs` is untouched.

## Testing Decisions

- A good test here observes a Consequence or a Tick outcome through an interface, never a private helper.
- **The dependency module** gets one verdict table test, built with `fixture(grid, rows)`, with one row per rule in each phase that decides it:
  - the self-span exemption;
  - emission no-contact;
  - the mover exemption;
  - lock cover;
  - Bang activation and a missed Bang;
  - replacement at an anchor;
  - subtree suppression;
  - a dynamic write on its own Expression.
- **The ordering defect** gets one focused test that hands the Turn phase a Reserved write onto an attempted computation, as source-module-depth/10 did for its defensive rejection.
- **Behaviour is pinned by what already exists**, which must pass unchanged: the Tick tests and the tick/* family suites, schedule reuse and its property, the Bang Function and Halt tests, and the nested-expression property.
- **Gates:** clippy and nextest on `orcvs` and `console`, then the workspace runs and doctests, with `PROPTEST_CASES=32`.

## Out of Scope

- Deciding bang-function ticket 08 or observable-schedule ticket 02.
- Deriving Reservations once for the Language Map and the Tick, or one Output declaration for a Function (architecture candidate 3).
- Making the schedule cache key the scheduling input by construction (architecture candidate 5).
- Any change to which Dependencies exist, to activation, or to diagnostics.

## Further Notes

The design was settled by grilling architecture candidates 1 and 4 of the fourth pass on 2026-10-14.
