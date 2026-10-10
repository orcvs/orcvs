//! The Writes, `@^ @v @< @> n value` and `@$ column row value`, written as a
//! performer sees them Tick after Tick.
//!
//! A directional Write writes `value` to the pair the Read with the same
//! arrow reads, and the absolute Write to the pair at a Position. The pair is
//! its Output Portal, known only at its Turn, so these tests also state the
//! order that Turn takes against the readers of that pair and against what
//! feeds the Write.

use std::collections::BTreeMap;

use lang::Tick;

use super::observed::{Observed, observe_at, observed, rows_of, source_of};
use super::plan;
use crate::grid::Grid;
use crate::source::Cells;

/// The first Tick of `rows`, observed.
fn first(grid: Grid, rows: &[&str]) -> Observed {
    observe_at(grid, rows, [0]).remove(0)
}

/// The Turn each computation took in Tick zero of `rows`, by anchor.
fn turns(grid: Grid, rows: &[&str]) -> BTreeMap<(usize, usize), Option<usize>> {
    let source = source_of(grid, rows);
    let map = source.shared_language_map();
    let bytes = source.snapshot();
    let (_, states) = plan(grid, Cells::of(bytes.as_bytes()), &map, Tick::ZERO);
    map.schedule_cache()
        .schedule(grid, &map)
        .lookup
        .nodes()
        .iter()
        .zip(&states)
        .map(|(node, state)| ((node.anchor.x(), node.anchor.y()), state.turn()))
        .collect()
}

/// Asserts that the computation at `earlier` took its Turn before the one at
/// `later`, both having taken one.
fn turn_before(
    turns: &BTreeMap<(usize, usize), Option<usize>>,
    earlier: (usize, usize),
    later: (usize, usize),
) {
    let turn =
        |anchor| turns[&anchor].unwrap_or_else(|| panic!("{anchor:?} took no Turn: {turns:?}"));
    assert!(turn(earlier) < turn(later), "{turns:?}");
}

fn diagnostic(x: usize, y: usize, message: &str) -> (usize, usize, String) {
    (x, y, message.to_string())
}

fn quiet(tick: &Observed) {
    assert!(tick.diagnostics.is_empty(), "{:?}", tick.diagnostics);
}

#[test]
fn a_write_answers_its_value_at_its_pair_and_writes_nothing_south() {
    let tick = first(Grid::with_shape(8, 3), &["@$0602C4", "xxxxxxxx"]);
    assert_eq!(tick.rows, ["@$0602C4", "xxxxxxxx", "      C4"]);
    quiet(&tick);
}

#[test]
fn a_value_is_written_as_it_is_spelled() {
    // `C4` spells a Number and a Note, `G4` only a Note, `0A` only a
    // Number, and `c4` a sharp: each is written back as its Cells.
    for value in ["C4", "G4", "0A", "c4"] {
        let row = format!("@$0601{value}");
        let tick = first(Grid::with_shape(8, 2), &[&row]);
        assert_eq!(tick.rows[1], format!("      {value}"), "{value}");
        quiet(&tick);
    }
}

#[test]
fn a_bang_in_value_is_written_as_a_bang_and_activates_the_root_below_it() {
    // The Bang lands at (0, 1), north of Raw Play's anchor.
    let mut source = source_of(Grid::with_shape(8, 3), &["@$0001**", "", "!>007FC4"]);
    let plan = source.execute(Tick::ZERO);
    assert!(plan.diagnostics.is_empty(), "{:?}", plan.diagnostics);
    assert_eq!(plan.play_commands.len(), 1);
    assert_eq!(rows_of(&source)[1], "**      ");
}

#[test]
fn a_bang_written_onto_a_roots_anchor_activates_it_without_overwriting_it() {
    // As a Copy's Bang does: Raw Play at (0, 2) plays and keeps its Cells.
    let mut source = source_of(Grid::with_shape(8, 3), &["@$0002**", "", "!>007FC4"]);
    let plan = source.execute(Tick::ZERO);
    assert!(plan.diagnostics.is_empty(), "{:?}", plan.diagnostics);
    assert_eq!(plan.play_commands.len(), 1);
    assert_eq!(rows_of(&source)[2], "!>007FC4");
}

#[test]
fn a_bang_written_onto_an_occupied_non_root_diagnoses_and_writes_nothing() {
    // As a Copy's Bang does: the `C4` at (0, 1) is no root to activate.
    let tick = first(Grid::with_shape(8, 2), &["@$0001**", "C4"]);
    assert_eq!(tick.rows[1], "C4      ");
    assert_eq!(
        tick.diagnostics,
        [diagnostic(0, 0, "@$ cannot activate an occupied non-root")]
    );
}

#[test]
fn a_root_a_writes_bang_activates_is_read_the_same_tick() {
    // The Bang lands at (8, 1), east of `*v`, which emits `vv` at (6, 2).
    // `&$` at (0, 1) reads that pair: it waits for `*v`, which the Bang
    // made active in this Tick.
    let grid = Grid::with_shape(10, 3);
    let rows = ["@$0801**", "&$0602*v"];
    let tick = first(grid, &rows);
    assert_eq!(tick.rows[2], "vv    vv  ");
    quiet(&tick);
    let turns = turns(grid, &rows);
    turn_before(&turns, (0, 0), (6, 1));
    turn_before(&turns, (6, 1), (0, 1));
}

#[test]
fn a_write_nested_in_a_root_a_writes_bang_activates_goes_writers_first() {
    // The Bang at (0, 1) activates Raw Play at (0, 2), whose nested Write
    // puts `C4` at (10, 3). `&$` at (10, 0) reads that pair, though it
    // stands before the nested Write in Grid order.
    let grid = Grid::with_shape(16, 4);
    let rows = ["@$0001**  &$0A03", "", "!>007F@$0A03C4"];
    let tick = first(grid, &rows);
    assert_eq!(&tick.rows[1][10..12], "C4");
    assert_eq!(&tick.rows[3][10..12], "C4");
    quiet(&tick);
    let turns = turns(grid, &rows);
    turn_before(&turns, (6, 2), (10, 0));
}

#[test]
fn a_root_a_static_bang_activates_is_read_the_same_tick() {
    // The control: Equality's Bang at (8, 1) is known before the Tick.
    let tick = first(Grid::with_shape(14, 3), &["        .=0101", "&$0602*v"]);
    assert_eq!(tick.rows[2], "vv    vv      ");
    quiet(&tick);
}

/// The diagnostic for a Bang that reaches a root after its Turn.
const MISSED_BANG: &str = "Bang reached a root that has taken its Turn";

#[test]
fn a_bang_onto_a_root_that_fed_the_write_is_missed() {
    // Halt at (2, 1) locks the Write below it, so it takes its Turn first,
    // inert: Equality answers no Bang. The Write's `**` at (0, 1) reaches
    // Halt after its Turn, and a Bang is never stored, so Halt misses it.
    // Nothing is stopped: the `**` display is still written.
    let mut source = source_of(
        Grid::with_shape(10, 3),
        &[".=0102    ", "  *!      ", "  @$0001**"],
    );
    let plan = source.execute(Tick::ZERO);
    assert_eq!(
        observed(&source, &plan).diagnostics,
        [diagnostic(2, 1, MISSED_BANG)]
    );
    assert!(plan.locks.is_empty(), "{:?}", plan.locks);
    assert_eq!(rows_of(&source), [".=0102    ", "***!      ", "  @$0001**"]);
}

#[test]
fn a_bang_onto_a_root_that_fed_an_earlier_write_is_missed() {
    // Halt locks the first Write, so it takes its Turn before it, and the
    // first Write goes before the second in Grid order. The second Write's
    // Bang reaches Halt after its Turn; the first Write's `C4` still lands.
    let tick = first(
        Grid::with_shape(10, 4),
        &[".=0102    ", "  *!      ", "  @$0803C4", "@$0001**  "],
    );
    assert_eq!(tick.diagnostics, [diagnostic(2, 1, MISSED_BANG)]);
    assert_eq!(tick.rows[3], "@$0001**C4");
    assert_eq!(&tick.rows[1][..2], "**");
}

#[test]
fn a_missed_bang_still_activates_the_roots_still_to_take_their_turn() {
    // The second Write's `**` at (4, 1) has Halt, which has taken its Turn,
    // to the west and Raw Play, which has not, to the east.
    let mut source = source_of(
        Grid::with_shape(14, 4),
        &[".=0102", "  *!  !>007FC4", "  @$0803C4", "@$0401**"],
    );
    let plan = source.execute(Tick::ZERO);
    assert_eq!(
        observed(&source, &plan).diagnostics,
        [diagnostic(2, 1, MISSED_BANG)]
    );
    assert_eq!(plan.play_commands.len(), 1);
    assert_eq!(rows_of(&source)[1], "  *!**!>007FC4");
}

#[test]
fn a_lock_from_a_halt_the_tick_joined_onto_a_taken_root_is_missed() {
    // The Read nested in the Write reads Equality's Bang, so Equality takes
    // its Turn before the Write's `**` activates Halt. Halt then locks
    // Equality after its Turn: the lock is missed at Equality.
    let tick = first(
        Grid::with_shape(12, 5),
        &["", "  *!", "  .=0101", "", "@$0001&$0203"],
    );
    // The Read on the last row also answers below the Source.
    assert_eq!(
        tick.diagnostics,
        [
            diagnostic(6, 4, "result \"**\" falls below the Source"),
            diagnostic(2, 2, "lock reached a root that has taken its Turn"),
        ]
    );
}

#[test]
fn a_root_a_write_activates_writes_onto_a_taken_function_as_feedback() {
    // The Read nested in the Write reads Equality's Bang at (4, 4), so the
    // Write's `**` at (0, 1) activates Raw Play after Equality's Turn. The
    // Addition nested in Raw Play then writes `7F` over Equality's spelling:
    // Equality keeps this Tick's Turn and is gone from the next.
    let ticks = observe_at(
        Grid::with_shape(12, 5),
        &["@$0001&$0404", "", "!>00.+3F40C4", "    .=0101", ""],
        0..2,
    );
    assert_eq!(ticks[0].rows[3], "    7F0101  ");
    quiet(&ticks[0]);
    assert_eq!(ticks[1].rows[3], "    7F0101  ");
    assert!(
        ticks[1]
            .diagnostics
            .iter()
            .all(|(.., message)| message != "spatial output reached an executed computation"),
        "{:?}",
        ticks[1].diagnostics
    );
}

#[test]
fn an_empty_or_partially_written_operand_makes_the_write_invalid() {
    // The Language Map diagnoses the unwritten operand, and the Tick writes
    // nothing and does not report it again.
    for row in [
        "@>  C4", "@>01  ", "@>01C ", "@$  01C4", "@$0001  ", "@^01xy",
    ] {
        let rows = [row, "xxxxxxxx"];
        let source = source_of(Grid::with_shape(8, 2), &rows);
        assert!(
            source
                .language_map()
                .diagnostics()
                .any(|diagnostic| diagnostic.start() == 0),
            "{row:?}"
        );
        let tick = first(Grid::with_shape(8, 2), &rows);
        assert_eq!(tick.rows[1], "xxxxxxxx", "{row:?}");
        quiet(&tick);
    }
}

#[test]
fn a_write_addresses_the_pair_the_read_with_the_same_arrow_reads() {
    // The Write stands at (4, 3) of a 12 by 7 Grid. Its `G4` lands on the
    // pair it addresses; the Read in the same place reads that pair back.
    let grid = Grid::with_shape(12, 7);
    let cell = |rows: &[String], (x, y): (usize, usize)| rows[y][x..x + 2].to_string();
    for (arrow, n) in [
        ("^", "00"),
        ("^", "02"),
        ("v", "01"),
        ("v", "03"),
        ("<", "01"),
        ("<", "03"),
        (">", "00"),
        (">", "02"),
    ] {
        let write = format!("    @{arrow}{n}G4");
        let mut rows = vec![""; 7];
        rows[3] = &write;
        let tick = first(grid, &rows);
        quiet(&tick);
        let landed: Vec<(usize, usize)> = (0..7)
            .flat_map(|y| (0..11).map(move |x| (x, y)))
            .filter(|&(x, y)| (x, y) != (8, 3) && cell(&tick.rows, (x, y)) == "G4")
            .collect();
        assert_eq!(landed.len(), 1, "@{arrow}{n}: {:?}", tick.rows);
        let (x, y) = landed[0];
        if y == 3 && (4..8).contains(&x) {
            // The pair is the Function's own spelling or operand, which the
            // Read reads as it stands.
            continue;
        }
        // The Read in the Write's place, with `G4` standing at that pair.
        let mut source = vec![vec![b' '; 12]; 7];
        source[3][4..8].copy_from_slice(format!("&{arrow}{n}").as_bytes());
        source[y][x..x + 2].copy_from_slice(b"G4");
        let source: Vec<String> = source
            .into_iter()
            .map(|row| String::from_utf8(row).expect("ASCII Source"))
            .collect();
        let source: Vec<&str> = source.iter().map(String::as_str).collect();
        let read = first(grid, &source);
        assert_eq!(
            cell(&read.rows, (4, 4)),
            "G4",
            "&{arrow}{n}: {:?}",
            read.rows
        );
    }
    // `@$` and `&$` address the same Position.
    let tick = first(Grid::with_shape(8, 4), &["@$0302G4"]);
    assert_eq!(tick.rows[2], "   G4   ");
    let read = first(Grid::with_shape(8, 4), &["&$0302", "", "   G4"]);
    assert_eq!(read.rows[1], "G4      ");
}

#[test]
fn a_write_at_distance_zero_steps_itself() {
    // `@>0002` writes `02` onto its own `n`, so the next Tick it writes two
    // pairs east of the operand, and from then on it holds.
    let ticks = observe_at(Grid::with_shape(8, 1), &["@>0002"], 0..3);
    let rows: Vec<&str> = ticks.iter().map(|tick| tick.rows[0].as_str()).collect();
    assert_eq!(rows, ["@>0202  ", "@>020202", "@>020202"]);
    for tick in &ticks {
        quiet(tick);
    }
}

#[test]
fn a_nested_write_returns_its_value_and_writes_its_pair() {
    // The inner Write writes `C4` at 01 01 and returns it, and the outer
    // Write writes the same `C4` at 00 00, over its own spelling: one value
    // lands at both Positions.
    let tick = first(Grid::with_shape(14, 2), &["@$0000@$0101C4"]);
    assert_eq!(tick.rows, ["C40000@$0101C4", " C4           "]);
    quiet(&tick);
    // A nested Write's value is its parent's operand.
    let tick = first(Grid::with_shape(12, 2), &[".+@$06010503"]);
    assert_eq!(tick.rows, [".+@$06010503", "08    05    "]);
    quiet(&tick);
}

#[test]
fn a_destination_outside_the_grid_or_cut_short_diagnoses_and_writes_nothing() {
    for (row, message) in [
        ("@$0800C4", "result \"C4\" falls outside the Grid"),
        ("@$0002C4", "result \"C4\" falls outside the Grid"),
        ("@$FFFFC4", "result \"C4\" falls outside the Grid"),
        ("@>FFC4", "result \"C4\" falls outside the Grid"),
        ("@^01C4", "result \"C4\" falls outside the Grid"),
        ("@$0700C4", "result \"C4\" crosses the row edge"),
    ] {
        let tick = first(Grid::with_shape(8, 2), &[row, "xxxxxxxx"]);
        assert_eq!(
            tick.rows,
            [format!("{row:8}"), "xxxxxxxx".to_owned()],
            "{row}"
        );
        assert_eq!(tick.diagnostics, [diagnostic(0, 0, message)], "{row}");
    }
    // Nested, the Write still returns its value.
    let tick = first(Grid::with_shape(12, 2), &[".+@$FF010503"]);
    assert_eq!(tick.rows[1], "08          ");
    assert_eq!(
        tick.diagnostics,
        [diagnostic(2, 0, "result \"05\" falls outside the Grid")]
    );
}

#[test]
fn a_static_reader_north_of_the_write_sees_it_the_same_tick() {
    // `=>` at (4, 0) copies (2, 0), which the Write at (0, 1) writes.
    let grid = Grid::with_shape(8, 2);
    let rows = ["  D4=>  ", "@$0200C4"];
    let tick = first(grid, &rows);
    assert_eq!(tick.rows, ["  C4=>C4", "@$0200C4"]);
    quiet(&tick);
    turn_before(&turns(grid, &rows), (0, 1), (4, 0));
}

#[test]
fn a_dynamic_reader_west_of_the_write_sees_it_the_same_tick() {
    // `&$0601` at (0, 0) reads (6, 1), which the Write at (8, 0) writes.
    let grid = Grid::with_shape(16, 3);
    let rows = ["&$0601  @$0601E4", "      D4"];
    let tick = first(grid, &rows);
    assert_eq!(tick.rows[1], "E4    E4        ");
    quiet(&tick);
    turn_before(&turns(grid, &rows), (8, 0), (0, 0));
}

#[test]
fn a_nested_reader_in_another_expression_sees_the_write_the_same_tick() {
    // The `&$` nested in `.+` at (0, 0) reads (6, 1), which the Write at
    // (0, 2) writes.
    let grid = Grid::with_shape(12, 4);
    let rows = [".+&$060101", "      05", "@$060108"];
    let tick = first(grid, &rows);
    assert_eq!(tick.rows[1], "0908  08    ");
    quiet(&tick);
    turn_before(&turns(grid, &rows), (0, 2), (2, 0));
}

#[test]
fn a_reader_that_feeds_the_write_sees_it_on_the_next_tick() {
    // The counter: the nested Read is upstream of the Write and reads `05`
    // before the Write puts `06` there, so it advances one step per Tick.
    let ticks = observe_at(
        Grid::with_shape(16, 3),
        &["@$0602.+&$060201", "", "      05"],
        0..3,
    );
    let counted: Vec<&str> = ticks.iter().map(|tick| &tick.rows[2][6..8]).collect();
    assert_eq!(counted, ["06", "07", "08"]);
    for tick in &ticks {
        quiet(tick);
    }
}

#[test]
fn independent_writes_to_one_pair_take_their_turns_in_grid_order() {
    let grid = Grid::with_shape(8, 3);
    let rows = ["@$0002C4", "@$0002D4"];
    let tick = first(grid, &rows);
    assert_eq!(tick.rows[2], "D4      ");
    quiet(&tick);
    turn_before(&turns(grid, &rows), (0, 0), (0, 1));
}

#[test]
fn a_static_writer_that_does_not_feed_the_write_writes_after_it() {
    // `.+0102` writes `03` at (0, 1), the pair the later Write writes.
    let grid = Grid::with_shape(8, 3);
    let rows = [".+0102", "", "@$0001C4"];
    let tick = first(grid, &rows);
    assert_eq!(tick.rows[1], "03      ");
    quiet(&tick);
    turn_before(&turns(grid, &rows), (0, 2), (0, 0));
}

#[test]
fn a_write_onto_a_function_still_to_come_suppresses_or_replaces_it() {
    // A Number over `.+` suppresses it: nothing is answered south of it.
    let tick = first(Grid::with_shape(8, 3), &["@$0001C4", ".+0102", "xx"]);
    assert_eq!(tick.rows[1..], ["C40102  ", "xx      "]);
    quiet(&tick);
    // A Function spelling the nested Read passes on replaces `.+` with `.-`.
    let tick = first(
        Grid::with_shape(12, 5),
        &["@$0001&$0003", ".+0302", "xx", ".-0101"],
    );
    // The nested Read also answers `.-` through its own Output Portal.
    assert_eq!(tick.rows[1..3], [".-0302.-    ", "01          "]);
    quiet(&tick);
}

#[test]
fn a_write_onto_a_function_that_has_taken_its_turn_is_seen_next_tick() {
    // The nested `.+` feeds the Write and has taken its Turn when the Write
    // puts `02` over its spelling: it answers this Tick, and from the next
    // the Write carries the literal it left.
    let ticks = observe_at(Grid::with_shape(12, 2), &["@$0600.+0101"], 0..2);
    assert_eq!(ticks[0].rows, ["@$0600020101", "      02    "]);
    quiet(&ticks[0]);
    assert_eq!(ticks[1].rows[0], "@$0600020101");
    assert!(
        ticks[1]
            .diagnostics
            .iter()
            .all(|(.., message)| message != "spatial output reached an executed computation"),
        "{:?}",
        ticks[1].diagnostics
    );
}

#[test]
fn a_static_writer_a_nested_read_waits_on_goes_before_the_write() {
    // The Read nested in the Write's value reads (2, 1), which `=^` at
    // (2, 2) writes after the Write in Grid order. The Read finds it at its
    // Turn and it goes first, rather than closing a cycle with the Write.
    let grid = Grid::with_shape(12, 4);
    let rows = ["@$0003&$0201", "", "  =^", "  D4"];
    let tick = first(grid, &rows);
    assert_eq!(tick.rows[3], "D4D4        ");
    quiet(&tick);
    let turns = turns(grid, &rows);
    turn_before(&turns, (2, 2), (6, 0));
    turn_before(&turns, (6, 0), (0, 0));
}

#[test]
fn a_cycle_through_a_real_dependency_is_diagnosed() {
    // The Read nested in the Write nested in `.+` reads (0, 1), which `.+`
    // writes: `.+` waits on the Write, which waits on the Read, which waits
    // on `.+`. The Addition below depends on none of it.
    let tick = first(Grid::with_shape(16, 4), &[".+@$0003&$000101", "", ".+0102"]);
    assert_eq!(
        tick.diagnostics,
        [diagnostic(0, 0, "same-Tick dependency cycle")]
    );
    assert_eq!(tick.rows[3], "03              ");
}

#[test]
fn a_read_nested_in_a_write_copies_one_pair_to_another_place() {
    // Orca's Generator: a Note, a Number and a Bang are carried as read.
    let tick = first(Grid::with_shape(12, 4), &["G4", "@$0603&$0000"]);
    assert_eq!(tick.rows[3], "      G4    ");
    quiet(&tick);
    let tick = first(Grid::with_shape(12, 4), &["0A", "@$0603&$0000"]);
    assert_eq!(tick.rows[3], "      0A    ");
    quiet(&tick);
    // The Bang Equality writes at (8, 1) is copied to (0, 3), north of Raw
    // Play, which it activates.
    let mut source = source_of(
        Grid::with_shape(14, 5),
        &["        .=0101", "", "@$0003&$0801", "", "!>007FC4"],
    );
    let plan = source.execute(Tick::ZERO);
    assert!(plan.diagnostics.is_empty(), "{:?}", plan.diagnostics);
    assert_eq!(plan.play_commands.len(), 1);
}
