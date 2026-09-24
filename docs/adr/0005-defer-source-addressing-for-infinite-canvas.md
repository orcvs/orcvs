# Defer Source addressing for an infinite canvas

Status: partially superseded by [ADR 0049](0049-a-position-is-two-numbers.md), which fixes a 256 by 256 limit and spells a Position as two Numbers. The `&` prefix and the four directional Address Functions stand.

Every Grid is now 256 by 256 ([ADR 0054](0054-a-grid-is-always-256-by-256.md)), not the 64 by 64
default named below; `COL_COUNT` and `ROW_COUNT` (`orcvs/src/grid.rs`) are the only statement of it.
There is no longer an exploratory default, so the paragraph below is left as written as a record of
the decision it made at the time, not as a description of the Grid. The sparse infinite canvas is a v2
concern; a resizable but bounded Grid is the nearer question, and it is the same rearchitecture
either way. See `.scratch/grid-boundedness/`.

Orcvs may evolve from its current finite Grid into a sparse infinite canvas, so the Orca Function audit will not prescribe relative offsets, two-digit absolute coordinates, or a 256×256 Source limit. The Address family uses `&` as its prefix. Its initial Functions address the four directions with `&^`, `&v`, `&<`, and `&>`. More complex Cell addresses are intentionally deferred. Non-spatial Functions may proceed independently, and the console's 64×64 default remains an exploratory size rather than an address-space contract.
