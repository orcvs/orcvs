# 10 — Source derives no grid arithmetic

**What to build:** Take the last of the grid arithmetic out of the Source. A Tick decides where each Expression's result goes, whether it falls below the bottom row, and whether it fits before the row edge — all of it currently derived by hand from the column count. Those are facts about the Grid, and the Source should ask for them.

**Blocked by:** 09 — Rendering addresses Cells through the Grid.

**Status:** resolved

- [x] The Grid answers which row a position is in, and whether a value of a given width fits in that row from that position.
- [x] A Tick derives no dimensions itself: the result destination, the discard below the bottom row, and the discard at the row edge all go through the Grid.
- [x] The Source is constructed from a Grid rather than from the whole console options, and so is the commander that owns it.
- [x] The Source's public interface, its Cell stores, and the change set it returns continue to speak in indices.
- [x] The empty-Source Tick case is removed: a Source cannot be built without Cells, so a Tick over one is not a case to handle.
- [x] Tick behaviour tests run on a rectangular Source and cover placement, bottom-row discard, and row-edge discard.

## Comments

**2026-08-26 — implemented (agent)**

The Grid gained three queries. `position_at(idx)` names the Cell an index addresses — the inverse of `index`, and so the Grid's answer to which row and column an index lands in. `below(pos)` is the Position one row down, `None` in the bottom row; it sits beside `down`, which clamps for cursor movement, and the two are asserted against each other so the difference is not something a later edit can quietly collapse. `fits(pos, width)` answers whether a value that wide fits in `pos`'s row counting `pos` as its first Cell.

A Tick now states no dimension of its own. The destination is `below` the Expression's start, and `None` from it *is* the bottom-row discard — the two facts that used to be `start + cols` and `target >= count` are now one question with one answer. The row-edge discard is `fits`, so `% cols` and the `col + width > cols` comparison are both gone. `Source` holds a `Grid` instead of an `Opts`, and `SourceCommander::spawn` takes one too; `App` already built a Grid and now hands it over rather than cloning options into it.

`Opts` lost `cols`, `rows` and `count()` with it. This goes slightly past the criteria, which only asked that the Source be constructed from a Grid — but leaving the dimensions on `Opts` would have left a second place to read them from, which is the thing this issue exists to remove. `Opts::new()` takes no arguments now and got a `Default` impl to match.

The Source still speaks in indices at every boundary: `set`, `unset`, `get`, `Cell.idx`, `Change.idx`, and the `inner`/`glyphs`/`parsed` stores are all unchanged. Positions appear only inside `execute` and `check_idx`, which asks the Grid whether an index names a Cell rather than comparing against a length itself.

Source tests moved from a square 10 x 10 to a rectangular 10 x 6, which is what makes criterion 6 mean anything — a transposed or self-derived dimension addresses different Cells on a rectangle and the placement, bottom-row and row-edge tests all catch it. The empty-Source Tick test is deleted rather than ported: `Grid::new` asserts at least one column and one row, so `Source::new` cannot be handed a shape with no Cells and the underflow that test guarded against is unreachable.

Honest accounting of the red steps: the three Grid queries had genuine behavioural reds. Everything downstream was a compile error against the new signatures, followed by the rectangular-grid test updates, which were index arithmetic rather than new coverage. 78 tests pass (76 before, plus 3 Grid tests, minus the deleted empty-Source one), the doctest passes, and clippy reports nothing new on the touched files.
