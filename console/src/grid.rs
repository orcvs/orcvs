use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_GRID_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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
/// The shape a console starts with, until something states another one. A
/// Grid's dimensions are its own: they are stated here as Cell counts, and
/// derived from nothing else.
///
pub const DEFAULT_COL_COUNT: usize = 32;
pub const DEFAULT_ROW_COUNT: usize = 32;

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
    pub fn index(&self, pos: Position) -> usize {
        self.assert_owns(pos);
        pos.y * self.cols + pos.x
    }

    /// Whether `pos` was minted by this Grid or one of its copies.
    #[inline]
    pub fn owns(&self, pos: Position) -> bool {
        self.id == pos.grid_id
    }

    ///
    /// The Cell an index addresses, and so which row and column it is in.
    /// `None` when the index is past the last Cell. The inverse of `index`.
    ///
    #[inline]
    pub fn position_at(&self, idx: usize) -> Option<Position> {
        self.position(idx % self.cols, idx / self.cols)
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
    /// Whether a value `width` Cells wide fits in `pos`'s row, counting `pos`
    /// as its first Cell. A row is the whole horizontal extent there is:
    /// nothing continues onto the next one.
    ///
    /// A foreign Position is refused: its identity names the Grid that minted it.
    ///
    #[inline]
    pub fn fits(&self, pos: Position, width: usize) -> bool {
        self.assert_owns(pos);
        width <= self.cols.saturating_sub(pos.x)
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
    pub fn rows(&self) -> impl Iterator<Item = impl Iterator<Item = Position>> {
        // Captured by value: Grid and Position are allocation-free Copy values,
        // so the returned iterators borrow nothing.
        let (id, cols, rows) = (self.id, self.cols, self.rows);

        (0..rows).map(move |y| (0..cols).map(move |x| Position { grid_id: id, x, y }))
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

        let rows: Vec<Vec<Position>> = grid.rows().map(|row| row.collect()).collect();

        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].len(), 4);
        assert_eq!(rows[1].len(), 4);

        // rows top to bottom, each row left to right
        assert_eq!(rows[0], vec![at(0, 0), at(1, 0), at(2, 0), at(3, 0)]);
        assert_eq!(rows[1], vec![at(0, 1), at(1, 1), at(2, 1), at(3, 1)]);

        // flattened, exactly the Source's own index order
        let indices: Vec<usize> = rows.iter().flatten().map(|p| grid.index(*p)).collect();
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

        let index = |x, y| grid.index(grid.position(x, y).expect("inside the grid"));

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
    fn test_grid_identity_survives_copying() {
        let grid = Grid::new(4, 2);
        let copied = grid;
        let position = grid.position(1, 0).expect("inside the Grid");

        assert!(copied.owns(position));
        assert_eq!(copied.index(position), 1);
    }

    #[test]
    fn test_grid_indices_cover_every_cell_exactly_once() {
        trace();

        let grid = Grid::new(4, 2);

        // every index names a Position, and that Position converts back to the
        // index it came from: no two Cells share an index, and none is missed
        for idx in 0..grid.count() {
            let position = grid.position_at(idx).expect("inside the grid");

            assert_eq!(grid.index(position), idx);
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

        // the inverse of `index`: row order, first row then second
        assert_eq!(grid.position_at(0), Some(at(0, 0)));
        assert_eq!(grid.position_at(3), Some(at(3, 0)));
        assert_eq!(grid.position_at(4), Some(at(0, 1)));
        assert_eq!(grid.position_at(7), Some(at(3, 1)));

        // past the last Cell: no Position, rather than one wrapped back inside
        assert_eq!(grid.position_at(8), None);
        assert_eq!(grid.position_at(100), None);
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
    fn test_grid_answers_whether_a_width_fits_in_the_row() {
        trace();

        let grid = Grid::new(4, 2);
        let at = |x, y| grid.position(x, y).expect("inside the grid");

        // from the first column the whole row is available
        assert!(grid.fits(at(0, 1), 4));
        assert!(!grid.fits(at(0, 1), 5));

        // from the third column of a 4 column Grid, two Cells are left
        assert!(grid.fits(at(2, 0), 2));
        assert!(!grid.fits(at(2, 0), 3));

        // the last column holds one Cell, and nothing wraps onto the next row
        assert!(grid.fits(at(3, 0), 1));
        assert!(!grid.fits(at(3, 0), 2));
    }
}
