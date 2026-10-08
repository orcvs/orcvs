# An error never stops the performance

Status: accepted. Amends the cycle clauses of [ADR 0034](0034-execute-against-live-typed-expressions.md), [ADR 0014](0014-spatial-functions-preserve-performative-behaviour.md) and [ADR 0060](0060-value-inputs-wait-placement-tests-occupancy.md), which reject every effect of a Tick that holds a same-Tick dependency cycle. Number 0064 is skipped: commits on `main` cite it for a decision that was withdrawn.

The no-Turn guarantee below is amended by [ADR 0068](0068-a-discovered-cycle-preserves-completed-turns.md) for a cycle discovered during a Tick: completed Turns retain their Effects, and every remaining computation in the affected Expressions stops.

**An error costs the Expressions it reaches, never the performance.** Orcvs is played live, and a performer makes mistakes while it plays. Every Tick publishes the effects of every Expression that does not depend on a fault, whatever else in the Grid is wrong. No error, in the Source or in Orcvs itself, rejects a whole Tick.

Most errors already behave this way. A type, domain or decode failure diagnoses one computation; a refused write keeps the Function's answer for its parent; a parent's error leaves its child's write standing ([ADR 0034](0034-execute-against-live-typed-expressions.md)). This decision makes the rule general and removes the two exceptions.

**A same-Tick dependency cycle stops the Expressions on it and the Expressions that depend on them.** An Expression depends on a stopped one when any of its computations needs a stopped computation first: it reads Cells that computation writes, its root is activated by that computation's Bang or locked by its Halt, or that computation's write covers it. The stopped set grows until no other Expression depends on it. Every Expression outside it is ordered and takes its Turns exactly as it would if the stopped Expressions were absent.

Nothing in a stopped Expression takes a Turn, so no part of the cycle executes and no order is chosen for it. Nothing is delayed to a later Tick. These are the two guarantees [ADR 0034](0034-execute-against-live-typed-expressions.md) gave by rejecting the whole Tick, and both still hold. What changes is that independent Expressions are no longer rejected with the cycle.

**Every stopped Expression is diagnosed.** Each cycle is diagnosed once, at the first computation in Parser order that lies on it. Each stopped Expression that holds no computation on a cycle, and that activation can reach this Tick, is diagnosed at its root as waiting on that cycle; one nothing activates would take no Turn without the cycle either, so it has nothing to wait for. A performer can therefore tell a cycle from what it starved, and nothing goes silent.

**An ordering defect stops one Turn, not the Tick.** When a write or a Halt lock would reach a computation that has already taken its Turn, the schedule was wrong. That Turn's write or lock is refused and diagnosed, and the Tick continues.

## Consequences

A cycle in one corner of the Grid no longer stops a counter, a Play Command or a Bang anywhere else. A layout that feeds a cycle still publishes what it computes before the cycle, so one Tick can show part of a connected arrangement and not the rest; which part is decided by the dependency edges and is the same every Tick. The scheduler's exceptions that avoid ordering edges because a cycle would "cost the whole Grid its Tick" now protect only the Expressions involved, and keep their place because those Expressions are still worth playing.

## Considered options

Rejecting the whole Tick is simple and was the rule. It makes one mistake silence everything, which a live performance cannot afford. Delaying a cycle to the next Tick hides the fault and makes outcomes depend on Tick history. Breaking the cycle at an arbitrary edge would execute part of it in an order the Source does not determine.
