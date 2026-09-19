# The Grid is bounded by definition

Status: accepted. Answers what an edge *is*, which [ADR 0005](0005-defer-source-addressing-for-infinite-canvas.md) left open when it deferred Source addressing. It does not reopen that deferral and it does not decide whether Orcvs becomes an infinite canvas.

The Grid is part of the language: a defined rectangle, at least one column and one row. A Position outside it does not exist. That is what Grid already says, and this decision records it as meaning rather than as a convenience of the bounded prototype.

[ADR 0036](0036-reserve-result-cells-before-their-width-exists.md) therefore stands unchanged. A Sequence-capable computation reserves its destination through the end of that destination's row because a row is the whole horizontal extent there is. The reservation orders Turns; it is not a bounds check that could be rewritten as a declared maximum width or resolved after evaluation. An unbounded row would give that rule no referent.

Leaving the Grid is a terminal state. A Self-Banging Function that would move out of the Grid changes its current Span to `**` ([ADR 0006](0006-bang-activation-includes-self-banging-functions.md), [ADR 0014](0014-spatial-functions-preserve-performative-behaviour.md), [ADR 0020](0020-order-tick-effects-by-source-position.md)). A Jump Function whose destination lies outside the Grid diagnoses and writes nothing. Those rules need no replacement stop condition and no replacement diagnosis: there is an outside, and it is not a Position.

The execution path needs no other change. Edge behaviour on the Tick path refuses rather than clamps; the clamping helpers are Cursor-only; `lang` has no Grid concept; Tick planning is already O(expressions). Presentation is not the Grid: a Render Frame, a Paint, and the viewport can show a subset, zoom, pan, or letterbox, and this decision does not constrain them. [ADR 0038](0038-the-console-owns-the-source-grid-transform.md) already owns the transform.

`CellIndex` as a scalar `y * cols + x`, and the absence of any Grid resize path, are a rearchitecture under a growable Grid of any kind — not only an infinite one. That work is not this decision. A later canvas, if any, is a change of extent: how large a Grid is, or how a console shows one, not a change in what "outside" means.

## Rejected alternatives

**Bounded for now.** Keep the current rectangle as staging and write Reservation, Self-Bang, and Jump so they could survive the edge going away. That invents a row-end replacement and a Self-Bang stop condition the language does not have, for a canvas question this ADR does not answer, while `Reserved::Row` is about to become reachable and Self-Banging movement already dies at the edge.

## Consequences

`CONTEXT.md` is unchanged. Grid, Span, Reservation, Bang, and Jump Function already mean this.

[ADR 0005](0005-defer-source-addressing-for-infinite-canvas.md) remains the addressing deferral. This decision does not retract it and does not implement it. [ADR 0049](0049-a-position-is-two-numbers.md) later ended that deferral: a Grid is at most 256 by 256, which settles the canvas question this decision left open.
