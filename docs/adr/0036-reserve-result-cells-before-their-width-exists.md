# Reserve result Cells before their width exists

Status: accepted. Lifts the "variable-width Sequence projection" deferral of [ADR 0032](0032-schedule-tick-execution-by-dependency.md).

A dependency edge must name every Cell a write touches, and [ADR 0032](0032-schedule-tick-execution-by-dependency.md) fixes the schedule before any Function evaluates. A Sequence has no width until it is evaluated, which is why the executor refused one outright and planned no write at all. Tick scheduling therefore reserves the Cells a result *can* reach, derived from what its Functions declare, and execution delivers a Sequence through the same Portal a scalar uses.

Sequence-capability is derivable before evaluation because a Sequence can only reach a computation from a nested child. [ADR 0034](0034-execute-against-live-typed-expressions.md) makes a spatial write literal characters that the receiving operand decodes by its declared literal type, and [ADR 0007](0007-sequence-is-a-first-class-value.md) gives a Sequence no literal spelling to decode, so an operand written over from Source is always one Atom. A computation can answer a Sequence only if its Function answers one whatever its operands carry, or if it widens over an operand a child already answers a Sequence to. That propagates bottom-up over the tree the scheduler already holds.

Both facts are declared per Function beside its kind, its pervasion, and its Bang, and neither is inferred from a family prefix or from a signature. Pervasion cannot stand in for the second: Equality broadcasts to find its comparison pairs and still answers one scalar under [ADR 0011](0011-general-arithmetic-wraps-over-bytes.md), so it is pervasive and never widens. No Function answers a Sequence today, so every computation is reserved a scalar Cell pair and no existing schedule changed when the columns arrived; ADR 0007's Range, Reverse, Concatenate, and Replace then arrive as one table row each rather than as a scheduling change.

A scalar computation reserves the Cell pair it already reserved, and refuses any other encoding width, which is what keeps a one-Cell result at the end of a row from being written into Cells the schedule never named. A Sequence-capable computation reserves its destination through the end of that destination's row. No Span reaches past the row it begins in, so the rest of the row is the smallest reservation that can name every Cell such a write might reach, and it is available before any width is.

The reservation orders Turns; the admitted write decides what happened to a Cell. The two coincide for a scalar and come apart for a Sequence, so a computation inside a reservation the encoding stopped short of is ordered after its producer and then not suppressed, replaced, or activated — it was never written over. Every admitted write is a subset of what was reserved, so each relationship execution reads is one a dependency edge already named.

Nothing else is added to deliver a Sequence into Source. [ADR 0009](0009-portals-resolve-tick-plan-destinations.md)'s Portal already refuses an encoding wider than its destination row entire, which is [ADR 0007](0007-sequence-is-a-first-class-value.md)'s complete-fit rule, and one admitted write already fans out Cell by Cell when the Tick Plan resolves, which is [ADR 0020](0020-order-tick-effects-by-source-position.md)'s conflict resolution. A Sequence therefore inherits both rather than restating them for a second width, and an empty Sequence still plans no write and reaches no Portal.

Of the three rules a delivered result applies, only suppression changes. Bang activation and Function replacement each read the answer rather than the Cells, and Bang and a Function Atom are single Atoms by construction, so no Sequence satisfies either. A Sequence carrying a Function spelling writes those Cells as ordinary Source content under ADR 0007, to be parsed by the following Tick. Replacement gains one clause: [ADR 0034](0034-execute-against-live-typed-expressions.md) already refuses a replacement at an original anchor that changes activation requirements or output kind, and a replacement that changes the reserved width is refused with them, because the schedule reserved from the Function it found there.

## Rejected alternatives

Reserving the rest of the row for every computation needs no declaration and no propagation. It also orders every computation against every later Expression in its destination row, which manufactures cycles between Expressions that never touch and would reject Ticks that work today. The declared columns exist to keep the scalar case exactly as wide as it was.

Diagnosing at execution time, when a wide write reaches an already-executed computation, is the failure the schedule exists to prevent. ADR 0034 calls an output reaching an executed Function "a bug in the ordering, not a supported case", and a rule that discovers the width too late to order by it rejects the whole Tick rather than delivering the result.

## Consequences

A Function that answers a Sequence widens the reservation of every pervasive ancestor between it and its root, so a deeply nested Range orders its root against the whole of the destination row. That is the cost of a reservation stated before a width exists, and it is paid only by the expressions that can produce one.

Newly anchored code, arbitrary computed destinations, and explicit delayed feedback remain deferred as ADR 0032 left them. A destination that a Tick computes would change which Cells a reservation covers after the schedule was fixed, which is the same problem this decision solves for width and is not solved here.
