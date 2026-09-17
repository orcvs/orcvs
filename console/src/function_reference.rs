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
//! planned order, left to right, with the column band index `k` used below, is:
//!
//! | Columns   | `k` | Group                                                |
//! |-----------|-----|-------------------------------------------------------|
//! | `0..16`   | 0   | Arithmetic (ticket 01): `.+ .- .| .x ./ .% .< .> .=`   |
//! | `16..32`  | 1   | Numeric Conversion (ticket 02): `.v .^`                |
//! | `32..48`  | 2   | Sequence (ticket 02): `:- :# :< :& :? :=` (results up to 14 Cells) |
//! | `48..64`  | 3   | Tick (this ticket): `~. ~* ~% ~+ ~> ~?`                |
//! | `64..96`  | 4..5 | Jumps, Directional Bangs, Self-Banging, Halt (two columns: `&^ &v &< &>`, `*^ *v *< *> *!`, `^^ vv << >>` move and need room to move without leaving their area) |
//! | `96..112` | 6   | MIDI: `!> !~ !% !c !b !$` (examples ~10 Cells wide)     |
//!
//! Each group's header Comment claims the rest of its *Grid* row per the
//! Comment Language Unit — the whole row, not just its own 16 Cells — so two
//! group headers cannot share a row once more than one group exists. This is
//! resolved by a staircase: the group in column band `k` puts its header on
//! row `k` and starts its examples on row `k + 1`. A row is parsed left to
//! right, so every column west of band `k` was already claimed by its own
//! Expression or blank row before the parser reaches column `16 * k`, and
//! every column east of it has not started yet (its own header is a later
//! row), so the header's rightward claim only ever swallows blank padding.
//! Arithmetic keeps `k = 0` from ticket 01 and does not move.
//!
//! Rows: the tallest group decides the Grid's row count — currently
//! Arithmetic, whose header plus 10 examples of 3 rows each reaches row 30 —
//! rounded up to 32, the next multiple of 8. Conversion, Sequence, and Tick
//! all start later (rows 1, 2, and 3) but finish well inside that height —
//! Tick's header plus six examples of 3 rows each reaches row 21 — so this
//! ticket does not grow the Grid's row count either; a later group taller
//! than 32 rows would widen the whole Grid's row count, since every column
//! shares one row axis. Tick's column does widen the Grid: its widest row is
//! Random's eight-Cell Expression, `~?010010`, reaching column 55 and
//! rounding the Grid's width up from 48 to 56.
//!
//! # Checked-in text is ragged, not a padded rectangle
//!
//! `function_reference.orcvs` stores each line with its trailing whitespace
//! removed, and stops at the last line with any content — no trailing blank
//! rows. Editors and formatters strip trailing whitespace on save, which
//! would otherwise turn a checked-in padded rectangle ragged on its next
//! untouched edit and fail a loader that demanded one. `source_from_reference_text`
//! instead derives the Grid's width from the widest line and its height from
//! the line count, each rounded up to the Sector Seam spacing, and pads
//! every short line and every row past the last line with empty Cells — see
//! its doc comment.
//!

use orcvs::grid::Grid;
use orcvs::source::Source;

const REFERENCE: &str = include_str!("../assets/function_reference.orcvs");

/// The Sector Seam spacing every Grid dimension here is a multiple of.
const SECTOR_SEAM: usize = 8;

///
/// The Function reference Source, rebuilt fresh each call.
///
pub(crate) fn function_reference() -> Source {
    source_from_reference_text(REFERENCE)
}

///
/// Parses `text` into a Source whose Grid is exactly wide and tall enough for
/// it: the widest line's Cell count and the line count, each rounded up to
/// the Sector Seam spacing.
///
/// A short line is padded with empty Cells rather than required to reach the
/// Grid's width, and any row past the last line is left entirely empty —
/// both are ordinary unset Cells, not a special case, since [`Source::get`]
/// already reads an unset Cell back as empty. This tolerates a ragged
/// `text`, which is what an editor or formatter that strips trailing
/// whitespace leaves behind: a padded rectangle with every trailing space
/// removed is ragged the moment one row's content ends before another's.
///
fn source_from_reference_text(text: &str) -> Source {
    let lines: Vec<&str> = text.lines().collect();
    let widest = lines
        .iter()
        .map(|line| line.chars().count())
        .max()
        .unwrap_or(0);
    let columns = round_up_to_sector_seam(widest);
    let rows = round_up_to_sector_seam(lines.len());

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

/// Rounds `value` up to the next multiple of [`SECTOR_SEAM`] (`0` stays `0`).
fn round_up_to_sector_seam(value: usize) -> usize {
    value.div_ceil(SECTOR_SEAM) * SECTOR_SEAM
}

#[cfg(test)]
mod tests {
    use super::{function_reference, source_from_reference_text};
    use orcvs::source::Tick;

    ///
    /// A checked-in, padded rectangle is not the only shape the loader must
    /// accept: an editor or formatter that strips trailing whitespace turns
    /// every padded row short of the widest into a ragged one, and a blank
    /// row into an empty line. The loader rounds the widest line and the
    /// line count up to the Sector Seam spacing instead of asserting a
    /// rectangle already at that shape, so this ragged text — never checked
    /// in, built by the test itself — loads rather than panics.
    ///
    #[test]
    fn a_ragged_text_with_stripped_trailing_whitespace_loads_at_rounded_up_dimensions() {
        // Three lines, none the same length: 6 Cells, 0 (a blank line
        // stripped bare), and 2. The widest, 6, rounds up to 8 Cells; the 3
        // lines round up to 8 rows.
        let source = source_from_reference_text(".+0102\n\n0C");
        let grid = source.grid();

        assert_eq!(grid.columns(), 8);
        assert_eq!(grid.rows(), 8);

        let read = |x, y| {
            let position = grid.position(x, y).expect("inside the rounded-up Grid");
            source.get(grid.index(position))
        };

        assert_eq!(read(0, 0), Some(".".to_string()));
        assert_eq!(read(5, 0), Some("2".to_string()));
        assert_eq!(
            read(6, 0),
            None,
            "a short line pads with empty Cells rather than reaching into the next row"
        );
        assert_eq!(read(0, 1), None, "a blank line stays entirely empty");
        assert_eq!(read(0, 2), Some("0".to_string()));
        assert_eq!(read(1, 2), Some("C".to_string()));
        assert_eq!(
            read(0, 7),
            None,
            "a row past the last line is empty padding, not an error"
        );
    }

    ///
    /// Every result row the Arithmetic, Conversion, Sequence, and Tick
    /// groups' examples write, read back after ticking the reference once.
    ///
    /// This is the test later groups extend: a later ticket adds its own
    /// `(column, row, expected)` entries for its own examples rather than a
    /// second assertion mechanism. `column` is the group's own anchor column
    /// (Arithmetic 0, Conversion 16, Sequence 32, Tick 48), since each
    /// group's result rows are read back from its own Cells, not always the
    /// leftmost ones.
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
            // Arithmetic (column 0)
            (0, 2, "03"),  // .+0102
            (0, 5, "02"),  // .-0503
            (0, 8, "04"),  // .|0307
            (0, 11, "0C"), // .x0304
            (0, 14, "04"), // ./0902
            (0, 17, "01"), // .%0902
            (0, 20, "03"), // .<0305
            (0, 23, "05"), // .>0305
            (0, 26, "**"), // .=0505 (equal)
            (0, 29, "  "), // .=0506 (not equal: no Cell write)
            // Conversion (column 16)
            (16, 3, "C4"), // .^3C (Number to Note)
            (16, 6, "3C"), // .vC4 (Note to Number)
            // Sequence (column 32)
            (32, 4, "01020304"),  // :-0104 (Number Range)
            (32, 7, "C4c4D4"),    // :#C4D4 (Note Range)
            (32, 10, "04030201"), // :<:-0104 (Reverse, over a nested Function operand)
            (32, 13, "010203"),   // :&01:-0203 (Concatenate)
            (32, 16, "01"),       // :?00:-0103 (Select)
            (32, 19, "010303"),   // :=01.+0102:-0103 (Replace, over nested Functions)
            (32, 22, "111213"),   // .+10:-0103 (Add, pervasive over a Sequence)
            // Tick (column 48). Clock, Delay, Euclidean, and Random depend
            // only on their operands, the absolute Tick, and (for Random)
            // this Function's own Grid Position — never on a previously
            // written Cell — so the checked-in result row already holds
            // what Tick 0 writes, exactly like Arithmetic.
            //
            // Increment and Interpolation are different: each reads its own
            // result Cell as the previous value before it writes a new one.
            // The checked-in text therefore holds a pre-Tick previous chosen
            // to make the change visible (Increment's `03`, Interpolation's
            // `00`), and this table asserts the value Tick 0 overwrites it
            // with — the two are deliberately not the same string.
            (48, 5, "00"),  // ~.0204 (Clock: step 0 of a 2-Tick, 4-step cycle)
            (48, 8, "**"),  // ~*0302 (Delay: every cycle Bangs at Tick 0)
            (48, 11, "**"), // ~%0308 (Euclidean: 03-08's pattern Bangs its first step)
            (48, 14, "00"), // ~+0104 (Increment: previous 03 -> (03+01)%04)
            (48, 17, "02"), // ~>0210 (Interpolation: previous 00 -> steps by 02 toward 10)
            (48, 20, "10"), // ~?010010 (Random: seed 01 at Position (48, 19), Tick 0)
        ];

        for (column, row, expected) in expected {
            let width = expected.chars().count();
            let actual: String = (0..width)
                .map(|offset| {
                    let position = grid
                        .position(column + offset, row)
                        .expect("inside the reference Grid");
                    source
                        .get(grid.index(position))
                        .unwrap_or_else(|| " ".to_string())
                })
                .collect();

            assert_eq!(
                actual, expected,
                "row {row}, column {column} did not read as written"
            );
        }
    }

    #[test]
    fn the_reference_grid_dimensions_are_multiples_of_the_sector_seam_spacing() {
        let grid = function_reference().grid();

        assert_eq!(grid.columns() % 8, 0);
        assert_eq!(grid.rows() % 8, 0);
    }
}
