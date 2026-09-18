use std::ops::Range;

use crate::grid::{Grid, Position};

///
/// A rectangle of Positions within a Grid, spanned from an anchor Cell to a
/// live end, with the Cursor on one of its Cells.
///
/// The Region has no corner of its own: it is the span between the anchor and
/// the live end, so moving the live end past the anchor flips the rectangle
/// rather than leaving a corner behind. The Cursor sits on the live end, except
/// after command A spans the whole Grid around it and leaves it where it was.
/// A Region always exists, and when the anchor sits on the live end it is that
/// one Cell — the ordinary state. See
/// `docs/adr/0046-the-primary-drag-selects-a-region.md`.
///
/// ```
/// use orcvs::grid::Grid;
/// use orcvs::region::Region;
///
/// let grid = Grid::new(8, 4);
/// let at = |x, y| grid.position(x, y).expect("inside the Grid");
///
/// // the Cursor above and left of the anchor spans the same rectangle as
/// // the Cursor below and right of it
/// let region = Region::span(grid, at(5, 3), at(2, 1));
/// assert_eq!((region.columns(), region.rows()), (2..6, 1..4));
/// assert_eq!(region.cursor(), at(2, 1));
/// ```
///
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Region {
    grid: Grid,
    anchor: Position,
    end: Position,
    cursor: Position,
}

impl Region {
    ///
    /// The Region of the one Cell at `position`.
    ///
    pub fn at(grid: Grid, position: Position) -> Self {
        Self::span(grid, position, position)
    }

    ///
    /// The Region from `anchor` to `cursor`, which is its live end, refusing a
    /// Position `grid` did not mint.
    ///
    pub fn span(grid: Grid, anchor: Position, cursor: Position) -> Self {
        grid.assert_owns(anchor);
        grid.assert_owns(cursor);
        Self {
            grid,
            anchor,
            end: cursor,
            cursor,
        }
    }

    ///
    /// The Region from `anchor` to `end` with the Cursor on `cursor`, which
    /// has to be one of its Cells.
    ///
    pub(crate) fn with_cursor(
        grid: Grid,
        anchor: Position,
        end: Position,
        cursor: Position,
    ) -> Self {
        grid.assert_owns(anchor);
        grid.assert_owns(end);
        let region = Self {
            grid,
            anchor,
            end,
            cursor,
        };
        assert!(region.contains(cursor), "the Cursor is outside its Region");
        region
    }

    ///
    /// Every Cell of `grid`, from its origin to its last Cell, with the Cursor
    /// left on `cursor`.
    ///
    /// The Cursor stays where it was, as a spreadsheet's active Cell does, so
    /// the Cursor follow of ADR 0045 does not move the Source View and the Cell
    /// a keystroke writes to next is the one it would have written to anyway.
    ///
    pub fn whole(grid: Grid, cursor: Position) -> Self {
        let last = grid
            .position(grid.columns() - 1, grid.rows() - 1)
            .expect("a Grid has at least one Cell");
        Self::with_cursor(grid, grid.origin(), last, cursor)
    }

    ///
    /// The Cell the Region was spanned from.
    ///
    pub fn anchor(&self) -> Position {
        self.anchor
    }

    ///
    /// The corner opposite the anchor: where a drag or Shift with an arrow
    /// moves the Region from.
    ///
    pub fn end(&self) -> Position {
        self.end
    }

    ///
    /// The Cursor: one of the Region's Cells, and its live end but for a
    /// Region command A spanned around it.
    ///
    pub fn cursor(&self) -> Position {
        self.cursor
    }

    ///
    /// The columns the Region covers, left to right.
    ///
    pub fn columns(&self) -> Range<usize> {
        let (a, b) = (self.anchor.x(), self.end.x());
        a.min(b)..a.max(b) + 1
    }

    ///
    /// The rows the Region covers, top to bottom.
    ///
    pub fn rows(&self) -> Range<usize> {
        let (a, b) = (self.anchor.y(), self.end.y());
        a.min(b)..a.max(b) + 1
    }

    ///
    /// The Region's top-left Cell, whichever corner the anchor and the Cursor
    /// sit on.
    ///
    pub fn top_left(&self) -> Position {
        self.grid
            .position(self.columns().start, self.rows().start)
            .expect("a Region's corner is a Cell of its Grid")
    }

    ///
    /// Whether the Region is the one Cell its anchor, live end and Cursor
    /// share.
    ///
    pub fn is_one_cell(&self) -> bool {
        self.anchor == self.end
    }

    ///
    /// Whether `position` is one of the Region's Cells.
    ///
    pub fn contains(&self, position: Position) -> bool {
        self.grid.assert_owns(position);
        self.columns().contains(&position.x()) && self.rows().contains(&position.y())
    }

    ///
    /// The Region's Positions, one iterator per row, top to bottom, each
    /// left to right.
    ///
    pub fn positions_by_row(&self) -> impl Iterator<Item = impl Iterator<Item = Position>> {
        let (grid, columns) = (self.grid, self.columns());
        self.rows().map(move |y| {
            columns.clone().map(move |x| {
                grid.position(x, y)
                    .expect("a Region's Cells are Cells of its Grid")
            })
        })
    }
}

#[cfg(test)]
mod tests {
    use super::Region;
    use crate::grid::Grid;

    #[test]
    fn a_region_spans_from_its_anchor_to_the_cursor_whichever_way_it_points() {
        let grid = Grid::new(10, 6);
        let at = |x, y| grid.position(x, y).expect("inside the Grid");

        let forward = Region::span(grid, at(2, 1), at(5, 3));
        let backward = Region::span(grid, at(5, 3), at(2, 1));
        let across = Region::span(grid, at(5, 1), at(2, 3));

        for region in [forward, backward, across] {
            assert_eq!(region.columns(), 2..6);
            assert_eq!(region.rows(), 1..4);
            assert_eq!(region.top_left(), at(2, 1));
            assert!(!region.is_one_cell());
        }
        assert_eq!(backward.anchor(), at(5, 3));
        assert_eq!(backward.cursor(), at(2, 1));
    }

    #[test]
    fn moving_the_cursor_past_the_anchor_flips_the_rectangle() {
        let grid = Grid::new(10, 6);
        let at = |x, y| grid.position(x, y).expect("inside the Grid");
        let anchor = at(4, 2);

        let right = Region::span(grid, anchor, at(6, 2));
        let left = Region::span(grid, anchor, at(1, 2));

        assert_eq!(right.columns(), 4..7);
        assert_eq!(left.columns(), 1..5);
        assert!(left.contains(anchor) && right.contains(anchor));
        assert!(!left.contains(at(5, 2)));
    }

    #[test]
    fn a_region_at_one_cell_is_that_cell() {
        let grid = Grid::new(3, 3);
        let cell = grid.position(1, 2).expect("inside the Grid");

        let region = Region::at(grid, cell);

        assert!(region.is_one_cell());
        assert_eq!((region.columns(), region.rows()), (1..2, 2..3));
        let cells: Vec<_> = region.positions_by_row().flatten().collect();
        assert_eq!(cells, vec![cell]);
    }

    #[test]
    fn a_region_walks_its_cells_row_by_row() {
        let grid = Grid::new(5, 5);
        let at = |x, y| grid.position(x, y).expect("inside the Grid");

        let region = Region::span(grid, at(3, 2), at(2, 1));
        let rows: Vec<Vec<_>> = region
            .positions_by_row()
            .map(|row| row.map(|p| (p.x(), p.y())).collect())
            .collect();

        assert_eq!(rows, vec![vec![(2, 1), (3, 1)], vec![(2, 2), (3, 2)]]);
    }

    #[test]
    fn the_whole_region_is_every_cell_of_the_grid_around_the_cursor() {
        let grid = Grid::new(4, 3);
        let cursor = grid.position(2, 1).expect("inside the Grid");

        let region = Region::whole(grid, cursor);

        assert_eq!((region.columns(), region.rows()), (0..4, 0..3));
        assert_eq!(region.cursor(), cursor);
        assert_eq!(region.anchor(), grid.origin());
        assert_eq!(region.end(), grid.position(3, 2).expect("inside the Grid"));
        assert!(!region.is_one_cell());
        assert_eq!(region.positions_by_row().flatten().count(), grid.count());
    }

    #[test]
    fn the_whole_region_of_a_one_cell_grid_is_one_cell() {
        let grid = Grid::new(1, 1);

        assert!(Region::whole(grid, grid.origin()).is_one_cell());
    }

    #[test]
    #[should_panic(expected = "Position belongs to another Grid")]
    fn a_region_refuses_a_position_another_grid_minted() {
        let grid = Grid::new(3, 3);
        let other = Grid::new(3, 3);

        Region::span(grid, grid.origin(), other.origin());
    }
}
