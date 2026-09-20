# Paint per-Cell cost

What one drawn Cell costs in `Paint::derive_with_colours`, and what it is allowed to cost.

`syntax-highlighting/08`–`10` moved Paint from two scalar projections to the parser's shared claim,
and added the Output Portal fact beside it. The per-Cell body gained an `Option<&Claim>` dereference,
a `HashMap<*const Claim, bool>` lookup, an `output_portal()` read, and the wider `claim_paint`
decision. Measured on CI, that made every drawn Cell about 5.5 times more expensive.

This effort establishes where that cost sits and brings it back down. ADR 0050 supersedes ADR 0044:
the Language Map answers every Paint distinction per Cell, once per Source revision, and the Render
Frame carries those finished answers into the console.

## Vocabulary

- **Per-Cell body** — the loop in `Paint::derive_with_colours` that runs once per drawn Position.
- **Drawn range** — `VisiblePositions`, the console's culling decision. The fitted series walks the
  whole Grid; the culled series walks a fixed 16x16 window whatever the Grid's size.
- **Series** — one named benchmark's points over `main`, as `benchmark-action/github-action-benchmark`
  stores them on `gh-pages`.

## Scope

In: the per-Cell body's cost; the Paint facts the Language Map derives once per Source revision; the
Render Frame boundary that carries them; and the floor the benchmark series should hold afterwards.

Out: the Render Frame's unrelated derivation cost (`render-frame-derivation`), the background-run
fold beyond what the same loop feeds it, the Cursor bloom (`render-frame-derivation/02`), and whether
the ratio gate should have blocked the merge (`benchmarks/07`).
