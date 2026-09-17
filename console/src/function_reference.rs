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
//! | `64..80`  | 4   | Jumps and Halt (this ticket): `&^ &v &< &>`, `*!`      |
//! | `80..96`  | 5   | Directional Bangs and Self-Banging (this ticket): `*^ *v *< *>`, `^^ vv << >>` |
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
//! Bands 4 and 5 each hold two of this ticket's four groups under one header
//! rather than stacking a second `||` Comment inside the band: band 4's
//! header reads `|| Jumps & Halt` and band 5's reads `|| Bang & Move`, each
//! short enough to stay inside its own 16 Cells rather than spilling into
//! the next band, matching every earlier header. A second header stacked
//! lower in a band would claim the rest of *its* row
//! too, which only works when every band east of it is still blank on that
//! exact row — true here for bands 4 and 5's own single header rows (4 and
//! 5), but a constraint worth stating for whoever adds a stacked header
//! later: **never row 6**. Ticket 05 puts MIDI's header there (`96..112`),
//! and a band-4 or band-5 header on row 6 would start further west, reach
//! that row first, and swallow MIDI's header along with everything east of
//! it. Ordinary (non-Comment) content from bands 4 or 5 on row 6 is fine —
//! only a header's whole-row claim reaches that far — and band 5 does in
//! fact use row 6 for the Delay expression of its first Directional Bang
//! example, below.
//!
//! # Movers need an area, not just a row
//!
//! Every other group's example is stable after the one Tick that computes
//! it: an Arithmetic sum does not change on Tick 1 just because Tick 0 already
//! wrote it. A Jump and a Halt are the same — a Jump re-reads and re-writes
//! the same value every Tick, and a locked Halt target simply never runs —
//! so their areas need only the columns their Expression and result occupy.
//! A Directional Bang's emission and a Self-Banging Function's own Span are
//! not stable: ADR 0006 moves them one Cell per Tick until something stops
//! them, so each of those examples reserves a small rectangle — its own
//! area — wide and tall enough for that path, with a blocking Cell placed so
//! the mover always stops inside it. Ticking the reference is this module's
//! test, not a guess: every area and blocker below came from running the
//! actual Tick loop over candidate layouts and reading back what moved,
//! what blocked cleanly, and what diagnosed, rather than from the language
//! rules alone.
//!
//! Two facts made the areas as small as they are:
//!
//! - A blocked mover's Span does not just stop; it becomes `**` for exactly
//!   one Tick and then clears to empty on the Tick after — the same way a
//!   Delay's own result Cell clears on a Tick it does not Bang. A settled
//!   mover's area therefore reads as entirely blank, not as a Bang
//!   permanently on display.
//! - A blocking Cell only needs to be a complete two-Cell Language Unit
//!   (`00` here throughout) placed where the mover's next full Span would
//!   land — immediately adjacent for a vertical mover, and for a horizontal
//!   mover, aligned so the *one* Cell it enters next is the near edge of
//!   that Unit. Both give a clean block with no diagnostic, per ADR 0006's
//!   "complete non-root contact adds no collision diagnostic."
//!
//! Each Directional Bang example's Bang source is a Delay, `~*1001`
//! (cycle 16), rather than the Equality every other group uses. An Equality
//! over two equal literals holds forever, so it Bangs every Tick — and once
//! its emission's destination Cell is no longer empty (occupied by the
//! mover it just wrote, or later by that mover's blocked `**`), every
//! following Tick's re-attempt is refused and diagnoses, forever. A Delay
//! whose cycle outlasts this module's own tick budget Bangs once, at Tick
//! 0, which is enough to emit the mover and never fires again — the same
//! reason `orcvs::source::tick`'s own
//! `an_emitted_self_banging_function_first_moves_on_the_following_tick` test
//! reaches for a Delay instead of an Equality.
//!
//! With those two facts, every mover in both bands is written, blocked, and
//! cleared by the Tick indexed 2 (the third Tick), and nothing in either
//! band's area changes again after that — `an_emitted_self_banging_function_
//! first_moves_on_the_following_tick` is why a Directional Bang's own
//! emission does not move until the Tick after it is written, one Tick later
//! than a Self-Banging Function already sitting in the checked-in text.
//!
//! Rows: the tallest group decides the Grid's row count. Arithmetic's header
//! plus 10 examples of 3 rows each reaches row 30; band 5's header plus its
//! eight movement examples reaches row 31 — still the taller of the two —
//! and both round up to 32, the next multiple of 8, so this ticket does not
//! grow the Grid's row count. Conversion, Sequence, and Tick all start later
//! (rows 1, 2, and 3) but finish well inside that height. Tick's column
//! widens the Grid on its own: its widest row is Random's eight-Cell
//! Expression, `~?010010`, reaching column 55 and rounding the Grid's width
//! up from 48 to 56. This ticket widens it again — band 5's West Directional
//! Bang example's Delay Expression, `~*1001`, reaches column 91 — rounding
//! the Grid's width up from 56 to 96, which is exactly band 5's own upper
//! bound and leaves ticket 05's MIDI band (`96..112`) untouched.
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

    /// A rectangle of Cells, half-open on both axes, matching one example's
    /// own area — the Jump, Directional Bang, Self-Banging, and Halt
    /// examples this ticket adds each own one, per the module doc's
    /// "Movers need an area, not just a row".
    struct Area {
        columns: std::ops::Range<usize>,
        rows: std::ops::Range<usize>,
    }

    impl Area {
        fn contains(&self, x: usize, y: usize) -> bool {
            self.columns.contains(&x) && self.rows.contains(&y)
        }
    }

    /// Every area this ticket's own examples may act inside, band 4 then
    /// band 5, top to bottom within each — the same rectangles the module
    /// doc names and the same ones ticking determined, not a guess.
    fn movement_areas() -> Vec<Area> {
        vec![
            // Band 4 (64..80): Jumps and Halt. Each Jump is stable after
            // Tick 0 (re-reading and re-writing the same input every Tick),
            // and a locked Halt target never runs, but every example still
            // gets its own area for the same reason the one-Tick test
            // excludes this ticket's groups as a whole.
            Area {
                columns: 64..66,
                rows: 5..8,
            }, // &^: output(5), jump(6), input(7)
            Area {
                columns: 64..66,
                rows: 9..12,
            }, // &v: input(9), jump(10), output(11)
            Area {
                columns: 64..70,
                rows: 13..14,
            }, // &<: output, jump, input, one row
            Area {
                columns: 64..70,
                rows: 15..16,
            }, // &>: input, jump, output, one row
            Area {
                columns: 64..72,
                rows: 17..20,
            }, // Halt: Equality, *!, locked root
            // Band 5 (80..96): Directional Bangs and Self-Banging. Each
            // Directional Bang emits a Self-Banging Function that moves
            // until a blocking `00` stops it; each standalone Self-Banging
            // Function does the same without an emission step.
            Area {
                columns: 80..86,
                rows: 6..10,
            }, // *v: Delay, bang result + *v, emission, blocker
            Area {
                columns: 80..88,
                rows: 11..13,
            }, // *>: Delay, bang result + *> + emission + blocker
            Area {
                columns: 80..88,
                rows: 14..17,
            }, // *^: blocker, Delay + target, *^ + bang result
            Area {
                columns: 80..92,
                rows: 18..20,
            }, // *<: Delay, blocker + emission + *< + bang result
            Area {
                columns: 80..85,
                rows: 21..22,
            }, // >>: mover, gap, blocker
            Area {
                columns: 80..85,
                rows: 23..24,
            }, // <<: blocker, gap, mover
            Area {
                columns: 80..82,
                rows: 25..28,
            }, // ^^: blocker, gap, mover
            Area {
                columns: 80..82,
                rows: 29..32,
            }, // vv: mover, gap, blocker
        ]
    }

    /// The Tick group's own six result rows (column 48; ticket 03), which
    /// change every Tick by that group's own design — Clock, Delay, and
    /// Euclidean from the absolute Tick, Increment and Interpolation from
    /// their own previous value — and are therefore excluded from the "no
    /// Cell outside an area changes" assertion below for a reason that has
    /// nothing to do with this ticket's movers.
    fn tick_dynamic_result_areas() -> Vec<Area> {
        vec![
            Area {
                columns: 48..50,
                rows: 5..6,
            },
            Area {
                columns: 48..50,
                rows: 8..9,
            },
            Area {
                columns: 48..50,
                rows: 11..12,
            },
            Area {
                columns: 48..50,
                rows: 14..15,
            },
            Area {
                columns: 48..50,
                rows: 17..18,
            },
            Area {
                columns: 48..50,
                rows: 20..21,
            },
        ]
    }

    /// `area`'s Cells, one `String` per row, each row read west to east with
    /// an unset Cell standing for a space — the same shape
    /// `orcvs::source::tick`'s own `rows_of` test helper reads a whole Grid
    /// as, narrowed to one rectangle.
    fn area_text(source: &super::Source, grid: super::Grid, area: &Area) -> Vec<String> {
        area.rows
            .clone()
            .map(|y| {
                area.columns
                    .clone()
                    .map(|x| {
                        let position = grid.position(x, y).expect("inside the reference Grid");
                        source
                            .get(grid.index(position))
                            .unwrap_or_else(|| " ".to_string())
                    })
                    .collect::<String>()
            })
            .collect()
    }

    ///
    /// Playing the reference is not a one-Tick affair for this ticket's four
    /// groups: a Directional Bang's emission and a Self-Banging Function's
    /// own Span move every Tick until a blocking Cell stops them, per
    /// ADR 0006. This ticks the reference five times — Ticks 0 through 4 —
    /// and asserts two things throughout: no Cell outside an example's own
    /// area ever changes (excepting the Tick group's own dynamic result
    /// rows, unrelated to this ticket), and every mover has stopped moving,
    /// and its Bang display has cleared, by the Tick indexed 2 — after
    /// which nothing in any area changes again either. The module doc's
    /// "Movers need an area, not just a row" section is where those two
    /// Tick counts and the clearing behaviour come from: this module's own
    /// Tick loop, read back, not the language rules alone.
    ///
    #[test]
    fn playing_the_reference_repeatedly_changes_only_each_examples_own_area() {
        let mut source = function_reference();
        let grid = source.grid();

        let read = |source: &super::Source, x: usize, y: usize| -> Option<String> {
            let position = grid.position(x, y).expect("inside the reference Grid");
            source.get(grid.index(position))
        };
        let snapshot = |source: &super::Source| -> Vec<Option<String>> {
            (0..grid.rows())
                .flat_map(|y| (0..grid.columns()).map(move |x| (x, y)))
                .map(|(x, y)| read(source, x, y))
                .collect()
        };
        let index_of = |x: usize, y: usize| y * grid.columns() + x;

        let movement = movement_areas();
        let tick_dynamic = tick_dynamic_result_areas();
        let excluded = |x: usize, y: usize| {
            movement.iter().any(|area| area.contains(x, y))
                || tick_dynamic.iter().any(|area| area.contains(x, y))
        };

        let before = snapshot(&source);
        let mut settled: Option<Vec<Option<String>>> = None;

        for tick in 0..5u64 {
            let plan = source.execute(Tick::new(tick));
            assert!(
                plan.diagnostics.is_empty(),
                "Tick {tick} diagnosed: {:?}",
                plan.diagnostics
            );

            let now = snapshot(&source);
            for y in 0..grid.rows() {
                for x in 0..grid.columns() {
                    if excluded(x, y) {
                        continue;
                    }
                    assert_eq!(
                        now[index_of(x, y)],
                        before[index_of(x, y)],
                        "Cell ({x}, {y}), outside every example's area, changed on Tick {tick}"
                    );
                }
            }

            if tick == 2 {
                // Every mover has been written, blocked, and cleared by
                // now — see the exact per-area content asserted just below —
                // so this is the snapshot later Ticks must not move away
                // from.
                settled = Some(now);

                let expect_area = |area: &Area, expected: &[&str]| {
                    assert_eq!(
                        area_text(&source, grid, area),
                        expected,
                        "area {:?}x{:?} did not read as settled at Tick 2",
                        area.columns,
                        area.rows
                    );
                };

                let areas = movement_areas();
                expect_area(&areas[0], &["05", "&^", "05"]); // &^
                expect_area(&areas[1], &["06", "&v", "06"]); // &v
                expect_area(&areas[2], &["07&<07"]); // &<
                expect_area(&areas[3], &["08&>08"]); // &>
                expect_area(&areas[4], &[".=0909  ", "***!    ", "  .+0304"]); // Halt
                expect_area(&areas[5], &["~*1001", "  *v  ", "      ", "  00  "]); // *v
                expect_area(&areas[6], &["~*1001  ", "  *>  00"]); // *>
                expect_area(&areas[7], &["00      ", "  ~*1001", "*^      "]); // *^
                expect_area(&areas[8], &["      ~*1001", "00  *<      "]); // *<
                expect_area(&areas[9], &["   00"]); // >>
                expect_area(&areas[10], &["00   "]); // <<
                expect_area(&areas[11], &["00", "  ", "  "]); // ^^
                expect_area(&areas[12], &["  ", "  ", "00"]); // vv
            }
        }

        let settled = settled.expect("Tick 2 ran");
        let after = snapshot(&source);
        for area in &movement {
            for y in area.rows.clone() {
                for x in area.columns.clone() {
                    assert_eq!(
                        after[index_of(x, y)],
                        settled[index_of(x, y)],
                        "Cell ({x}, {y}), in a settled area, changed after Tick 2"
                    );
                }
            }
        }
    }
}
