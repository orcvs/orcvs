# 19 — Give a Position its Grid identity

**What to build:** Make a Position say which Grid minted it, so a Grid cannot be asked about a Position from somewhere else. A Position is a bare pair of usizes today, so every Grid query that takes one — `index`, `fits`, `below`, `up`, `down`, `left`, `right` — will answer a foreign Position as if it belonged, and cannot tell that it does not.

**Blocked by:** None (can start immediately).

**Status:** needs-triage

- [ ] A Position records the Grid that minted it, and a Grid can tell its own Positions from another Grid's.
- [ ] Asking a Grid about a Position it did not mint is either unexpressible or refused — not silently answered.
- [ ] The doc comments on `index` and `fits` that currently describe the undetectable foreign Position say what the type guarantees instead.
- [ ] `test_grid_fits_a_foreign_position_inside_its_own_columns`, which pins the wrong answer, is replaced by a test of the new behaviour.
- [ ] A Grid stays cheap to copy and a Position stays a plain value: no allocation, no registry, no runtime lookup on the render path, which asks for a Position per Cell per Render Frame.

## Notes

Found while reviewing the issue 10 branch, not yet triaged by a human.

CONTEXT.md states the rule the type does not yet enforce: "A Position can be obtained only from the Grid that contains it, so a Position outside its Grid does not exist." The first half is true — `Grid::position` is the only constructor. The second half is not: once minted, a Position outgrows the Grid that made it and can be handed to any other.

`fits` is where this is visible. Its doc used to claim a foreign Position always answers `false`, on the reasoning that its column must be past this Grid's last column. That reasoning only holds when the foreign column exceeds this Grid's width. `Grid::new(10, 2).position(1, 0)` handed to `Grid::new(4, 2).fits(pos, 2)` answers `true` — verified, and now pinned by a test in `console/src/grid.rs` commented as a known limitation pointing here. `index` carries the same caveat in its doc, where a foreign Position silently produces an index for a Cell in a different Source.

Nothing is broken today: there is one Grid per Source and no path that mixes two. This is a latent defect, and the argument for fixing it is that the docs currently have to explain a hazard that the type could remove. It is worth doing when a second Grid appears — a second Source, a viewport, or anything that renders one shape over another — and probably not before.

Filed separately from issue 18 rather than folded into it: 18 repairs a build that does not compile, this changes a domain type and the shape of the Grid's whole interface. They share no code, no acceptance criteria, and no natural order.

## Comments

**2026-08-26 — filed (agent)**

Filed alongside the `fits` doc correction on the issue 10 branch, which pins the counter-example rather than fixing it and names this ticket in the comment.
