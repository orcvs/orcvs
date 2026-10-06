//! Track, `@t index count`, read as a performer sees it Tick after Tick.
//!
//! Track reads the Cell pair `index % count` pairs east of its last operand
//! with Jump's rules, and writes what it reads through its Output Portal. The
//! pair is known only at Track's Turn, so these tests also state the order
//! that Turn takes against the writers of the Cells it reads.

use std::collections::BTreeMap;

use lang::{MidiChannel, Note, PlayCommand, Tick, Velocity};

use super::execution::ComputationState;
use super::{plan, plan_carrying};
use crate::grid::{CellIndex, Grid, Position};
use crate::source::{Cells, Source, TickPlan};

///
/// One Tick as a performer observes it: the Grid after the Tick, and each
/// diagnostic with the Grid column and row of its first Cell.
///
#[derive(Debug, PartialEq)]
struct Observed {
    rows: Vec<String>,
    diagnostics: Vec<(usize, usize, String)>,
}

/// A Source holding `rows`, each padded to the Grid's width, above blank rows
/// to the Grid's height.
fn source_of(grid: Grid, rows: &[&str]) -> Source {
    let mut source = Source::new(grid);
    let width = grid.columns();
    let text: String = rows.iter().map(|row| format!("{row:width$}")).collect();
    let text = format!("{text:count$}", count = grid.count());
    assert_eq!(text.len(), grid.count(), "the rows fit the Grid");
    for (index, byte) in text.bytes().enumerate() {
        source
            .set(
                grid.cell_index(index).expect("inside the Grid"),
                &char::from(byte).to_string(),
            )
            .unwrap();
    }
    source
}

fn rows_of(grid: Grid, source: &Source) -> Vec<String> {
    source
        .snapshot()
        .into_bytes()
        .chunks(grid.columns())
        .map(|row| String::from_utf8(row.to_vec()).expect("ASCII Source"))
        .collect()
}

fn observed(grid: Grid, source: &Source, plan: &TickPlan) -> Observed {
    Observed {
        rows: rows_of(grid, source),
        diagnostics: plan
            .diagnostics
            .iter()
            .map(|diagnostic| {
                let anchor = diagnostic.anchor();
                (anchor.x(), anchor.y(), diagnostic.message.clone())
            })
            .collect(),
    }
}

/// Runs the Ticks in `ticks` against `rows` through [`Source::execute`].
fn observe_at(grid: Grid, rows: &[&str], ticks: impl IntoIterator<Item = u64>) -> Vec<Observed> {
    let mut source = source_of(grid, rows);
    ticks
        .into_iter()
        .map(|tick| {
            let plan = source.execute(Tick::new(tick));
            observed(grid, &source, &plan)
        })
        .collect()
}

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
    anchored_turns(&source, &states)
}

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

fn diagnostic(x: usize, y: usize, message: &str) -> (usize, usize, String) {
    (x, y, message.to_string())
}

#[test]
fn track_writes_the_pair_its_operands_select_south() {
    let tick = first(Grid::with_shape(12, 2), &["@t0103C4D4E4"]);
    assert_eq!(tick.rows, ["@t0103C4D4E4", "D4          "]);
}

#[test]
fn a_nested_index_or_count_moves_the_pairs_east_with_the_last_operand() {
    // `.+0001` answers `01` as the index and writes it south of itself too.
    let index = first(Grid::with_shape(16, 2), &["@t.+000103C4D4E4"]);
    assert_eq!(index.rows, ["@t.+000103C4D4E4", "D401            "]);
    // A nested count is the last operand, so the pairs follow its own.
    let count = first(Grid::with_shape(16, 2), &["@t01.+0003C4D4E4"]);
    assert_eq!(count.rows, ["@t01.+0003C4D4E4", "D4  03          "]);
}

#[test]
fn track_answers_what_a_jump_answers_for_the_cells_it_reads() {
    let grid = Grid::with_shape(12, 2);
    // Empty Cells clear the Output Portal.
    assert_eq!(
        first(grid, &["@t0103C4  E4", "xx"]).rows,
        ["@t0103C4  E4", "            "]
    );
    // A Note spelling that is no Number is the Note.
    assert_eq!(
        first(grid, &["@t0103C4G4E4"]).rows,
        ["@t0103C4G4E4", "G4          "]
    );
    // A pair that straddles two Language Units, holds a partial unit, or
    // starts a Comment is not one Language Unit.
    for partial in ["@t0103C.+0101", "@t0103C4D 4", "@t0103C||4E4"] {
        let tick = first(Grid::with_shape(14, 2), &[partial]);
        assert_eq!(&tick.rows[1][..2], "  ", "{partial:?}");
        assert!(
            tick.diagnostics
                .contains(&diagnostic(0, 0, "@t has partial or invalid input")),
            "{partial:?}: {:?}",
            tick.diagnostics
        );
    }
}

#[test]
fn a_function_track_copies_replaces_the_function_where_it_lands() {
    // Track writes `.+` over the anchor of `.-0302`, which then answers as
    // Add in the same Tick, as a Jump's copy would make it. The `.+0101`
    // Track reads from is itself a root and writes `02` south.
    let tick = first(Grid::with_shape(14, 3), &["@t0102C4.+0101", ".-0302"]);
    assert_eq!(
        tick.rows,
        ["@t0102C4.+0101", ".+0302  02    ", "05            "]
    );
    assert!(tick.diagnostics.is_empty(), "{:?}", tick.diagnostics);
}

#[test]
fn a_bang_track_reads_is_relayed_and_activates_the_root_it_lands_on() {
    // Equality writes `**` into pair 1 this Tick. Track relays it onto the
    // anchor of Raw Play, which plays and keeps its Cells.
    let mut source = source_of(
        Grid::with_shape(14, 3),
        &["        .=0101", "@t0103C4  E4", "!>007FC4"],
    );
    let plan = source.execute(Tick::ZERO);
    assert!(plan.diagnostics.is_empty(), "{:?}", plan.diagnostics);
    assert_eq!(plan.play_commands.len(), 1);
    assert_eq!(rows_of(source.grid(), &source)[2], "!>007FC4      ");
}

#[test]
fn an_empty_operand_leaves_track_pending() {
    for rows in [["@t  03C4D4E4", "xx"], ["@t01  C4D4E4", "xx"]] {
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
fn a_zero_count_diagnoses_at_the_turn() {
    let tick = first(Grid::with_shape(12, 2), &["@t0100C4D4E4"]);
    assert_eq!(tick.rows[1], "            ");
    assert_eq!(
        tick.diagnostics,
        [diagnostic(0, 0, "@t cannot wrap at a zero count")]
    );
}

#[test]
fn a_pair_past_the_row_edge_diagnoses() {
    let tick = first(Grid::with_shape(12, 2), &["@t0304C4D4E4"]);
    assert_eq!(tick.rows[1], "            ");
    assert_eq!(
        tick.diagnostics,
        [diagnostic(0, 0, "@t has partial or invalid input")]
    );
}

#[test]
fn a_clock_driven_track_holds_each_pair_for_the_rate_and_wraps_at_the_count() {
    // `~.0203` steps every second Tick through `00`, `01`, `02`.
    let grid = Grid::with_shape(16, 2);
    let rows = ["@t~.020303C4D4E4"];
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
    let selected: Vec<String> = quiet_rows(grid, &["  ~.0103", "@t  03C4D4E4"], 4)
        .into_iter()
        .map(|tick| tick[2][..2].to_string())
        .collect();
    assert_eq!(selected, ["C4", "D4", "E4", "C4"]);
}

#[test]
fn a_writer_below_the_selected_pair_takes_its_turn_first() {
    // `&^` at (8, 1) copies `D4` from (8, 2) into pair 1. It stands after
    // Track in Grid order, so Track waits for it.
    let grid = Grid::with_shape(12, 3);
    let rows = ["@t0103C4  E4", "        &^", "        D4"];
    assert_eq!(first(grid, &rows).rows[1], "D4      &^  ");
    let turns = turns(grid, &rows);
    assert!(turns[&(8, 1)] < turns[&(0, 0)], "{turns:?}");
}

#[test]
fn a_writer_east_of_the_selected_pair_takes_its_turn_first() {
    // `&<` at (10, 0) copies `D4` from (12, 0) into pair 1.
    let grid = Grid::with_shape(14, 2);
    let rows = ["@t0104C4  &<D4"];
    assert_eq!(first(grid, &rows).rows[1], "D4            ");
    let turns = turns(grid, &rows);
    assert!(turns[&(10, 0)] < turns[&(0, 0)], "{turns:?}");
}

#[test]
fn a_writer_of_an_unselected_pair_is_not_ordered_against_track() {
    let grid = Grid::with_shape(12, 3);
    let rows = ["@t0003C4  E4", "        &^", "        D4"];
    assert_eq!(first(grid, &rows).rows[1], "C4      &^  ");
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
    let source = source_of(grid, &["@t0001  E4", ".+0000", ".x0203"]);
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
    let tick = first(Grid::with_shape(12, 2), &[".+@t01020305"]);
    assert_eq!(tick.rows, [".+@t01020305", "0805        "]);
}

#[test]
fn a_timed_play_plays_the_note_a_nested_track_supplies() {
    // Equality's Bang at (0, 1) activates Timed Play at (2, 1). Its length
    // is Track's pair 0, and Track reads pair 1.
    let mut source = source_of(Grid::with_shape(18, 3), &[".=0101", "  !~017F@t010204C4"]);
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
