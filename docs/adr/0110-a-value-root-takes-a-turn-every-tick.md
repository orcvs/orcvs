# A value root takes a Turn every Tick

Status: accepted. Decided by Toby on 2026-11-15, in `.scratch/requirements-snapshot/` (decision
D5). Amends [ADR 0006](0006-bang-activation-includes-self-banging-functions.md)'s clause that an
ordinary root Expression is inert until Bang activation. Records as the decision what
[ADR 0032](0032-schedule-tick-execution-by-dependency.md) kept as the first implementation's
policy.

ADR 0006 says that "an ordinary root Expression is inert until Bang activation", with Self-Banging
Functions as "the explicit exception in activation source". Read literally, every root but a mover
waits for a Bang, a value root included. ADR 0032 then kept "the current Function-family activation
policy—value roots calculate each Tick and terminal roots require a Bang" for its first
implementation, without saying it amended ADR 0006. Every later decision assumes that policy:
[ADR 0073](0073-a-function-with-nothing-to-write-answers-nothing.md) has Equality, Every and
Euclidean answer a Clear "on every Turn they do not Bang", which they can do only if they take a
Turn on every Tick. The Function table declares it per Function, the Function reference teaches it,
and every example depends on it. The requirements snapshot found the two readings in conflict and
could not state either as binding.

## Decision

**A root whose Function answers a value takes a Turn on every Tick, with nothing delivered to it.**
Add, Clock, Every, Euclidean, Equality, Increment, the Reads, Writes and Copies, and every other
value Function are intrinsically active at the root. So are the Self-Banging Functions and the Bang
Function, as ADRs 0006 and 0072 already say.

**A root whose Function performs any other effect is inert until a Bang reaches it.** The Terminal
Output Functions (the `!` family), the Directional Bang Functions and Halt take a Turn only when a
Bang activates them during the Tick. This is the part of ADR 0006's rule that stands.

**Where a root's activation comes from is declared per Function, in the Function table.** It is not
derived from whether the Function answers a value, because a Self-Banging Function answers an
effect and is active anyway. A nested Function takes its Turn from the root that owns it, so what it
declares is never read.

Halt still withholds a Turn from the root south of it, whichever kind that root is, and the
scheduling, ordering and Bang-lifetime rules of ADR 0032 and its successors are unchanged.

## Considered options

- **Keep ADR 0006 as written: every root but a mover waits for a Bang.** Every pulse-driven Source
  would go silent, because `~*0104` above a Play would never take the Turn that writes its `**`; the
  site's examples, the Function reference's checked examples and the first-result tutorial would all
  need rework, and the language would need an always-active source other than a mover, which no
  decision provides. It is also further from Orca, whose clock and delay operators run every frame.
- **Leave the question open.** The requirements would keep describing the most basic activation
  rule as observed behaviour rather than a decision.

## Consequences

- `docs/requirements/evaluation.md` states the rule as EVAL-050 and drops it from its open decisions.
- No code changes: `lang/src/atom.rs` already declares every value Function, Self-Banging Function
  and the Bang Function `Intrinsic`, and every Terminal Output, Directional Bang and Halt Function
  `Bang`.
