# 19 — Give a Position its Grid identity

**What to build:** Make a Position say which Grid minted it, so a Grid cannot be asked about a Position from somewhere else. A Position is a bare pair of usizes today, so every Grid query that takes one — `index`, `fits`, `below`, `up`, `down`, `left`, `right` — will answer a foreign Position as if it belonged, and cannot tell that it does not.

**Blocked by:** None (can start immediately).

**Status:** resolved

- [x] A Position records the Grid that minted it, and a Grid can tell its own Positions from another Grid's.
- [x] Asking a Grid about a Position it did not mint is either unexpressible or refused — not silently answered.
- [x] The doc comments on `index` and `fits`, which state the single-Grid invariant as a convention, say what the type enforces instead.
- [x] A test covers a Grid refusing a Position another Grid minted. Nothing is left to invert: the two tests that exercised foreign Positions were deleted when the single-Grid invariant was adopted.
- [x] A Grid stays cheap to copy and a Position stays a plain value: no allocation, no registry, no runtime lookup on the render path, which asks for a Position per Cell per Render Frame.

## Answer

Every Grid receives an allocation-free atomic identity at construction, and every Position carries the identity that minted it. All Grid queries accepting a Position now share one ownership guard and refuse foreign values. Copying a Grid preserves its identity; deserializing one creates a fresh identity. Tests cover foreign refusal and copied ownership without adding a registry, allocation, or render-path lookup.

## Notes

Found while reviewing the issue 10 branch, not yet triaged by a human.

CONTEXT.md states the rule the type does not yet enforce: "A Position can be obtained only from the Grid that contains it, so a Position outside its Grid does not exist." The first half is true — `Grid::position` is the only constructor. The second half is not: once minted, a Position outgrows the Grid that made it and can be handed to any other.

`fits` is where this is visible. Its doc used to claim a foreign Position always answers `false`, on the reasoning that its column must be past this Grid's last column. That reasoning only holds when the foreign column exceeds this Grid's width. `Grid::new(10, 2).position(1, 0)` handed to `Grid::new(4, 2).fits(pos, 2)` answers `true` — verified on the issue 10 branch, though no test records it now: the tests that exercised foreign Positions were deleted when the single-Grid invariant was adopted. `index` now states the invariant rather than a caveat, but has the same exposure: a foreign Position silently produces an index for a Cell in a different Source.

Nothing is broken today: there is one Grid per Source and no path that mixes two. This is a latent defect, and the argument for fixing it is that the docs currently have to explain a hazard that the type could remove. It is worth doing when a second Grid appears — a second Source, a viewport, or anything that renders one shape over another — and probably not before.

Filed separately from issue 18 rather than folded into it: 18 repairs a build that does not compile, this changes a domain type and the shape of the Grid's whole interface. They share no code, no acceptance criteria, and no natural order.

## Comments

**2026-08-26 — filed (agent)**

Filed alongside the `fits` doc correction on the issue 10 branch, which pins the counter-example rather than fixing it and names this ticket in the comment.

**2026-08-26 — what the issue 10 branch left here for this ticket to undo (review)**

`console/src/grid.rs` carries `test_grid_fits_a_foreign_position_inside_its_own_columns`, which asserts the answer this ticket exists to make unaskable: a Position minted at column 1 by `Grid::new(10, 2)`, handed to `Grid::new(4, 2).fits(pos, 2)`, answers `true`. Criterion 4 names it. The reciprocal is worth stating plainly — that test is not a regression guard, it is a record of a limitation, so when this ticket lands it inverts or goes. A green `fits` counter-example after the fix would mean the fix did not land.

`Grid::fits` was made total on the same branch: `self.cols - pos.x` became `self.cols.saturating_sub(pos.x)`. That is why this is a correctness-of-meaning ticket rather than a crash. Before the change, a foreign Position whose column was past this Grid's last one panicked on the underflow; now it answers `false`. The panic was the only loud signal a foreign Position has ever produced, and removing it was right — `fits` should answer a question, not abort — but it does mean every foreign Position now gets a plausible answer, which is exactly the hazard the doc on `fits` had to be rewritten to describe instead of the guarantee it used to claim.

One more query changed shape in the same round, and it is on this ticket's list. `Grid::down` now delegates to `below` and clamps with `unwrap_or(pos)` rather than computing `(pos.y + 1).min(self.rows - 1)`. For a Position this Grid minted the two are identical. For a foreign Position below this Grid's bottom row they differ: the old code clamped it onto this Grid's last row, the new one hands it back unchanged. Both answers are meaningless, which is the point — what `down` answers a foreign Position is now an accident of the delegation rather than a considered clamp, and no test can tell the difference until a Position knows which Grid minted it.

**2026-08-26 — resolved in favour of the stronger guarantee (agent)**

Rebasing onto main brought a reworded `Grid::index` doc: "Total under the console's single-Grid invariant: the application owns one authoritative Grid, and every runtime Position is minted from it. Position deliberately carries no Grid identity; constructing multiple Grids and mixing their Positions is outside the supported domain." That is a stronger statement than the branch's, which only observed that mixing was not prevented, and it is the one adopted.

`fits` now says the same, so the two queries state one invariant rather than two accounts of the same hazard. `saturating_sub` stays: `index` is total and asserts nothing, and `fits` matching it costs nothing.

Both tests that constructed a second Grid are deleted — `test_grid_fits_nothing_past_its_last_column` and `test_grid_fits_a_foreign_position_inside_its_own_columns`. Under a declared invariant they pinned behaviour outside the supported domain, which makes the unsupported look guaranteed, and this ticket would have had to undo them. No in-domain coverage was lost: `test_grid_answers_whether_a_width_fits_in_the_row` still covers the last column in both directions.

This ticket is now the only place the invariant is written down as something to enforce rather than observe. Nothing pins the counter-example any more, so the argument for doing it is the invariant itself: main declares mixing unsupported, and only the type can make that true.
