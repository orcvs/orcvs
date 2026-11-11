//! Pitch Functions Tick by Tick: Up and Down at a root, nested, and supplying
//! a Play's note.

use lang::{Length, MidiChannel, Note, PlayCommand, Velocity};

use crate::grid::Grid;

use super::fixture::{messages, tick_by_tick};
use super::observed::{first, quiet, raw};

#[test]
fn up_and_down_write_their_note_through_the_output_portal_as_roots() {
    for (expression, answer) in [("#+C402", "D4    "), ("#-D402", "C4    ")] {
        let tick = first(Grid::with_shape(6, 2), &[expression, ""]);
        assert_eq!(tick.rows, [expression, answer], "{expression}");
        quiet(&tick);
    }
}

#[test]
fn a_nested_up_or_down_returns_its_note_and_writes_its_own_output_portal() {
    // The nested Function writes its Note south of its own anchor and returns
    // it to the root, whose answer lands south of the root: `#+C402` hands D4
    // to Down, and `#-D402` hands C4 to Up.
    for (expression, answer) in [("#-#+C40202", "C4D4      "), ("#+#-D40202", "D4C4      ")] {
        let tick = first(Grid::with_shape(10, 2), &[expression, ""]);
        assert_eq!(tick.rows, [expression, answer], "{expression}");
        quiet(&tick);
    }
}

#[test]
fn an_answer_outside_the_note_range_diagnoses_and_writes_nothing() {
    for (expression, message) in [
        (
            "#+G901",
            "#+ cannot answer semitone 128 (decimal), outside the Note range 00–7F",
        ),
        (
            "#-C/01",
            "#- cannot answer semitone -1 (decimal), outside the Note range 00–7F",
        ),
    ] {
        let (plans, grids, _) = tick_by_tick(Grid::with_shape(6, 2), &[expression, "A4"], 1);
        assert_eq!(messages(&plans[0]), [message], "{expression}");
        assert_eq!(grids[0][1], "A4    ", "{expression}");
    }
}

#[test]
fn a_nested_up_in_a_plays_note_operand_plays_the_transposed_note() {
    let (plans, grids, _) = tick_by_tick(
        Grid::with_shape(12, 4),
        &[".=0101", "", "!>007F#+C402", ""],
        1,
    );
    assert_eq!(plans[0].play_commands, vec![raw(0, 0x7F, 0x3E)]);
    assert!(
        plans[0].diagnostics.is_empty(),
        "{:?}",
        plans[0].diagnostics
    );
    assert_eq!(grids[0][3], "      D4    ");
}

#[test]
fn a_nested_down_in_a_timed_plays_note_operand_plays_the_transposed_note() {
    let (plans, grids, _) = tick_by_tick(
        Grid::with_shape(14, 4),
        &[".=0101", "", "!~007F#-D40204", ""],
        1,
    );
    assert_eq!(
        plans[0].play_commands,
        vec![PlayCommand::Timed {
            channel: MidiChannel::try_from(0).unwrap(),
            velocity: Velocity::try_from(0x7F).unwrap(),
            note: Note::try_from(0x3C).unwrap(),
            length: Length::from(4),
        }]
    );
    assert!(
        plans[0].diagnostics.is_empty(),
        "{:?}",
        plans[0].diagnostics
    );
    assert_eq!(grids[0][3], "      C4      ");
}

#[test]
fn an_up_above_a_note_slot_writes_the_note_the_play_sounds() {
    // The Output Portal of the Up is the Play's note slot, so what it writes
    // is what the Bang plays.
    let (plans, grids, _) = tick_by_tick(
        Grid::with_shape(12, 3),
        &[".=0101", "      #+C402", "!>007F"],
        1,
    );
    assert_eq!(plans[0].play_commands, vec![raw(0, 0x7F, 0x3E)]);
    assert!(
        plans[0].diagnostics.is_empty(),
        "{:?}",
        plans[0].diagnostics
    );
    assert_eq!(&grids[0][2][..8], "!>007FD4");
}

#[test]
fn a_nested_up_outside_the_note_range_sends_nothing() {
    let (plans, _, _) = tick_by_tick(
        Grid::with_shape(12, 4),
        &[".=0101", "", "!>007F#+G901", ""],
        1,
    );
    assert_eq!(plans[0].play_commands, vec![]);
    // The Up says why it failed, and the Play reports the nested computation
    // that returned it nothing, as for any failed nested Function.
    assert_eq!(
        messages(&plans[0]),
        [
            "#+ cannot answer semitone 128 (decimal), outside the Note range 00–7F",
            "nested computation at column 6, row 2 returned nothing"
        ]
    );
}
