# A Position is two Numbers

Status: accepted. Supersedes the addressing deferral in [ADR 0005](0005-defer-source-addressing-for-infinite-canvas.md) and closes the infinite canvas that [ADR 0043](0043-the-grid-is-bounded-by-definition.md) left open.

**A Grid is at most 256 columns by 256 rows, and that limit is part of the language.** A shape wider or taller than that is not a Grid, just as one with no columns is not. Every Position can then be spelled as two Numbers under [ADR 0010](0010-two-cell-hexadecimal-numbers.md): column first, then row, each `00`–`FF`, counted from `00 00` at the top-left. A Grid may be smaller than the limit on either axis; its shape is still its own, and the default stays exploratory.

**A pair of Numbers the Grid does not reach names nothing.** Under a Grid narrower or shorter than 256, `(column, row)` past its extent is not a Position, exactly as ADR 0043 defines outside. A Function whose destination or source is such a pair diagnoses and writes nothing, as a Jump Function out of the Grid already does. Portals ([ADR 0009](0009-portals-resolve-tick-plan-destinations.md)) resolve such a pair to a Position or refuse it; nothing about Tick commit changes.

This decision fixes the address space and how a Position is spelled. It adds no Function. The absolute Address Functions that read or write at a spelled Position are follow-up work under the `&` prefix.

## Rejected alternatives

**A limit on addressing only.** Allow larger Grids and let two Numbers reach only their top-left 256 by 256. That brings back an outside that exists but cannot be named: Positions no Source could address, and a boundary that moves depending on whether a Cell was reached by the Cursor or by a Function. ADR 0043 refused exactly that kind of outside.

**Keep the deferral for an infinite canvas.** ADR 0005 declined two-digit absolute Positions to keep a sparse unbounded canvas possible. A canvas has no Position a byte pair can spell, so every absolute Address Function would wait on a question no one is working on. Spending two Cells per axis matches how a Number is already written, and gives every Position a Source can hold a spelling the Source can write.

**Base-36 single-glyph axes, as Orca does.** ADR 0010 already rejected a single glyph per Number; an axis spelled differently from every other Number would be a second numeric notation.

## Consequences

A stored Source whose Grid exceeds 256 on either axis no longer loads. No default or shipped Grid has ever been that large.

`CellIndex` is bounded by 65,536 Cells. A growable Grid, if one comes, grows within the limit rather than past it.
