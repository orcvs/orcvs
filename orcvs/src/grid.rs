use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_GRID_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct GridId(u64);

impl GridId {
    fn new() -> Self {
        let id = NEXT_GRID_ID
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |id| id.checked_add(1))
            .expect("Grid identity space exhausted");
        Self(id)
    }
}

///
/// A valid position within a Grid. A Position can only be obtained from the
/// Grid that contains it, so a Position outside its Grid cannot exist.
///
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Position {
    grid_id: GridId,
    x: usize,
    y: usize,
}

impl Position {
    #[inline]
    pub fn x(&self) -> usize {
        self.x
    }

    #[inline]
    pub fn y(&self) -> usize {
        self.y
    }
}

///
/// The linear index of a Cell in a Grid: `y * cols + x`.
///
/// Like a Position, a CellIndex can only be obtained from the Grid that
/// contains it, so an index outside its Grid cannot exist. It is a type of its
/// own rather than a bare `usize` because this crate threads several unrelated
/// index spaces — offsets within a row, Cell counts, positions in a partition —
/// and nothing but the type distinguishes them at a glance.
///
/// Indices order row-major within one Grid. Grid identity is the first field
/// so that the derived ordering agrees with equality: two indices compare
/// `Equal` only when they name the same Cell of the same Grid. Every index of
/// one Grid shares that Grid's identity, so their order is decided by `idx`
/// alone; an ordering across Grids is arbitrary but total, which is what
/// ordered collections require of `Ord`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct CellIndex {
    grid_id: GridId,
    idx: usize,
}

impl CellIndex {
    ///
    /// The index as a number, for addressing the Cells of a Source.
    ///
    #[inline]
    pub fn get(self) -> usize {
        self.idx
    }
}

///
/// The shape a console starts with, until something states another one. A
/// Grid's dimensions are its own: they are stated here as Cell counts, and
/// derived from nothing else.
///
/// The default is 40 by 25 — a Grid that reads left to right in time, in the
/// proportion a console is most often given. Cells are square, so these counts
/// are the Grid's aspect ratio, and a console opened in that proportion spends
/// all of its area on the Grid rather than on letterboxing.
///
pub const DEFAULT_COL_COUNT: usize = 40;
pub const DEFAULT_ROW_COUNT: usize = 25;

///
/// The fixed rectangular shape a Source occupies: its column and row counts,
/// and the valid positions within them. The Grid is the shape; the Source is
/// the contents.
///
#[cfg_attr(feature = "persistence", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "persistence",
    serde(try_from = "PersistedGrid", into = "PersistedGrid")
)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Grid {
    id: GridId,
    cols: usize,
    rows: usize,
}

#[cfg(feature = "persistence")]
#[derive(serde::Serialize, serde::Deserialize)]
struct PersistedGrid {
    cols: usize,
    rows: usize,
}

#[cfg(feature = "persistence")]
impl From<Grid> for PersistedGrid {
    fn from(grid: Grid) -> Self {
        Self {
            cols: grid.cols,
            rows: grid.rows,
        }
    }
}

#[cfg(feature = "persistence")]
impl TryFrom<PersistedGrid> for Grid {
    type Error = &'static str;

    fn try_from(grid: PersistedGrid) -> Result<Self, Self::Error> {
        if grid.cols == 0 || grid.rows == 0 {
            return Err("persisted Grid dimensions must be greater than zero");
        }
        if grid.cols.checked_mul(grid.rows).is_none() {
            return Err("persisted Grid Cell count is too large");
        }

        Ok(Self::new(grid.cols, grid.rows))
    }
}

impl Grid {
    ///
    /// A Grid has at least one column and one row.
    ///
    pub fn new(cols: usize, rows: usize) -> Self {
        assert!(cols > 0, "cols must be greater than zero");
        assert!(rows > 0, "rows must be greater than zero");
        assert!(
            cols.checked_mul(rows).is_some(),
            "Grid Cell count is too large"
        );

        Self {
            id: GridId::new(),
            cols,
            rows,
        }
    }

    ///
    /// The only way to obtain a Position. `None` when (x, y) is outside this Grid.
    ///
    pub fn position(&self, x: usize, y: usize) -> Option<Position> {
        if x < self.cols && y < self.rows {
            Some(Position {
                grid_id: self.id,
                x,
                y,
            })
        } else {
            None
        }
    }

    ///
    /// The first Position of this Grid.
    ///
    #[inline]
    pub fn origin(&self) -> Position {
        Position {
            grid_id: self.id,
            x: 0,
            y: 0,
        }
    }

    ///
    /// The linear index of a Position in this Grid.
    ///
    /// A foreign Position is refused: its identity names the Grid that minted it.
    ///
    #[inline]
    pub fn index(&self, pos: Position) -> CellIndex {
        self.assert_owns(pos);
        CellIndex {
            grid_id: self.id,
            idx: pos.y * self.cols + pos.x,
        }
    }

    ///
    /// The only way to obtain a CellIndex from a bare number. `None` when the
    /// number is past the last Cell, so an index this Grid cannot address
    /// never comes into being.
    ///
    #[inline]
    pub fn cell_index(&self, idx: usize) -> Option<CellIndex> {
        (idx < self.count()).then_some(CellIndex {
            grid_id: self.id,
            idx,
        })
    }

    ///
    /// The index `offset` Cells along `pos`'s row. `None` when the offset runs
    /// past the row's end: a row is the whole horizontal extent there is, and
    /// an index that wrapped onto the next row would name a Cell the caller
    /// did not ask for.
    ///
    #[inline]
    pub fn offset_in_row(&self, pos: Position, offset: usize) -> Option<CellIndex> {
        self.assert_owns(pos);
        self.position(pos.x.checked_add(offset)?, pos.y)
            .map(|pos| self.index(pos))
    }

    /// Whether `pos` was minted by this Grid or one of its copies.
    #[inline]
    pub fn owns(&self, pos: Position) -> bool {
        self.id == pos.grid_id
    }

    ///
    /// The Cell an index addresses, and so which row and column it is in. The
    /// inverse of `index`.
    ///
    /// Total: a CellIndex can only come from a Grid, so it is in range for the
    /// Grid that minted it. A foreign index is refused.
    ///
    #[inline]
    pub fn position_at(&self, idx: CellIndex) -> Position {
        self.assert_owns_index(idx);
        self.position(idx.idx % self.cols, idx.idx / self.cols)
            .expect("a CellIndex is inside the Grid that minted it")
    }

    ///
    /// The Position one row below `pos`. `None` in the bottom row, where
    /// there is no row below. Unlike `down`, which clamps for cursor
    /// movement, this never answers with a Cell the caller did not ask for.
    ///
    #[inline]
    pub fn below(&self, pos: Position) -> Option<Position> {
        self.assert_owns(pos);
        self.position(pos.x, pos.y + 1)
    }

    ///
    /// The Position `columns` Cells east and `rows` Cells south of `pos`, or
    /// `None` where that Cell is outside this Grid.
    ///
    /// Negative offsets go west and north. Like `below` and unlike `up`,
    /// `down`, `left` and `right`, it never clamps: a displacement that leaves
    /// the Grid is a destination that does not exist, which is what ADR 0009
    /// wants reported rather than silently moved to an edge Cell the caller did
    /// not ask for.
    ///
    /// A displaced Cell stays inside its own row only when `rows` is zero;
    /// there is no wrapping, because a column past the last one is outside the
    /// Grid in the same way a row past the last one is.
    ///
    #[inline]
    pub(crate) fn displaced(&self, pos: Position, columns: i16, rows: i16) -> Option<Position> {
        self.assert_owns(pos);
        let x = pos.x.checked_add_signed(isize::from(columns))?;
        let y = pos.y.checked_add_signed(isize::from(rows))?;
        self.position(x, y)
    }

    ///
    /// The Position one row above `pos`, clamped at the top row.
    ///
    #[inline]
    pub fn up(&self, pos: Position) -> Position {
        self.assert_owns(pos);
        Position {
            grid_id: self.id,
            x: pos.x,
            y: pos.y.saturating_sub(1),
        }
    }

    ///
    /// The Position one row below `pos`, clamped at the bottom row. The row
    /// below is `below`'s answer; clamping is all this adds, so the two cannot
    /// disagree about where one row down is.
    ///
    #[inline]
    pub fn down(&self, pos: Position) -> Position {
        self.below(pos).unwrap_or(pos)
    }

    ///
    /// The Position one column left of `pos`, clamped at the first column.
    ///
    #[inline]
    pub fn left(&self, pos: Position) -> Position {
        self.assert_owns(pos);
        Position {
            grid_id: self.id,
            x: pos.x.saturating_sub(1),
            y: pos.y,
        }
    }

    ///
    /// The Position one column right of `pos`, clamped at the last column.
    ///
    #[inline]
    pub fn right(&self, pos: Position) -> Position {
        self.assert_owns(pos);
        Position {
            grid_id: self.id,
            x: (pos.x + 1).min(self.cols - 1),
            y: pos.y,
        }
    }

    ///
    /// The Positions of this Grid in render order: one iterator per row, top to
    /// bottom, each yielding that row's Positions left to right. The render
    /// path states no bound of its own, so a swapped axis is not expressible.
    ///
    pub fn positions_by_row(&self) -> impl Iterator<Item = impl Iterator<Item = Position>> {
        // Captured by value: Grid and Position are allocation-free Copy values,
        // so the returned iterators borrow nothing.
        let (id, cols, rows) = (self.id, self.cols, self.rows);

        (0..rows).map(move |y| (0..cols).map(move |x| Position { grid_id: id, x, y }))
    }

    ///
    /// How many Cells occupy each row of this finite Grid: the column count
    /// half of the shape a Source occupies.
    ///
    #[inline]
    pub fn columns(&self) -> usize {
        self.cols
    }

    ///
    /// How many rows of Cells this finite Grid has: the row count half of the
    /// shape a Source occupies, and the number of iterators
    /// [`positions_by_row`](Self::positions_by_row) yields.
    ///
    #[inline]
    pub fn rows(&self) -> usize {
        self.rows
    }

    ///
    /// How many Cells this Grid has: one per Position it yields.
    ///
    #[inline]
    pub fn count(&self) -> usize {
        self.cols * self.rows
    }

    #[inline]
    pub(crate) fn assert_owns(&self, pos: Position) {
        assert!(self.owns(pos), "Position belongs to another Grid");
    }

    #[inline]
    pub(crate) fn assert_owns_index(&self, idx: CellIndex) {
        assert!(self.id == idx.grid_id, "CellIndex belongs to another Grid");
    }
}

#[cfg(test)]
mod test {

    use crate::{
        grid::{Grid, Position},
        test::trace,
    };

    #[test]
    fn test_grid_yields_its_rows_in_render_order() {
        trace();

        // Rectangular on purpose: a transposed implementation yields 4 rows of
        // 2 Positions and fails here.
        let grid = Grid::new(4, 2);
        let at = |x, y| grid.position(x, y).expect("inside the grid");

        let rows: Vec<Vec<Position>> = grid.positions_by_row().map(|row| row.collect()).collect();

        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].len(), 4);
        assert_eq!(rows[1].len(), 4);

        // rows top to bottom, each row left to right
        assert_eq!(rows[0], vec![at(0, 0), at(1, 0), at(2, 0), at(3, 0)]);
        assert_eq!(rows[1], vec![at(0, 1), at(1, 1), at(2, 1), at(3, 1)]);

        // flattened, exactly the Source's own index order
        let indices: Vec<usize> = rows
            .iter()
            .flatten()
            .map(|p| grid.index(*p).get())
            .collect();
        assert_eq!(indices, (0..grid.count()).collect::<Vec<usize>>());
    }

    #[test]
    #[should_panic(expected = "cols must be greater than zero")]
    fn test_grid_cannot_have_zero_cols() {
        trace();

        let _ = Grid::new(0, 2);
    }

    #[test]
    #[should_panic(expected = "rows must be greater than zero")]
    fn test_grid_cannot_have_zero_rows() {
        trace();

        let _ = Grid::new(4, 0);
    }

    #[test]
    fn test_grid_mints_positions_inside_it() {
        trace();

        let grid = Grid::new(4, 2);

        let pos = grid.position(3, 1).expect("3, 1 is inside a 4 x 2 grid");

        assert_eq!(pos.x(), 3);
        assert_eq!(pos.y(), 1);
    }

    #[test]
    fn test_grid_refuses_positions_outside_it() {
        trace();

        let grid = Grid::new(4, 2);

        // past the last column
        assert_eq!(grid.position(4, 0), None);
        // past the last row
        assert_eq!(grid.position(0, 2), None);
        // transposed: valid in a 2 x 4 grid, not in a 4 x 2 one
        assert_eq!(grid.position(1, 3), None);
    }

    #[test]
    fn test_grid_origin_is_the_first_position() {
        trace();

        let grid = Grid::new(4, 2);

        let origin = grid.origin();

        assert_eq!(origin.x(), 0);
        assert_eq!(origin.y(), 0);
    }

    #[test]
    fn test_grid_converts_positions_to_indices_in_row_order() {
        trace();

        let grid = Grid::new(4, 2);

        let index = |x, y| {
            grid.index(grid.position(x, y).expect("inside the grid"))
                .get()
        };

        // first row runs 0..4, second row starts at 4
        assert_eq!(index(0, 0), 0);
        assert_eq!(index(3, 0), 3);
        assert_eq!(index(0, 1), 4);
        assert_eq!(index(3, 1), 7);
    }

    #[test]
    #[should_panic(expected = "Position belongs to another Grid")]
    fn test_grid_refuses_a_position_minted_by_another_grid() {
        let first = Grid::new(4, 2);
        let second = Grid::new(4, 2);
        let foreign = first.position(1, 0).expect("inside the first Grid");

        second.index(foreign);
    }

    #[test]
    fn test_cell_index_ordering_agrees_with_equality_across_grids() {
        trace();

        // `Ord` requires `a.cmp(&b) == Equal` exactly when `a == b`. Two Grids
        // can each mint an index for the same number, and equality reports
        // them different because their Grid identities differ. An ordering
        // that answers `Equal` there contradicts equality and corrupts any
        // ordered collection holding indices from more than one Grid.
        let first = Grid::new(4, 2);
        let second = Grid::new(4, 2);
        let a = first.cell_index(1).expect("inside the first Grid");
        let b = second.cell_index(1).expect("inside the second Grid");

        assert_ne!(a, b, "indices from different Grids are not equal");
        assert_ne!(
            a.cmp(&b),
            std::cmp::Ordering::Equal,
            "ordering must agree with equality"
        );

        // The corruption this prevents: a BTreeSet keyed on CellIndex holds
        // both, rather than collapsing them into one.
        let set = std::collections::BTreeSet::from([a, b]);
        assert_eq!(set.len(), 2);
    }

    #[test]
    fn test_grid_identity_survives_copying() {
        let grid = Grid::new(4, 2);
        let copied = grid;
        let position = grid.position(1, 0).expect("inside the Grid");

        assert!(copied.owns(position));
        assert_eq!(copied.index(position).get(), 1);
    }

    #[test]
    fn test_grid_indices_cover_every_cell_exactly_once() {
        trace();

        let grid = Grid::new(4, 2);

        // every index names a Position, and that Position converts back to the
        // index it came from: no two Cells share an index, and none is missed
        for idx in 0..grid.count() {
            let cell = grid.cell_index(idx).expect("inside the grid");
            let position = grid.position_at(cell);

            assert_eq!(grid.index(position), cell);
        }
    }

    #[test]
    fn test_grid_moves_up_and_stops_at_the_top_row() {
        trace();

        let grid = Grid::new(4, 2);
        let at = |x, y| grid.position(x, y).expect("inside the grid");

        // from the bottom row up to the top
        assert_eq!(grid.up(at(3, 1)), at(3, 0));
        // already at the top row: stays
        assert_eq!(grid.up(at(3, 0)), at(3, 0));
    }

    #[test]
    fn test_grid_moves_down_and_stops_at_the_bottom_row() {
        trace();

        let grid = Grid::new(4, 2);
        let at = |x, y| grid.position(x, y).expect("inside the grid");

        // from the top row down to the bottom
        assert_eq!(grid.down(at(3, 0)), at(3, 1));
        // already at the bottom row of a 2 row grid: stays
        assert_eq!(grid.down(at(3, 1)), at(3, 1));
    }

    #[test]
    fn test_grid_moves_left_and_stops_at_the_first_column() {
        trace();

        let grid = Grid::new(4, 2);
        let at = |x, y| grid.position(x, y).expect("inside the grid");

        assert_eq!(grid.left(at(3, 1)), at(2, 1));
        assert_eq!(grid.left(at(1, 1)), at(0, 1));
        // already at the first column: stays
        assert_eq!(grid.left(at(0, 1)), at(0, 1));
    }

    #[test]
    fn test_grid_moves_right_and_stops_at_the_last_column() {
        trace();

        let grid = Grid::new(4, 2);
        let at = |x, y| grid.position(x, y).expect("inside the grid");

        assert_eq!(grid.right(at(0, 1)), at(1, 1));
        // past the second column, which is the last one only in a 2 x 4 grid
        assert_eq!(grid.right(at(1, 1)), at(2, 1));
        assert_eq!(grid.right(at(2, 1)), at(3, 1));
        // already at the last column: stays
        assert_eq!(grid.right(at(3, 1)), at(3, 1));
    }

    #[test]
    fn test_grid_names_the_cell_an_index_addresses() {
        trace();

        let grid = Grid::new(4, 2);
        let at = |x, y| grid.position(x, y).expect("inside the grid");

        let cell = |idx| grid.cell_index(idx).expect("inside the grid");

        // the inverse of `index`: row order, first row then second
        assert_eq!(grid.position_at(cell(0)), at(0, 0));
        assert_eq!(grid.position_at(cell(3)), at(3, 0));
        assert_eq!(grid.position_at(cell(4)), at(0, 1));
        assert_eq!(grid.position_at(cell(7)), at(3, 1));

        // past the last Cell: no index at all, rather than one wrapped back
        // inside. An index this Grid cannot address never comes into being.
        assert_eq!(grid.cell_index(8), None);
        assert_eq!(grid.cell_index(100), None);
    }

    #[test]
    fn test_grid_answers_the_row_below_and_stops_past_the_bottom_row() {
        trace();

        let grid = Grid::new(4, 2);
        let at = |x, y| grid.position(x, y).expect("inside the grid");

        // one row down, same column
        assert_eq!(grid.below(at(3, 0)), Some(at(3, 1)));

        // the bottom row has no row below it, and `below` says so rather than
        // clamping the way cursor movement does
        assert_eq!(grid.below(at(3, 1)), None);
        assert_eq!(grid.down(at(3, 1)), at(3, 1));
    }

    #[test]
    fn test_grid_displaces_in_both_axes_and_stops_at_every_edge() {
        trace();

        let grid = Grid::new(4, 3);
        let at = |x, y| grid.position(x, y).expect("inside the grid");
        let middle = at(1, 1);

        // Both axes move independently, and each sign moves the way its name
        // says: positive columns east, positive rows south. A transposed or
        // sign-flipped offset lands on none of these four.
        assert_eq!(grid.displaced(middle, 0, -1), Some(at(1, 0)));
        assert_eq!(grid.displaced(middle, 0, 1), Some(at(1, 2)));
        assert_eq!(grid.displaced(middle, -1, 0), Some(at(0, 1)));
        assert_eq!(grid.displaced(middle, 1, 0), Some(at(2, 1)));
        assert_eq!(grid.displaced(middle, 2, 1), Some(at(3, 2)));

        // Every edge answers `None` rather than a clamped Cell, including the
        // one a wrapping index would have reached: the Cell after the last
        // column of a row exists in the Grid, and it is not this row's.
        assert_eq!(grid.displaced(at(0, 0), 0, -1), None);
        assert_eq!(grid.displaced(at(0, 0), -1, 0), None);
        assert_eq!(grid.displaced(at(3, 0), 1, 0), None);
        assert_eq!(grid.displaced(at(0, 2), 0, 1), None);

        // Zero is the Cell itself, which is what makes a Function that
        // declines no default declare nothing special.
        assert_eq!(grid.displaced(middle, 0, 0), Some(middle));
    }
}

///
/// The Grid laws CONTEXT.md states, over shapes and Positions no example names.
///
/// "A Position can be obtained only from the Grid that contains it, so a
/// Position outside its Grid does not exist; the Grid converts between a
/// Position and the index the Source addresses Cells by." Every property below
/// is one clause of that sentence, or of the Grid entry's own "A Grid has at
/// least one column and one row, and a position outside it does not exist".
///
/// The generated value is the shape alone, and each case then sweeps every
/// Position inside it. That split is the point rather than an economy: the
/// glossary quantifies over *every* Position a Grid mints, so drawing one
/// Position per case would leave the quantifier itself unchecked.
///
/// `.scratch/property-testing/spec.md` says to prefer an exhaustive loop where
/// the domain is small enough for one, and 4,096 dimension pairs is small
/// enough. The rule is about the domain a property covers, not the value it
/// draws, and the sweep is what separates the two here: a case covers a whole
/// shape's worth of Positions, so these properties range over the (shape,
/// Position) space — some 4 x 10^6 pairs — and, for `offset_in_row`, the
/// (shape, Position, offset) space those shapes open, which runs to some 10^8
/// triples. Neither is within an enumeration's reach, which is what makes a
/// sample the right instrument rather than a concession. What the sample is
/// spent on is the shape, and the shapes the rule would worry about losing are
/// the edge ones, which `dimensions` draws by weight rather than by luck:
/// `generated_grids_include_the_one_column_and_one_row_cases` is what says so.
/// A pull request draws 32 shapes and the merge tier 256 — draws rather than
/// distinct pairs, since the weighted arms repeat the 1 x 1 Grid and the other
/// edges on purpose.
///
/// This replaces the effort's wiring seed, whose own comment said that proving
/// the round trip for every minted Position belonged to
/// `.scratch/property-testing/issues/02-grid-position-round-trip.md`.
/// `every_position_the_grid_mints_round_trips_through_its_index` is that
/// proof, so keeping the seed beside it would be a second, weaker statement of
/// the same law.
///
/// Dimensions start at one because `Grid::new` refuses zero: both counts are
/// `assert!`ed rather than `debug_assert!`ed, so a Grid of no Cells cannot be
/// constructed in a release build either, and no property here has to admit
/// one. `mod test`'s `test_grid_cannot_have_zero_cols` and
/// `test_grid_cannot_have_zero_rows` pin that, and the `persistence`
/// `TryFrom<PersistedGrid>` refuses the same shape with an error rather than a
/// panic, so a deserialized Grid cannot arrive empty either.
///
/// The `cfg` matches the `[target.'cfg(not(target_arch = "wasm32"))'.dev-dependencies]`
/// table that declares proptest, so a WASM build never sees the dependency.
///
#[cfg(all(test, not(target_arch = "wasm32")))]
mod property {

    use crate::grid::{Grid, Position};
    use proptest::prelude::*;
    use proptest::test_runner::{Config, TestRunner};
    // Aliased because `Cell` is a domain noun in this file and in CONTEXT.md.
    // What `std::cell::Cell` holds here is a draw count.
    use std::cell::Cell as Counter;

    /// The largest side a generated Grid has. Every property sweeps every Cell
    /// of the shape it draws, so this bounds a case's work as well as the
    /// domain: at the largest shape that is 4,096 Cells, and `offset_in_row`
    /// asks about every offset from each of them.
    const MAX_SIDE: usize = 64;

    /// How far past the last column, row, or Cell a sweep looks. One is the
    /// first refusal; two says the refusal is a comparison rather than an
    /// off-by-one that happens to answer once more.
    const PAST_THE_END: usize = 2;

    ///
    /// A Grid's dimensions: at least one column and one row, and no larger
    /// than a case can sweep.
    ///
    /// A weighted union rather than a pair of ranges, because the shapes where
    /// the edges coincide are the ones the arithmetic is most likely to get
    /// wrong and the ones a uniform draw from `1..=64` reaches once in 64
    /// cases. In a one-column Grid every Cell is both the first and the last of
    /// its row, in a one-row Grid the Cell index and the column are the same
    /// number, and a 1 x 1 Grid is every edge at once.
    /// `generated_grids_include_the_one_column_and_one_row_cases` is what says
    /// they are actually drawn rather than merely drawable.
    ///
    /// The rectangular arm keeps its own weight because a transposed
    /// implementation — `x * rows + y` for an index, or a
    /// `positions_by_row()` that yields columns — agrees with a correct one on
    /// every square Grid.
    ///
    fn dimensions() -> impl Strategy<Value = (usize, usize)> {
        prop_oneof![
            4 => (1usize..=MAX_SIDE, 1usize..=MAX_SIDE),
            1 => (Just(1usize), 1usize..=MAX_SIDE),
            1 => (1usize..=MAX_SIDE, Just(1usize)),
            1 => Just((1usize, 1usize)),
        ]
    }

    ///
    /// Whether `pos` is a Cell of `grid`: minted by it, and inside it.
    ///
    /// `owns` alone is not that claim. `up`, `down`, `left` and `right` build a
    /// Position from its fields rather than asking `position` for one, so a
    /// clamp that let a column run past the last would still produce a
    /// Position carrying this Grid's identity. Asking `position` for the same
    /// coordinates is what refuses it, and comparing the answer keeps the
    /// identity in the comparison.
    ///
    /// The two terms are not independent: `Position` derives `PartialEq` over
    /// `grid_id`, so the equality already implies `owns`. The call stays
    /// because acceptance line 5 of `property-testing/02` asks for `owns` in
    /// those words and this is where every property states it; it is the
    /// literal form of a claim the equality subsumes, not a second check.
    ///
    fn is_a_cell_of(grid: Grid, pos: Position) -> bool {
        grid.owns(pos) && grid.position(pos.x(), pos.y()) == Some(pos)
    }

    ///
    /// Every Position a Grid of these dimensions mints, in row order.
    ///
    /// The bounds come from the generated dimensions rather than from the Grid,
    /// so a sweep never agrees with the Grid's own answer about its shape by
    /// construction. Obtained through `position` rather than through `rows`,
    /// for the same reason: `rows` is itself under test in
    /// `rows_yields_every_cell_of_the_grid_once`.
    ///
    fn every_position(grid: Grid, cols: usize, rows: usize) -> impl Iterator<Item = Position> {
        (0..rows).flat_map(move |y| {
            (0..cols).map(move |x| grid.position(x, y).expect("inside the Grid"))
        })
    }

    proptest! {
        ///
        /// "A Grid has at least one column and one row, and a position outside
        /// it does not exist": `position` answers `Some` exactly at the
        /// coordinates the Grid has a Cell for, and the Position it answers
        /// with is the one that was asked for.
        ///
        /// The sweep runs past both dimensions, so every case states the
        /// refusal as well as the acceptance, and the far coordinates below
        /// state that the refusal is a comparison against the dimensions
        /// rather than a bound on how far outside a caller may ask.
        ///
        #[test]
        fn a_position_exists_exactly_where_the_grid_has_a_cell(
            (cols, rows) in dimensions(),
        ) {
            let grid = Grid::new(cols, rows);

            for y in 0..rows + PAST_THE_END {
                for x in 0..cols + PAST_THE_END {
                    let inside = x < cols && y < rows;
                    let answered = grid.position(x, y);

                    prop_assert_eq!(
                        answered.is_some(),
                        inside,
                        "({}, {}) in a {} x {} Grid",
                        x, y, cols, rows,
                    );
                    if let Some(pos) = answered {
                        prop_assert_eq!((pos.x(), pos.y()), (x, y));
                        prop_assert!(is_a_cell_of(grid, pos));
                    }
                }
            }

            prop_assert_eq!(grid.position(usize::MAX, 0), None);
            prop_assert_eq!(grid.position(0, usize::MAX), None);
            prop_assert_eq!(grid.position(usize::MAX, usize::MAX), None);

            // The one Position a Grid hands out without being asked for
            // coordinates is the first Cell it has, so it is subject to the
            // same law.
            prop_assert_eq!(Some(grid.origin()), grid.position(0, 0));
            prop_assert!(is_a_cell_of(grid, grid.origin()));
        }

        ///
        /// "The Grid converts between a Position and the index the Source
        /// addresses Cells by": `position_at` inverts `index` for every
        /// Position the Grid mints, and the index it converts to is one the
        /// Grid would answer for that number.
        ///
        /// The round trip alone would hold for an `index` that mapped two
        /// Cells to the same number, as long as `position_at` mapped it back
        /// to whichever one was asked. It cannot: a round trip that survives
        /// for every Cell of the shape is injective, because a collision
        /// returns at most one of the two Positions that reached it. That is
        /// what sweeping the whole shape buys over sampling one Position.
        ///
        #[test]
        fn every_position_the_grid_mints_round_trips_through_its_index(
            (cols, rows) in dimensions(),
        ) {
            let grid = Grid::new(cols, rows);

            for pos in every_position(grid, cols, rows) {
                let idx = grid.index(pos);

                prop_assert_eq!(grid.position_at(idx), pos);
                // The Source addresses Cells by this number, and the shape has
                // `cols * rows` of them. Bounded by the draw rather than by
                // `count`, so the Grid does not get to answer for its own
                // size here.
                prop_assert!(idx.get() < cols * rows);
                // And the number names the same Cell coming the other way, so
                // the two ways of obtaining an index cannot disagree.
                prop_assert_eq!(grid.cell_index(idx.get()), Some(idx));
            }
        }

        ///
        /// The other direction of the same conversion: `index` inverts
        /// `position_at` for every `CellIndex` the Grid answers with, and the
        /// Position it passes through is a Cell of the Grid.
        ///
        /// `position_at` is documented as total for an index its own Grid
        /// minted, so a case failing by panic is this property failing.
        ///
        #[test]
        fn every_index_the_grid_answers_round_trips_through_its_position(
            (cols, rows) in dimensions(),
        ) {
            let grid = Grid::new(cols, rows);

            // `cols * rows` rather than `count`, for the reason
            // `every_position` gives: a sweep bounded by the Grid's own answer
            // about its size covers whatever that answer says, so a `count`
            // that under-reported by one would make this skip the last Cell
            // rather than fail.
            for i in 0..cols * rows {
                let cell = grid.cell_index(i).expect("below the Cell count");
                prop_assert_eq!(cell.get(), i);

                let pos = grid.position_at(cell);
                prop_assert!(is_a_cell_of(grid, pos));
                prop_assert_eq!(grid.index(pos), cell);
            }
        }

        ///
        /// "An index this Grid cannot address never comes into being":
        /// `cell_index` answers `Some` exactly below the Cell count.
        ///
        /// Stated separately from the round trip because it is the half a
        /// round trip cannot see. Every index the round trip walks is one
        /// `cell_index` already answered, so an implementation that minted an
        /// index past the last Cell would round-trip that index too.
        ///
        /// The count is the drawn `cols * rows`, and `count` is checked
        /// against it rather than used as the bound. `cell_index` is written
        /// as `idx < self.count()`, so asking whether it agrees with
        /// `i < grid.count()` is one expression compared with itself: it holds
        /// whatever `count` returns, including `cols * rows + 1`. Naming the
        /// Cell count from the shape is what makes this property state the law
        /// its name claims.
        ///
        #[test]
        fn an_index_exists_exactly_below_the_cell_count(
            (cols, rows) in dimensions(),
        ) {
            let grid = Grid::new(cols, rows);
            let count = cols * rows;

            prop_assert_eq!(grid.count(), count);

            for i in 0..count + PAST_THE_END {
                prop_assert_eq!(
                    grid.cell_index(i).is_some(),
                    i < count,
                    "index {} of a {} x {} Grid",
                    i, cols, rows,
                );
            }

            prop_assert_eq!(grid.cell_index(usize::MAX), None);
        }

        ///
        /// "One repaint of the console, in which every Position the Grid yields
        /// is drawn once": `positions_by_row` yields exactly `count`
        /// Positions, each index appearing once, in the order the Source stores
        /// its Cells.
        ///
        /// Equality with `0..cols * rows` states all three at once — a repeat, an
        /// omission, or a swapped axis each make the sequence differ somewhere
        /// — and it states them for oblong shapes, where a transposed
        /// implementation is distinguishable at all.
        ///
        #[test]
        fn positions_by_row_yields_every_cell_of_the_grid_once(
            (cols, rows) in dimensions(),
        ) {
            let grid = Grid::new(cols, rows);

            let yielded: Vec<Vec<Position>> = grid.positions_by_row().map(|row| row.collect()).collect();

            prop_assert_eq!(yielded.len(), rows);
            for row in &yielded {
                prop_assert_eq!(row.len(), cols);
            }

            let positions: Vec<Position> = yielded.into_iter().flatten().collect();
            prop_assert_eq!(positions.len(), cols * rows);
            for pos in &positions {
                prop_assert!(is_a_cell_of(grid, *pos));
            }

            prop_assert_eq!(
                positions.iter().map(|pos| grid.index(*pos).get()).collect::<Vec<usize>>(),
                (0..cols * rows).collect::<Vec<usize>>()
            );
        }

        ///
        /// "A row is the whole horizontal extent there is": `offset_in_row`
        /// answers `Some` exactly while the offset Cell is still in the row it
        /// started from, and the index it answers with names that Cell.
        ///
        /// The sweep asks every offset from every Cell, one and two past the
        /// last the row admits, so each case states both the last acceptance
        /// and the first refusal of every row. `usize::MAX` is asked as well
        /// and takes two different paths to the same answer: from column zero
        /// the sum is representable and simply lands outside the Grid, and from
        /// any other column it overflows, which is why the addition is checked.
        ///
        /// Both production call sites — `Portal::admit` and `tick::Lookup::at`
        /// — ask on behalf of a run of Cells they already hold, so `width - 1`
        /// is the offset each hands over and the row's last Cell is what
        /// decides whether a write is refused or a destination has
        /// relationships at all. There was no test of this method before this
        /// one.
        ///
        #[test]
        fn an_offset_in_row_stays_inside_the_row_it_started_in(
            (cols, rows) in dimensions(),
        ) {
            let grid = Grid::new(cols, rows);

            for pos in every_position(grid, cols, rows) {
                let start = grid.index(pos);

                // The first offset outside the row is `cols - pos.x()`, so the
                // range reaches one past that.
                for offset in 0..cols - pos.x() + PAST_THE_END {
                    let answered = grid.offset_in_row(pos, offset);
                    let inside = pos.x() + offset < cols;

                    prop_assert_eq!(
                        answered.is_some(),
                        inside,
                        "offset {} from ({}, {}) in a {} x {} Grid",
                        offset, pos.x(), pos.y(), cols, rows,
                    );

                    if let Some(cell) = answered {
                        let landed = grid.position_at(cell);
                        prop_assert_eq!(landed.y(), pos.y(), "left its own row");
                        prop_assert_eq!(landed.x(), pos.x() + offset);
                        prop_assert_eq!(cell.get(), start.get() + offset);
                    }
                }

                // No offset at all is the Position's own Cell, which is what a
                // one-Cell write asks for.
                prop_assert_eq!(grid.offset_in_row(pos, 0), Some(start));
                prop_assert_eq!(grid.offset_in_row(pos, usize::MAX), None);
            }
        }

        ///
        /// "The Cursor holds no dimensions and does no clamping of its own —
        /// the Grid answers where a move lands": every move lands on a Cell of
        /// this Grid, one step along one axis, or stays where the Grid ends.
        ///
        /// `is_a_cell_of` rather than `owns` because these four are the only
        /// methods that build a Position from its fields instead of asking
        /// `position` for one. A clamp that ran one column past the last would
        /// mint a Position this Grid owns and cannot address, and `index`
        /// would then hand the Source a Cell in the next row.
        ///
        /// The two vertical edge behaviours are stated together because the
        /// names do not distinguish them: `down` clamps and answers a Position,
        /// `below` answers `Option` and gives `None` in the bottom row. Each
        /// branch below names both, so neither can be read as the other.
        ///
        /// The inverses — `up` after `down`, `left` after `right` — are what
        /// says a move is one Cell rather than merely a Cell in the right
        /// direction, without restating the saturating arithmetic the methods
        /// are written in.
        ///
        #[test]
        fn every_move_lands_on_a_cell_of_the_grid(
            (cols, rows) in dimensions(),
        ) {
            let grid = Grid::new(cols, rows);

            for pos in every_position(grid, cols, rows) {
                for landed in [grid.up(pos), grid.down(pos), grid.left(pos), grid.right(pos)] {
                    prop_assert!(
                        is_a_cell_of(grid, landed),
                        "({}, {}) left a {} x {} Grid",
                        landed.x(), landed.y(), cols, rows,
                    );
                }

                // One axis at a time.
                prop_assert_eq!(grid.up(pos).x(), pos.x());
                prop_assert_eq!(grid.down(pos).x(), pos.x());
                prop_assert_eq!(grid.left(pos).y(), pos.y());
                prop_assert_eq!(grid.right(pos).y(), pos.y());

                if pos.y() + 1 < rows {
                    let below = grid.below(pos).expect("a row below");
                    prop_assert!(is_a_cell_of(grid, below));
                    prop_assert_eq!((below.x(), below.y()), (pos.x(), pos.y() + 1));
                    prop_assert_eq!(grid.down(pos), below);
                    prop_assert_eq!(grid.up(below), pos);
                } else {
                    // The bottom row: `below` says there is no row below, and
                    // `down` clamps to the Cell it was given.
                    prop_assert_eq!(grid.below(pos), None);
                    prop_assert_eq!(grid.down(pos), pos);
                }

                if pos.y() > 0 {
                    prop_assert_eq!(grid.up(pos).y(), pos.y() - 1);
                    prop_assert_eq!(grid.below(grid.up(pos)), Some(pos));
                } else {
                    prop_assert_eq!(grid.up(pos), pos);
                }

                if pos.x() + 1 < cols {
                    prop_assert_eq!(grid.right(pos).x(), pos.x() + 1);
                    prop_assert_eq!(grid.left(grid.right(pos)), pos);
                } else {
                    prop_assert_eq!(grid.right(pos), pos);
                }

                if pos.x() > 0 {
                    prop_assert_eq!(grid.left(pos).x(), pos.x() - 1);
                    prop_assert_eq!(grid.right(grid.left(pos)), pos);
                } else {
                    prop_assert_eq!(grid.left(pos), pos);
                }
            }
        }
    }

    ///
    /// The generator reaches the shapes where a Grid's edges coincide: one
    /// column, one row, the 1 x 1 Grid that is both, and an oblong one where a
    /// transposed implementation is distinguishable.
    ///
    /// A property is only as good as what its generator produces, and none of
    /// the properties above can tell a shape it never saw from one it saw and
    /// handled. Driving the runner directly is what lets the draws be counted
    /// across cases; the count is asserted afterwards, where `proptest!` would
    /// have had nowhere to put it. This is the same guard, for the same
    /// reason, as `lang::parser`'s
    /// `generated_source_covers_the_space_the_incomplete_hash_and_the_comment_introducer`.
    ///
    /// The case count is pinned rather than taken from `PROPTEST_CASES`,
    /// because the claim is about the generator rather than about the Grid.
    /// Nothing is constructed per case, so 256 draws of two numbers cost
    /// nothing either tier would want back.
    ///
    #[test]
    fn generated_grids_include_the_one_column_and_one_row_cases() {
        let config = Config {
            cases: 256,
            source_file: Some(file!()),
            ..Config::default()
        };
        let one_column = Counter::new(0usize);
        let one_row = Counter::new(0usize);
        let single_cell = Counter::new(0usize);
        let oblong = Counter::new(0usize);

        TestRunner::new(config)
            .run(&dimensions(), |(cols, rows)| {
                if cols == 1 {
                    one_column.set(one_column.get() + 1);
                }
                if rows == 1 {
                    one_row.set(one_row.get() + 1);
                }
                if cols == 1 && rows == 1 {
                    single_cell.set(single_cell.get() + 1);
                }
                if cols > 1 && rows > 1 && cols != rows {
                    oblong.set(oblong.get() + 1);
                }
                Ok(())
            })
            .unwrap_or_else(|error| panic!("{error}"));

        assert!(one_column.get() > 0, "no generated Grid had one column");
        assert!(one_row.get() > 0, "no generated Grid had one row");
        assert!(
            single_cell.get() > 0,
            "no generated Grid held a single Cell"
        );
        assert!(
            oblong.get() > 0,
            "no generated Grid had both sides above one and unequal",
        );
    }
}
