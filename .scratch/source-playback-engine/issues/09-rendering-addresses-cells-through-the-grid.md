# 09 — Rendering addresses Cells through the Grid

**What to build:** Make the render path ask the Grid which Cells exist and in what order, instead of deriving the Grid's extent for itself. Rendering a rectangular Source currently reads the wrong Cell, because the render loop iterates columns as rows; it survives only because the default Source is square.

**Blocked by:** 08 — Grid owns positions and cursor movement.

**Status:** resolved

- [x] A Grid yields its rows, and each row yields the Positions in it, in render order.
- [x] The console renders one row per iteration of what the Grid yields, and never states a loop bound of its own — a swapped axis is not expressible.
- [x] Looking up a Cell by position returns that Cell on a rectangular Source, and the index conversion's assert that the position is inside the Grid is removed rather than corrected.
- [x] A behaviour test on a rectangular Source fails before this change and passes after, without requiring a rendering context.

## Notes

Ticket 08 removed the assert that used to fire when the render loop asked for a position outside the Grid. Nothing observable changed, because the default Source is square — but on a non-square Source the transposed loop now renders a silent field of blanks with dead clicks, where it previously panicked. Until this ticket lands there is no longer a loud failure for the swap.

## Comments

**2026-08-26 — implemented (agent)**

`Grid::rows()` yields an iterator per row, each yielding that row's Positions left to right. `Console` iterates it, one horizontal per row, and reads no dimension of its own — the transposed loop bound is gone because there is no bound left to state. `App::get`, `App::terminator`, `in_marker_block` and `Cursor::is_at` all take Positions now, and `get`'s unmintable-position fallback is deleted along with its TODO, that case no longer being reachable from the render path.

`Grid::rows()` collided with the existing `rows()` count accessor, so the two counts became `col_count()` and `row_count()`. Both were used only by grid tests.

Honest accounting of the red steps: `Grid::rows()` and `App::get` had genuine behavioural reds on a rectangular Grid; the rest were compile errors against new signatures. `Console::update` needs an egui context and remains untested — criterion 2 is met structurally, by removing the bound rather than by covering it.

Two pre-existing rendering defects surfaced and are filed rather than fixed here: a Cell with content but no glyph renders as background (issue 15), and the highlight and space glyphs are both constructed as markers (issue 16). Together they explain why a lone character is invisible and why the whole background renders as markers.

**2026-08-26 — follow-up (agent)**

`col_count()` and `row_count()`, added here when `Grid::rows()` took the old count accessor's name, are deleted. Nothing picked them up: ticket 10 moved Source's arithmetic onto `position_at`, `below`, `fits` and `count`, and no open issue asks the Grid for a dimension. `count()` stays. The two grid tests that read the accessors now ask the live API instead, and the shape test asserts that a 4 x 2 Grid refuses `(1, 3)` — the transposed axis this ticket existed to remove was invisible to the assertion it replaces.
