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
//! by a `||` Comment naming it. A Comment claims the rest of its Grid row, so
//! only the first header actually parses as a Comment Language Unit; every
//! header east of it becomes that Comment's own text instead of a Comment of
//! its own. That has no consequence — a Comment is never evaluated, and
//! Source Paint colours the whole row as a Comment either way — provided the
//! row holds nothing else: an Expression east of a header would fall inside
//! that Comment and silently stop being parsed and run. All six headers
//! therefore sit on row 0, and every group's own examples start on row 1. A
//! group that stacks a second header inside its own band follows the same
//! rule: that header's row must hold no Expression east of it either.
//!
//! | Columns   | Group                                                       |
//! |-----------|--------------------------------------------------------------|
//! | `0..16`   | Arithmetic: `.+ .- .| .x ./ .% .< .> .=`                      |
//! | `16..32`  | Numeric Conversion: `.v .^`                                   |
//! | `32..48`  | Sequence: `:- :# :< :& :? :=` (results up to 14 Cells)        |
//! | `48..64`  | Tick: `~. ~* ~% ~+ ~> ~?`                                     |
//! | `64..80`  | Jumps and Halt: `&^ &v &< &>`, `*!`                           |
//! | `80..96`  | Directional Bangs and Self-Banging: `*^ *v *< *>`, `^^ vv << >>` |
//! | `96..112` | MIDI: `!> !~ !% !c !b`                                        |
//!
//! # Source Functions need an area, not just a row
//!
//! Every value-answering group's example is stable after the one Tick that
//! computes it: an Arithmetic sum does not change on Tick 1 just because
//! Tick 0 already wrote it. A Jump and a Halt are the same — a Jump re-reads
//! and re-writes the same value every Tick, and a locked Halt target simply
//! never runs — so their areas need only the columns their Expression and
//! result occupy.
//!
//! A Directional Bang Function's emission and a Self-Banging Function's own
//! Span are not stable: ADR 0006 moves them one Cell per Tick until
//! something stops them. Rather than an ordinary blocking Cell — which turns
//! a stopped mover's Span into `**` for one Tick and then clears it to blank
//! forever, exactly the vanishing this reference should not show while it
//! plays — each mover that can be walled in comes to rest directly south of
//! an already-active Halt Function (CONTEXT.md's Halt entry). Halt locks its
//! target's Turn before it runs, so a Cell it protects is never overwritten
//! at all: the mover's own glyph stands forever once it arrives.
//!
//! Halt only ever locks the root directly south of itself, and that geometry
//! decides which movers can be walled and which cannot:
//!
//! - **North** (`^^`, and the `^^` a Directional Bang North emits) travels
//!   there cleanly. Rows are discrete: the row the mover is about to leave
//!   is either fully occupied or not there yet, never partly one and partly
//!   the other, so an already-active Halt several rows north locks it the
//!   moment it arrives, with room to spare for the Directional Bang's own
//!   one-shot Delay well away from Halt's own neighbourhood.
//! - **West and East**, placed at rest from the outset (`<<`, `>>`), are
//!   walled the same way — Halt is active before the mover ever exists, so
//!   its own driving Equality has the whole neighbourhood to itself. Real
//!   travel cannot do this safely: a horizontal Span slides through the
//!   target one column at a time, so the Tick immediately before it fully
//!   aligns leaves it overlapping only part of Halt's target, and Halt
//!   reports "target is not an Expression root" for that partial match
//!   rather than locking it silently.
//! - **South** (`vv`), also placed at rest, hits a stricter version of the
//!   same limit: reaching a Cell south of Halt from the north means passing
//!   through Halt's own row first, which is impossible without colliding
//!   with Halt outright, in flight or at rest. `vv` is instead placed
//!   immediately south of its own Halt from Tick 0, the one Cell Halt can
//!   reach without ever touching it.
//! - **The Directional Bang South, West, and East Functions cannot be
//!   walled at all**, in flight or at rest. Each needs its own one-shot
//!   Delay touching it to emit exactly once, and that Delay's four-digit
//!   operand always reaches into the same one or two Cells a second,
//!   permanently active Bang source would need in order to keep the
//!   mover's own Halt lit — every placement that avoids one collision runs
//!   into the other. `*v`, `*<`, and `*>` therefore still emit into an
//!   ordinary blocking Cell and clear the Tick after, the same as every
//!   mover did before Halt walled the rest of them in.
//!
//! # MIDI activation needs no area, only a Bang source
//!
//! A Terminal Output Function "never writes a Cell result" (CONTEXT.md's
//! Terminal Output Function entry), so nothing about a `!`-spelled
//! Expression itself ever changes once checked in, and no row is set aside
//! south of it the way a value-answering group sets one aside for its result.
//! What changes is each example's own Bang source: a Euclidean, `~%0104`
//! (the `X...` pattern), Bangs once every four Ticks — slow enough to hear
//! each event distinctly at the default tempo — and clears to blank between
//! Bangs the same way Tick's own Delay and Euclidean examples do.
//!
//! Activation is vertical, the same alignment
//! `orcvs::source::tick::a_select_bang_activates_an_aligned_terminal_root`
//! proves for a Select-forwarded Bang: each Euclidean's Expression, its own
//! Bang-display Cells one row south, and the gated Terminal Output
//! Function's root one row further south all share one column. A blank row
//! then separates one example from the next, so each spans four rows
//! (Euclidean Expression, its Bang display, the gated Function, and a blank
//! row) rather than the three-row cadence a value-answering group uses,
//! since a Terminal Output Function's own row replaces a result row rather
//! than sitting beside one.
//!
//! The Function table's only other Terminal Output Function, Application
//! Command `!$`, is not among these five: ADR 0008 and ADR 0019 defer it
//! until Orcvs settles a command value encoding, so it has no variant in
//! `lang::Function` and nothing to write an example of.
//!
//! # Checked-in text is ragged, not a padded rectangle
//!
//! `function_reference.orcvs` stores each line with its trailing whitespace
//! removed, and stops at the last line with any content — no trailing blank
//! rows. Editors and formatters strip trailing whitespace on save, which
//! would otherwise turn a checked-in padded rectangle ragged on its next
//! untouched edit and fail a loader that demanded one.
//! `source_from_reference_text` instead places the text on the one 256 by 256
//! Grid (ADR 0054), and every short line and every row past the last line
//! reads as empty Cells — see its doc comment.
//!
//! # Completeness and diagnostic-cleanliness are proven, not asserted
//!
//! `every_function_in_the_table_has_a_worked_example` reads
//! `lang::Function::ALL` and asserts every spelling anchors the root of some
//! Expression in the reference's own `LanguageMap` — a Function nested
//! inside another's operand does not count, since a worked example asks for
//! the Function itself to be exercised, not merely referred to.
//!
//! `every_example_expression_parses_without_a_diagnostic_outside_a_result_row`
//! reads the same `LanguageMap`, built directly from the checked-in text
//! before any Tick runs, and asserts every Diagnostic's Span sits wholly
//! inside a written result Cell: a value such as `03` sits at a row start
//! with no Function before it, so the Parser's greedy two-Cell Function read
//! refuses it a Cell at a time (ADR 0018) — a fact about the checked-in
//! placeholder rather than about the example that computes it, and never a
//! Tick-time Diagnostic. Every other Cell, operands and Source Function
//! areas alike, is held to the same standard.
//!
//! Ticking the reference once is not enough on its own to prove a result was
//! actually written rather than merely agreeing with a checked-in
//! placeholder that already held the right answer — a Concatenate example
//! once survived exactly that way, its Turn silently withheld by a static
//! parse fault the Tick Plan never diagnosed, until the diagnostic sweep
//! above caught the fault that had been masking it.
//! `ticking_the_reference_once_writes_every_result_row_exactly_as_written`
//! therefore asserts a matching [`orcvs::source::CellWrite`] in the Tick
//! Plan for every expected result, not only the settled Cell it reads back
//! afterwards.
//!

use orcvs::grid::Grid;
use orcvs::source::Source;

const REFERENCE: &str = include_str!("../assets/function_reference.orcvs");

///
/// The Function reference Source, rebuilt fresh each call.
///
pub(crate) fn function_reference() -> Source {
    source_from_reference_text(REFERENCE)
}

///
/// Places `text` on the one Grid (ADR 0054): line *n* is row *n*, character
/// *m* is column *m*.
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
    let grid = Grid::new();
    let mut source = Source::new(grid);
    for (y, line) in text.lines().enumerate() {
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
    use super::{function_reference, source_from_reference_text};
    use orcvs::grid::{CellIndex, Grid};
    use orcvs::source::{
        BendLsb, BendMsb, CellContent, CellWrite, ControlValue, Controller, LanguageUnitKind,
        Length, MidiChannel, Note, PlayCommand, Source, Tick, Velocity,
    };

    ///
    /// A checked-in, padded rectangle is not the only shape the loader must
    /// accept: an editor or formatter that strips trailing whitespace turns
    /// every padded row short of the widest into a ragged one, and a blank
    /// row into an empty line. The loader places the text on the one 256 by
    /// 256 Grid instead of asserting a rectangle, so this ragged text — never
    /// checked in, built by the test itself — loads rather than panics.
    ///
    #[test]
    fn a_ragged_text_with_stripped_trailing_whitespace_loads_onto_the_one_grid() {
        // Three lines, none the same length: 6 Cells, 0 (a blank line
        // stripped bare), and 2. The Grid is the one shape regardless.
        let source = source_from_reference_text(".+0102\n\n0C");
        let grid = source.grid();

        assert_eq!(grid.columns(), 256);
        assert_eq!(grid.rows(), 256);

        assert_eq!(read_cell(&source, grid, 0, 0), Some(".".to_string()));
        assert_eq!(read_cell(&source, grid, 5, 0), Some("2".to_string()));
        assert_eq!(
            read_cell(&source, grid, 6, 0),
            None,
            "a short line pads with empty Cells rather than reaching into the next row"
        );
        assert_eq!(
            read_cell(&source, grid, 0, 1),
            None,
            "a blank line stays entirely empty"
        );
        assert_eq!(read_cell(&source, grid, 0, 2), Some("0".to_string()));
        assert_eq!(read_cell(&source, grid, 1, 2), Some("C".to_string()));
        assert_eq!(
            read_cell(&source, grid, 0, 7),
            None,
            "a row past the last line is empty padding, not an error"
        );
    }

    /// One Cell of `source` at `(x, y)`, read back through `grid` — the one
    /// read every test below shares rather than each writing its own copy.
    fn read_cell(source: &Source, grid: Grid, x: usize, y: usize) -> Option<String> {
        let position = grid.position(x, y).expect("inside the reference Grid");
        source.get(grid.index(position))
    }

    /// The [`CellIndex`] `grid` mints for `(x, y)`.
    fn cell_index(grid: Grid, x: usize, y: usize) -> CellIndex {
        let position = grid.position(x, y).expect("inside the reference Grid");
        grid.index(position)
    }

    /// One example's expected written result: the Cells starting at
    /// `(column, row)`, read west to east, and the string one Tick writes
    /// there — or a run of spaces when the example writes nothing there.
    struct ExpectedResult {
        column: usize,
        row: usize,
        expected: &'static str,
    }

    /// Every result row the Arithmetic, Conversion, Sequence, and Tick
    /// groups' examples write, where `column` is each group's own anchor
    /// column (Arithmetic 0, Conversion 16, Sequence 32, Tick 48) since each
    /// group's result rows are read back from its own Cells, not always the
    /// leftmost ones.
    ///
    /// Shared by [`ticking_the_reference_once_writes_every_result_row_exactly_as_written`],
    /// which proves these are actually written, and
    /// `every_example_expression_parses_without_a_diagnostic_outside_a_result_row`,
    /// which excludes them from the pre-Tick diagnostic sweep: a written
    /// result such as `03` sits at a row start with no Function before it,
    /// so the Parser's greedy two-Cell Function read refuses it (ADR 0018)
    /// and diagnoses every one of its Cells, a fact about the checked-in
    /// placeholder rather than about the example that computes it.
    fn expected_results() -> Vec<ExpectedResult> {
        let result = |column, row, expected| ExpectedResult {
            column,
            row,
            expected,
        };
        vec![
            // Arithmetic (column 0)
            result(0, 2, "03"),  // .+0102
            result(0, 5, "02"),  // .-0503
            result(0, 8, "04"),  // .|0307
            result(0, 11, "0C"), // .x0304
            result(0, 14, "04"), // ./0902
            result(0, 17, "01"), // .%0902
            result(0, 20, "03"), // .<0305
            result(0, 23, "05"), // .>0305
            result(0, 26, "**"), // .=0505 (equal)
            result(0, 29, "  "), // .=0506 (not equal: no Cell write)
            // Conversion (column 16)
            result(16, 2, "C4"), // .^3C (Number to Note)
            result(16, 5, "3C"), // .vC4 (Note to Number)
            // Sequence (column 32)
            result(32, 2, "01020304"), // :-0104 (Number Range)
            result(32, 5, "C4c4D4"),   // :#C4D4 (Note Range)
            result(32, 8, "04030201"), // :<:-0104 (Reverse, over a nested Function operand)
            result(32, 11, "010203"), // :&.+0001:-0203 (Concatenate, over a nested Function's Atom answer)
            result(32, 14, "01"),     // :?00:-0103 (Select)
            result(32, 17, "010303"), // :=01.+0102:-0103 (Replace, over nested Functions)
            result(32, 20, "111213"), // .+10:-0103 (Add, pervasive over a Sequence)
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
            result(48, 2, "00"), // ~.0204 (Clock: step 0 of a 2-Tick, 4-step cycle)
            result(48, 5, "**"), // ~*0302 (Delay: every cycle Bangs at Tick 0)
            result(48, 8, "**"), // ~%0308 (Euclidean: 03-08's pattern Bangs its first step)
            result(48, 11, "00"), // ~+0104 (Increment: previous 03 -> (03+01)%04)
            result(48, 14, "02"), // ~>0210 (Interpolation: previous 00 -> steps by 02 toward 10)
            result(48, 17, "07"), // ~?010010 (Random: seed 01 at this Function's own Grid Position, Tick 0)
        ]
    }

    /// Every Cell a Jump reads as its spatial input, or an ordinary blocking
    /// Language Unit occupies, in the checked-in text: present with no
    /// Function before it, so the Parser's greedy two-Cell Function read
    /// refuses it exactly as it refuses a written result — a fact about the
    /// raw spatial value it carries, never about the Jump or the mover whose
    /// Turn reads it. Named explicitly, the same way [`expected_results`]
    /// is, rather than by a blanket area: a mistyped operand elsewhere in a
    /// Source Function's own Expression is still expected to diagnose.
    fn spatial_literal_cells() -> Vec<ExpectedResult> {
        let literal = |column, row, expected| ExpectedResult {
            column,
            row,
            expected,
        };
        vec![
            literal(64, 3, "05"),  // &^'s input
            literal(64, 5, "06"),  // &v's input
            literal(68, 9, "07"),  // &<'s input
            literal(64, 11, "08"), // &>'s input
            literal(82, 10, "00"), // *v's blocker
            literal(80, 13, "00"), // *<'s blocker
            literal(86, 16, "00"), // *>'s blocker
        ]
    }

    #[test]
    fn ticking_the_reference_once_writes_every_result_row_exactly_as_written() {
        let mut source = function_reference();
        let grid = source.grid();

        let plan = source.execute(Tick::ZERO);
        assert!(
            plan.diagnostics.is_empty(),
            "every included Expression must parse and schedule cleanly \
             (this alone does not prove any Turn ran): {:?}",
            plan.diagnostics
        );

        for result in expected_results() {
            let ExpectedResult {
                column,
                row,
                expected,
            } = result;
            let width = expected.chars().count();
            let cells: Vec<CellIndex> = (0..width)
                .map(|offset| cell_index(grid, column + offset, row))
                .collect();

            if expected.trim().is_empty() {
                for &cell in &cells {
                    assert!(
                        !plan.writes.iter().any(|write| write.cell == cell),
                        "row {row}, column {column} was written despite expecting no result"
                    );
                }
            } else {
                for (offset, ch) in expected.chars().enumerate() {
                    let content =
                        CellContent::new(ch as u8).expect("a printable ASCII result Cell");
                    let write = CellWrite {
                        cell: cells[offset],
                        content,
                    };
                    assert!(
                        plan.writes.contains(&write),
                        "row {row}, column {column} was not written by this Tick, \
                         though the checked-in text reads {expected:?} — a Turn withheld \
                         by a fault the Tick Plan never diagnosed would pass a read-back \
                         check alone"
                    );
                }
            }

            let actual: String = cells
                .iter()
                .map(|&cell| source.get(cell).unwrap_or_else(|| " ".to_string()))
                .collect();
            assert_eq!(
                actual, expected,
                "row {row}, column {column} did not read as written"
            );
        }
    }

    #[test]
    fn the_reference_loads_onto_the_one_grid_rather_than_one_rounded_up_from_its_text() {
        let grid = function_reference().grid();

        assert_eq!((grid.columns(), grid.rows()), (256, 256));
    }

    /// A rectangle of Cells, half-open on both axes, matching one example's
    /// own area.
    #[derive(Clone)]
    struct Area {
        columns: std::ops::Range<usize>,
        rows: std::ops::Range<usize>,
    }

    impl Area {
        fn contains(&self, x: usize, y: usize) -> bool {
            self.columns.contains(&x) && self.rows.contains(&y)
        }
    }

    /// One resting Cell a settled mover's own glyph must still occupy: the
    /// two Cells `expected` names, at `(column, row)`.
    struct RestingGlyph {
        column: usize,
        row: usize,
        expected: &'static str,
    }

    /// Every area a Jump, Halt, Directional Bang, or Self-Banging example
    /// may act inside — the Cells its own Expression, emission, or Halt
    /// wall occupies — named rather than indexed positionally.
    struct SourceFunctionAreas {
        jump_north: Area,
        jump_south: Area,
        jump_west: Area,
        jump_east: Area,
        halt: Area,
        directional_north: Area,
        directional_south: Area,
        directional_west: Area,
        directional_east: Area,
        self_banging_north: Area,
        self_banging_south: Area,
        self_banging_west: Area,
        self_banging_east: Area,
    }

    impl SourceFunctionAreas {
        fn new() -> Self {
            Self {
                // Band 4 (64..80): Jumps and Halt. Each Jump is stable after
                // Tick 0 (re-reading and re-writing the same input every
                // Tick), and a locked Halt target never runs, but every
                // example still gets its own area for the same reason the
                // one-Tick test excludes this group as a whole.
                jump_north: Area {
                    columns: 64..66,
                    rows: 1..4,
                }, // &^: output, jump, input
                jump_south: Area {
                    columns: 64..66,
                    rows: 5..8,
                }, // &v: input, jump, output
                jump_west: Area {
                    columns: 64..70,
                    rows: 9..10,
                }, // &<: output, jump, input, one row
                jump_east: Area {
                    columns: 64..70,
                    rows: 11..12,
                }, // &>: input, jump, output, one row
                halt: Area {
                    columns: 64..72,
                    rows: 13..16,
                }, // Halt: Equality, *!, locked root
                // Band 5 (80..96): Directional Bangs and Self-Banging.
                directional_north: Area {
                    columns: 80..90,
                    rows: 1..6,
                }, // *^: Equality, Halt, target, Delay, *^
                directional_south: Area {
                    columns: 80..86,
                    rows: 7..11,
                }, // *v: Delay, *v, emission, blocker (cannot be walled — see module doc)
                directional_west: Area {
                    columns: 80..92,
                    rows: 12..14,
                }, // *<: blocker, *<, emission, Delay (cannot be walled)
                directional_east: Area {
                    columns: 80..88,
                    rows: 15..17,
                }, // *>: Delay, *>, emission, blocker (cannot be walled)
                self_banging_north: Area {
                    columns: 80..86,
                    rows: 18..23,
                }, // ^^: Equality, Halt, target, travel, start
                self_banging_south: Area {
                    columns: 80..86,
                    rows: 24..27,
                }, // vv: Equality, Halt, vv (at rest from Tick 0)
                self_banging_west: Area {
                    columns: 80..86,
                    rows: 28..31,
                }, // <<: Equality, Halt, << (at rest from Tick 0)
                self_banging_east: Area {
                    columns: 80..86,
                    rows: 32..35,
                }, // >>: Equality, Halt, >> (at rest from Tick 0)
            }
        }

        fn all(&self) -> [&Area; 13] {
            [
                &self.jump_north,
                &self.jump_south,
                &self.jump_west,
                &self.jump_east,
                &self.halt,
                &self.directional_north,
                &self.directional_south,
                &self.directional_west,
                &self.directional_east,
                &self.self_banging_north,
                &self.self_banging_south,
                &self.self_banging_west,
                &self.self_banging_east,
            ]
        }
    }

    /// The five movers Halt actually walls in, and the glyph each must still
    /// show at its own resting Cell once settled: the Directional Bang
    /// North's emitted `^^`, the standalone `^^`, and the three Self-Banging
    /// Functions placed at rest from Tick 0. `*v`, `*<`, and `*>`'s emitted
    /// movers are not among these — the module doc's "Source Functions need
    /// an area, not just a row" section is why Halt cannot reach them.
    fn walled_resting_glyphs() -> Vec<RestingGlyph> {
        let glyph = |column, row, expected| RestingGlyph {
            column,
            row,
            expected,
        };
        vec![
            glyph(82, 3, "^^"),  // *^'s emitted ^^, settled south of its Halt
            glyph(82, 20, "^^"), // standalone ^^, settled south of its Halt
            glyph(82, 26, "vv"), // standalone vv, at rest from Tick 0
            glyph(82, 30, "<<"), // standalone <<, at rest from Tick 0
            glyph(82, 34, ">>"), // standalone >>, at rest from Tick 0
        ]
    }

    /// The Tick group's own six dynamic result Cells (column 48), derived
    /// from [`expected_results`] rather than a second hand-kept coordinate
    /// list: Clock, Delay, and Euclidean change with the absolute Tick,
    /// Increment and Interpolation with their own previous value, so all six
    /// change every Tick by that group's own design and are excluded from
    /// the "no Cell outside an area changes" sweep below for a reason that
    /// has nothing to do with Source Functions.
    fn tick_dynamic_result_areas() -> Vec<Area> {
        expected_results()
            .into_iter()
            .filter(|result| result.column == 48)
            .map(|result| Area {
                columns: result.column..result.column + 2,
                rows: result.row..result.row + 1,
            })
            .collect()
    }

    /// The column the MIDI group's band starts at.
    const MIDI_BAND_COLUMN: usize = 96;

    /// The MIDI group's own five Bang-display Cells, one pair per Terminal
    /// Output example — column `96..98`, one row south of that example's
    /// Euclidean root, exactly the default output Portal every Value
    /// Function that can emit Bang uses. `~% 01 04` Bangs on the "X..."
    /// pattern (`euclidean_places_its_hits_where_the_adr_formula_does`
    /// proves this pattern from the same `~%` formula this module's own
    /// Euclidean examples use), so each of these Cells reads `**` on every
    /// fourth Tick starting at Tick 0 and clears to blank on the three Ticks
    /// between — the same clearing behaviour Tick's own Delay and Euclidean
    /// examples already rely on — which is why these five Cells, and no
    /// others in the MIDI group, are excluded from "no Cell outside an
    /// example's own area changes" below. Every Terminal Output Function's
    /// own Expression is stable forever after being written once: it
    /// performs an effect and never receives a Cell write of its own.
    fn midi_bang_result_areas() -> Vec<Area> {
        [2, 6, 10, 14, 18]
            .into_iter()
            .map(|row| Area {
                columns: MIDI_BAND_COLUMN..MIDI_BAND_COLUMN + 2,
                rows: row..(row + 1),
            })
            .collect()
    }

    /// `area`'s Cells, one `String` per row, each row read west to east with
    /// an unset Cell standing for a space.
    fn area_text(source: &Source, grid: Grid, area: &Area) -> Vec<String> {
        area.rows
            .clone()
            .map(|y| {
                area.columns
                    .clone()
                    .map(|x| read_cell(source, grid, x, y).unwrap_or_else(|| " ".to_string()))
                    .collect::<String>()
            })
            .collect()
    }

    ///
    /// Playing the reference is not a one-Tick affair for the Jump, Halt,
    /// Directional Bang, and Self-Banging examples: a Directional Bang's
    /// emission and a Self-Banging Function's own Span move every Tick until
    /// something stops them, per ADR 0006. This ticks the reference five
    /// times — Ticks 0 through 4 — and asserts three things: no Cell outside
    /// an example's own area ever changes (excepting the Tick group's own
    /// dynamic result rows and the MIDI group's own Bang-display Cells,
    /// unrelated to Source Functions); every mover Halt can wall in still
    /// shows its own glyph at rest by the Tick indexed 2; and nothing in any
    /// Source Function area changes again after that.
    ///
    #[test]
    fn playing_the_reference_repeatedly_changes_only_each_examples_own_area() {
        let mut source = function_reference();
        let grid = source.grid();

        let source_functions = SourceFunctionAreas::new();
        let tick_dynamic = tick_dynamic_result_areas();
        let midi_bang = midi_bang_result_areas();
        let excluded = |x: usize, y: usize| {
            source_functions
                .all()
                .iter()
                .any(|area| area.contains(x, y))
                || tick_dynamic.iter().any(|area| area.contains(x, y))
                || midi_bang.iter().any(|area| area.contains(x, y))
        };

        let snapshot = |source: &Source| -> Vec<Option<String>> {
            (0..grid.rows())
                .flat_map(|y| (0..grid.columns()).map(move |x| (x, y)))
                .map(|(x, y)| read_cell(source, grid, x, y))
                .collect()
        };
        let index_of = |x: usize, y: usize| y * grid.columns() + x;

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
                // Every mover Halt can wall in has been locked in place by
                // now, and every mover it cannot has been written, blocked,
                // and cleared — see the module doc's "Source Functions need
                // an area, not just a row" — so this is the snapshot later
                // Ticks must not move away from.
                settled = Some(now);

                for glyph in walled_resting_glyphs() {
                    let width = glyph.expected.chars().count();
                    let actual: String = (0..width)
                        .map(|offset| {
                            read_cell(&source, grid, glyph.column + offset, glyph.row)
                                .unwrap_or_else(|| " ".to_string())
                        })
                        .collect();
                    assert_eq!(
                        actual, glyph.expected,
                        "the mover settled at ({}, {}) did not keep its own glyph on display",
                        glyph.column, glyph.row
                    );
                }

                let expect_area = |area: &Area, expected: &[&str]| {
                    assert_eq!(
                        area_text(&source, grid, area),
                        expected,
                        "area {:?}x{:?} did not read as settled at Tick 2",
                        area.columns,
                        area.rows
                    );
                };
                expect_area(&source_functions.jump_north, &["05", "&^", "05"]);
                expect_area(&source_functions.jump_south, &["06", "&v", "06"]);
                expect_area(&source_functions.jump_west, &["07&<07"]);
                expect_area(&source_functions.jump_east, &["08&>08"]);
                expect_area(
                    &source_functions.halt,
                    &[".=0909  ", "***!    ", "  .+0304"],
                );
            }
        }

        let settled = settled.expect("Tick 2 ran");
        let after = snapshot(&source);
        for area in source_functions.all() {
            for y in area.rows.clone() {
                for x in area.columns.clone() {
                    assert_eq!(
                        after[index_of(x, y)],
                        settled[index_of(x, y)],
                        "Cell ({x}, {y}), in a settled Source Function area, changed after Tick 2"
                    );
                }
            }
        }
    }

    /// The five expected [`PlayCommand`]s the MIDI examples emit when their
    /// own Euclidean Bang source Bangs, in the group's own row order (Raw
    /// Play, Timed Play, Monophonic Play, Control Change, Pitch Bend) — the
    /// operands `console/assets/function_reference.orcvs` checks in for
    /// each, read as the domain types the Function table binds them to.
    /// Channel, velocity, note, controller, value, and bend operands were
    /// chosen to be plausible MIDI content (a mid-velocity Middle C on
    /// channel `00`, a Volume Control Change (`07`), a Pitch Bend above
    /// centre (`40`)) rather than boundary values, since this is a worked
    /// example and not a domain-boundary test — those already live in
    /// `lang`. Timed Play's and Monophonic Play's lengths (`04` Ticks each)
    /// match their own Euclidean's four-Tick period, so neither note is
    /// asked to sound again before the one before it has ended.
    fn expected_midi_commands() -> [PlayCommand; 5] {
        [
            // !>0064C4 — Raw Play: channel 00, velocity 64 (100), note C4.
            PlayCommand::Raw {
                channel: MidiChannel::try_from(0x00).unwrap(),
                velocity: Velocity::try_from(0x64).unwrap(),
                note: Note::try_from(0x3C).unwrap(),
            },
            // !~0064E404 — Timed Play: channel 00, velocity 64, note E4,
            // length 04 Ticks.
            PlayCommand::Timed {
                channel: MidiChannel::try_from(0x00).unwrap(),
                velocity: Velocity::try_from(0x64).unwrap(),
                note: Note::try_from(0x40).unwrap(),
                length: Length::from(0x04),
            },
            // !%0164G404 — Monophonic Play: channel 01, velocity 64, note G4,
            // length 04 Ticks.
            PlayCommand::Mono {
                channel: MidiChannel::try_from(0x01).unwrap(),
                velocity: Velocity::try_from(0x64).unwrap(),
                note: Note::try_from(0x43).unwrap(),
                length: Length::from(0x04),
            },
            // !c000764 — Control Change: channel 00, controller 07 (Volume),
            // value 64.
            PlayCommand::ControlChange {
                channel: MidiChannel::try_from(0x00).unwrap(),
                controller: Controller::try_from(0x07).unwrap(),
                value: ControlValue::try_from(0x64).unwrap(),
            },
            // !b000050 — Pitch Bend: channel 00, lsb 00, msb 50 (above
            // centre).
            PlayCommand::PitchBend {
                channel: MidiChannel::try_from(0x00).unwrap(),
                lsb: BendLsb::try_from(0x00).unwrap(),
                msb: BendMsb::try_from(0x50).unwrap(),
            },
        ]
    }

    ///
    /// The MIDI group: five Terminal Output Functions (`!>`, `!~`, `!%`,
    /// `!c`, `!b` — every Terminal Output Function the Function table
    /// holds; `!$` Application Command is not among them, since ADR 0008
    /// and ADR 0019 defer it until Orcvs has a command value encoding, so it
    /// has no variant in `lang::Function` to give an example of), each
    /// gated by its own Euclidean Bang source, `~%0104`.
    ///
    /// Every Terminal Output Function "never writes a Cell result"
    /// (CONTEXT.md's Terminal Output Function entry), so no row is set aside
    /// south of any of these five Expressions — unlike every value-answering
    /// group's own result row — and the only Cells that ever change while
    /// ticking this group are the Euclidean sources' own Bang-display Cells,
    /// [`midi_bang_result_areas`], which this test names explicitly and
    /// excludes rather than asserting fixed content for, since each toggles
    /// between `**` and blank forever rather than settling.
    ///
    /// This ticks the reference across two full Euclidean cycles (Ticks `0`
    /// through `8`) and asserts, at every Tick: no diagnostic; no
    /// [`orcvs::source::CellWrite`] whose Position falls inside the MIDI
    /// band outside the five Bang-display Cells; and no Cell inside the
    /// MIDI band changes except those same five Cells. Across the whole run
    /// it also asserts every one of the five expected [`PlayCommand`]s was
    /// emitted on at least one Tick — proof the group plays, not merely
    /// that it declines to write.
    ///
    #[test]
    fn ticking_the_midi_group_writes_no_cell_and_emits_every_terminal_output_function() {
        let mut source = function_reference();
        let grid = source.grid();

        let midi_columns = MIDI_BAND_COLUMN..grid.columns();
        let midi_rows = 0..grid.rows();
        let midi_coordinates: Vec<(usize, usize)> = midi_rows
            .clone()
            .flat_map(|y| midi_columns.clone().map(move |x| (x, y)))
            .collect();
        let bang_cells = midi_bang_result_areas();
        let excluded = |x: usize, y: usize| bang_cells.iter().any(|area| area.contains(x, y));

        let before: Vec<Option<String>> = midi_coordinates
            .iter()
            .map(|&(x, y)| read_cell(&source, grid, x, y))
            .collect();

        let expected = expected_midi_commands();
        let mut emitted = [false; 5];

        for tick in 0..9u64 {
            let plan = source.execute(Tick::new(tick));
            assert!(
                plan.diagnostics.is_empty(),
                "Tick {tick} diagnosed: {:?}",
                plan.diagnostics
            );

            for write in &plan.writes {
                let position = grid.position_at(write.cell);
                let in_midi_band = position.x() >= MIDI_BAND_COLUMN;
                let is_bang_display = bang_cells
                    .iter()
                    .any(|area| area.contains(position.x(), position.y()));
                assert!(
                    !in_midi_band || is_bang_display,
                    "Tick {tick} wrote Cell {position:?} inside the MIDI band \
                     but outside a Euclidean Bang-display Cell — no Terminal \
                     Output Function ever writes a Cell"
                );
            }

            for (index, command) in expected.iter().enumerate() {
                if plan.play_commands.contains(command) {
                    emitted[index] = true;
                }
            }

            let now: Vec<Option<String>> = midi_coordinates
                .iter()
                .map(|&(x, y)| read_cell(&source, grid, x, y))
                .collect();
            for (offset, &(x, y)) in midi_coordinates.iter().enumerate() {
                if excluded(x, y) {
                    continue;
                }
                assert_eq!(
                    now[offset], before[offset],
                    "Cell ({x}, {y}), in the MIDI band but outside a Euclidean \
                     Bang-display Cell, changed on Tick {tick}"
                );
            }
        }

        assert!(
            emitted.iter().all(|&fired| fired),
            "not every MIDI example fired within nine Ticks: {emitted:?}"
        );
    }

    ///
    /// Every Function in `lang::Function::ALL` — the table's own
    /// enumeration, generated by `define_functions!` beside the enum itself
    /// (`lang/src/atom.rs`) — anchors the root of at least one Expression in
    /// the reference's own [`orcvs::source::LanguageMap`]: a Function nested
    /// inside another's operand is not a worked example of that Function on
    /// its own, so only an [`orcvs::source::ExpressionEntry::root`] counts.
    /// Deriving the list from the table itself, rather than a hand-written
    /// one, is the point: a Function added to `define_functions!` without a
    /// worked example fails this test instead of a checklist nobody
    /// remembered to update.
    ///
    /// A Function's spelling is read through its existing
    /// `impl std::fmt::Display for Function` (`f.write_str(self.spelling())`,
    /// `lang/src/atom.rs`) rather than a new accessor: `Function::spelling`
    /// is `pub(crate)` to `lang`, but `Function` and its `Display` impl are
    /// already public, so nothing new is exposed to answer this — no
    /// public-API risk.
    ///
    #[test]
    fn every_function_in_the_table_has_a_worked_example() {
        let source = function_reference();
        let map = source.language_map();

        let function_at_anchor: std::collections::HashMap<(usize, usize), lang::Function> = map
            .units()
            .filter_map(|unit| match unit.kind() {
                LanguageUnitKind::Function(function) => {
                    Some(((unit.anchor().x(), unit.anchor().y()), function))
                }
                _ => None,
            })
            .collect();

        let documented: Vec<lang::Function> = map
            .expressions()
            .filter_map(|expression| expression.root())
            .filter_map(|root| function_at_anchor.get(&(root.x(), root.y())).copied())
            .collect();

        let missing: Vec<String> = lang::Function::ALL
            .iter()
            .filter(|function| !documented.contains(function))
            .map(|function| format!("{function} ({function:?})"))
            .collect();

        assert!(
            missing.is_empty(),
            "the Function reference has no worked example, as the root of an Expression, for: {missing:?}"
        );
    }

    ///
    /// Every example Expression in the reference parses without a
    /// Diagnostic, reading the reference's own
    /// [`orcvs::source::LanguageMap`] built directly from the checked-in
    /// text — the same Map `Source::language_map` exposes for live syntax
    /// highlighting — before any Tick has run.
    ///
    /// A written result such as `02` sits at a row start with no Function
    /// before it, so the Parser's greedy two-Cell Function read refuses it
    /// and advances one Cell at a time (ADR 0018), naming every one of its
    /// Cells "unknown function" or "invalid Language Unit character" — a
    /// fact about the checked-in placeholder, not about the example that
    /// computes it, and never a Tick-time Diagnostic:
    /// `orcvs::source::tick`'s scheduling only ever visits Expressions with
    /// a `function_candidate`, and a bare written result never parses as
    /// one (proven separately by
    /// `ticking_the_reference_once_writes_every_result_row_exactly_as_written`'s
    /// `plan.diagnostics.is_empty()`).
    ///
    /// A Diagnostic is excused only when its whole Span sits inside a
    /// written result Cell or a named spatial literal Cell —
    /// [`expected_results`]'s and [`spatial_literal_cells`]'s own lists, no
    /// blanket area — so a mistyped operand anywhere else in a Source
    /// Function's own Expression still fails this test.
    ///
    #[test]
    fn every_example_expression_parses_without_a_diagnostic_outside_a_result_row() {
        let source = function_reference();
        let map = source.language_map();

        let excused: Vec<ExpectedResult> = expected_results()
            .into_iter()
            .chain(spatial_literal_cells())
            .collect();
        let excluded = |x: usize, y: usize| {
            excused.iter().any(|result| {
                result.row == y
                    && (result.column..result.column + result.expected.chars().count()).contains(&x)
            })
        };

        let offending: Vec<String> = map
            .diagnostics()
            .filter(|diagnostic| {
                diagnostic
                    .span()
                    .positions()
                    .any(|position| !excluded(position.x(), position.y()))
            })
            .map(|diagnostic| {
                let anchor = diagnostic.anchor();
                format!("({}, {}): {}", anchor.x(), anchor.y(), diagnostic.message)
            })
            .collect();

        assert!(
            offending.is_empty(),
            "unexpected Diagnostics outside every excluded result Cell's Span: {offending:?}"
        );
    }
}
