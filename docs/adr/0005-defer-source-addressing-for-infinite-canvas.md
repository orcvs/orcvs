# Defer Source addressing for an infinite canvas

The console's default is 40 by 25, not the 64 by 64 named below; `DEFAULT_COL_COUNT` and
`DEFAULT_ROW_COUNT` (`orcvs/src/grid.rs`) are the only statement of it. The number below is left as
written — the decision it carries is that the default is exploratory rather than an address-space
contract, which holds whatever the default happens to be. The sparse infinite canvas is a v2
concern; a resizable but bounded Grid is the nearer question, and it is the same rearchitecture
either way. See `.scratch/grid-boundedness/`.

Orcvs may evolve from its current finite Grid into a sparse infinite canvas, so the Orca Function audit will not prescribe relative offsets, two-digit absolute coordinates, or a 256×256 Source limit. The Address family uses `&` as its prefix. Its initial Functions address the four directions with `&^`, `&v`, `&<`, and `&>`. More complex Cell addresses are intentionally deferred. Non-spatial Functions may proceed independently, and the console's 64×64 default remains an exploratory size rather than an address-space contract.
