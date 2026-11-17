//! Sources stated as rows, for tests and benchmarks.
//!
//! A test that needs a Source to drive something else states the rows a
//! reader would see and addresses a later edit by column and row. The Grid's
//! width and every flat index follow from the rows, so an Expression that
//! grows by an operand changes the row that holds it and nothing else.

use super::{Source, SourceCommander};
use crate::grid::{CellIndex, Grid};

///
/// The index `grid` mints for `idx`. A Cell is named by an index its Grid
/// minted, so a test states the number and the Grid answers with the Cell.
///
/// # Panics
///
/// If `idx` is past the Grid's last Cell.
///
pub fn cell(grid: Grid, idx: usize) -> CellIndex {
    grid.cell_index(idx).expect("inside the Grid")
}

///
/// The index `grid` mints for the Cell at `column` of `row`.
///
/// # Panics
///
/// If the Cell is outside the Grid.
///
pub fn at(grid: Grid, column: usize, row: usize) -> CellIndex {
    grid.index(grid.position(column, row).expect("inside the Grid"))
}

///
/// The smallest Grid that holds `rows`: as wide as the widest of them and as
/// tall as their count.
///
/// A Function that writes beside or below itself needs the Cells it writes
/// to, and an Expression needs a Cell for each operand it leaves blank. The
/// rows state that room as trailing spaces or an empty row.
///
/// # Panics
///
/// If `rows` is empty or every row is.
///
pub fn grid_of(rows: &[&str]) -> Grid {
    let width = rows.iter().map(|row| row.len()).max().unwrap_or(0);
    Grid::with_shape(width, rows.len())
}

///
/// Builds one Source Snapshot from `rows`, padded to the Grid's width.
///
pub fn snapshot(grid: Grid, rows: &[&str]) -> String {
    let width = grid.count() / rows.len();
    rows.iter().map(|row| format!("{row:width$}")).collect()
}

///
/// One Source built from `rows`, its Cells set one at a time as an editor
/// sets them.
///
/// Shared by every helper that runs a Tick against stated rows, so that
/// comparing two of them compares what they do differently and never two
/// ways of building a Source.
///
/// # Panics
///
/// If a row holds a character a Cell refuses.
///
pub fn seeded_source(grid: Grid, rows: &[&str]) -> Source {
    let mut source = Source::new(grid);
    for (index, byte) in snapshot(grid, rows).bytes().enumerate() {
        source
            .set(cell(grid, index), &char::from(byte).to_string())
            .unwrap();
    }
    source
}

///
/// The committed Source as one string per Grid row.
///
/// A Self-Banging Function is specified by where its Cells are, so a test
/// of one states a Grid and compares a Grid. Reading the same fact off a
/// list of Cell indices states the arithmetic instead of the geometry, and
/// an expectation nobody can picture is one nobody can check.
///
/// # Panics
///
/// If the Source holds a byte outside ASCII.
///
pub fn rows_of(grid: Grid, source: &Source) -> Vec<String> {
    source
        .snapshot()
        .into_bytes()
        .chunks(grid.columns())
        .map(|row| String::from_utf8(row.to_vec()).expect("ASCII Source"))
        .collect()
}

///
/// Row `row` of `snapshot`, a Source Snapshot of `grid`.
///
/// # Panics
///
/// If `snapshot` has no such row.
///
pub fn row(grid: Grid, snapshot: &str, row: usize) -> &str {
    let width = grid.columns();
    &snapshot[row * width..(row + 1) * width]
}

///
/// A shared Source holding `rows`, in the Grid [`grid_of`] gives them.
///
/// # Panics
///
/// If a row holds a character a Cell refuses.
///
pub fn commander(rows: &[&str]) -> SourceCommander {
    let source = SourceCommander::new(grid_of(rows));
    for (row, content) in rows.iter().enumerate() {
        write_at(&source, 0, row, content);
    }
    source
}

///
/// Sets the Cells of `row` from `column` on to the characters of `content`,
/// one at a time as an editor sets them.
///
/// # Panics
///
/// If `content` runs past the row's end or holds a character a Cell refuses.
///
pub fn write_at(source: &SourceCommander, column: usize, row: usize, content: &str) {
    let grid = source.grid();
    for (offset, content) in content.chars().enumerate() {
        source
            .set(at(grid, column + offset, row), &content.to_string())
            .unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_grid_is_as_wide_as_its_widest_row_and_as_tall_as_its_rows() {
        let grid = grid_of(&[".=0101", "", "!>007FC4"]);
        assert_eq!((grid.columns(), grid.rows()), (8, 3));
    }

    #[test]
    fn a_commander_holds_its_rows_padded_to_the_grids_width() {
        let source = commander(&[".=0101", "", "  !>007FC4"]);
        assert_eq!(
            source.snapshot(),
            snapshot(source.grid(), &[".=0101", "", "  !>007FC4"])
        );
    }

    #[test]
    fn an_edit_lands_at_its_column_and_row() {
        let source = commander(&[".=0101", "", "!>007FC4"]);
        write_at(&source, 5, 0, "2");
        write_at(&source, 6, 2, "D5");
        assert_eq!(
            source.snapshot(),
            snapshot(source.grid(), &[".=0102", "", "!>007FD5"])
        );
    }

    #[test]
    fn a_row_of_a_snapshot_is_that_rows_cells() {
        let source = commander(&[".=0101", "", "!>007FC4"]);
        assert_eq!(row(source.grid(), &source.snapshot(), 2), "!>007FC4");
        assert_eq!(row(source.grid(), &source.snapshot(), 0), ".=0101  ");
    }

    #[test]
    fn a_column_and_row_name_the_cell_a_flat_index_names() {
        let grid = Grid::with_shape(10, 4);
        assert_eq!(at(grid, 6, 2), cell(grid, 26));
    }
}
