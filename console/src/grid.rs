///
/// A valid position within a Grid. A Position can only be obtained from the
/// Grid that contains it, so a Position outside its Grid cannot exist.
///
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Position {
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
/// The fixed rectangular shape a Source occupies: its column and row counts,
/// and the valid positions within them. The Grid is the shape; the Source is
/// the contents.
///
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Grid {
    cols: usize,
    rows: usize,
}

impl Grid {
    ///
    /// A Grid has at least one column and one row.
    ///
    pub fn new(cols: usize, rows: usize) -> Self {
        assert!(cols > 0, "cols must be greater than zero");
        assert!(rows > 0, "rows must be greater than zero");

        Self { cols, rows }
    }

    ///
    /// The only way to obtain a Position. `None` when (x, y) is outside this Grid.
    ///
    pub fn position(&self, x: usize, y: usize) -> Option<Position> {
        if x < self.cols && y < self.rows {
            Some(Position { x, y })
        } else {
            None
        }
    }

    ///
    /// The first Position of this Grid.
    ///
    #[inline]
    pub fn origin(&self) -> Position {
        Position { x: 0, y: 0 }
    }

    ///
    /// The linear index of a Position in this Grid.
    ///
    /// Total: a Position can only be obtained from a Grid, so it is in range
    /// for the Grid that minted it. A Position is a bare pair carrying no grid
    /// identity, so passing one minted by a *different* Grid is not prevented;
    /// there is one Grid per Source and no path that mixes two.
    ///
    #[inline]
    pub fn index(&self, pos: Position) -> usize {
        pos.y * self.cols + pos.x
    }

    ///
    /// The Position one row above `pos`, clamped at the top row.
    ///
    #[inline]
    pub fn up(&self, pos: Position) -> Position {
        Position {
            x: pos.x,
            y: pos.y.saturating_sub(1),
        }
    }

    ///
    /// The Position one row below `pos`, clamped at the bottom row.
    ///
    #[inline]
    pub fn down(&self, pos: Position) -> Position {
        Position {
            x: pos.x,
            y: (pos.y + 1).min(self.rows - 1),
        }
    }

    ///
    /// The Position one column left of `pos`, clamped at the first column.
    ///
    #[inline]
    pub fn left(&self, pos: Position) -> Position {
        Position {
            x: pos.x.saturating_sub(1),
            y: pos.y,
        }
    }

    ///
    /// The Position one column right of `pos`, clamped at the last column.
    ///
    #[inline]
    pub fn right(&self, pos: Position) -> Position {
        Position {
            x: (pos.x + 1).min(self.cols - 1),
            y: pos.y,
        }
    }

    #[inline]
    pub fn cols(&self) -> usize {
        self.cols
    }

    #[inline]
    pub fn rows(&self) -> usize {
        self.rows
    }

    #[inline]
    pub fn count(&self) -> usize {
        self.cols * self.rows
    }
}

#[cfg(test)]
mod test {

    use crate::{grid::Grid, test::trace};

    #[test]
    fn test_grid_keeps_its_columns_and_rows() {
        trace();

        let grid = Grid::new(4, 2);

        assert_eq!(grid.cols(), 4);
        assert_eq!(grid.rows(), 2);
        assert_eq!(grid.count(), 8);
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
    fn test_grid_indices_cover_every_cell_exactly_once() {
        trace();

        let grid = Grid::new(4, 2);

        let mut seen: Vec<usize> = Vec::new();
        for y in 0..grid.rows() {
            for x in 0..grid.cols() {
                seen.push(grid.index(grid.position(x, y).expect("inside the grid")));
            }
        }

        assert_eq!(seen, (0..grid.count()).collect::<Vec<usize>>());
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
}
