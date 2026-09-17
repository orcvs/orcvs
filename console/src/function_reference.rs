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
//! | `48..64`  | 3   | Tick (ticket 03): `~. ~* ~% ~+ ~> ~?`                  |
//! | `64..80`  | 4   | Jumps and Halt (ticket 04): `&^ &v &< &>`, `*!`        |
//! | `80..96`  | 5   | Directional Bangs and Self-Banging (ticket 04): `*^ *v *< *>`, `^^ vv << >>` |
//! | `96..112` | 6   | MIDI (this ticket): `!> !~ !% !c !b`                   |
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
//! up from 48 to 56. Ticket 04 widened it again — band 5's West Directional
//! Bang example's Delay Expression, `~*1001`, reaches column 91 — rounding
//! the Grid's width up from 56 to 96, which is exactly band 5's own upper
//! bound. This ticket widens it once more: Timed Play's and Monophonic
//! Play's Expressions are ten Cells each (`!~0064E408`, `!%0164G404`),
//! starting at column 96 like every MIDI example's Euclidean source above
//! them and so reaching column 105 — rounding the Grid's width up from 96 to
//! 112, exactly band 6's own upper bound. Band 6's five examples, each four
//! rows tall (below), reach only row 26, well inside the height Arithmetic
//! already pins, so this ticket does not grow the Grid's row count either.
//!
//! # MIDI activation needs no area, only a Bang source
//!
//! Every MIDI example is stable after the one Tick that computes it, the same
//! as Arithmetic, Conversion, Sequence, and the Jumps: a Terminal Output
//! Function "never writes a Cell result" (CONTEXT.md's Terminal Output
//! Function entry), so nothing about a `!`-spelled Expression itself ever
//! changes once checked in, and no row is reserved south of it the way every
//! value-answering group reserves one for its result. What does change is
//! each example's own Bang source, exactly as Tick's own Delay and Euclidean
//! examples already do (ticket 03).
//!
//! Each of the five examples is gated by its own Euclidean, `~%0104` — the
//! `X...` pattern, one Bang every four Ticks starting at Tick 0 — rather
//! than a Delay or an always-true Equality, for a reason particular to this
//! band: the ticket asks for a Bang source that "fires at a rate slow
//! enough to hear distinct events at the default tempo" (20 BPM, 750 ms per
//! Tick, from `orcvs::opts::Opts::new`), and neither alternative does that.
//! A Delay whose cycle outlasts the tick budget — band 5's own choice —
//! Bangs exactly once and never again, which does not "fire at a rate" at
//! all. An always-true Equality Bangs every single Tick: band 5 avoids that
//! choice because a Directional Bang's emission occupies a Cell, so every
//! Tick after the first re-attempts a write into an occupied destination and
//! diagnoses forever, but that failure mode does not apply here — a Terminal
//! Output Function has no destination Cell to occupy, so an always-true
//! Equality would perform cleanly on every Tick with no diagnostic. It would
//! simply not read as a distinct event, firing on every Tick rather than at
//! any chosen rate. Euclidean answers a Bang only on the Ticks its rhythm
//! places an onset at (CONTEXT.md's Euclidean Function entry) and the
//! Absence Marker on the rest, so `~% 01 04` gives a genuine repeating pulse
//! at a rate the two operands choose, clearing to blank between Bangs the
//! same way Delay's own result Cell does.
//!
//! Activation is vertical, the same alignment
//! `orcvs::source::tick::a_select_bang_activates_an_aligned_terminal_root`
//! proves for a Select-forwarded Bang activating a Raw Play root two rows
//! south: each Euclidean's Expression sits on one row, its own Bang-display
//! Cells sit one row south at the same column (the ordinary default Portal
//! every Bang-capable Value Function uses), and the gated Terminal Output
//! Function's root sits one row further south, at that same column — sharing
//! the Bang-display Cells' anchor column one row south, exactly the
//! "vertically aligned root" CONTEXT.md's Bang entry describes. A blank row
//! then separates one example from the next, so each example spans four
//! rows (Euclidean Expression, its Bang display, the gated Function, and a
//! blank row) rather than the three-row cadence a value-answering group
//! uses, since a Terminal Output Function's own row replaces a result row
//! rather than sitting beside one.
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
//! untouched edit and fail a loader that demanded one. `source_from_reference_text`
//! instead derives the Grid's width from the widest line and its height from
//! the line count, each rounded up to the Sector Seam spacing, and pads
//! every short line and every row past the last line with empty Cells — see
//! its doc comment.
//!
//! # Completeness and diagnostic-cleanliness are proven, not asserted
//!
//! Two tests close the loop the intent states rather than leaving it as
//! prose. `every_function_in_the_table_has_a_worked_example` reads
//! `lang::Function::ALL` — the table's own enumeration, from
//! `define_functions!` — and asserts each spelling anchors at least one
//! `orcvs::source::LanguageUnitKind::Function` unit in the reference's own
//! `LanguageMap`; a Function added to the table without an example fails
//! the build rather than a checklist going stale. It reads the spelling
//! through `Function`'s existing `Display` impl, so no new public API is
//! needed.
//!
//! `every_example_expression_parses_without_a_diagnostic_outside_a_result_row`
//! reads the same `LanguageMap`, built directly from the checked-in text
//! before any Tick runs, and asserts every Diagnostic's Span sits wholly
//! inside a named excluded area: the one-Tick result triples, the movement
//! areas' non-Function Cells, or the MIDI band's Bang-display Cells. A
//! written result such as `02` sits at a row start with no Function before
//! it, so the Parser's greedy two-Cell Function read refuses it a Cell at a
//! time (ADR 0018) — a fact about the checked-in placeholder, never about
//! the example that computes it, and never a Tick-time Diagnostic: Tick
//! scheduling only ever visits Expressions with a `function_candidate`, and
//! a bare written result never parses as one.
//!
//! This diagnostic-cleanliness proof caught a real defect rather than only
//! guarding against future ones. Ticket 02's Concatenate example,
//! `:&01:-0203`, spelled its left operand as a bare literal `01` against
//! its `AtomOrSequence`-declared slot, which the Parser resolves to
//! `Token::Atom` for parsing purposes. `Token::Atom::decode` always refuses
//! a literal by design (`lang/src/expression.rs`): a generic Atom operand
//! "has no literal reading" because nothing in the declaration says
//! whether it means a Number, a Note, or a Char, so "the only thing that
//! can stand at the position is a nested Function's typed answer" — the
//! same rule ADR 0007 states for a Sequence-typed operand, extended to the
//! Atom alternative Concatenate accepts beside it. The Expression therefore
//! diagnosed at the static parse stage, and Tick scheduling's
//! `syntax_valid` carried the same fact forward: `syntax_blocks`
//! (`orcvs/src/source/tick/execution.rs`) silently withheld Concatenate's
//! own Turn every Tick, with no Diagnostic of its own, because its
//! operands never changed from the checked-in text `syntax_valid` already
//! judged invalid. The checked-in result, `010203`, was a hand-typed
//! placeholder that happened to already match — never an actual Tick
//! answer — which is exactly what
//! `ticking_the_reference_once_writes_every_result_row_exactly_as_written`
//! could not catch: it only reads settled Cells back, never asks whether a
//! Tick wrote them. The fix, `:&.+0001:-0203`, gives the left operand a
//! nested Function's Atom answer (Add's `01`) instead of a bare literal,
//! keeping the same worked answer while letting Concatenate's Turn
//! actually run: corrupting the result Cell by hand and re-ticking now
//! shows it overwritten, where before the fix it stood untouched.
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
    use orcvs::source::{
        BendLsb, BendMsb, ControlValue, Controller, LanguageUnitKind, Length, MidiChannel, Note,
        PlayCommand, Tick, Velocity,
    };

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

    /// Every result row the Arithmetic, Conversion, Sequence, and Tick
    /// groups' examples write, one `(column, row, expected)` triple each,
    /// where `column` is the group's own anchor column (Arithmetic 0,
    /// Conversion 16, Sequence 32, Tick 48) since each group's result rows
    /// are read back from its own Cells, not always the leftmost ones.
    ///
    /// Shared by [`ticking_the_reference_once_writes_every_result_row_exactly_as_written`],
    /// which reads these Cells back after ticking, and
    /// `every_example_expression_parses_without_a_diagnostic_outside_a_result_row`,
    /// which excludes them from the pre-Tick diagnostic sweep: a written
    /// result such as `03` sits at a row start with no Function before it,
    /// so the Parser's greedy two-Cell Function read refuses it (ADR 0018)
    /// and diagnoses every one of its Cells, a fact about the checked-in
    /// placeholder rather than about the example that computes it.
    fn one_tick_result_triples() -> Vec<(usize, usize, &'static str)> {
        vec![
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
            (32, 13, "010203"), // :&.+0001:-0203 (Concatenate, over a nested Function's Atom answer)
            (32, 16, "01"),     // :?00:-0103 (Select)
            (32, 19, "010303"), // :=01.+0102:-0103 (Replace, over nested Functions)
            (32, 22, "111213"), // .+10:-0103 (Add, pervasive over a Sequence)
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
        ]
    }

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

        for (column, row, expected) in one_tick_result_triples() {
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

    /// The MIDI group's own five Bang-display Cells (ticket 05), one pair per
    /// Terminal Output example — column `96..98`, one row south of that
    /// example's Euclidean root, exactly the default output Portal every
    /// Value Function that can emit Bang uses (the same Cells `~*0302`'s `**`
    /// occupies south of Tick's own Delay, and `~%0308`'s south of Tick's own
    /// Euclidean). `~% 01 04` Bangs on the "X..." pattern
    /// (`euclidean_places_its_hits_where_the_adr_formula_does` proves this
    /// pattern from the same `~%` formula this module's own Euclidean
    /// example uses), so this Cell reads `**` on every fourth Tick starting
    /// at Tick 0 and clears to blank on the three Ticks between — the same
    /// clearing behaviour ticket 04's Delay and Tick's own Euclidean already
    /// rely on — which is why these five Cells, and no others in the MIDI
    /// group, are excluded from "no Cell outside an example's own area
    /// changes" below. Every Terminal Output Function's own Expression is
    /// stable forever after being written once: it performs an effect and
    /// never receives a Cell write of its own, so re-evaluating it on a
    /// later Tick reads back exactly what ticket 05 checked in.
    fn midi_bang_result_areas() -> Vec<Area> {
        [8, 12, 16, 20, 24]
            .into_iter()
            .map(|row| Area {
                columns: 96..98,
                rows: row..(row + 1),
            })
            .collect()
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
        let midi_bang = midi_bang_result_areas();
        let excluded = |x: usize, y: usize| {
            movement.iter().any(|area| area.contains(x, y))
                || tick_dynamic.iter().any(|area| area.contains(x, y))
                || midi_bang.iter().any(|area| area.contains(x, y))
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

    /// The five expected [`PlayCommand`]s this ticket's MIDI examples emit
    /// when their own Euclidean Bang source Bangs, in the group's own row
    /// order (Raw Play, Timed Play, Monophonic Play, Control Change, Pitch
    /// Bend) — the operands `console/assets/function_reference.orcvs` checks
    /// in for each, read as the domain types the Function table binds them
    /// to. Channel, velocity, note, controller, value, and bend operands were
    /// chosen to be plausible MIDI content (a mid-velocity Middle C on
    /// channel `00`, a Volume Control Change (`07`), a Pitch Bend above
    /// centre (`40`)) rather than boundary values, since this is a worked
    /// example and not a domain-boundary test — those already live in
    /// `lang`.
    fn expected_midi_commands() -> [PlayCommand; 5] {
        [
            // !>0064C4 — Raw Play: channel 00, velocity 64 (100), note C4.
            PlayCommand::Raw {
                channel: MidiChannel::try_from(0x00).unwrap(),
                velocity: Velocity::try_from(0x64).unwrap(),
                note: Note::try_from(0x3C).unwrap(),
            },
            // !~0064E408 — Timed Play: channel 00, velocity 64, note E4,
            // length 08 Ticks.
            PlayCommand::Timed {
                channel: MidiChannel::try_from(0x00).unwrap(),
                velocity: Velocity::try_from(0x64).unwrap(),
                note: Note::try_from(0x40).unwrap(),
                length: Length::from(0x08),
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
    /// Ticket 05's MIDI group: five Terminal Output Functions (`!>`, `!~`,
    /// `!%`, `!c`, `!b` — every Terminal Output Function the Function table
    /// holds; `!$` Application Command is not among them, since ADR 0008 and
    /// ADR 0019 defer it until Orcvs has a command value encoding, so it has
    /// no variant in `lang::Function` to give an example of), each gated by
    /// its own Euclidean Bang source, `~%0104` — the `X...` pattern
    /// `euclidean_places_its_hits_where_the_adr_formula_does` in `lang`
    /// checks, one Bang every four Ticks starting at Tick 0. At the default
    /// 20 BPM tempo (`orcvs::opts::Opts::new`'s `Bpm::new(20)`, 750 ms per
    /// Tick), that is a Bang roughly every three seconds — slow enough to
    /// hear each group's note, chord change, controller sweep, or bend as
    /// its own event, unlike a Bang that repeated every Tick. A Delay was
    /// not used instead: cycle `~*1001` Bangs exactly once and never again
    /// inside any tick budget this module ticks, which does not "fire at a
    /// rate" the way this ticket calls for. An Equality that holds forever
    /// was not used either: unlike ticket 04's Directional Bang, a Terminal
    /// Output Function never receives a Cell write of its own to occupy, so
    /// an always-true Equality would not diagnose here the way it does for a
    /// mover — but it would Bang every single Tick, which does not read as a
    /// distinct event at any tempo.
    ///
    /// Every Terminal Output Function "never writes a Cell result"
    /// (CONTEXT.md's Terminal Output Function entry), so no row is reserved
    /// south of any of these five Expressions — unlike every value-answering
    /// group's own result row — and the only Cells that ever change while
    /// ticking this group are the Euclidean sources' own Bang-display Cells,
    /// [`midi_bang_result_areas`], which this test names explicitly and
    /// excludes rather than asserting fixed content for, since each toggles
    /// between `**` and blank forever rather than settling.
    ///
    /// This ticks the reference across two full Euclidean cycles (Ticks `0`
    /// through `8`) and asserts, at every Tick: no diagnostic, no
    /// [`orcvs::source::CellWrite`] whose Position falls inside the MIDI
    /// band (column `96` onward — no other group ever writes there), and no
    /// Cell inside the MIDI band changes except the five Bang-display Cells.
    /// Across the whole run it also asserts every one of the five expected
    /// [`PlayCommand`]s was emitted on at least one Tick — proof the group
    /// plays, not merely that it declines to write, which is what the Tick
    /// Plan's own `play_commands` exposes for exactly this purpose (the same
    /// field `orcvs::source::tick`'s own
    /// `a_select_bang_activates_an_aligned_terminal_root` test reads).
    ///
    #[test]
    fn ticking_the_midi_group_writes_no_cell_and_emits_every_terminal_output_function() {
        let mut source = function_reference();
        let grid = source.grid();

        let read = |source: &super::Source, x: usize, y: usize| -> Option<String> {
            let position = grid.position(x, y).expect("inside the reference Grid");
            source.get(grid.index(position))
        };

        let midi_columns = 96..grid.columns();
        let midi_rows = 0..grid.rows();
        let midi_coordinates: Vec<(usize, usize)> = midi_rows
            .clone()
            .flat_map(|y| midi_columns.clone().map(move |x| (x, y)))
            .collect();
        let bang_cells = midi_bang_result_areas();
        let excluded = |x: usize, y: usize| bang_cells.iter().any(|area| area.contains(x, y));

        let before: Vec<Option<String>> = midi_coordinates
            .iter()
            .map(|&(x, y)| read(&source, x, y))
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
                let in_midi_band = position.x() >= 96;
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
                .map(|&(x, y)| read(&source, x, y))
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
    /// (`lang/src/atom.rs`) — anchors at least one
    /// [`LanguageUnitKind::Function`] Language Unit somewhere in the
    /// reference's own [`orcvs::source::LanguageMap`]. Deriving the list from
    /// the table itself, rather than a hand-written one, is the point: a
    /// Function added to `define_functions!` without an example here fails
    /// this test instead of a checklist nobody remembered to update.
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

        let documented: Vec<lang::Function> = map
            .units()
            .filter_map(|unit| match unit.kind() {
                LanguageUnitKind::Function(function) => Some(function),
                _ => None,
            })
            .collect();

        let missing: Vec<String> = lang::Function::ALL
            .iter()
            .filter(|function| !documented.contains(function))
            .map(|function| format!("{function} ({function:?})"))
            .collect();

        assert!(
            missing.is_empty(),
            "the Function reference has no worked example for: {missing:?}"
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
    /// A Diagnostic is excused only when its whole Span sits inside one of
    /// three named areas, each reused from an existing helper rather than a
    /// second hand-maintained coordinate list:
    ///
    /// - [`one_tick_result_triples`]'s result Cells (Arithmetic, Conversion,
    ///   Sequence, and Tick's own answers);
    /// - [`movement_areas`]'s non-Function Cells — the mover, jump, and
    ///   blocker values a Jump, Halt, Directional Bang, or Self-Banging
    ///   example writes or reads spatially — filtered to exclude whichever
    ///   of an area's own Cells anchor a [`LanguageUnitKind::Function`]
    ///   unit, so a mistyped Function spelling inside a movement area still
    ///   fails this test;
    /// - [`midi_bang_result_areas`]'s Bang-display Cells.
    ///
    /// This test caught a real defect rather than only guarding against
    /// future ones: see the module doc's "Completeness and
    /// diagnostic-cleanliness are proven, not asserted" section for what it
    /// found in ticket 02's Concatenate example.
    ///
    #[test]
    fn every_example_expression_parses_without_a_diagnostic_outside_a_result_row() {
        let source = function_reference();
        let map = source.language_map();

        let function_cells: std::collections::HashSet<(usize, usize)> = map
            .units()
            .filter(|unit| matches!(unit.kind(), LanguageUnitKind::Function(_)))
            .flat_map(|unit| unit.span().positions())
            .map(|position| (position.x(), position.y()))
            .collect();

        let one_tick = one_tick_result_triples();
        let movement = movement_areas();
        let midi_bang = midi_bang_result_areas();

        let excluded = |x: usize, y: usize| {
            one_tick.iter().any(|&(column, row, expected)| {
                row == y && (column..column + expected.chars().count()).contains(&x)
            }) || (movement.iter().any(|area| area.contains(x, y))
                && !function_cells.contains(&(x, y)))
                || midi_bang.iter().any(|area| area.contains(x, y))
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
            "unexpected Diagnostics outside every excluded result Cell range: {offending:?}"
        );
    }
}
