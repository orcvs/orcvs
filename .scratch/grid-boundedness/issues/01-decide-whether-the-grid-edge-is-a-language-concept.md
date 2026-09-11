# 01 — Decide whether the Grid's edge is a language concept

**What to build:** An ADR settling whether the Source Grid's boundary is permanent language
vocabulary or a convenience of the bounded prototype, and what each dependent semantic means under
the answer.

**Blocked by:** None — can start immediately.

**Status:** ready-for-human

**Tags:** release/v1

- [ ] The ADR answers one question plainly: is the Grid bounded by definition, or bounded for now?
- [ ] It says what bounds a Reservation. `Reserved::Row` is `grid.cols() - output.x()`
      (`orcvs/src/source/tick.rs:142`) and orders Turns rather than merely checking a bound. If the
      edge is permanent, ADR 0036 stands unchanged and this is recorded as deliberate. If it is not,
      the ADR says what replaces "through the end of its row" — a declared maximum width, a
      reservation resolved after evaluation, or something else — because an unbounded row gives that
      rule no referent.
- [ ] It says whether leaving the Grid is a terminal state. Today it is the only way a Self-Banging
      Function stops (ADR 0006, ADR 0014, ADR 0020, `CONTEXT.md` under Bang and Jump Function). If
      the edge is not permanent, the ADR says what stops a Self-Banging Function instead, and what
      an out-of-Grid Jump diagnoses when there is no out of Grid.
- [ ] It records that the execution path needs no other change: edge behaviour on the Tick path
      refuses rather than clamps, the clamping helpers are Cursor-only, `lang` has no Grid concept,
      and Tick planning is already O(expressions).
- [ ] It records that `CellIndex` as a scalar `y * cols + x`, and the absence of any Grid resize
      path, are rearchitecture under a growable Grid of any kind — not only an infinite one.
- [ ] It does not decide whether Orcvs becomes an infinite canvas. It decides what the language
      means by an edge, so that answering the canvas question later is a change of extent rather
      than a change of meaning.
- [ ] `CONTEXT.md` is updated only if the answer changes what Grid, Span, Reservation, Bang or Jump
      Function mean. A decision to keep the edge permanent changes no glossary text.

## Comments

`ready-for-human`: this is a language design decision, not an implementation. An agent can draft the
ADR once the answer exists.

The cost of answering late is not evenly distributed. Deciding the edge is permanent costs nothing
at any time — it ratifies what is already written. Deciding it is not costs almost nothing today,
while `Reserved::Row` is unreachable and Bang movement is unwritten, and costs a language migration
once `sequence-values/03` and `05` and `spatial-tick-planning/03`, `04` and `05` have landed. Those
five issues are blocked on this one for that reason, not because the work is unclear.
