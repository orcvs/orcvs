//! Push, `@t index count value`, written as a performer sees it Tick after
//! Tick.
//!
//! Push writes `value` into the pair `index % count` pairs east of its
//! default Output Portal: a lane of `count` pairs on the row below it,
//! starting directly under its anchor. The pair is its Output Portal, known
//! only at its Turn, so these tests also state the order that Turn takes
//! against what feeds Push and against the readers of that pair.

use super::observed::{
    MISSED_BANG, diagnostic, first, observe_at, quiet, source_of, turn_before, turns,
};
use crate::grid::Grid;

#[test]
fn push_writes_its_value_into_the_selected_pair_of_the_lane_below_it() {
    // Pair 00 is the Cell pair directly under Push's anchor, and each step
    // of `index % count` is one pair east. Only that pair is written.
    for (row, lane) in [
        ("@t0003C4", "C4xxxxxxxx"),
        ("@t0103C4", "xxC4xxxxxx"),
        ("@t0203C4", "xxxxC4xxxx"),
        ("@t0403C4", "xxC4xxxxxx"),
        ("@t0405C4", "xxxxxxxxC4"),
    ] {
        let tick = first(Grid::with_shape(10, 2), &[row, "xxxxxxxxxx"]);
        assert_eq!(tick.rows, [format!("{row:10}"), lane.to_owned()], "{row}");
        quiet(&tick);
    }
}

#[test]
fn a_nested_operand_does_not_move_the_lane() {
    // `.+0002` answers `02` as the index and writes it south of itself too.
    let index = first(Grid::with_shape(12, 2), &["@t.+000203C4"]);
    assert_eq!(index.rows, ["@t.+000203C4", "  02C4      "]);
    quiet(&index);
    // A nested `value` answers `0A` south of itself and to Push, which
    // writes it into pair 01 of the same lane.
    let value = first(Grid::with_shape(12, 2), &["@t0103.+0505"]);
    assert_eq!(value.rows, ["@t0103.+0505", "  0A  0A    "]);
    quiet(&value);
}

#[test]
fn a_nested_push_returns_its_value_to_its_parent() {
    // Push writes `05` into pair 01 of its lane, under its own index, and
    // returns it to the Addition, which answers `0F` under its anchor.
    let tick = first(Grid::with_shape(12, 2), &[".+@t0103050A"]);
    assert_eq!(tick.rows, [".+@t0103050A", "0F  05      "]);
    quiet(&tick);
}

#[test]
fn a_zero_count_diagnoses_at_the_turn() {
    let tick = first(Grid::with_shape(8, 2), &["@t0100C4", "xxxxxxxx"]);
    assert_eq!(tick.rows, ["@t0100C4", "xxxxxxxx"]);
    assert_eq!(
        tick.diagnostics,
        [diagnostic(0, 0, "@t cannot wrap at a zero count")]
    );
}

#[test]
fn a_pair_past_the_row_edge_or_a_lane_below_the_grid_diagnoses_and_writes_nothing() {
    for ((columns, rows), row, message) in [
        ((8, 2), "@t0405C4", "result \"C4\" falls outside the Grid"),
        ((9, 2), "@t0405C4", "result \"C4\" crosses the row edge"),
        ((8, 1), "@t0003C4", "result \"C4\" falls outside the Grid"),
    ] {
        let tick = first(Grid::with_shape(columns, rows), &[row]);
        assert_eq!(tick.rows[0], format!("{row:columns$}"), "{row}");
        if rows > 1 {
            assert_eq!(tick.rows[1].trim(), "", "{row}");
        }
        assert_eq!(tick.diagnostics, [diagnostic(0, 0, message)], "{row}");
    }
    // Nested, Push still returns its value: pair 05 of its lane starts at
    // column 12, past the row edge.
    let tick = first(Grid::with_shape(12, 2), &[".+@t0506050A"]);
    assert_eq!(tick.rows, [".+@t0506050A", "0F          "]);
    assert_eq!(
        tick.diagnostics,
        [diagnostic(2, 0, "result \"05\" falls outside the Grid")]
    );
}

#[test]
fn an_empty_or_partially_written_operand_makes_push_invalid() {
    // The Language Map diagnoses the unwritten operand, and the Tick writes
    // nothing and does not report it again.
    for row in [
        "@t  03G4C4",
        "@t01  G4C4",
        "@t0103  C4",
        "@t0 03G4C4",
        "@t0103G C4",
        "@t0103xyC4",
    ] {
        let rows = [row, "xxxxxxxxxx"];
        let source = source_of(Grid::with_shape(10, 2), &rows);
        assert!(
            source
                .language_map()
                .diagnostics()
                .any(|diagnostic| diagnostic.start() == 0),
            "{row:?}"
        );
        let tick = first(Grid::with_shape(10, 2), &rows);
        assert_eq!(tick.rows, rows, "{row:?}");
        quiet(&tick);
    }
}

#[test]
fn a_function_standing_in_the_lane_takes_its_turn_and_meets_the_write() {
    // Push takes its Turn first. Over the anchor of `.+0102`, a pair that
    // spells no Function suppresses it: nothing is answered south of it.
    let tick = first(Grid::with_shape(8, 3), &["@t0003C4", ".+0102", "xx"]);
    assert_eq!(tick.rows[1..], ["C40102  ", "xx      "]);
    quiet(&tick);
    // Over an operand, `.+` still takes its Turn and adds what Push wrote.
    let tick = first(Grid::with_shape(8, 3), &["@t0203C4", ".+0102", "xx"]);
    assert_eq!(tick.rows[1..], [".+01C4  ", "C5      "]);
    quiet(&tick);
    // A Function spelling the nested Read passes on replaces `.+` with `.-`,
    // which the Read also answers through its own Output Portal.
    let tick = first(
        Grid::with_shape(12, 5),
        &["@t0003&$0003", ".+0302", "xx", ".-0101"],
    );
    assert_eq!(tick.rows[1..3], [".-0302.-    ", "01          "]);
    quiet(&tick);
}

#[test]
fn a_clock_writing_the_index_is_read_the_same_tick() {
    // `~.0103` at (2, 0) writes steps `00`, `01` and `02` into Push's index.
    let grid = Grid::with_shape(8, 3);
    let rows = ["  ~.0103", "@t  03C4"];
    let ticks = observe_at(grid, &rows, 0..3);
    let lanes: Vec<&str> = ticks.iter().map(|tick| tick.rows[2].as_str()).collect();
    assert_eq!(lanes, ["C4      ", "C4C4    ", "C4C4C4  "]);
    for tick in &ticks {
        quiet(tick);
    }
    turn_before(&turns(grid, &rows), (2, 0), (0, 1));
}

#[test]
fn a_count_written_the_same_tick_is_read_the_same_tick() {
    // `.+0102` at (4, 0) writes `03` into Push's count: `05 % 03` is pair 2.
    let grid = Grid::with_shape(10, 3);
    let rows = ["    .+0102", "@t05  C4"];
    let tick = first(grid, &rows);
    assert_eq!(tick.rows[1..], ["@t0503C4  ", "    C4    "]);
    quiet(&tick);
    turn_before(&turns(grid, &rows), (4, 0), (0, 1));
}

#[test]
fn a_value_written_the_same_tick_is_carried_the_same_tick() {
    // `.+0102` at (6, 0) writes `03` into Push's `value`.
    let grid = Grid::with_shape(12, 3);
    let rows = ["      .+0102", "@t0103"];
    let tick = first(grid, &rows);
    assert_eq!(tick.rows[1..], ["@t010303    ", "  03        "]);
    quiet(&tick);
    turn_before(&turns(grid, &rows), (6, 0), (0, 1));
}

#[test]
fn readers_of_the_selected_pair_before_and_after_push_see_it_the_same_tick() {
    // Push writes `C4` at (10, 3). `&$0A03` at (0, 0) and `=^` at (10, 2)
    // read it and stand before Push in Grid order; `=v` at (10, 4) reads it
    // and stands after.
    let grid = Grid::with_shape(14, 6);
    let rows = ["&$0A03", "", "@t0506C4  =^", "", "          =v"];
    let tick = first(grid, &rows);
    assert_eq!(
        tick.rows,
        [
            "&$0A03        ",
            "C4        C4  ",
            "@t0506C4  =^  ",
            "          C4  ",
            "          =v  ",
            "          C4  ",
        ]
    );
    quiet(&tick);
    let turns = turns(grid, &rows);
    for reader in [(0, 0), (10, 2), (10, 4)] {
        turn_before(&turns, (0, 2), reader);
    }
}

#[test]
fn a_track_whose_list_is_the_lane_reads_what_push_writes_in_the_same_tick() {
    // Track stands on the row below Push, west of the lane, so its list is
    // Push's lane and the same `index` and `count` select the same pair.
    const LANE: [&str; 4] = ["C4", "D4", "E4", "F4"];
    for (index, count) in [
        ("00", "03"),
        ("01", "03"),
        ("02", "03"),
        ("05", "03"),
        ("07", "04"),
        ("FF", "FF"),
        ("FF", "04"),
    ] {
        let grid = Grid::with_shape(14, 3);
        let rows = [
            format!("      @t{index}{count}0A"),
            format!("&t{index}{count}{}", LANE.concat()),
        ];
        let rows: Vec<&str> = rows.iter().map(String::as_str).collect();
        let tick = first(grid, &rows);
        quiet(&tick);
        let landed: Vec<usize> = (0..LANE.len())
            .filter(|pair| &tick.rows[1][6 + 2 * pair..8 + 2 * pair] == "0A")
            .collect();
        assert_eq!(landed.len(), 1, "@t{index}{count}: {:?}", tick.rows);
        assert_eq!(&tick.rows[2][..2], "0A", "&t{index}{count}");
        turn_before(&turns(grid, &rows), (6, 0), (0, 1));
    }
}

#[test]
fn a_track_that_feeds_push_reads_its_write_on_the_next_tick() {
    // The counter: Track at (0, 1) reads pair 00 of Push's lane, (6, 1), and
    // writes it at (0, 2), which the Read nested in Push's value reads. So
    // Track feeds Push, reads `05` before Push writes one more there, and
    // advances one step per Tick with no cycle diagnosed.
    let ticks = observe_at(
        Grid::with_shape(22, 3),
        &["      @t0001.+01&$0002", "&t000105"],
        0..3,
    );
    let read: Vec<&str> = ticks.iter().map(|tick| &tick.rows[2][..2]).collect();
    assert_eq!(read, ["05", "06", "07"]);
    let lane: Vec<&str> = ticks.iter().map(|tick| &tick.rows[1][6..8]).collect();
    assert_eq!(lane, ["06", "07", "08"]);
    for tick in &ticks {
        quiet(tick);
    }
}

#[test]
fn a_bang_onto_a_root_that_fed_push_is_missed() {
    // The Read nested in Push's value reads the `**` Equality at (12, 2)
    // writes, so Equality goes first, and Halt at (12, 1), which would lock
    // it, goes before it, inert: Equality at (14, 0) answers no Bang. Push
    // carries the `**` to pair 05 of its lane, (10, 1), where it reaches
    // Halt after its Turn, and a Bang is never stored, so Halt misses it.
    // Nothing is stopped: the `**` display is still written.
    let grid = Grid::with_shape(20, 4);
    let rows = [
        "@t0506&$0C03  .=0102",
        "            *!",
        "            .=0101",
    ];
    let tick = first(grid, &rows);
    assert_eq!(tick.diagnostics, [diagnostic(12, 1, MISSED_BANG)]);
    assert_eq!(
        tick.rows,
        [
            "@t0506&$0C03  .=0102",
            "      **  ***!      ",
            "            .=0101  ",
            "            **      ",
        ]
    );
    let turns = turns(grid, &rows);
    turn_before(&turns, (12, 1), (12, 2));
    turn_before(&turns, (12, 2), (6, 0));
    turn_before(&turns, (6, 0), (0, 0));
}
