# Track reads a Portal as Jump does

Status: accepted. Supersedes the List clauses of [ADR 0063](0063-a-list-is-cells-in-a-claim-not-a-value.md): Orcvs has no List claim, no literal count read when the Source is parsed, and no untyped Item. ADR 0063's removal of the Sequence value and the `:` family stands. Amends [ADR 0032](0032-schedule-tick-execution-by-dependency.md): a dependency can be found at a Function's Turn.

**Track is an ordinary value Function with an Input Portal: `@t index count`.** `index` and `count` are Number operands like any other. Either can be a literal, a nested Function or Cells a Portal writes, and an empty one leaves Track pending ([ADR 0066](0066-an-unwritten-operand-leaves-its-function-pending.md)). At its Turn Track reads the two Cells `index % count` pairs east of its last operand, so `@t0103C4D4E4` reads `D4`. It answers what it reads through its Output Portal and, nested, as its Return. A `count` of `00` is diagnosed at the Turn, as a wrap by zero is for every other Function. A pair past the row edge is diagnosed, as a Jump's Input Portal outside the Grid is.

**Track reads its Input Portal exactly as a Jump does ([ADR 0014](0014-spatial-functions-preserve-performative-behaviour.md)).** Empty Cells are copied: Track clears its Output Portal and, nested, leaves its parent pending (ADR 0066). `**` answers a Bang, which Track relays through its Output Portal and which activates a root there. A Function spelling answers that Function, so Function Replacement applies where it lands. A Number or a Note answers that Atom. Anything else, such as a partial pair, a Comment or Cells that straddle two Language Units, is diagnosed as partial or invalid input. Track has no reading rules of its own.

**The Cells east of Track are ordinary Source.** Track claims only its operands. The Cells it selects from are parsed, painted and edited as any other Source, and the pair Track reads is whatever working Source holds there at its Turn. Whether a bare value such as `C4` standing on its own is diagnosed is a question for the Parser. It applies equally to Cells a Jump reads, so this decision does not answer it.

**Track's read is ordered by ADR 0032's rule, completed at Track's Turn.** A Function goes after whatever writes the Cells it reads. A Jump's Input Portal is a fixed offset from its anchor, so its dependency is known before the Tick. Track's pair depends on `index` and `count`, which are themselves inputs, ordered before Track by the same rule. Once they settle, Track's pair is known. Any writer of that pair that has not yet taken its Turn goes first, and Track takes its Turn after it. If that writer waits on Track, the two form a same-Tick dependency cycle, diagnosed under [ADR 0065](0065-an-error-never-stops-the-performance.md). The loop that orders Turns continues during the Tick so that a dependency found at a Turn can join it. Nothing is reserved for Track before the Tick, and every other Function is ordered as before.

## Amendment: static and dynamic Input Portals

An Input Portal is named by when its position is known. A static Input Portal's position is a fixed offset from its Function's anchor, known when the Source is parsed: a Jump's Input Portal is static, as are Increment's and Interpolation's, so its dependency is known before the Tick. A dynamic Input Portal's position is selected by its Function's operands: Track's Input Portal is dynamic, the only one, so its dependency is found at its Turn. The distinction is of position, not contents: every Input Portal's Cells are read from working Source at its Function's Turn.

## Considered options

- **A List in Track's claim (ADR 0063).** Track's Items were part of its Expression, its count was a literal fixed when the Source was parsed, and its Items were copied untyped. Track then needed its own parse, its own Language Unit kind, its own answer and its own copy rules, none of which Jump needs. Rejected for a Function that reads Cells and answers them.
- **Ordering Track after every writer east of it to the row edge.** This needs nothing from `count`, but it orders writers Track never reads and can report a cycle that does not exist.
- **Ordering Track after writers in the `count` pairs the Source holds when the Tick starts.** This freezes the count for the Tick and fails when the count is nested.
- **Not ordering Track's read, as Orca's `T` does.** Track would read whatever its turn in Grid order finds, so a writer east of it or below it would be seen a Tick late.

## Consequences

A cycle found at Track's Turn preserves completed Turns and stops the remaining
computations in every Expression it reaches, per [ADR 0068](0068-a-discovered-cycle-preserves-completed-turns.md).

When `index` and `count` are literals that nothing writes during the Tick, Track's pair is known before the Tick. Its dependency could then join the schedule built in advance, which keeps that schedule reusable across Ticks. This is an optional optimisation that never changes what Track reads. Add it only when a benchmark shows the Turn-time path costs something.

A Track that reads `**` activates the root its Output Portal lands on, as a Jump does. This differs from ADR 0063, where a copied `**` was characters and activated nothing.
