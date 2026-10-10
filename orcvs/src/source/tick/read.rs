//! The directional Reads, `&^ &v &< &> n`, read as a performer sees them Tick
//! after Tick.
//!
//! A Read reads the Cell pair `n` Portals from its `n` operand in its arrow's
//! direction with Copy's rules, and writes what it reads through its Output
//! Portal. The pair is known only at the Read's Turn, so these tests also
//! state the order that Turn takes against the writers of the Cells it reads.

use std::collections::BTreeMap;

use lang::{MidiChannel, Note, PlayCommand, Tick, Velocity};

use super::observed::{Observed, observe_at, rows_of, source_of};
use super::plan;
use crate::grid::Grid;
use crate::source::Cells;

/// The first Tick of `rows`, observed.
fn first(grid: Grid, rows: &[&str]) -> Observed {
    observe_at(grid, rows, [0]).remove(0)
}

/// The Grid rows after `ticks` consecutive Ticks from Tick zero, one entry per
/// Tick, for a Source whose Ticks diagnose nothing.
fn quiet_rows(grid: Grid, rows: &[&str], ticks: u64) -> Vec<Vec<String>> {
    observe_at(grid, rows, 0..ticks)
        .into_iter()
        .map(|tick| {
            assert!(tick.diagnostics.is_empty(), "{:?}", tick.diagnostics);
            tick.rows
        })
        .collect()
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

/// Asserts that the computation at `writer` took its Turn before the one at
/// `read`, both having taken one.
fn turn_before(
    turns: &BTreeMap<(usize, usize), Option<usize>>,
    writer: (usize, usize),
    read: (usize, usize),
) {
    let turn =
        |anchor| turns[&anchor].unwrap_or_else(|| panic!("{anchor:?} took no Turn: {turns:?}"));
    assert!(turn(writer) < turn(read), "{turns:?}");
}

fn diagnostic(x: usize, y: usize, message: &str) -> (usize, usize, String) {
    (x, y, message.to_string())
}

/// The diagnostic a Read spelled `spelling` gives a pair it cannot read.
fn invalid(spelling: &str) -> String {
    format!("{spelling} has partial or invalid input")
}

#[test]
fn a_distance_counts_portals_from_the_n_operand() {
    // Step `00` is the operand itself, and each step east is one pair.
    let own = first(Grid::with_shape(6, 2), &["&>00C4"]);
    assert_eq!(own.rows, ["&>00C4", "00    "]);
    let next = first(Grid::with_shape(6, 2), &["&>01C4"]);
    assert_eq!(next.rows, ["&>01C4", "C4    "]);
    let second = first(Grid::with_shape(8, 2), &["&>02C4D4"]);
    assert_eq!(second.rows, ["&>02C4D4", "D4      "]);
    for tick in [own, next, second] {
        assert!(tick.diagnostics.is_empty(), "{:?}", tick.diagnostics);
    }
}

#[test]
fn every_read_at_distance_zero_answers_its_own_operand() {
    for spelling in ["&^", "&v", "&<", "&>"] {
        let row = format!("{spelling}00");
        let tick = first(Grid::with_shape(4, 3), &["", &row]);
        assert_eq!(tick.rows, ["    ", row.as_str(), "00  "], "{spelling}");
        assert!(tick.diagnostics.is_empty(), "{:?}", tick.diagnostics);
    }
}

#[test]
fn north_and_south_count_rows_in_the_operand_columns() {
    // The operand stands at column 2, so each Read reads column 2.
    let north = first(Grid::with_shape(4, 4), &["  F4", "  E4", "&^02"]);
    assert_eq!(north.rows[3], "F4  ");
    let south = first(Grid::with_shape(4, 4), &["&v02", "", "  E4"]);
    assert_eq!(south.rows[1], "E4  ");
    for tick in [north, south] {
        assert!(tick.diagnostics.is_empty(), "{:?}", tick.diagnostics);
    }
}

#[test]
fn a_south_read_one_row_down_reads_the_pair_beside_its_output_portal() {
    // The Output Portal is below the spelling, at column 0; the pair one row
    // south of the operand is at column 2, so the Read writes beside it.
    let tick = first(Grid::with_shape(4, 2), &["&v01", "  D4"]);
    assert_eq!(tick.rows, ["&v01", "D4D4"]);
    assert!(tick.diagnostics.is_empty(), "{:?}", tick.diagnostics);
}

#[test]
fn a_west_read_counts_pairs_back_over_its_own_spelling() {
    // From the operand at column 6, one pair west is the spelling itself and
    // three pairs west is column 0.
    let tick = first(Grid::with_shape(8, 2), &["G4  &<03"]);
    assert_eq!(tick.rows, ["G4  &<03", "    G4  "]);
    assert!(tick.diagnostics.is_empty(), "{:?}", tick.diagnostics);
}

#[test]
fn a_west_read_one_portal_away_writes_a_copy_of_its_own_function() {
    let tick = first(Grid::with_shape(4, 2), &["&<01"]);
    assert_eq!(tick.rows, ["&<01", "&<  "]);
    assert!(tick.diagnostics.is_empty(), "{:?}", tick.diagnostics);
}

#[test]
fn a_nested_n_counts_from_the_slot_it_occupies() {
    // Step `00` is `.+`, steps `01` and `02` its operands, and step `03` the
    // `C4`. `.+` also writes its answer south of itself.
    let second_operand = first(Grid::with_shape(10, 2), &["&>.+0101C4"]);
    assert_eq!(second_operand.rows, ["&>.+0101C4", "0102      "]);
    let past = first(Grid::with_shape(10, 2), &["&>.+0102C4"]);
    assert_eq!(past.rows, ["&>.+0102C4", "C403      "]);
    // Track counts its list from the end of its last operand instead, nested
    // Functions included: its pair 01 is `D4`.
    let track = first(Grid::with_shape(16, 2), &["&t.+010003C4D4E4"]);
    assert_eq!(track.rows[1], "D401            ");
    for tick in [second_operand, past, track] {
        assert!(tick.diagnostics.is_empty(), "{:?}", tick.diagnostics);
    }
}

#[test]
fn an_n_a_portal_writes_this_tick_is_read_at_the_turn() {
    // The Read's operand is empty in the Source. `.+0101` writes `02` into it
    // before the Read's Turn, so it reads two pairs east.
    let grid = Grid::with_shape(8, 3);
    let rows = ["  .+0101", "&>  C4D4"];
    assert_eq!(
        quiet_rows(grid, &rows, 1),
        [["  .+0101", "&>02C4D4", "D4      "]]
    );
}

#[test]
fn a_clock_writing_n_is_read_the_same_tick() {
    // `~.0103` writes `00`, `01`, `02` into the south Read's operand each
    // Tick, so the Read answers its operand and then each pair below it.
    let grid = Grid::with_shape(8, 4);
    let selected: Vec<String> = quiet_rows(grid, &["  ~.0103", "&v", "      ", "  E4"], 4)
        .into_iter()
        .map(|tick| tick[2][..2].to_string())
        .collect();
    assert_eq!(selected, ["00", "  ", "E4", "00"]);
}

#[test]
fn a_clock_driven_n_steps_east_and_wraps_at_the_modulus() {
    // `~.0203` steps every second Tick through `00`, `01`, `02`.
    let grid = Grid::with_shape(8, 3);
    let selected: Vec<String> = quiet_rows(grid, &["  ~.0203", "&>  C4D4"], 7)
        .into_iter()
        .map(|tick| tick[2][..2].to_string())
        .collect();
    assert_eq!(selected, ["00", "00", "C4", "C4", "D4", "D4", "00"]);
}

#[test]
fn a_read_answers_what_a_copy_answers_for_the_cells_it_reads() {
    let grid = Grid::with_shape(8, 2);
    // Empty Cells clear the Output Portal without a diagnostic.
    let empty = first(grid, &["&>01  D4", "xx"]);
    assert_eq!(empty.rows, ["&>01  D4", "        "]);
    assert!(empty.diagnostics.is_empty(), "{:?}", empty.diagnostics);
    // Read as a Copy reads it: Number before Note, so `D4` is `0xD4` and
    // `G4`, which spells no Number, is the Note. Both write back as read.
    assert_eq!(first(grid, &["&>01G4"]).rows[1], "G4      ");
    assert_eq!(first(grid, &["&>01D4"]).rows[1], "D4      ");
    // A pair that straddles two Language Units or holds a partial unit is not
    // one Language Unit.
    for partial in ["&>03C4C.+0101", "&>03C4C4D 4"] {
        let tick = first(Grid::with_shape(14, 2), &[partial, "xx"]);
        assert_eq!(&tick.rows[1][..2], "xx", "{partial:?}");
        assert!(
            tick.diagnostics.contains(&diagnostic(0, 0, &invalid("&>"))),
            "{partial:?}: {:?}",
            tick.diagnostics
        );
    }
}

#[test]
fn a_comment_at_the_selected_pair_diagnoses() {
    // The selected pair (8, 0) is the `||` that opens a Comment.
    let tick = first(Grid::with_shape(12, 2), &["&>03C4C4||E4", "xx"]);
    assert_eq!(tick.rows[1], "xx          ");
    assert_eq!(tick.diagnostics, [diagnostic(0, 0, &invalid("&>"))]);
}

#[test]
fn a_pair_straddling_a_comment_introducer_diagnoses() {
    // The selected pair (8, 0) is `|4`: the second Cell of the `||` that
    // opens the Comment at (7, 0), and the first Cell of its text.
    let tick = first(Grid::with_shape(12, 2), &["&>03C4C||4E4", "xx"]);
    assert_eq!(tick.rows[1], "xx          ");
    assert_eq!(tick.diagnostics, [diagnostic(0, 0, &invalid("&>"))]);
}

#[test]
fn a_function_a_read_copies_replaces_the_function_where_it_lands() {
    // The Read writes `.+` over the anchor of `.-0302`, which then answers as
    // Add in the same Tick. The `.+0101` it reads from is itself a root and
    // writes `02` south.
    let tick = first(Grid::with_shape(14, 3), &["&>03C4  .+0101", ".-0302"]);
    assert_eq!(
        tick.rows,
        ["&>03C4  .+0101", ".+0302  02    ", "05            "]
    );
    assert!(tick.diagnostics.is_empty(), "{:?}", tick.diagnostics);
}

#[test]
fn a_bang_a_read_reads_is_relayed_and_activates_the_root_it_lands_on() {
    // Equality writes `**` into the selected pair this Tick. The Read relays
    // it onto the anchor of Raw Play, which plays and keeps its Cells.
    let mut source = source_of(
        Grid::with_shape(14, 3),
        &["        .=0101", "&>03C4  E4", "!>007FC4"],
    );
    let plan = source.execute(Tick::ZERO);
    assert!(plan.diagnostics.is_empty(), "{:?}", plan.diagnostics);
    assert_eq!(plan.play_commands.len(), 1);
    assert_eq!(rows_of(&source)[2], "!>007FC4      ");
}

#[test]
fn an_empty_or_partially_written_operand_makes_the_read_invalid() {
    // The Language Map diagnoses the unwritten operand, and the Tick writes
    // nothing and does not report it again.
    for row in ["&>  C4", "&>0 C4", "&v  ", "&<0 ", "&^  "] {
        let rows = ["", row, "xx"];
        let source = source_of(Grid::with_shape(6, 3), &rows);
        assert!(
            source
                .language_map()
                .diagnostics()
                .any(|diagnostic| diagnostic.start() == 6),
            "{row:?}"
        );
        let tick = first(Grid::with_shape(6, 3), &rows);
        assert_eq!(tick.rows[2], "xx    ", "{row:?}");
        assert!(
            tick.diagnostics.is_empty(),
            "{row:?}: {:?}",
            tick.diagnostics
        );
    }
}

#[test]
fn a_pair_at_the_row_edge_is_read() {
    let tick = first(Grid::with_shape(10, 2), &["&>03C4D4E4"]);
    assert_eq!(tick.rows[1], "E4        ");
    assert!(tick.diagnostics.is_empty(), "{:?}", tick.diagnostics);
}

#[test]
fn a_pair_cut_short_by_the_row_edge_diagnoses_and_writes_nothing() {
    let tick = first(Grid::with_shape(9, 2), &["&>03C4D4E", "xx"]);
    assert_eq!(tick.rows[1], "xx       ");
    assert_eq!(tick.diagnostics, [diagnostic(0, 0, &invalid("&>"))]);
}

#[test]
fn a_pair_outside_the_grid_in_any_direction_diagnoses_and_writes_nothing() {
    // The Read stands at (2, 1) of a 6 by 3 Grid, so its operand is at (4, 1).
    for (spelling, n) in [
        ("&^", "02"),
        ("&v", "02"),
        ("&<", "03"),
        ("&>", "01"),
        ("&>", "FF"),
        ("&<", "FF"),
    ] {
        let row = format!("  {spelling}{n}");
        let tick = first(Grid::with_shape(6, 3), &["", &row, "xx"]);
        assert_eq!(tick.rows[2], "xx    ", "{row:?}");
        assert_eq!(
            tick.diagnostics,
            [diagnostic(2, 1, &invalid(spelling))],
            "{row:?}"
        );
    }
    // A west pair that would start one Cell before the row.
    let tick = first(Grid::with_shape(6, 2), &[" &<02", "xxxxxx"]);
    assert_eq!(tick.rows[1], "xxxxxx");
    assert_eq!(tick.diagnostics, [diagnostic(1, 0, &invalid("&<"))]);
}

#[test]
fn a_writer_before_a_north_read_in_grid_order_is_read_the_same_tick() {
    // `.+0102` at (2, 0) writes `03` into the selected pair (2, 1).
    let grid = Grid::with_shape(8, 4);
    let rows = ["  .+0102", "", "&^01"];
    assert_eq!(first(grid, &rows).rows[3], "03      ");
    let turns = turns(grid, &rows);
    turn_before(&turns, (2, 0), (0, 2));
}

#[test]
fn a_writer_before_a_west_read_in_grid_order_is_read_the_same_tick() {
    // `.+0101` at (0, 0) writes `02` into (0, 1), three pairs west of the
    // operand of the Read at (4, 1).
    let grid = Grid::with_shape(8, 3);
    let rows = [".+0101", "    &<03"];
    let tick = first(grid, &rows);
    assert_eq!(tick.rows, [".+0101  ", "02  &<03", "    02  "]);
    assert!(tick.diagnostics.is_empty(), "{:?}", tick.diagnostics);
    turn_before(&turns(grid, &rows), (0, 0), (4, 1));
}

#[test]
fn a_nested_n_that_writes_the_selected_pair_takes_its_turn_first() {
    // The nested `=^` in the north Read's slot copies `01` from below it to
    // the pair above it, and returns `01` as `n`: the Read reads what its
    // own operand wrote.
    let grid = Grid::with_shape(4, 3);
    let rows = ["", "&^=^", "  01"];
    let tick = first(grid, &rows);
    assert_eq!(tick.rows, ["  01", "&^=^", "0101"]);
    assert!(tick.diagnostics.is_empty(), "{:?}", tick.diagnostics);
    turn_before(&turns(grid, &rows), (2, 1), (0, 1));
    // The nested `.+` in the south Read's slot writes `01` below itself, the
    // pair its answer selects.
    let grid = Grid::with_shape(8, 2);
    let rows = ["&v.+0001"];
    let tick = first(grid, &rows);
    assert_eq!(tick.rows, ["&v.+0001", "0101    "]);
    assert!(tick.diagnostics.is_empty(), "{:?}", tick.diagnostics);
    turn_before(&turns(grid, &rows), (2, 0), (0, 0));
}

#[test]
fn a_writer_east_of_the_selected_pair_takes_its_turn_first() {
    // `=<` at (8, 0) copies `D4` from (10, 0) into the selected pair (6, 0).
    let grid = Grid::with_shape(12, 2);
    let rows = ["&>02C4  =<D4"];
    assert_eq!(first(grid, &rows).rows[1], "D4          ");
    let turns = turns(grid, &rows);
    turn_before(&turns, (8, 0), (0, 0));
}

#[test]
fn a_writer_below_the_selected_pair_takes_its_turn_first() {
    // The south Read selects (2, 1). `=^` at (2, 2) copies `D4` from (2, 3)
    // into it, and stands after the Read in Grid order, so the Read waits.
    let grid = Grid::with_shape(4, 4);
    let rows = ["&v01", "", "  =^", "  D4"];
    assert_eq!(first(grid, &rows).rows[1], "D4D4");
    let turns = turns(grid, &rows);
    turn_before(&turns, (2, 2), (0, 0));
}

#[test]
fn a_writer_after_a_west_read_in_grid_order_takes_its_turn_first() {
    // The west Read at (4, 0) selects (2, 0). `=^` at (2, 1) copies `E4` from
    // (2, 2) into it.
    let grid = Grid::with_shape(8, 3);
    let rows = ["    &<02", "  =^", "  E4"];
    let tick = first(grid, &rows);
    assert_eq!(tick.rows[1], "  =^E4  ");
    assert!(tick.diagnostics.is_empty(), "{:?}", tick.diagnostics);
    let turns = turns(grid, &rows);
    turn_before(&turns, (2, 1), (4, 0));
}

#[test]
fn a_writer_of_an_unselected_pair_is_not_ordered_against_the_read() {
    let grid = Grid::with_shape(10, 3);
    let rows = ["&>01C4  E4", "        =^", "        D4"];
    assert_eq!(first(grid, &rows).rows[1], "C4      =^");
    let turns = turns(grid, &rows);
    assert_eq!(turns[&(0, 0)], Some(0), "{turns:?}");
}

#[test]
fn competing_writes_to_the_selected_pair_reach_the_read_as_they_reach_the_source() {
    // `=<` at (8, 0) and `=^` at (6, 1) both write the selected pair. The
    // later Turn wins each Cell, and the Read reads the pair after both.
    let grid = Grid::with_shape(12, 3);
    let rows = ["&>02C4  =<D4", "      =^", "      E4"];
    let tick = first(grid, &rows);
    assert_eq!(&tick.rows[1][..2], &tick.rows[0][6..8]);
    assert_eq!(&tick.rows[1][..2], "E4");
    let turns = turns(grid, &rows);
    turn_before(&turns, (8, 0), (0, 0));
    turn_before(&turns, (6, 1), (0, 0));
}

#[test]
fn a_writer_that_waits_on_the_read_forms_a_cycle_and_the_rest_of_the_tick_runs() {
    // The Read writes (0, 1), which the `=>` at (2, 1) copies to (4, 1), which
    // the `=>` at (6, 1) copies over the `=^` at (8, 1). That `=^` writes the
    // selected pair, so the Read finds at its Turn that it waits on a writer
    // that waits on it. The `=>` at (10, 1) reads Cells the cycle writes, and
    // the Addition below depends on none of it.
    let tick = first(
        Grid::with_shape(12, 5),
        &["&>03C4  E4", "  =>  =>=^=>", "        D4", ".+0102"],
    );
    assert_eq!(
        tick.diagnostics,
        [
            diagnostic(0, 0, "same-Tick dependency cycle"),
            diagnostic(10, 1, "waiting on a same-Tick dependency cycle"),
        ]
    );
    assert_eq!(
        tick.rows,
        [
            "&>03C4  E4  ",
            "  =>  =>=^=>",
            "        D4  ",
            ".+0102      ",
            "03          ",
        ]
    );
}

#[test]
fn a_read_of_cells_a_stopped_cycle_writes_waits_on_the_cycle() {
    // `.+0102` at (8, 0) writes over the `=^` below it, and the `=^` copies
    // its empty input over the Addition's spelling: a cycle the schedule
    // stops. The selected pair is the Cells the Addition writes, so the Read
    // is stopped with it.
    let tick = first(
        Grid::with_shape(14, 3),
        &["        .+0102", "&>03C4  =^E4", "xx"],
    );
    assert_eq!(tick.rows[2], "xx            ");
    assert_eq!(
        tick.diagnostics,
        [
            diagnostic(8, 0, "same-Tick dependency cycle"),
            diagnostic(0, 1, "waiting on a same-Tick dependency cycle"),
        ]
    );
}

#[test]
fn a_nested_read_returns_its_pair_and_writes_its_own_output_portal() {
    // The nested Read's operand is at (4, 0), so two pairs east is the `05`.
    // The right operand of `.+` is `03`.
    let tick = first(Grid::with_shape(10, 2), &[".+&>020305"]);
    assert_eq!(tick.rows, [".+&>020305", "0805      "]);
}

#[test]
fn a_nested_read_that_reads_empty_cells_makes_its_parent_invalid() {
    let tick = first(Grid::with_shape(10, 2), &[".+&>0203  ", "xxxx"]);
    assert_eq!(tick.rows, [".+&>0203  ", "xx        "]);
    assert_eq!(
        tick.diagnostics,
        [diagnostic(0, 0, "expected a number, found \"  \"")]
    );
}

#[test]
fn a_timed_play_plays_the_note_a_nested_read_supplies() {
    // Equality's Bang at (0, 1) activates Timed Play at (2, 1). The Read's
    // operand is at (10, 1), its length is the `04` at (12, 1), and two pairs
    // east of the operand is the `C4`.
    let mut source = source_of(Grid::with_shape(16, 3), &[".=0101", "  !~017F&>0204C4"]);
    let plan = source.execute(Tick::ZERO);
    assert!(plan.diagnostics.is_empty(), "{:?}", plan.diagnostics);
    assert_eq!(
        plan.play_commands,
        [PlayCommand::Timed {
            channel: MidiChannel::try_from(0x01).expect("a MIDI channel"),
            velocity: Velocity::try_from(0x7F).expect("a MIDI data byte"),
            note: Note::try_from(60).expect("a MIDI note"),
            length: crate::source::Length::from(0x04),
        }]
    );
}
