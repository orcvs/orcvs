//! Sources stated as rows, for tests and benchmarks.
//!
//! A test that needs a Source to drive something else states the rows a
//! reader would see and addresses a later edit by column and row. The Grid's
//! width and every flat index follow from the rows, so an Expression that
//! grows by an operand changes the row that holds it and nothing else.
//!
//! A Cell holds one ASCII character, so a row is measured in bytes. Each
//! entry point that takes text ([`grid_of`], [`snapshot`] and the helpers
//! built on them, and [`write_at`]) refuses text outside ASCII rather than
//! miscount it.

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
/// Refuses `text` unless it is ASCII, the one character set a Cell holds.
///
fn assert_ascii(text: &str) {
    assert!(
        text.is_ascii(),
        "{text:?} is not ASCII, and a Cell holds one ASCII character"
    );
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
/// If a row is not ASCII, if there are no rows, if there are only empty
/// rows, or if the rows are wider or more than [`Grid::with_shape`] allows.
///
pub fn grid_of(rows: &[&str]) -> Grid {
    rows.iter().for_each(|row| assert_ascii(row));
    let Some(width) = rows.iter().map(|row| row.len()).max() else {
        panic!("a Grid of no rows holds nothing");
    };
    assert!(width > 0, "a Grid of only empty rows holds nothing");
    Grid::with_shape(width, rows.len())
}

///
/// Builds one Source Snapshot of `grid` from `rows`: each row padded to the
/// Grid's width, and every Grid row past the last one stated left blank, so
/// no rows at all is a blank Snapshot.
///
/// # Panics
///
/// If a row is not ASCII or is wider than the Grid, or if `rows` states more
/// rows than the Grid has.
///
pub fn snapshot(grid: Grid, rows: &[&str]) -> String {
    let width = grid.columns();
    assert!(
        rows.len() <= grid.rows(),
        "{} rows are more rows than the Grid's {}",
        rows.len(),
        grid.rows()
    );
    let mut snapshot = String::with_capacity(grid.count());
    for row in rows {
        assert_ascii(row);
        assert!(
            row.len() <= width,
            "row {row:?} is wider than the Grid's {width} columns"
        );
        snapshot.push_str(&format!("{row:width$}"));
    }
    snapshot.push_str(&" ".repeat(grid.count() - snapshot.len()));
    snapshot
}

///
/// One Source built from `rows`, every Cell of its Grid set one at a time,
/// spaces included, as an editor sets them.
///
/// Shared by every helper that runs a Tick against stated rows, so that
/// comparing two of them compares what they do differently and never two
/// ways of building a Source.
///
/// # Panics
///
/// As [`snapshot`] does, or if a row holds a character a Cell refuses.
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
/// The committed Source as one string per row of its own Grid.
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
pub fn rows_of(source: &Source) -> Vec<String> {
    source
        .snapshot()
        .into_bytes()
        .chunks(source.grid().columns())
        .map(|row| String::from_utf8(row.to_vec()).expect("ASCII Source"))
        .collect()
}

///
/// Row `index` of `snapshot`, a Source Snapshot of `grid`.
///
/// # Panics
///
/// If `snapshot` is shorter than row `index` of `grid` needs, as it is when
/// `index` is past the Grid's last row.
///
pub fn row(grid: Grid, snapshot: &str, index: usize) -> &str {
    let width = grid.columns();
    &snapshot[index * width..(index + 1) * width]
}

///
/// A shared Source holding `rows`, in the Grid [`grid_of`] gives them.
///
/// Only the characters each row states are set, as an editor types them:
/// the padding past a row's end and every empty row are Cells never written,
/// so they are not edited and make no revision. [`seeded_source`] sets every
/// Cell of its Grid, spaces included.
///
/// # Panics
///
/// As [`grid_of`] does, or if a row holds a character a Cell refuses.
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
/// If `content` is not ASCII, if `row` or `column` is outside the Grid, if
/// `content` runs past the row's end, or if it holds a character a Cell
/// refuses.
///
pub fn write_at(source: &SourceCommander, column: usize, row: usize, content: &str) {
    assert_ascii(content);
    let grid = source.grid();
    // Checked before the loop, so an empty `content` is refused there too.
    at(grid, column, row);
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
    fn a_snapshot_is_as_wide_as_its_grid_whatever_its_rows_state() {
        let grid = Grid::with_shape(10, 4);
        let snapshot = snapshot(grid, &["", "ab", "", ""]);
        assert_eq!(row(grid, &snapshot, 1), "ab        ");
        assert_eq!(snapshot.len(), grid.count());
    }

    #[test]
    fn rows_a_snapshot_does_not_state_are_blank() {
        let grid = Grid::with_shape(4, 3);
        assert_eq!(snapshot(grid, &["ab"]), format!("ab{}", " ".repeat(10)));
    }

    #[test]
    fn a_seeded_source_holds_its_rows_at_the_grids_width() {
        let grid = Grid::with_shape(10, 4);
        let source = seeded_source(grid, &["", "ab"]);
        assert_eq!(
            rows_of(&source),
            ["          ", "ab        ", "          ", "          "]
        );
    }

    #[test]
    #[should_panic(expected = "wider than the Grid")]
    fn a_row_wider_than_its_grid_is_refused() {
        snapshot(Grid::with_shape(4, 2), &["abcde", ""]);
    }

    #[test]
    #[should_panic(expected = "more rows than the Grid")]
    fn more_rows_than_the_grid_holds_are_refused() {
        snapshot(Grid::with_shape(4, 1), &["a", "b"]);
    }

    #[test]
    #[should_panic(expected = "not ASCII")]
    fn a_grid_of_a_row_outside_ascii_is_refused() {
        grid_of(&["é"]);
    }

    #[test]
    #[should_panic(expected = "not ASCII")]
    fn a_snapshot_of_a_row_outside_ascii_is_refused() {
        snapshot(Grid::with_shape(4, 1), &["é"]);
    }

    #[test]
    #[should_panic(expected = "not ASCII")]
    fn an_edit_outside_ascii_is_refused() {
        let source = commander(&["ab"]);
        write_at(&source, 0, 0, "é");
    }

    #[test]
    #[should_panic(expected = "inside the Grid")]
    fn an_empty_edit_outside_the_grid_is_refused() {
        let source = commander(&[".=0101"]);
        write_at(&source, 0, 99, "");
    }

    #[test]
    #[should_panic(expected = "a Grid of no rows")]
    fn a_grid_of_no_rows_is_refused() {
        grid_of(&[]);
    }

    #[test]
    #[should_panic(expected = "a Grid of only empty rows")]
    fn a_grid_of_only_empty_rows_is_refused() {
        grid_of(&["", ""]);
    }

    #[test]
    fn a_column_and_row_name_the_cell_a_flat_index_names() {
        let grid = Grid::with_shape(10, 4);
        assert_eq!(at(grid, 6, 2), cell(grid, 26));
    }
}
