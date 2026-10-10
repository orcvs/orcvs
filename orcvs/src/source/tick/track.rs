//! Track, `&t index count`, read as a performer sees it Tick after Tick.
//!
//! Track reads the Cell pair `index % count` pairs east of its last operand
//! with Copy's rules, and writes what it reads through its Output Portal. The
//! pair is known only at Track's Turn, so these tests also state the order
//! that Turn takes against the writers of the Cells it reads.

use std::collections::{BTreeMap, BTreeSet};

use lang::{MidiChannel, Note, PlayCommand, Tick, Velocity};

use super::execution::ComputationState;
use super::observed::{
    diagnostic, first, observe_at, quiet_rows, raw_play, rows_of, source_of, turns,
};
use super::{plan, plan_carrying};
use crate::grid::{CellIndex, Grid, Position};
use crate::source::{Cells, Source};

fn anchored_turns(
    source: &Source,
    states: &[ComputationState],
) -> BTreeMap<(usize, usize), Option<usize>> {
    let map = source.shared_language_map();
    let grid = source.grid();
    map.schedule_cache()
        .schedule(grid, &map)
        .lookup
        .nodes()
        .iter()
        .zip(states)
        .map(|(node, state)| ((node.anchor.x(), node.anchor.y()), state.turn()))
        .collect()
}

#[test]
fn track_writes_the_pair_its_operands_select_south() {
    let tick = first(Grid::with_shape(12, 2), &["&t0103C4D4E4"]);
    assert_eq!(tick.rows, ["&t0103C4D4E4", "D4          "]);
}

#[test]
fn a_nested_index_or_count_moves_the_pairs_east_with_the_last_operand() {
    // `.+0001` answers `01` as the index and writes it south of itself too.
    let index = first(Grid::with_shape(16, 2), &["&t.+000103C4D4E4"]);
    assert_eq!(index.rows, ["&t.+000103C4D4E4", "D401            "]);
    // A nested count is the last operand, so the pairs follow its own.
    let count = first(Grid::with_shape(16, 2), &["&t01.+0003C4D4E4"]);
    assert_eq!(count.rows, ["&t01.+0003C4D4E4", "D4  03          "]);
}

#[test]
fn track_answers_what_a_copy_answers_for_the_cells_it_reads() {
    let grid = Grid::with_shape(12, 2);
    // Empty Cells clear the Output Portal.
    assert_eq!(
        first(grid, &["&t0103C4  E4", "xx"]).rows,
        ["&t0103C4  E4", "            "]
    );
    // A Note spelling that is no Number is the Note.
    assert_eq!(
        first(grid, &["&t0103C4G4E4"]).rows,
        ["&t0103C4G4E4", "G4          "]
    );
    // A pair that straddles two Language Units, holds a partial unit, or
    // starts a Comment is not one Language Unit.
    for partial in ["&t0103C.+0101", "&t0103C4D 4", "&t0103C||4E4"] {
        let tick = first(Grid::with_shape(14, 2), &[partial]);
        assert_eq!(&tick.rows[1][..2], "  ", "{partial:?}");
        assert!(
            tick.diagnostics
                .contains(&diagnostic(0, 0, "&t has partial or invalid input")),
            "{partial:?}: {:?}",
            tick.diagnostics
        );
    }
}

#[test]
fn a_function_track_copies_replaces_the_function_where_it_lands() {
    // Track writes `.+` over the anchor of `.-0302`, which then answers as
    // Add in the same Tick, as a Copy's write would make it. The `.+0101`
    // Track reads from is itself a root and writes `02` south.
    let tick = first(Grid::with_shape(14, 3), &["&t0102C4.+0101", ".-0302"]);
    assert_eq!(
        tick.rows,
        ["&t0102C4.+0101", ".+0302  02    ", "05            "]
    );
    assert!(tick.diagnostics.is_empty(), "{:?}", tick.diagnostics);
}

#[test]
fn a_bang_track_reads_overwrites_the_root_it_lands_on_and_activates_the_roots_aligned_with_it() {
    // Equality writes `**` into pair 1 this Tick. Track writes it over Raw
    // Play C4's anchor, so C4 does not run, and activates Raw Play D4 south
    // of it.
    let mut source = source_of(
        Grid::with_shape(14, 4),
        &["        .=0101", "&t0103C4  E4", "!>007FC4", "!>007FD4"],
    );
    let plan = source.execute(Tick::ZERO);
    assert!(plan.diagnostics.is_empty(), "{:?}", plan.diagnostics);
    assert_eq!(plan.play_commands, [raw_play(62)]);
    assert_eq!(rows_of(&source)[2], "**007FC4      ");
}

#[test]
fn an_empty_operand_makes_track_invalid() {
    // The Language Map diagnoses the unwritten operand, and the Tick writes
    // nothing and does not report it again.
    for rows in [["&t  03C4D4E4", "xx"], ["&t01  C4D4E4", "xx"]] {
        let source = source_of(Grid::with_shape(12, 2), &rows);
        assert!(
            source
                .language_map()
                .diagnostics()
                .any(|diagnostic| diagnostic.start() == 0),
            "{rows:?}"
        );
        let tick = first(Grid::with_shape(12, 2), &rows);
        assert_eq!(tick.rows[1], "xx          ", "{rows:?}");
        assert!(
            tick.diagnostics.is_empty(),
            "{rows:?}: {:?}",
            tick.diagnostics
        );
    }
}

#[test]
fn a_partially_written_operand_makes_track_invalid() {
    for row in ["&t0 03C4D4E4", "&t010 C4D4E4"] {
        let source = source_of(Grid::with_shape(12, 2), &[row, "xx"]);
        let diagnostics: Vec<_> = source.language_map().diagnostics().collect();
        assert!(
            diagnostics.iter().any(|diagnostic| {
                diagnostic.anchor().x() == 0
                    && diagnostic.anchor().y() == 0
                    && diagnostic.message == "expected a number, found \"0 \""
            }),
            "{row:?}: {diagnostics:?}"
        );
        let tick = first(Grid::with_shape(12, 2), &[row, "xx"]);
        assert_eq!(tick.rows[1], "xx          ", "{row:?}");
        assert!(
            tick.diagnostics.is_empty(),
            "{row:?}: {:?}",
            tick.diagnostics
        );
    }
}

#[test]
fn a_comment_aligned_with_the_selected_pair_diagnoses() {
    let tick = first(Grid::with_shape(12, 2), &["&t0103C4||E4", "xx"]);
    assert_eq!(tick.rows[1], "xx          ");
    assert_eq!(
        tick.diagnostics,
        [diagnostic(0, 0, "&t has partial or invalid input")]
    );
}

#[test]
fn a_zero_count_diagnoses_at_the_turn() {
    let tick = first(Grid::with_shape(12, 2), &["&t0100C4D4E4"]);
    assert_eq!(tick.rows[1], "            ");
    assert_eq!(
        tick.diagnostics,
        [diagnostic(0, 0, "&t cannot wrap at a zero count")]
    );
}

#[test]
fn a_pair_past_the_row_edge_diagnoses_as_a_copy_input_outside_the_grid_does() {
    // Both leave their Output Portal as it was and diagnose the read.
    let tick = first(Grid::with_shape(12, 2), &["&t0304C4D4E4", "xx"]);
    assert_eq!(tick.rows[1], "xx          ");
    assert_eq!(
        tick.diagnostics,
        [diagnostic(0, 0, "&t has partial or invalid input")]
    );
    // `=>` at (0, 0) reads the two Cells west of it, outside the Grid.
    let copy = first(Grid::with_shape(12, 2), &["=>", "xx"]);
    assert_eq!(copy.rows[1], "xx          ");
    assert_eq!(
        copy.diagnostics,
        [diagnostic(0, 0, "=> has partial or invalid input")]
    );
}

#[test]
fn a_selected_pair_straddling_the_row_edge_diagnoses() {
    let tick = first(Grid::with_shape(11, 2), &["&t0203C4D4E", "xx"]);
    assert_eq!(tick.rows[1], "xx         ");
    assert_eq!(
        tick.diagnostics,
        [diagnostic(0, 0, "&t has partial or invalid input")]
    );
}

#[test]
fn a_clock_driven_track_holds_each_pair_for_the_rate_and_wraps_at_the_count() {
    // `~.0203` steps every second Tick through `00`, `01`, `02`.
    let grid = Grid::with_shape(16, 2);
    let rows = ["&t~.020303C4D4E4"];
    let selected: Vec<String> = quiet_rows(grid, &rows, 7)
        .into_iter()
        .map(|tick| tick[1][..2].to_string())
        .collect();
    assert_eq!(selected, ["C4", "C4", "D4", "D4", "E4", "E4", "C4"]);
    // Tick 256 is step 128, and 128 % 3 is 2.
    let late = observe_at(grid, &rows, [256]).remove(0);
    assert_eq!(late.rows[1], "E402            ");
}

#[test]
fn a_clock_writing_the_index_is_read_the_same_tick() {
    let grid = Grid::with_shape(12, 3);
    let selected: Vec<String> = quiet_rows(grid, &["  ~.0103", "&t  03C4D4E4"], 4)
        .into_iter()
        .map(|tick| tick[2][..2].to_string())
        .collect();
    assert_eq!(selected, ["C4", "D4", "E4", "C4"]);
}

#[test]
fn a_writer_below_the_selected_pair_takes_its_turn_first() {
    // `=^` at (8, 1) copies `D4` from (8, 2) into pair 1. It stands after
    // Track in Grid order, so Track waits for it.
    let grid = Grid::with_shape(12, 3);
    let rows = ["&t0103C4  E4", "        =^", "        D4"];
    assert_eq!(first(grid, &rows).rows[1], "D4      =^  ");
    let turns = turns(grid, &rows);
    assert!(turns[&(8, 1)] < turns[&(0, 0)], "{turns:?}");
}

#[test]
fn a_deferred_track_and_independent_addition_run_beside_a_stopped_cycle() {
    let tick = first(
        Grid::with_shape(24, 5),
        &[
            "&t0103C4  E4",
            "        =^      .+0102",
            "        D4      =v",
            "                =^",
        ],
    );
    assert_eq!(
        tick.rows,
        [
            "&t0103C4D4E4            ",
            "D4      =^      .+0102  ",
            "        D4      03      ",
            "                =^      ",
            "                        ",
        ]
    );
    assert_eq!(
        tick.diagnostics,
        [diagnostic(16, 2, "same-Tick dependency cycle")]
    );
}

#[test]
fn a_writer_east_of_the_selected_pair_takes_its_turn_first() {
    // `=<` at (10, 0) copies `D4` from (12, 0) into pair 1.
    let grid = Grid::with_shape(14, 2);
    let rows = ["&t0104C4  =<D4"];
    assert_eq!(first(grid, &rows).rows[1], "D4            ");
    let turns = turns(grid, &rows);
    assert!(turns[&(10, 0)] < turns[&(0, 0)], "{turns:?}");
}

#[test]
fn a_writer_of_an_unselected_pair_is_not_ordered_against_track() {
    let grid = Grid::with_shape(12, 3);
    let rows = ["&t0003C4  E4", "        =^", "        D4"];
    assert_eq!(first(grid, &rows).rows[1], "C4      =^  ");
    let turns = turns(grid, &rows);
    assert_eq!(turns[&(0, 0)], Some(0), "{turns:?}");
    assert!(turns[&(0, 0)] < turns[&(8, 1)], "{turns:?}");
}

#[test]
fn a_writer_that_waits_on_track_forms_a_cycle_and_the_rest_of_the_tick_runs() {
    // Track writes `.+0000`'s left operand, and `.+` writes Track's pair 0:
    // `.+` waits on Track before the Tick, and Track finds at its Turn that
    // it waits on `.+`. `.x0203` stands apart and still publishes.
    let grid = Grid::with_shape(12, 4);
    let source = source_of(grid, &["&t0001  E4", ".+0000", ".x0203"]);
    let cell = |x, y| grid.index(grid.position(x, y).expect("inside the Grid"));
    let site = |x, y| grid.position(x, y).expect("inside the Grid");
    let destinations: BTreeMap<CellIndex, Vec<Position>> = [
        (cell(0, 0), vec![site(2, 1)]),
        (cell(0, 1), vec![site(6, 0)]),
    ]
    .into();
    let map = source.shared_language_map();
    let bytes = source.snapshot();
    let (plan, states) = plan_carrying(
        grid,
        Cells::of(bytes.as_bytes()),
        &map,
        &BTreeSet::new(),
        Tick::ZERO,
        &destinations,
    );
    let messages: Vec<_> = plan
        .diagnostics
        .iter()
        .map(|diagnostic| diagnostic.message.as_str())
        .collect();
    assert_eq!(messages, ["same-Tick dependency cycle"], "{messages:?}");
    let turns = anchored_turns(&source, &states);
    assert_eq!(turns[&(0, 0)], None);
    assert_eq!(turns[&(0, 1)], None);
    assert!(turns[&(0, 2)].is_some());
    let written: Vec<_> = plan.writes.iter().map(|write| write.cell).collect();
    assert_eq!(written, [cell(0, 3), cell(1, 3)]);
}

#[test]
fn a_nested_track_returns_its_pair_and_writes_its_own_output_portal() {
    // The right operand of `.+` is Track's pair 0, `03`; Track reads pair 1.
    let tick = first(Grid::with_shape(12, 2), &[".+&t01020305"]);
    assert_eq!(tick.rows, [".+&t01020305", "0805        "]);
}

#[test]
fn a_timed_play_plays_the_note_a_nested_track_supplies() {
    // Equality's Bang at (0, 1) activates Timed Play at (2, 1). Its length
    // is Track's pair 0, and Track reads pair 1.
    let mut source = source_of(Grid::with_shape(18, 3), &[".=0101", "  !~017F&t010204C4"]);
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

#[test]
fn a_track_reading_cells_a_stopped_cycle_writes_waits_on_the_cycle() {
    // `.+0102` at (8, 0) writes over the `=^` below it, and the `=^` copies
    // its empty input over the Addition's spelling: a cycle the schedule
    // stops. Track's pair 1 is the Cells the Addition writes, so Track
    // depends on the stopped Addition and is stopped with it, as a Copy
    // reading those Cells is.
    let grid = Grid::with_shape(14, 3);
    let tick = first(grid, &["        .+0102", "&t0103C4=^E4", "xx"]);
    assert_eq!(tick.rows[2], "xx            ");
    assert_eq!(
        tick.diagnostics,
        [
            diagnostic(8, 0, "same-Tick dependency cycle"),
            diagnostic(0, 1, "waiting on a same-Tick dependency cycle"),
        ]
    );
    // The `=>` at (10, 1) reads the same Cells through an anchored Input
    // Portal.
    let copy = first(grid, &["        .+0102", "        =^=>"]);
    assert_eq!(
        copy.diagnostics,
        [
            diagnostic(8, 0, "same-Tick dependency cycle"),
            diagnostic(10, 1, "waiting on a same-Tick dependency cycle"),
        ]
    );
}

#[test]
fn a_suppressed_nested_count_is_read_as_its_two_cells_and_the_pairs_follow_them() {
    // `.+0003` writes `03` over the nested `.+`'s anchor, which suppresses
    // it: Track's count is the `03` it reads there, so its last operand ends
    // two Cells after that anchor and pair 1 is the `02` the suppressed
    // Function left behind.
    let tick = first(Grid::with_shape(16, 3), &["    .+0003", "&t01.+0102C4D4E4"]);
    assert!(tick.diagnostics.is_empty(), "{:?}", tick.diagnostics);
    assert_eq!(
        tick.rows,
        ["    .+0003      ", "&t01030102C4D4E4", "02              "]
    );
    // The same Cells written that way in the Source read the same pair.
    let written = first(Grid::with_shape(16, 2), &["&t01030102C4D4E4"]);
    assert_eq!(written.rows[1], tick.rows[2]);
}

#[test]
fn a_track_whose_nested_count_is_suppressed_waits_for_the_writer_of_the_pair_after_it() {
    // `.+0003` at (8, 0) writes `03` over the anchor of the nested `.+0102`,
    // which suppresses it, so the count ends at column 10 and pair 2 is the
    // empty pair at column 14, past the Cells the suppressed Function
    // claimed. The `=^` at (14, 2) writes `D4` there after Track in Grid
    // order, so Track waits for it and reads `D4` on its retried Turn.
    let grid = Grid::with_shape(20, 4);
    let rows = [
        "        .+0003",
        "&t.+0002.+0102  E4F4",
        "              =^",
        "              D4",
    ];
    let tick = first(grid, &rows);
    assert!(tick.diagnostics.is_empty(), "{:?}", tick.diagnostics);
    assert_eq!(
        tick.rows,
        [
            "        .+0003      ",
            "&t.+0002030102D4E4F4",
            "D402          =^    ",
            "              D4    ",
        ]
    );
    let source = source_of(grid, &rows);
    let map = source.shared_language_map();
    let bytes = source.snapshot();
    let (_, states) = plan(
        grid,
        Cells::of(bytes.as_bytes()),
        &map,
        &BTreeSet::new(),
        Tick::ZERO,
    );
    let schedule = map.schedule_cache().schedule(grid, &map);
    let nodes = schedule.lookup.nodes();
    let interpretations: BTreeMap<_, _> = nodes
        .iter()
        .zip(&states)
        .map(|(node, state)| ((node.anchor.x(), node.anchor.y()), state.interpretations()))
        .collect();
    // The nested index completes once and is not interpreted again when
    // Track retries; the suppressed count is never interpreted.
    assert_eq!(
        interpretations,
        BTreeMap::from([
            ((0, 1), 1),
            ((2, 1), 1),
            ((8, 0), 1),
            ((8, 1), 0),
            ((14, 2), 1),
        ])
    );
    // The schedule built before the Tick orders Track ahead of the `=^`, so
    // the `=^` taking the earlier Turn is Track's wait.
    let scheduled = |x, y| {
        schedule
            .order
            .iter()
            .position(|&index| (nodes[index].anchor.x(), nodes[index].anchor.y()) == (x, y))
    };
    assert!(scheduled(0, 1) < scheduled(14, 2));
    let turns = anchored_turns(&source, &states);
    assert!(turns[&(14, 2)] < turns[&(0, 1)], "{turns:?}");
}

#[test]
fn a_nested_track_that_reads_empty_cells_makes_its_parent_invalid() {
    // Track reads pair 1, which is empty. It clears its own Output Portal
    // and returns the empty Cells, so the Addition's operand is unwritten:
    // the Addition writes nothing and is diagnosed.
    let tick = first(Grid::with_shape(12, 2), &[".+&t010203  ", "xxxx"]);
    assert_eq!(tick.rows, [".+&t010203  ", "xx          "]);
    assert_eq!(
        tick.diagnostics,
        [diagnostic(0, 0, "expected a number, found \"  \"")]
    );
}

#[test]
fn a_count_a_portal_writes_this_tick_is_read_at_the_turn() {
    // Track's count is empty in the Source. `.+0101` writes `02` into it
    // before Track's Turn, so index `03` selects pair 1.
    let grid = Grid::with_shape(12, 3);
    let rows = ["    .+0101", "&t03  C4D4E4"];
    assert_eq!(
        quiet_rows(grid, &rows, 1),
        [["    .+0101  ", "&t0302C4D4E4", "D4          "]]
    );
}

#[test]
fn a_writer_that_waits_on_track_through_the_source_forms_a_cycle() {
    // Track writes (0, 1), which the `=>` at (2, 1) copies to (4, 1), which
    // the `=>` at (6, 1) copies over the `=^` at (8, 1). That `=^` writes
    // Track's pair 1, so Track finds at its Turn that it waits on a writer
    // that waits on it. The `=>` at (10, 1) reads Cells the cycle writes and
    // waits on it, and the Addition below depends on none of it.
    let tick = first(
        Grid::with_shape(14, 5),
        &["&t0103C4  E4", "  =>  =>=^=>", "        D4", ".+0102"],
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
            "&t0103C4  E4  ",
            "  =>  =>=^=>  ",
            "        D4    ",
            ".+0102        ",
            "03            ",
        ]
    );
}

#[test]
fn a_zero_count_a_portal_writes_this_tick_diagnoses_at_the_turn() {
    // `.+0000` writes `00` into Track's empty count before Track's Turn.
    let tick = first(
        Grid::with_shape(12, 3),
        &["    .+0000", "&t03  C4D4E4", "xx"],
    );
    assert_eq!(tick.rows[1], "&t0300C4D4E4");
    assert_eq!(tick.rows[2], "xx          ");
    assert_eq!(
        tick.diagnostics,
        [diagnostic(0, 1, "&t cannot wrap at a zero count")]
    );
}

#[test]
fn a_count_of_one_selects_the_first_pair_for_every_index() {
    for index in ["00", "01", "07", "FF"] {
        let row = format!("&t{index}01C4D4");
        let tick = first(Grid::with_shape(10, 2), &[&row]);
        assert_eq!(tick.rows[1], "C4        ", "{index}");
        assert!(
            tick.diagnostics.is_empty(),
            "{index}: {:?}",
            tick.diagnostics
        );
    }
}

#[test]
fn all_empty_pairs_clear_the_output_portal_without_a_diagnostic() {
    for index in ["00", "01", "02"] {
        let row = format!("&t{index}03      ");
        let tick = first(Grid::with_shape(12, 2), &[&row, "xx"]);
        assert_eq!(tick.rows[1], "            ", "{index}");
        assert!(
            tick.diagnostics.is_empty(),
            "{index}: {:?}",
            tick.diagnostics
        );
    }
}

#[test]
fn a_partial_write_to_the_selected_pair_completes_it_cell_by_cell() {
    // `=^` at (7, 1) copies `4D` into Cells 7 and 8. Cell 7 already holds
    // `4`, and Cell 8 begins pair 1, whose Cell 9 holds `4` in the Source:
    // Track reads the `D4` the write and the Source make together.
    let grid = Grid::with_shape(12, 3);
    let tick = first(grid, &["&t0103C4 4E4", "       =^", "       4D"]);
    assert_eq!(tick.rows[0], "&t0103C4D4E4");
    assert_eq!(&tick.rows[1][..2], "D4");
}

#[test]
fn competing_writes_to_the_selected_pair_reach_track_as_they_reach_the_source() {
    // `=<` at (10, 0) and `=^` at (8, 1) both write pair 1. The later Turn
    // wins each Cell, and Track reads the pair after both.
    let grid = Grid::with_shape(14, 3);
    let rows = ["&t0104C4  =<D4", "        =^", "        E4"];
    let tick = first(grid, &rows);
    assert_eq!(&tick.rows[1][..2], &tick.rows[0][8..10]);
    assert_eq!(&tick.rows[1][..2], "E4");
    let turns = turns(grid, &rows);
    assert!(turns[&(10, 0)] < turns[&(0, 0)], "{turns:?}");
    assert!(turns[&(8, 1)] < turns[&(0, 0)], "{turns:?}");
}

#[test]
fn a_track_that_waits_reads_its_nested_index_again_unchanged() {
    // `.+0001` answers Track's index `01`, then Track waits for the `=^` that
    // writes pair 1. Its retried Turn reads the same Return, and the
    // Addition is interpreted once.
    let grid = Grid::with_shape(16, 3);
    let rows = ["&t.+000103C4  E4", "            =^", "            D4"];
    let source = source_of(grid, &rows);
    let map = source.shared_language_map();
    let bytes = source.snapshot();
    let (plan, states) = plan(
        grid,
        Cells::of(bytes.as_bytes()),
        &map,
        &BTreeSet::new(),
        Tick::ZERO,
    );
    assert!(plan.diagnostics.is_empty(), "{:?}", plan.diagnostics);
    let nodes = map.schedule_cache().schedule(grid, &map).lookup.nodes();
    let addition = nodes
        .iter()
        .position(|node| (node.anchor.x(), node.anchor.y()) == (2, 0))
        .expect("the nested Addition");
    assert_eq!(states[addition].interpretations(), 1);
    assert_eq!(first(grid, &rows).rows[1], "D401        =^  ");
    let turns = anchored_turns(&source, &states);
    assert!(turns[&(12, 1)] < turns[&(0, 0)], "{turns:?}");
}

#[test]
fn a_cycle_a_track_finds_after_another_track_waited_is_diagnosed_as_one_found_first() {
    // The Track at (0, 0) waits for the `=^` at (8, 1), so the rest of the
    // Tick is ordered from its Turn on. The Track at (0, 3) then finds that
    // its pair 1 is written by the `=^` at (8, 4), which waits on it through
    // the two `=>`. The `=>` at (10, 4) reads Cells the cycle writes, and the
    // Addition depends on none of it.
    let tick = first(
        Grid::with_shape(14, 8),
        &[
            "&t0103C4  E4",
            "        =^",
            "        D4",
            "&t0103C4  E4",
            "  =>  =>=^=>",
            "        D4",
            ".+0102",
        ],
    );
    assert_eq!(
        tick.diagnostics,
        [
            diagnostic(0, 3, "same-Tick dependency cycle"),
            diagnostic(10, 4, "waiting on a same-Tick dependency cycle"),
        ]
    );
    assert_eq!(
        tick.rows,
        [
            "&t0103C4D4E4  ",
            "D4      =^    ",
            "        D4    ",
            "&t0103C4  E4  ",
            "  =>  =>=^=>  ",
            "        D4    ",
            ".+0102        ",
            "03            ",
        ]
    );
}

#[test]
fn a_cycle_discovered_by_a_nested_track_stops_its_sibling() {
    let grid = Grid::with_shape(18, 4);
    let source = source_of(grid, &[".+&t0304.x0203", "=^        =>", ".+0102"]);
    let cell = |x, y| grid.index(grid.position(x, y).expect("inside the Grid"));
    let site = |x, y| grid.position(x, y).expect("inside the Grid");
    let destinations: BTreeMap<CellIndex, Vec<Position>> = [(cell(0, 1), vec![site(14, 0)])].into();
    let map = source.shared_language_map();
    let bytes = source.snapshot();
    let (planned, states) = plan_carrying(
        grid,
        Cells::of(bytes.as_bytes()),
        &map,
        &BTreeSet::new(),
        Tick::ZERO,
        &destinations,
    );
    let turns = anchored_turns(&source, &states);
    assert_eq!(
        turns[&(8, 0)],
        None,
        "a stopped Expression's sibling takes no Turn"
    );
    assert_eq!(turns[&(10, 1)], None, "the sibling's consumer also stops");
    assert!(turns[&(0, 2)].is_some(), "the independent Expression plays");
    assert!(!planned.writes.iter().any(|write| write.cell == cell(8, 1)));
    let independent: Vec<_> = planned
        .writes
        .iter()
        .filter(|write| write.cell == cell(0, 3) || write.cell == cell(1, 3))
        .map(|write| write.content.byte())
        .collect();
    assert_eq!(independent, b"03");
}

#[test]
fn a_late_cycle_preserves_a_completed_nested_operands_write() {
    let grid = Grid::with_shape(22, 4);
    let source = source_of(grid, &[".+&t.+000304.x0203", "=^            =>", ".+0102"]);
    let cell = |x, y| grid.index(grid.position(x, y).expect("inside the Grid"));
    let site = |x, y| grid.position(x, y).expect("inside the Grid");
    let destinations: BTreeMap<CellIndex, Vec<Position>> = [(cell(0, 1), vec![site(18, 0)])].into();
    let map = source.shared_language_map();
    let bytes = source.snapshot();
    let (planned, states) = plan_carrying(
        grid,
        Cells::of(bytes.as_bytes()),
        &map,
        &BTreeSet::new(),
        Tick::ZERO,
        &destinations,
    );
    assert!(
        planned
            .diagnostics
            .iter()
            .any(|diagnostic| { diagnostic.message == "same-Tick dependency cycle" }),
        "the selected writer closes an actual cycle"
    );
    let turns = anchored_turns(&source, &states);
    assert_eq!(
        turns[&(4, 0)],
        Some(0),
        "the nested index settles before the read is known"
    );
    for anchor in [(0, 0), (2, 0), (12, 0), (0, 1), (14, 1)] {
        assert_eq!(
            turns[&anchor], None,
            "{anchor:?} is stopped by the late cycle"
        );
    }
    assert!(turns[&(0, 2)].is_some(), "the independent Expression plays");
    let written: BTreeMap<_, _> = planned
        .writes
        .iter()
        .map(|write| (write.cell, write.content.byte()))
        .collect();
    assert_eq!(
        written,
        BTreeMap::from([
            (cell(4, 1), b'0'),
            (cell(5, 1), b'3'),
            (cell(0, 3), b'0'),
            (cell(1, 3), b'3'),
        ]),
        "the completed index and independent Expression keep their writes"
    );
}

#[test]
fn a_completed_operands_consumer_survives_a_late_cycle_after_an_earlier_wait() {
    let grid = Grid::with_shape(24, 5);
    let cell = |x, y| grid.index(grid.position(x, y).expect("inside the Grid"));
    let site = |x, y| grid.position(x, y).expect("inside the Grid");
    for earlier_wait in ["", "&t0001"] {
        let source = source_of(
            grid,
            &[earlier_wait, ".+&t.+000304.x0203", "=^    =>", ".x0102"],
        );
        let destinations: BTreeMap<CellIndex, Vec<Position>> = [
            (cell(0, 0), vec![site(18, 4)]),
            (cell(0, 2), vec![site(18, 1)]),
            (cell(0, 3), vec![site(6, 0)]),
        ]
        .into();
        let map = source.shared_language_map();
        let bytes = source.snapshot();
        let (planned, states) = plan_carrying(
            grid,
            Cells::of(bytes.as_bytes()),
            &map,
            &BTreeSet::new(),
            Tick::ZERO,
            &destinations,
        );
        let turns = anchored_turns(&source, &states);
        assert!(
            turns[&(4, 1)].is_some(),
            "the nested index completes: {earlier_wait:?}"
        );
        assert!(
            turns[&(6, 2)].is_some(),
            "its independent consumer takes a Turn: {earlier_wait:?}"
        );
        let written: BTreeMap<_, _> = planned
            .writes
            .iter()
            .map(|write| (write.cell, write.content.byte()))
            .collect();
        assert_eq!((written[&cell(8, 2)], written[&cell(9, 2)]), (b'0', b'3'));
        assert_eq!(turns[&(12, 1)], None, "the unfinished sibling stops");
        assert!(
            planned
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.message == "same-Tick dependency cycle")
        );
    }
}
