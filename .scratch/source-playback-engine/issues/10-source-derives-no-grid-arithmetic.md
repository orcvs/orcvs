# 10 — Source derives no grid arithmetic

**What to build:** Take the last of the grid arithmetic out of the Source. A Tick decides where each Expression's result goes, whether it falls below the bottom row, and whether it fits before the row edge — all of it currently derived by hand from the column count. Those are facts about the Grid, and the Source should ask for them.

**Blocked by:** 09 — Rendering addresses Cells through the Grid.

**Status:** resolved

- [x] The Grid answers which row a position is in, and whether a value of a given width fits in that row from that position.
- [x] A Tick derives no dimensions itself: the result destination, the discard below the bottom row, and the discard at the row edge all go through the Grid.
- [x] The Source is constructed from a Grid rather than from the whole console options, and so is the commander that owns it.
- [x] The Source's public interface, its Cell stores, and the change set it returns continue to speak in indices.
- [x] The empty-Source Tick case is removed: a Source cannot be built without Cells, so a Tick over one is not a case to handle.
- [x] Tick behaviour tests run on a rectangular Source and cover placement, bottom-row discard, and the row edge in both directions: a result reaching the last column exactly is committed, and one that would cross it is discarded.

## Comments

**2026-08-26 — implemented (agent)**

The Grid gained three queries. `position_at(idx)` names the Cell an index addresses — the inverse of `index`, and so the Grid's answer to which row and column an index lands in. `below(pos)` is the Position one row down, `None` in the bottom row; it sits beside `down`, which clamps for cursor movement, and the two are asserted against each other so the difference is not something a later edit can quietly collapse. `fits(pos, width)` answers whether a value that wide fits in `pos`'s row counting `pos` as its first Cell.

A Tick now states no dimension of its own. The destination is `below` the Expression's start, and `None` from it *is* the bottom-row discard — the two facts that used to be `start + cols` and `target >= count` are now one question with one answer. The row-edge discard is `fits`, so `% cols` and the `col + width > cols` comparison are both gone. `Source` holds a `Grid` instead of an `Opts`, and `SourceCommander::spawn` takes one too; `App` already built a Grid and now hands it over rather than cloning options into it.

`Opts` lost `cols`, `rows` and `count()` with it. This goes slightly past the criteria, which only asked that the Source be constructed from a Grid — but leaving the dimensions on `Opts` would have left a second place to read them from, which is the thing this issue exists to remove. `Opts::new()` takes no arguments now and got a `Default` impl to match.

The Source still speaks in indices at every boundary: `set`, `unset`, `get`, `Cell.idx`, `Change.idx`, and the `inner`/`glyphs`/`parsed` stores are all unchanged. Positions appear only inside `execute` and `check_idx`, which asks the Grid whether an index names a Cell rather than comparing against a length itself.

Source tests moved from a square 10 x 10 to a rectangular 10 x 6, which is what makes criterion 6 mean anything — a transposed or self-derived dimension addresses different Cells on a rectangle and the placement, bottom-row and row-edge tests all catch it. The empty-Source Tick test is deleted rather than ported: `Grid::new` asserts at least one column and one row, so `Source::new` cannot be handed a shape with no Cells and the underflow that test guarded against is unreachable.

Honest accounting of the red steps: the three Grid queries had genuine behavioural reds. Everything downstream was a compile error against the new signatures, followed by the rectangular-grid test updates, which were index arithmetic rather than new coverage. 78 tests pass (76 before, plus 3 Grid tests, minus the deleted empty-Source one), the doctest passes, and clippy reports nothing new on the touched files.

**2026-08-26 — unasked-for change (agent)**

A further change landed on this branch that no issue asks for. `App::select_at`, the public method that took an x and y and moved the Cursor if the Grid minted a Position for them, is deleted, and the console's click handler now calls `Cursor::select` directly with the Position `Grid::rows()` had already minted rather than taking it apart for the Grid to mint again. A `compile_fail` doctest on `App` records that no coordinate-taking selection remains, and the two tests that called `select_at` now use the test module's existing `select_or_panic`.

This goes beyond the criteria above, none of which mention selection; neither do issues 08 or 09. It is recorded here because it landed on the same branch as this issue's work, not because this issue called for it.

**2026-08-26 — review fix round (review)**

A code review of this branch produced a second round of changes. All of them are in this issue's territory and none was recorded until now.

*Genuine behavioural reds — two.* `Grid::fits` computed `self.cols - pos.x`, which underflows and panics when a Position's column is past this Grid's last one. Only a Position another Grid minted can reach that, but a panic is not an answer. It is `saturating_sub` now, so the question is always answerable. `test_grid_fits_nothing_past_its_last_column` (`console/src/grid.rs`) panicked on the underflow before the change and asserts `false` after. Separately, the `row()` test helper in `console/src/source/source.rs` built its own 10 x 6 Grid instead of reading the shape of the Source it was handed — the last thing in this crate restating a dimension, which is the whole point of this issue. The helpers now hang off a `SourceUnderTest` fixture that owns the Grid and the Source it minted from it, exposes no way to replace either, and derefs to the Source; two disagreeing shapes are no longer expressible, and a Source built outside the fixture has no `row` to call at all. `test_row_reads_the_shape_of_the_source_under_test` builds on 8 x 4 and failed against the old helper with ten Cells read from an eight-wide Source.

*A compile error, not a red.* `App::select` was added so the click handler stops reaching through the public `cursor` field, the way `write` and `delete` already do not. `test_select_moves_the_cursor_to_the_position` failed to compile against a method that did not exist and then passed. No behaviour was ever in question, and the Cursor's `select` is unchanged.

*Tests that passed before the change, proven guards by injecting the defect they target.* `test_result_reaching_the_last_column_exactly_is_committed` — every other Source test lands its result far from the row edge or one Cell short of fitting, so a `fits` written `<` rather than `<=` passed all of them. Injecting `width < self.cols.saturating_sub(pos.x)` makes this test, and only this test, fail. `test_grid_fits_a_foreign_position_inside_its_own_columns` came at it from the other side: the property the old `fits` doc claimed was written out as `assert!(!narrow.fits(foreign, 2))` and read failing, which falsified the doc rather than the code. The doc now claims only what is true and the counter-example stays pinned, pointing at issue 19.

*Refactors, deletions and documentation, no behavioural delta.* `Grid::down` delegates to `below` and clamps with `unwrap_or(pos)` instead of recomputing `(pos.y + 1).min(self.rows - 1)`, so the two cannot drift apart about where one row down is; identical for a Position this Grid minted, and guarded by `test_grid_moves_down_and_stops_at_the_bottom_row` and the `below`/`down` assertion in `test_grid_answers_the_row_below_and_stops_past_the_bottom_row`. `test_grid_keeps_its_columns_and_rows` is deleted as a duplicate — every assertion it made is already made by `test_grid_mints_positions_inside_it`, `test_grid_refuses_positions_outside_it` and `test_grid_yields_its_rows_in_render_order`. The Source tests stopped writing `10` and `60` by hand and ask the test Grid. The `compile_fail` doctest on `App` called `select_at`, a method that no longer exists, so it passed on any compile error at all and advertised an API a reader would search for in vain; it now demonstrates the guarantee positively, that a pair outside the Grid never becomes a Position. `DEFAULT_COL_COUNT`/`DEFAULT_ROW_COUNT` moved from `opts.rs` to `grid.rs` and stopped deriving from the marker spacing — recorded on issues 07, 14 and 17, where it actually bears. `Source::execute`'s `idx + i` walk is now commented as the one index this file still derives by hand, with the `fits` check above it named as its sole warrant.

82 unit tests and 2 doctests pass. The 78 in the implementation comment above was correct when written; this round is +1 `fits` underflow, -1 duplicate shape test, +1 exact fit, +1 `select`, +1 foreign-Position counter-example, +1 row-helper shape.

**2026-08-26 — criterion 6 sharpened (review)**

Criterion 6 read "cover placement, bottom-row discard, and row-edge discard". `test_result_reaching_the_last_column_exactly_is_committed` covers the other half of the row edge — a two-Cell result whose last Cell is the last column of its row, which must be committed and not discarded. The criterion implied that boundary and no test held it, which is how a `fits` written `<` for `<=` would have passed the whole suite.

The criterion now reads "the row edge in both directions", naming the commit and the discard separately, because a criterion that names only the discard is satisfied by an implementation that discards everything near the edge. It stays checked: the behaviour was already correct, and the test is what makes the claim mean something.
