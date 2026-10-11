//! The absolute Copy, `=$ src-column src-row dst-column dst-row`, written as
//! a performer sees it Tick after Tick.
//!
//! It reads the pair at its source Position with Copy's reading rules and
//! writes it to the pair at its destination Position, leaving the source as
//! it was. The source is a dynamic Input Portal and the destination a dynamic
//! Output Portal, both known only at its Turn, so these tests also state the
//! order that Turn takes against the writers of its source and the readers of
//! its destination.

use lang::Tick;

use super::observed::{
    MISSED_BANG, Observed, diagnostic, first, observe_at, observed, quiet, raw, rows_of, source_of,
    turn_before, turns,
};
use crate::grid::Grid;
use crate::source::{CellContent, CellWrite};

/// The diagnostic `=$` gives a source pair it cannot read.
const INVALID: &str = "=$ has partial or invalid input";

#[test]
fn an_absolute_copy_writes_the_source_pair_at_the_destination() {
    let tick = first(Grid::with_shape(10, 3), &["=$00010602", "C4", "xxxxxxxxxx"]);
    assert_eq!(tick.rows, ["=$00010602", "C4        ", "xxxxxxC4xx"]);
    quiet(&tick);
}

#[test]
fn an_unwritten_operand_makes_the_absolute_copy_invalid() {
    // The Language Map diagnoses the unwritten operand, and the Tick writes
    // nothing and does not report it again.
    for row in [
        "=$  010602",
        "=$00  0602",
        "=$0001  02",
        "=$000106  ",
        "=$0 010602",
        "=$00010x02",
    ] {
        let rows = [row, "C4", "xxxxxxxxxx"];
        let source = source_of(Grid::with_shape(10, 3), &rows);
        assert!(
            source
                .language_map()
                .diagnostics()
                .any(|diagnostic| diagnostic.start() == 0),
            "{row:?}"
        );
        let tick = first(Grid::with_shape(10, 3), &rows);
        assert_eq!(tick.rows[2], "xxxxxxxxxx", "{row:?}");
        quiet(&tick);
    }
}

/// Tick zero of `rows`, beneath, first, `=$` copying the pair at `source`
/// to 06 03, and then the composition `@$0603&$` reading the same pair, each
/// as observed. Row 3 is empty.
fn copy_and_composition(source: &str, rows: [&str; 2]) -> (Observed, Observed) {
    let grid = Grid::with_shape(16, 4);
    let copy = format!("=${source}0603");
    let composition = format!("@$0603&${source}");
    let observe = |row0: &str| first(grid, &[row0, rows[0], rows[1], ""]);
    (observe(&copy), observe(&composition))
}

#[test]
fn an_absolute_copy_writes_what_a_write_of_a_read_writes() {
    // A Number, a Note and a Function spelling, each read where it stands in
    // row 1, and the Bang Equality answers at (8, 2), each carried unchanged
    // to 06 03.
    for (source, rows, copied) in [
        ("0801", ["        C4", ""], "C4"),
        ("0801", ["        G4", ""], "G4"),
        ("0801", ["        0A", ""], "0A"),
        ("0801", ["        .+0101", ""], ".+"),
        ("0802", ["        .=0101", ""], "**"),
    ] {
        let (copy, composition) = copy_and_composition(source, rows);
        assert_eq!(&copy.rows[3][6..8], copied, "{rows:?}");
        assert_eq!(copy.rows[3], composition.rows[3], "{rows:?}");
        quiet(&copy);
        quiet(&composition);
    }
}

#[test]
fn an_absolute_copy_diagnoses_the_pairs_a_read_diagnoses() {
    // A Comment introducer, `C|` straddling one, a partial pair, and a pair
    // straddling a Function and its operand. Neither form writes.
    for (source, row1) in [
        ("0801", "        ||E4"),
        ("0801", "        C||4"),
        ("0A01", "        C4D 4"),
        ("0901", "        .+0102"),
    ] {
        let (copy, composition) = copy_and_composition(source, [row1, ""]);
        assert_eq!(copy.rows[3].trim(), "", "{row1:?}");
        assert_eq!(composition.rows[3], copy.rows[3], "{row1:?}");
        assert!(
            copy.diagnostics.contains(&diagnostic(0, 0, INVALID)),
            "{row1:?}: {:?}",
            copy.diagnostics
        );
        assert!(
            composition
                .diagnostics
                .contains(&diagnostic(6, 0, "&$ has partial or invalid input")),
            "{row1:?}: {:?}",
            composition.diagnostics
        );
    }
}

#[test]
fn an_empty_source_clears_the_destination() {
    let tick = first(Grid::with_shape(12, 3), &["=$0A000002", "", "xx"]);
    assert_eq!(tick.rows[2], "            ");
    quiet(&tick);
}

#[test]
fn a_bang_copied_onto_a_roots_anchor_overwrites_it_and_activates_the_roots_aligned_with_it() {
    // Equality writes `**` at (10, 1), and `=$` copies it over Raw Play C4's
    // anchor, so C4 does not run, and activates Raw Play D4 south of it.
    let mut source = source_of(
        Grid::with_shape(16, 4),
        &["=$0A010002.=0101", "", "!>007FC4", "!>007FD4"],
    );
    let plan = source.execute(Tick::ZERO);
    assert!(plan.diagnostics.is_empty(), "{:?}", plan.diagnostics);
    assert_eq!(plan.play_commands, [raw(0, 0x7F, 62)]);
    assert_eq!(rows_of(&source)[2], "**007FC4        ");
}

#[test]
fn a_root_whose_anchor_a_copied_bang_covers_takes_its_turn_after_the_copy() {
    // Equality at (2, 0) stands before `=$` in Grid order and does not feed
    // it, so `=$` goes first and Equality, covered, answers nothing south of
    // it. The Equality at (10, 0) feeds `=$` and goes before it.
    let grid = Grid::with_shape(16, 3);
    let rows = ["  .=0101  .=0101", "", "=$0A010200"];
    let tick = first(grid, &rows);
    assert_eq!(tick.rows[..2], ["  **0101  .=0101", "          **    "]);
    quiet(&tick);
    let turns = turns(grid, &rows);
    turn_before(&turns, (10, 0), (0, 2));
    turn_before(&turns, (0, 2), (2, 0));
}

#[test]
fn a_source_or_destination_outside_the_grid_or_cut_short_diagnoses_and_writes_nothing() {
    for (row, message) in [
        ("=$0C010001", INVALID),
        ("=$00030001", INVALID),
        ("=$FFFF0001", INVALID),
        ("=$0B010001", INVALID),
        ("=$00010C01", "result \"C4\" falls outside the Grid"),
        ("=$00010003", "result \"C4\" falls outside the Grid"),
        ("=$0001FFFF", "result \"C4\" falls outside the Grid"),
        ("=$00010B01", "result \"C4\" crosses the row edge"),
    ] {
        let tick = first(Grid::with_shape(12, 3), &[row, "C4xxxxxxxxxG", ""]);
        assert_eq!(tick.rows[1..], ["C4xxxxxxxxxG", "            "], "{row}");
        assert_eq!(tick.diagnostics, [diagnostic(0, 0, message)], "{row}");
    }
}

#[test]
fn an_absolute_copy_answers_only_at_its_destination() {
    // Row 1 under `=$` keeps its `xx`: nothing is written south.
    let tick = first(
        Grid::with_shape(14, 4),
        &["=$0C000003  C4", "xxxxxxxxxx", "", ""],
    );
    assert_eq!(tick.rows[1], "xxxxxxxxxx    ");
    assert_eq!(tick.rows[3], "C4            ");
    quiet(&tick);
}

#[test]
fn a_nested_absolute_copy_returns_the_pair_it_copies() {
    // The nested `=$` copies the `05` at (4, 1) to (6, 3) and returns it, and
    // `.+` answers `0F` under its anchor. Nothing lands at (2, 1).
    let tick = first(
        Grid::with_shape(14, 4),
        &[".+=$040106030A", "    05", "", ""],
    );
    assert_eq!(
        tick.rows[1..],
        ["0F  05        ", "              ", "      05      "]
    );
    quiet(&tick);
}

#[test]
fn a_nested_absolute_copy_whose_destination_is_outside_the_grid_still_returns_its_pair() {
    // Column 20 is past the row edge: `=$` writes nothing and diagnoses, and
    // `.+` still adds the `05` it copied.
    let tick = first(
        Grid::with_shape(14, 4),
        &[".+=$040120030A", "    05", "", ""],
    );
    assert_eq!(
        tick.rows[1..],
        ["0F  05        ", "              ", "              "]
    );
    assert_eq!(
        tick.diagnostics,
        [diagnostic(2, 0, "result \"05\" falls outside the Grid")]
    );
}

#[test]
fn a_static_writer_of_the_source_takes_its_turn_first() {
    // Before `=$` in Grid order: `.+0102` at (2, 0) writes `03` at (2, 1).
    let grid = Grid::with_shape(10, 4);
    let rows = ["  .+0102", "", "=$02010603"];
    let tick = first(grid, &rows);
    assert_eq!(&tick.rows[3][6..8], "03");
    quiet(&tick);
    turn_before(&turns(grid, &rows), (2, 0), (0, 2));
    // After it: `=^` at (2, 2) copies `D4` into (2, 1). `=$` finds it at
    // its Turn and lets it go first, which is the yield and not a cycle.
    let grid = Grid::with_shape(10, 5);
    let rows = ["=$02010604", "", "  =^", "  D4"];
    let tick = first(grid, &rows);
    assert_eq!(&tick.rows[4][6..8], "D4");
    quiet(&tick);
    turn_before(&turns(grid, &rows), (2, 2), (0, 0));
}

#[test]
fn readers_of_the_destination_see_the_write_the_same_tick() {
    // `&$0603` at (0, 0) reads the destination before `=$` in Grid order,
    // and `=>` at (8, 3) reads it west of itself after `=$`.
    let grid = Grid::with_shape(12, 6);
    let rows = ["&$0603", "", "=$00050603", "        =>", "", "C4"];
    let tick = first(grid, &rows);
    assert_eq!(tick.rows[1], "C4          ");
    assert_eq!(tick.rows[3], "      C4=>C4");
    quiet(&tick);
    let turns = turns(grid, &rows);
    turn_before(&turns, (0, 2), (0, 0));
    turn_before(&turns, (0, 2), (8, 3));
}

#[test]
fn a_source_that_is_the_destination_is_rewritten_unchanged() {
    // The Tick Plan carries the write of `C4` over itself, so a `=$` that
    // wrote nothing would fail here though the Cells read the same.
    let grid = Grid::with_shape(10, 2);
    let mut source = source_of(grid, &["=$00010001", "C4"]);
    for tick in 0..3 {
        let plan = source.execute(Tick::new(tick));
        let tick = observed(&source, &plan);
        assert_eq!(tick.rows[1], "C4        ");
        quiet(&tick);
        for (index, cell) in [(10, b'C'), (11, b'4')] {
            let write = CellWrite {
                cell: grid.cell_index(index).expect("inside the Grid"),
                content: CellContent::new(cell).expect("a printable Cell"),
            };
            assert!(plan.writes.contains(&write), "{:?}", plan.writes);
        }
    }
}

#[test]
fn a_number_pair_overlapping_its_destination_by_one_cell_is_copied_each_tick() {
    // `=$` copies the `01` operand of `.+` one Cell east, over the start of
    // `.+`'s second operand. Only Number Cells change, so the Language Units
    // stay where they were: on Tick 1 the source pair is still the aligned
    // operand, now `00`, and `=$` copies it again. `.+` reads each write in
    // the Tick it lands. Nothing straddles and nothing is a cycle.
    let ticks = observe_at(Grid::with_shape(10, 3), &["=$04010501", "  .+0102"], 0..2);
    assert_eq!(ticks[0].rows[1..], ["  .+0012  ", "  12      "]);
    assert_eq!(ticks[1].rows[1..], ["  .+0002  ", "  02      "]);
    for tick in &ticks {
        quiet(tick);
    }
}

#[test]
fn pairs_that_overlap_by_one_cell_write_then_diagnose_straddling_input() {
    // Straddling needs a Language Unit to move, which only a Function
    // spelling does. On Tick 0 `=$` copies `.+` one Cell east, over the
    // second Cell of its own spelling, which suppresses that `.+` this Tick.
    // On Tick 1 the source pair holds the `.` left behind and the first Cell
    // of the `.+` written there.
    let ticks = observe_at(Grid::with_shape(10, 3), &["=$02010301", "  .+0102"], 0..2);
    assert_eq!(ticks[0].rows[1], "  ..+102  ");
    quiet(&ticks[0]);
    assert_eq!(ticks[1].diagnostics, [diagnostic(0, 0, INVALID)]);
}

#[test]
fn an_absolute_copy_nested_where_its_parent_writes_its_source_is_a_cycle() {
    // `.+` answers at (0, 1), the pair its nested `=$` reads: `.+` waits on
    // `=$`, which waits on `.+`. The Addition below depends on none of it.
    let tick = first(Grid::with_shape(14, 4), &[".+=$000106030A", "", ".+0102"]);
    assert_eq!(
        tick.diagnostics,
        [diagnostic(0, 0, "same-Tick dependency cycle")]
    );
    assert_eq!(tick.rows[3], "03            ");
}

#[test]
fn a_bang_copied_onto_a_root_that_fed_the_copy_is_missed() {
    // Halt at (2, 1) locks `=$` below it, so it takes its Turn first, inert:
    // `.=0102` answers no Bang. `=$` copies the `**` `.=0101` answers at
    // (6, 4) to (0, 1), where it reaches Halt after its Turn, and Halt misses
    // it. The `**` display is still written.
    let grid = Grid::with_shape(12, 5);
    let rows = [".=0102", "  *!", "  =$06040001", "      .=0101"];
    turn_before(&turns(grid, &rows), (2, 1), (2, 2));
    let mut source = source_of(grid, &rows);
    let plan = source.execute(Tick::ZERO);
    assert_eq!(
        observed(&source, &plan).diagnostics,
        [diagnostic(2, 1, MISSED_BANG)]
    );
    assert!(plan.locks.is_empty(), "{:?}", plan.locks);
    assert_eq!(rows_of(&source)[1], "***!        ");
}
