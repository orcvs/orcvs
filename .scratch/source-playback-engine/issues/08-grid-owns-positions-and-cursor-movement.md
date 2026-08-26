# 08 — Grid owns positions and cursor movement

**What to build:** Introduce the Grid — the fixed rectangular shape a Source occupies — as the only thing that knows the Source's dimensions and the only thing that can name a position within them. Move cursor navigation behind it, so the cursor stops carrying its own copy of the dimensions and its own clamping.

**Blocked by:** 07 — Rename the visual marker grid.

**Status:** resolved

- [x] A Grid is constructed with its column and row counts and cannot be constructed with a zero dimension.
- [x] A Position can be obtained only from a Grid, and a Position outside its Grid cannot be constructed; converting a Position to an index is total and asserts nothing.
- [x] Movement is the Grid's: moving in any direction returns a Position, clamping at the edges exactly as the cursor does today.
- [x] The cursor holds a Position, selection and blink state, and nothing else — no dimensions, no clamping, no arithmetic over the Grid's extent.
- [x] Position replaces the console's Coord throughout; the old addressing module is deleted, along with the unused Coord in the language crate.
- [x] Movement tests run on a rectangular grid and cover every edge in both axes, so a swapped axis fails.

## Comments

**2026-08-26 — implemented (agent)**

`Grid` and `Position` live in the console crate. A `Grid` asserts non-zero dimensions at construction, carrying the asserts out of `App::new`. `Position` has private fields and is minted only by `Grid::position`, which returns `None` outside, so `Grid::index` is total and `App::index`'s assert that the position was inside the Grid is gone. Movement is the Grid's and preserves the cursor's old clamping exactly.

`Cursor` now holds a Position, selection and blink state and nothing else. `App` owns the Grid and drives movement. `coord.rs` and the unused `Coord` in the language crate are deleted. `Coord::from_index` and `Coord::index` were not carried onto `Grid`: neither had a production caller, and ticket 10 adds index-to-position when it has one.

Grid tests run on a rectangular grid throughout, and `App`'s index test was strengthened from square to 10x4, so transposing columns and rows now fails.

Review raised five points. Three are fixed here, none changing behaviour for valid input: the test helpers now panic rather than silently writing to the previously selected Cell, the marker-spacing assert moved onto the narrowed integer it actually divides by, and the doc comments no longer claim a Position is proof of range across different Grids. The other two are filed: the marker block is a Cell too wide (issue 14), and the transposed render loop is this ticket's successor, issue 09, which now records that 08 removed the assert that used to make the swap loud.

Known duplication until 09 and 10 land: `App` builds both a `Grid` and an `Opts` from the same dimensions, and the render loop still reads them from `Opts`. `Grid::cols`, `rows` and `count` have no production caller yet.
