# A Source Bundle states two facts

Status: resolved

## Problem Statement

Someone changing how a Source-writing Function behaves has to work out what its Source Bundle means, and today that meaning is spread over four modules. The Interpreter's crate declares the three bundles: Advance, Emit and Fire. The Portal module matches on them to decide which Cells a bundle writes. Tick scheduling has three single-purpose predicates, each matching on them for one ordering rule: whether a bundle moves, whether it clears its own Span, and whether it emits without vacating. Execution matches on them again to choose between firing, placing, leaving `**` on refusal, or diagnosing.

These facts are related only by doc comments that say "as with `advances`". Adding Fire for the Bang Function meant editing all four modules. A fourth bundle, or any change to Advance, would mean finding every match again. Nothing tests a bundle's meaning directly: it can only be seen end to end through a Tick.

## Solution

Every question those four modules ask comes down to two facts about the bundle:

- **Clears its own Span**: the bundle's first write is spaces over the Cells the producer stands in. True for Advance and Fire.
- **Writes displaced**: the bundle writes a spelling at the producer's Span displaced. True for Advance and Emit.

`SourceBundle` states these two facts itself. Every other question is derived from them where it is asked:

- the Cells it writes;
- whether a reservation over its own Span is the design rather than a cycle;
- whether the roots it contacts are ordered after it;
- whether an intrinsically active contact is exempt;
- what a refused write costs.

Advance, Emit and Fire stay as names. The language's behaviour does not change.

## User Stories

1. As a maintainer adding a Source Bundle, I want to state its two facts in one place, so that every rule that depends on the bundle follows without edits elsewhere.
2. As a maintainer changing Advance, I want one definition to change, so that I cannot update the Portal sites and forget the schedule.
3. As a maintainer reading Tick scheduling, I want each bundle rule written against a named fact, so that I can see why a Self-Banging Function is exempt from its own reservation and a Directional Bang Function is not.
4. As a maintainer, I want the self-overlap exemption to read "clears its own Span", so that the Bang Function and the Self-Banging Functions share one stated reason.
5. As a maintainer, I want the rule "an emission orders no contacts" to read "does not clear its own Span", so that ADR 0060 rule 3b's reasoning, that occupancy alone orders nothing, has one home.
6. As a maintainer, I want the intrinsically active contact exemption to read "clears its own Span and writes displaced", so that a mover's stance toward an intrinsic root is stated once.
7. As a maintainer, I want the refusal cost to read off "clears its own Span", so that ADR 0006's rule, that a producer leaving its Cells reports there with `**` while one staying diagnoses, is the same fact the `SourceBundle` doc already names.
8. As a maintainer, I want the Portal module to build a bundle's writes from the two facts, so that its own-Span site and displaced site come from the declaration rather than from a variant match.
9. As a maintainer, I want execution to stop matching on the variants to choose its refusal arm, so that a fourth bundle cannot fall into the wrong arm silently.
10. As a maintainer, I want the Bang Function still to take its own Turn, which activates its aligned roots and clears its Span, so that ADR 0072's behaviour is untouched.
11. As a maintainer, I want a table test that asserts both facts for every Source-writing Function, so that a bundle's meaning is tested at its interface rather than only through a Tick.
12. As a maintainer, I want every existing Tick, placement and Bang Function test to pass unchanged, so that the refactor is shown to move no behaviour.
13. As a maintainer deciding ticket 08 in bang-function, I want the mover-to-contacted-root rule written against the two facts, so that keeping or removing that edge is a one-line change.
14. As a reader of the glossary, I want a Source Bundle entry, so that "bundle" in code and ADRs has a domain definition.
15. As a performer, I want every Grid to play exactly as before.
16. As a reviewer, I want the change to add no Bang-specific rule and change no scheduling edge, so that it needs no language decision.

## Implementation Decisions

- **Scope is facts only.** The policy states facts. It does not execute a bundle behind a write port: that seam would have one adapter, execution, which makes it a hypothetical seam. Execution keeps its own code and reads the facts.
- **Home.** The two facts are methods on `SourceBundle` in the `lang` crate, beside the enum and the `SourceEffect` displacement. `lang` already answers geometry relative to the anchor, and `orcvs` turns it into Cells in one place, the Portal module.
- **Names.** `clears_own_span()` and `writes_displaced()`. Each is an exhaustive match on the variant inside `lang`, so adding a variant forces both answers.
- **Portal sites.** A Source-writing Function's writes are its own Span if the bundle clears it, then the displaced destination if it writes one, in that order. This is the emission order ADR 0020 resolves and ADR 0004's bundle declares. For the three bundles this gives exactly today's sites:
  - Advance: own Span, then destination;
  - Emit: destination only;
  - Fire: own Span only.
- **Schedule.** The three predicates in Tick scheduling are deleted, and the producer-edge rule reads the facts directly:
  - The self-overlap exemption is `clears_own_span()`.
  - Contacts are ordered after the producer only when the bundle clears its own Span; an emission orders none, per ADR 0060 rule 3b.
  - An intrinsically active contact is exempt only when the bundle also writes displaced, which is the mover case.
  - For Fire, contact ordering is vacuous. Its only contact is itself, and the self-overlap exemption already covers it. No edge changes.
- **Execution.** A Fire still routes to the Bang Function's Turn. For a bundle that writes displaced, the refusal arm reads `clears_own_span()` where it reads "advancing" today: true leaves `**` in the producer's Span, false diagnoses. The admitted-move check that excludes the producer's own Cells reads the same fact.
- **Glossary.** `CONTEXT.md` gains a Source Bundle entry. It names the three bundles by the two facts and states that refusal cost follows from the first.
- **Ordering with bang-function ticket 08.** This lands first and keeps today's mover-to-contacted-root edge. Ticket 08 then changes one rule.
- **Unchanged.**
  - Replacement still compares the declared Source write as a whole, so its guard is untouched.
  - `SourceEffect` keeps its displacement, spelling and bundle fields.
  - No ADR is reopened.

## Testing Decisions

- A good test here observes a bundle's facts or a Tick's outcome through an existing interface, never a private helper of scheduling or execution.
- **The policy's interface test** is the existing per-Function Source-effect table test in `lang`. It already states every Source-writing Function's displacement, spelling and bundle. It is extended to assert both facts for each one, so the two facts are pinned for Advance, Emit and Fire through every Function that declares them.
- **Behaviour is pinned by what already exists.** These suites must pass unchanged:
  - the Tick tests;
  - the placement tests and their property;
  - the Bang Function tests, E1 to E9 and tickets 01 to 04;
  - the Function reference settled-area test.

  No new end-to-end test is added, because no outcome changes.
- **Prior art:**
  - the Source-effect table test;
  - the `DECLARED_CHANGES` replacement guard in the Function table, which asserts declared facts per Function;
  - the `BangAnswer` column, which moved a composed predicate into a declared fact the same way.
- **Gates:** clippy and nextest on `lang` and its dependents, then the workspace runs and doctests, with `PROPTEST_CASES=32`.

## Out of Scope

- Deciding bang-function ticket 08, the mover-to-contacted-root edge.
- Moving bundle execution behind a port, or merging the Bang Function's Turn into a generic bundle executor.
- The other architecture-review candidates:
  - deriving reservations once;
  - separating dependency derivation from graph scheduling;
  - further Function-table roles;
  - the shared Tick fixture;
  - Turn-state verbs.
- Any change to the three bundles' behaviour, spellings or displacements.

## Further Notes

`SourceBundle` was introduced with the Directional Bang Functions (Advance and Emit). That commit made three scheduling rules ask about the bundle rather than whether a Function writes Source. Fire joined with the Bang Function. The two facts were found by reading the four match sites: every question they ask is one fact, its negation, or the two together.
