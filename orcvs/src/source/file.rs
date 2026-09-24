//!
//! The Source File: a Source written out as plain text (ADR 0054,
//! `.scratch/menu-structure/spec.md`).
//!
//! One line per row, one character per Cell, a space for an empty Cell. Line
//! *n* is row *n* and character *m* is column *m*, so a Source File states
//! Cells and never a shape: a short line, a missing line, or trailing spaces
//! an editor stripped all read back as empty Cells of the one 256 by 256 Grid.
//!
//! [`write()`] produces LF-terminated lines, trailing spaces trimmed, up to the
//! last row holding content. [`read()`] accepts LF or CRLF and refuses the whole
//! text — constructing no Source — at the first byte that is not printable
//! ASCII or space, the first line past the 256th, or the first character past
//! the 256th on a line, naming the line and column where it stopped.
//!

use std::fmt;

use crate::grid::Grid;

use super::{CellContent, CellWrite, Source};

/// The byte an empty Cell holds, and the byte [`write()`] trims from a row's end.
const SPACE: u8 = b' ';

///
/// Why [`read()`] refused a Source File.
///
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Refusal {
    /// A byte that is not printable ASCII or space: a tab, a lone carriage
    /// return, any other control byte, or any byte of a non-ASCII character.
    NotACell(u8),
    /// A 257th line. The Grid has 256 rows.
    TooManyLines,
    /// A 257th character on one line. The Grid has 256 columns.
    LineTooLong,
}

///
/// A Source File [`read()`] refused whole, and where it stopped.
///
/// `line` and `column` count from one, as an editor does: the refused byte is
/// row `line - 1`, column `column - 1`. A [`Refusal::TooManyLines`] names the
/// first column of the 257th line; a [`Refusal::LineTooLong`] names the 257th
/// column of its line.
///
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SourceFileError {
    line: usize,
    column: usize,
    refusal: Refusal,
}

impl SourceFileError {
    /// The line, counted from one, the refusal was found on.
    pub fn line(&self) -> usize {
        self.line
    }

    /// The column, counted from one, the refusal was found at.
    pub fn column(&self) -> usize {
        self.column
    }

    /// Why the text is not a Source File.
    pub fn refusal(&self) -> Refusal {
        self.refusal
    }

    /// The refusal at zero-based `row` and `column`.
    fn at(row: usize, column: usize, refusal: Refusal) -> Self {
        Self {
            line: row + 1,
            column: column + 1,
            refusal,
        }
    }
}

impl fmt::Display for SourceFileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "line {}, column {}: ", self.line, self.column)?;
        match self.refusal {
            Refusal::NotACell(b'\t') => {
                write!(f, "a tab is not a Cell; use spaces")
            }
            Refusal::NotACell(b'\r') => {
                write!(
                    f,
                    "a carriage return outside a CRLF line ending is not a Cell"
                )
            }
            Refusal::NotACell(byte) => {
                write!(f, "byte 0x{byte:02X} is not printable ASCII or space")
            }
            Refusal::TooManyLines => write!(
                f,
                "a Source File has at most {} lines",
                crate::grid::ROW_COUNT
            ),
            Refusal::LineTooLong => write!(
                f,
                "a line holds at most {} characters",
                crate::grid::COL_COUNT
            ),
        }
    }
}

impl std::error::Error for SourceFileError {}

///
/// The Source `text` states, on a new Grid of the one shape, or the refusal
/// that stopped it. A refusal constructs no Source: the text is refused whole.
///
/// Bytes rather than a `str`, so a file that is not UTF-8 is refused at the
/// byte that makes it so, with its line and column, rather than before
/// anything could say where.
///
/// ```
/// use orcvs::source::file;
///
/// let source = file::read(b".+0102\r\n\n  0C").expect("a Source File");
/// let grid = source.grid();
/// let at = |x, y| source.get(grid.index(grid.position(x, y).unwrap()));
///
/// assert_eq!((grid.columns(), grid.rows()), (256, 256));
/// assert_eq!(at(1, 0), Some("+".to_string()));
/// assert_eq!(at(0, 1), None);
/// assert_eq!(at(3, 2), Some("C".to_string()));
///
/// let Err(refused) = file::read(b"..\n.\t") else {
///     panic!("a tab is not a Cell");
/// };
/// assert_eq!((refused.line(), refused.column()), (2, 2));
/// ```
///
pub fn read(text: &[u8]) -> Result<Source, SourceFileError> {
    let grid = Grid::new();
    let (columns, rows) = (grid.columns(), grid.rows());
    let mut writes = Vec::new();

    // The final line's terminator ends that line rather than starting another,
    // so a file that ends in LF, as every file `write` produces does, has as
    // many lines as it has line terminators.
    let body = text.strip_suffix(b"\n").unwrap_or(text);
    for (row, line) in body.split(|byte| *byte == b'\n').enumerate() {
        if row >= rows {
            return Err(SourceFileError::at(row, 0, Refusal::TooManyLines));
        }
        let line = line.strip_suffix(b"\r").unwrap_or(line);
        for (column, byte) in line.iter().copied().enumerate() {
            if column >= columns {
                return Err(SourceFileError::at(row, column, Refusal::LineTooLong));
            }
            let content = CellContent::new(byte).ok_or(SourceFileError::at(
                row,
                column,
                Refusal::NotACell(byte),
            ))?;
            if byte != SPACE {
                let position = grid
                    .position(column, row)
                    .expect("checked against the Grid");
                writes.push(CellWrite {
                    cell: grid.index(position),
                    content,
                });
            }
        }
    }

    // Every byte is a `CellContent` before the Source sees it, so the Source
    // is built the one way block edits are, and its invariants are its own.
    let mut source = Source::new(grid);
    source.write_cells(&writes);
    Ok(source)
}

///
/// The Source File text of `source`: one line per row up to the last row
/// holding content, each with its trailing spaces trimmed and ended by LF.
/// A Source with no content writes the empty text.
///
/// ```
/// use orcvs::grid::Grid;
/// use orcvs::source::{Source, file};
///
/// let grid = Grid::new();
/// let mut source = Source::new(grid);
/// source.set(grid.cell_index(2).unwrap(), "1").unwrap();
/// source.set(grid.cell_index(2 * 256).unwrap(), "*").unwrap();
///
/// assert_eq!(file::write(&source), "  1\n\n*\n");
/// ```
///
pub fn write(source: &Source) -> String {
    let grid = source.grid();
    let cells = source.snapshot();
    let rows = cells
        .as_bytes()
        .chunks_exact(grid.columns())
        .map(|row| {
            let end = row
                .iter()
                .rposition(|byte| *byte != SPACE)
                .map_or(0, |at| at + 1);
            &row[..end]
        })
        .collect::<Vec<_>>();
    let Some(last) = rows.iter().rposition(|row| !row.is_empty()) else {
        return String::new();
    };

    let mut text = String::with_capacity(rows[..=last].iter().map(|row| row.len() + 1).sum());
    for row in &rows[..=last] {
        // A Source holds printable ASCII in every Cell, so every row is UTF-8.
        text.push_str(std::str::from_utf8(row).expect("a Source row is printable ASCII"));
        text.push('\n');
    }
    text
}

#[cfg(test)]
mod test {
    use super::{Refusal, SourceFileError, read, write};
    use crate::grid::{COL_COUNT, Grid, ROW_COUNT};
    use crate::source::Source;

    fn refused(text: &[u8]) -> SourceFileError {
        match read(text) {
            Ok(_) => panic!(
                "{:?} was read as a Source File",
                String::from_utf8_lossy(text)
            ),
            Err(refusal) => refusal,
        }
    }

    fn at(source: &Source, x: usize, y: usize) -> Option<String> {
        let grid = source.grid();
        source.get(grid.index(grid.position(x, y).expect("inside the Grid")))
    }

    fn row_of(width: usize, byte: u8) -> Vec<u8> {
        vec![byte; width]
    }

    fn lines(count: usize, line: &[u8]) -> Vec<u8> {
        let mut text = Vec::new();
        for _ in 0..count {
            text.extend_from_slice(line);
            text.push(b'\n');
        }
        text
    }

    #[test]
    fn reading_places_line_n_character_m_at_column_m_row_n() {
        let source = read(b"ab\n  c\n").expect("a Source File");

        assert_eq!((source.grid().columns(), source.grid().rows()), (256, 256));
        assert_eq!(at(&source, 0, 0), Some("a".to_string()));
        assert_eq!(at(&source, 1, 0), Some("b".to_string()));
        assert_eq!(at(&source, 2, 1), Some("c".to_string()));
        assert_eq!(at(&source, 0, 1), None);
    }

    #[test]
    fn reading_treats_short_and_missing_lines_as_empty_cells() {
        let source = read(b".\n\n...").expect("a Source File");

        assert_eq!(
            at(&source, 1, 0),
            None,
            "a short line pads with empty Cells"
        );
        assert_eq!(at(&source, 0, 1), None, "an empty line is empty Cells");
        assert_eq!(at(&source, 2, 2), Some(".".to_string()));
        assert_eq!(at(&source, 0, 3), None, "a missing line is empty Cells");
        assert_eq!(at(&source, 255, 255), None);
    }

    #[test]
    fn reading_the_empty_text_is_the_empty_source() {
        for text in [&b""[..], b"\n"] {
            let source = read(text).expect("a Source File");
            assert!(source.snapshot().bytes().all(|byte| byte == b' '));
        }
    }

    #[test]
    fn reading_accepts_crlf_as_it_accepts_lf() {
        let crlf = read(b".+0102\r\n\r\n  0C\r\n").expect("a CRLF Source File");
        let lf = read(b".+0102\n\n  0C\n").expect("an LF Source File");

        assert_eq!(crlf.snapshot(), lf.snapshot());
        assert_eq!(at(&crlf, 6, 0), None, "the CR is a line ending, not a Cell");
    }

    #[test]
    fn a_line_without_a_final_terminator_is_read() {
        let source = read(b"a\nb").expect("a Source File");
        assert_eq!(at(&source, 0, 1), Some("b".to_string()));
    }

    #[test]
    fn reading_accepts_256_lines_of_256_characters() {
        let text = lines(ROW_COUNT, &row_of(COL_COUNT, b'x'));
        let source = read(&text).expect("the whole Grid is a Source File");

        assert!(source.snapshot().bytes().all(|byte| byte == b'x'));
        assert_eq!(at(&source, 255, 255), Some("x".to_string()));

        let unterminated = &text[..text.len() - 1];
        assert!(read(unterminated).is_ok(), "the last line's LF is optional");
    }

    #[test]
    fn reading_refuses_a_257th_line() {
        let refusal = refused(&lines(ROW_COUNT + 1, b"."));
        assert_eq!(
            (refusal.line(), refusal.column(), refusal.refusal()),
            (257, 1, Refusal::TooManyLines)
        );

        // An empty 257th line is still a 257th line.
        let mut text = lines(ROW_COUNT, b".");
        text.push(b'\n');
        assert_eq!(refused(&text).refusal(), Refusal::TooManyLines);
    }

    #[test]
    fn reading_refuses_a_257th_character() {
        let mut text = lines(3, b".");
        text.extend_from_slice(&row_of(COL_COUNT + 1, b'y'));
        let refusal = refused(&text);

        assert_eq!(
            (refusal.line(), refusal.column(), refusal.refusal()),
            (4, 257, Refusal::LineTooLong)
        );
        // Trailing spaces count: a line is its characters, not its content.
        let spaces = [&b"."[..], &row_of(COL_COUNT, b' ')].concat();
        assert_eq!(refused(&spaces).refusal(), Refusal::LineTooLong);
        // A CRLF ending is not a character of the line it ends.
        let crlf = [&row_of(COL_COUNT, b'z')[..], b"\r\n"].concat();
        assert!(read(&crlf).is_ok());
    }

    #[test]
    fn reading_refuses_every_byte_that_is_not_printable_ascii_or_space() {
        for byte in u8::MIN..=u8::MAX {
            if byte == b'\n' {
                continue;
            }
            let accepted = byte == b' ' || byte.is_ascii_graphic();
            let text = [b'a', b'\n', b'b', byte, b'c'];

            match read(&text) {
                Ok(source) => {
                    assert!(accepted, "byte 0x{byte:02X} was read as a Cell");
                    assert_eq!(
                        at(&source, 1, 1),
                        (byte != b' ').then(|| char::from(byte).to_string())
                    );
                }
                Err(refusal) => {
                    assert!(!accepted, "byte 0x{byte:02X} was refused");
                    assert_eq!(
                        (refusal.line(), refusal.column(), refusal.refusal()),
                        (2, 2, Refusal::NotACell(byte))
                    );
                }
            }
        }
    }

    #[test]
    fn reading_refuses_each_class_of_byte_that_is_not_a_cell() {
        // A tab, a lone CR, a control byte, DEL, and a UTF-8 character: each
        // refused at the column it sits in, the first refusal in the text.
        for (text, column, byte) in [
            (&b"ab\tc"[..], 3, b'\t'),
            (b"ab\rc", 3, b'\r'),
            (b"\x00", 1, 0x00),
            (b"abc\x7f", 4, 0x7f),
            ("ab\u{e9}".as_bytes(), 3, 0xc3),
            (b"a\x1b\t", 2, 0x1b),
        ] {
            let refusal = refused(text);
            assert_eq!(
                (refusal.line(), refusal.column(), refusal.refusal()),
                (1, column, Refusal::NotACell(byte)),
                "{:?}",
                String::from_utf8_lossy(text)
            );
        }
    }

    #[test]
    fn a_refusal_names_its_line_and_column() {
        assert_eq!(
            refused(b"..\n.\t").to_string(),
            "line 2, column 2: a tab is not a Cell; use spaces"
        );
        assert_eq!(
            refused(b"\x01").to_string(),
            "line 1, column 1: byte 0x01 is not printable ASCII or space"
        );
    }

    #[test]
    fn writing_ends_every_row_up_to_the_last_with_content_and_trims_trailing_spaces() {
        let grid = Grid::new();
        let mut source = Source::new(grid);
        source.set(grid.cell_index(1).unwrap(), "a").unwrap();
        source.set(grid.cell_index(3).unwrap(), "b").unwrap();
        source
            .set(grid.cell_index(3 * COL_COUNT + 255).unwrap(), "z")
            .unwrap();

        let text = write(&source);
        assert_eq!(text, format!(" a b\n\n\n{}z\n", " ".repeat(255)));
        assert!(!text.contains('\r'));
    }

    #[test]
    fn writing_the_empty_source_is_the_empty_text() {
        assert_eq!(write(&Source::new(Grid::new())), "");
    }

    #[test]
    fn writing_the_last_row_writes_256_lines() {
        let grid = Grid::new();
        let mut source = Source::new(grid);
        source
            .set(grid.cell_index(grid.count() - 1).unwrap(), "!")
            .unwrap();

        let text = write(&source);
        assert_eq!(text.lines().count(), ROW_COUNT);
        assert_eq!(read(text.as_bytes()).unwrap().snapshot(), source.snapshot());
    }
}

///
/// The round trip `write` then `read`, over any Source of the one Grid.
///
/// The `cfg` matches the `[target.'cfg(not(target_arch = "wasm32"))'.dev-dependencies]`
/// table that declares proptest, so a WASM build never sees the dependency.
///
#[cfg(all(test, not(target_arch = "wasm32")))]
mod property {
    use super::{read, write};
    use crate::grid::{COL_COUNT, Grid, ROW_COUNT};
    use crate::source::{CellContent, CellWrite, Source};
    use proptest::prelude::*;

    /// Any Cell content, space included: a written space is an empty Cell.
    fn content() -> impl Strategy<Value = u8> {
        prop_oneof![
            4 => 0x21u8..=0x7e,
            1 => Just(b' '),
        ]
    }

    /// Where a write lands. The edges carry their own weight: the last column
    /// is where a trimmed line ends at full length and the last row is where
    /// the written text reaches 256 lines.
    fn cell() -> impl Strategy<Value = (usize, usize)> {
        prop_oneof![
            4 => (0..COL_COUNT, 0..ROW_COUNT),
            1 => (Just(COL_COUNT - 1), 0..ROW_COUNT),
            1 => (0..COL_COUNT, Just(ROW_COUNT - 1)),
            1 => (0..COL_COUNT, Just(0usize)),
        ]
    }

    /// The Cells a Source of the one Grid is written with: a few, or many,
    /// or every Cell of one row. Drawn as coordinates rather than as a
    /// `Source`, which has no `Debug` for proptest to report a failure with.
    fn writes() -> impl Strategy<Value = Vec<((usize, usize), u8)>> {
        prop_oneof![
            prop::collection::vec((cell(), content()), 0..64),
            prop::collection::vec((cell(), content()), 0..4096),
            (0..ROW_COUNT, prop::collection::vec(content(), COL_COUNT)).prop_map(|(y, row)| {
                row.into_iter()
                    .enumerate()
                    .map(|(x, byte)| ((x, y), byte))
                    .collect()
            }),
        ]
    }

    /// The Source `writes` states, on a new Grid of the one shape.
    fn source_of(writes: &[((usize, usize), u8)]) -> Source {
        let grid = Grid::new();
        let mut source = Source::new(grid);
        let writes = writes
            .iter()
            .map(|&((x, y), byte)| CellWrite {
                cell: grid.index(grid.position(x, y).expect("inside the Grid")),
                content: CellContent::new(byte).expect("printable ASCII"),
            })
            .collect::<Vec<_>>();
        source.write_cells(&writes);
        source
    }

    proptest! {
        ///
        /// Every Source reads back from the text it writes with the same
        /// Cells, on the same shape.
        ///
        #[test]
        fn every_source_round_trips_through_write_then_read(writes in writes()) {
            let source = source_of(&writes);
            let text = write(&source);
            let read = read(text.as_bytes());

            prop_assert!(read.is_ok(), "{:?}", read.as_ref().err());
            let read = read.unwrap();
            prop_assert_eq!(read.snapshot(), source.snapshot());
            prop_assert_eq!(
                (read.grid().columns(), read.grid().rows()),
                (source.grid().columns(), source.grid().rows())
            );
            // Written text holds no trailing spaces and no trailing empty line.
            prop_assert!(text.lines().all(|line| !line.ends_with(' ')));
            prop_assert!(!text.ends_with("\n\n"));
        }
    }
}
