# 12 — Fit a Sequence-capable Output Portal to its answer

**What to build:** A Sequence-capable root's Output Portal highlight covers its answer rather than the rest of the row. It shows at least four Cells from the Output Portal, whether empty or written. Past those four, it extends by each following Cell pair that holds written content and stops at the first blank pair. It never reaches past the root's Reservation, so it is clipped at the row edge. A scalar root keeps its Cell pair.

Four is the minimum because a Function that never writes more than two Cells would be declared scalar. Being Sequence-capable only matters when an answer can be longer. It also lets a viewer tell a Sequence-capable root from a scalar one before any Tick.

The Reservation itself is unchanged: Tick scheduling still reserves from the Output Portal to the end of the row (ADR 0036), and `10`'s agreement test still compares the Reservation with the scheduler. Only the highlight narrows, and it still reads the current Source revision alone (`05`).

**Blocked by:** None (can start immediately). `06` and `10` are resolved.

**Status:** ready-for-agent

- [ ] A Sequence-capable root's highlight covers exactly its written answer where the answer is four Cells or longer: `01020304` south of `:-0104`, `C4c4D4` south of `:#C4D4`, `04030201` south of `:<:-0104`, and `010203` south of `:&.+0001:-0203`.
- [ ] An empty Sequence-capable Output Portal shows four tinted Cells before any Tick.
- [ ] A one-Atom answer shows four tinted Cells.
- [ ] The highlight stops at the first blank Cell pair after the minimum.
- [ ] Near the row edge, the highlight is clipped to the Reservation.
- [ ] A regression test uses a two-column layout like the one this was found in, where Sequence roots on the left and scalar roots on the right share rows. The left roots' tint does not reach the right roots' Expressions or their scalar Output Portals.
- [ ] Scalar roots, the Reservation, Tick scheduling and the agreement test are unchanged. Every existing Tick test passes unchanged.
- [ ] `05`'s Width row, the theme documentation and `06`'s Sequence case describe the fitted highlight.

## Comments

Found from a screenshot on 2026-09-19. With the highlight covering the whole Reservation, the left column's Sequence roots tinted the rest of every Output Portal row, running under the right column's Tick Expressions and their scalar answers.

Known limit: content written directly after an answer is absorbed into the highlight past the fourth Cell. That covers stale Cells a shorter answer left behind (a Tick writes only the answer's width and does not clear past it) and text written by hand. `13` records the alternative that avoids this.
