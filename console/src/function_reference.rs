//!
//! The Function reference: a checked-in Source text showing every Function in
//! the Function table, one worked example each — the Expression, and on the
//! row directly south the result one Tick writes there.
//!
//! `.scratch/function-reference/spec.md` states the intent; this module and
//! `assets/function_reference.orcvs` are the one checked-in Source it comes
//! from. The console opens on it whenever no stored Source is available (no
//! `persistence` feature, no stored revision, or a stored revision this build
//! refuses), and `Console`'s File menu offers it as an explicit "load"
//! action that replaces the running Source, Grid included.
//!
//! # Column layout
//!
//! Every group occupies one column two Sector Seams (16 Cells) wide, headed
//! by a `||` Comment naming it, with each example's Expression row, its
//! result row directly south, and one blank row. Groups sit side by side so
//! later tickets can add their own column without moving this one; the
//! planned order, left to right, is:
//!
//! | Columns   | Group                                                |
//! |-----------|-------------------------------------------------------|
//! | `0..16`   | Arithmetic (this ticket): `.+ .- .| .x ./ .% .< .> .=` |
//! | `16..32`  | Numeric Conversion: `.v .^`                            |
//! | `32..48`  | Sequence: `:- :# :< :& :? :=` (results up to 14 Cells) |
//! | `48..64`  | Tick: `~. ~* ~% ~+ ~> ~?`                              |
//! | `64..96`  | Jumps, Directional Bangs, Self-Banging, Halt (two columns: `&^ &v &< &>`, `*^ *v *< *> *!`, `^^ vv << >>` move and need room to move without leaving their area) |
//! | `96..112` | MIDI: `!> !~ !% !c !b !$` (examples ~10 Cells wide)     |
//!
//! Each group's header Comment claims the rest of its Grid row per the
//! Comment Language Unit, which is the whole Grid width rather than one
//! group's 16 Cells — a later ticket adding a second column on row 0 must
//! either give its header a row the Arithmetic column has already finished
//! with, or otherwise avoid two group headers sharing a row. This ticket
//! leaves that unresolved because only one group exists so far.
//!
//! Rows: the Arithmetic column needs 1 header row plus 10 examples of 3 rows
//! each (31 rows), so the Grid is padded to 32 — the next multiple of 8 — with
//! one trailing blank row. A later group taller than 32 rows widens the whole
//! Grid's row count; every column shares one row axis.
//!

use orcvs::grid::Grid;
use orcvs::source::Source;

const REFERENCE: &str = include_str!("../assets/function_reference.orcvs");

/// The Sector Seam spacing every Grid dimension here is a multiple of.
const SECTOR_SEAM: usize = 8;

///
/// The Function reference Source, rebuilt fresh each call.
///
/// The Grid's dimensions come from `REFERENCE` itself — its longest line and
/// its line count — rather than from a value stated separately in code, so
/// the two cannot drift apart. Every line must be the same width: a ragged
/// checked-in text is a defect in the reference, not a shape to tolerate.
///
pub(crate) fn function_reference() -> Source {
    let lines: Vec<&str> = REFERENCE.lines().collect();
    let columns = lines.first().map_or(0, |line| line.chars().count());
    let rows = lines.len();

    assert!(
        lines.iter().all(|line| line.chars().count() == columns),
        "the Function reference text must be a rectangle: every row the same width"
    );
    assert_eq!(
        columns % SECTOR_SEAM,
        0,
        "the Function reference's column count must be a multiple of the Sector Seam spacing"
    );
    assert_eq!(
        rows % SECTOR_SEAM,
        0,
        "the Function reference's row count must be a multiple of the Sector Seam spacing"
    );

    let grid = Grid::new(columns, rows);
    let mut source = Source::new(grid);
    for (y, line) in lines.into_iter().enumerate() {
        for (x, content) in line.chars().enumerate() {
            if content != ' ' {
                let position = grid.position(x, y).expect("inside the reference Grid");
                let cell = grid.index(position);
                source
                    .set(cell, &content.to_string())
                    .expect("the Function reference holds only printable ASCII");
            }
        }
    }

    source
}

#[cfg(test)]
mod tests {
    use super::function_reference;
    use orcvs::source::Tick;

    ///
    /// Every result row the Arithmetic group's examples write, read back
    /// after ticking the reference once.
    ///
    /// This is the test later groups extend: a later ticket adds its own
    /// `(row, expected)` entries for its own examples rather than a second
    /// assertion mechanism.
    ///
    #[test]
    fn ticking_the_reference_once_writes_every_result_row_exactly_as_written() {
        let mut source = function_reference();
        let grid = source.grid();

        let plan = source.execute(Tick::ZERO);
        assert!(
            plan.diagnostics.is_empty(),
            "every worked example must be a valid Expression: {:?}",
            plan.diagnostics
        );

        let expected = [
            (2, "03"),  // .+0102
            (5, "02"),  // .-0503
            (8, "04"),  // .|0307
            (11, "0C"), // .x0304
            (14, "04"), // ./0902
            (17, "01"), // .%0902
            (20, "03"), // .<0305
            (23, "05"), // .>0305
            (26, "**"), // .=0505 (equal)
            (29, "  "), // .=0506 (not equal: no Cell write)
        ];

        for (row, expected) in expected {
            let start = grid.position(0, row).expect("inside the reference Grid");
            let end = grid.position(1, row).expect("inside the reference Grid");
            let actual: String = [start, end]
                .into_iter()
                .map(|position| {
                    source
                        .get(grid.index(position))
                        .unwrap_or_else(|| " ".to_string())
                })
                .collect();

            assert_eq!(actual, expected, "row {row} did not read as written");
        }
    }

    #[test]
    fn the_reference_grid_dimensions_are_multiples_of_the_sector_seam_spacing() {
        let grid = function_reference().grid();

        assert_eq!(grid.columns() % 8, 0);
        assert_eq!(grid.rows() % 8, 0);
    }
}
