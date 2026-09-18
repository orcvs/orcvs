use std::ops::Range;

use crate::grid::{Grid, Position};

///
/// A rectangle of Positions within a Grid, spanned from an anchor Cell to the
/// Cursor.
///
/// The Region has no corner of its own: it is the span between the two, so
/// moving the Cursor past the anchor flips the rectangle rather than leaving a
/// corner behind. A Region always exists, and when the anchor sits on the
/// Cursor it is that one Cell — the ordinary state. See
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
    /// The Region from `anchor` to `cursor`, refusing a Position `grid` did
    /// not mint.
    ///
    pub fn span(grid: Grid, anchor: Position, cursor: Position) -> Self {
        grid.assert_owns(anchor);
        grid.assert_owns(cursor);
        Self {
            grid,
            anchor,
            cursor,
        }
    }

    ///
    /// Every Cell of `grid`, anchored at its last Cell with the Cursor on its
    /// origin.
    ///
    /// The Cursor takes the origin so the Cursor follow of ADR 0045 brings the
    /// Grid's start into view rather than its far corner, and so the Cell a
    /// keystroke writes to next is the first one.
    ///
    pub fn whole(grid: Grid) -> Self {
        let last = grid
            .position(grid.columns() - 1, grid.rows() - 1)
            .expect("a Grid has at least one Cell");
        Self::span(grid, last, grid.origin())
    }

    ///
    /// The Cell the Region was spanned from.
    ///
    pub fn anchor(&self) -> Position {
        self.anchor
    }

    ///
    /// The Region's live end, which is the Cursor.
    ///
    pub fn cursor(&self) -> Position {
        self.cursor
    }

    ///
    /// The columns the Region covers, left to right.
    ///
    pub fn columns(&self) -> Range<usize> {
        let (a, b) = (self.anchor.x(), self.cursor.x());
        a.min(b)..a.max(b) + 1
    }

    ///
    /// The rows the Region covers, top to bottom.
    ///
    pub fn rows(&self) -> Range<usize> {
        let (a, b) = (self.anchor.y(), self.cursor.y());
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
    /// Whether the Region is the one Cell its anchor and Cursor share.
    ///
    pub fn is_one_cell(&self) -> bool {
        self.anchor == self.cursor
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
    fn the_whole_region_is_every_cell_of_the_grid() {
        let grid = Grid::new(4, 3);

        let region = Region::whole(grid);

        assert_eq!((region.columns(), region.rows()), (0..4, 0..3));
        assert_eq!(region.cursor(), grid.origin());
        assert_eq!(region.positions_by_row().flatten().count(), grid.count());
    }

    #[test]
    #[should_panic(expected = "Position belongs to another Grid")]
    fn a_region_refuses_a_position_another_grid_minted() {
        let grid = Grid::new(3, 3);
        let other = Grid::new(3, 3);

        Region::span(grid, grid.origin(), other.origin());
    }
}
