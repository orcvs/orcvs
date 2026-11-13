//! Pitch Functions Tick by Tick: each at a root, nested, and supplying a
//! Play's note or a Chord Play's root and set.

use lang::{Length, MidiChannel, Note, PlayCommand, Velocity};

use crate::grid::Grid;

use super::fixture::{messages, tick_by_tick};
use super::observed::{first, quiet, quiet_plan, raw};

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
fn a_nested_up_in_a_plays_note_operand_plays_the_raised_note() {
    let (plans, grids, _) = tick_by_tick(
        Grid::with_shape(12, 4),
        &[".=0101", "", "!>007F#+C402", ""],
        1,
    );
    assert_eq!(plans[0].play_commands, vec![raw(0, 0x7F, 0x3E)]);
    quiet_plan(&plans[0]);
    assert_eq!(grids[0][3], "      D4    ");
}

#[test]
fn a_nested_down_in_a_timed_plays_note_operand_plays_the_lowered_note() {
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
    quiet_plan(&plans[0]);
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
    quiet_plan(&plans[0]);
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

#[test]
fn pick_and_stack_write_their_answers_through_the_output_portal_as_roots() {
    for (expression, answer) in [("#pC4io04", "G4      "), ("#sio0004", "M7      ")] {
        let tick = first(Grid::with_shape(8, 2), &[expression, ""]);
        assert_eq!(tick.rows, [expression, answer], "{expression}");
        quiet(&tick);
    }
}

#[test]
fn a_nested_stack_writes_its_own_output_portal_and_answers_picks_set() {
    // `#sio0003` writes MA south of its own anchor and returns it, and index
    // 02 of MA is 7, so Pick's answer lands south of the root as G4.
    let tick = first(Grid::with_shape(14, 2), &["#pC4#sio000302", ""]);
    assert_eq!(tick.rows, ["#pC4#sio000302", "G4  MA        "]);
    quiet(&tick);
}

#[test]
fn a_nested_clock_steps_pick_through_the_set_tick_by_tick() {
    let (plans, grids, _) = tick_by_tick(Grid::with_shape(12, 2), &["#pC4io~.0107", ""], 3);
    for (tick, row) in [
        (0, "C4    00    "),
        (1, "D4    01    "),
        (2, "E4    02    "),
    ] {
        assert_eq!(grids[tick][1], row, "Tick {tick}");
        quiet_plan(&plans[tick]);
    }
}

#[test]
fn a_pick_above_the_note_range_and_a_stack_no_name_fits_diagnose_and_write_nothing() {
    for (expression, message) in [
        (
            "#pG9MA01",
            "#p cannot answer semitone 131 (decimal), outside the Note range 00–7F",
        ),
        (
            "#spM0003",
            "#s finds no Interval Set for pM from degree 00 with size 03",
        ),
    ] {
        let (plans, grids, _) = tick_by_tick(Grid::with_shape(8, 2), &[expression, "A4"], 1);
        assert_eq!(messages(&plans[0]), [message], "{expression}");
        assert_eq!(grids[0][1], "A4      ", "{expression}");
    }
}

#[test]
fn a_nested_pick_in_a_plays_note_operand_plays_the_tone_it_picks() {
    let (plans, grids, _) = tick_by_tick(
        Grid::with_shape(14, 4),
        &[".=0101", "", "!>007F#pC4io04", ""],
        1,
    );
    assert_eq!(plans[0].play_commands, vec![raw(0, 0x7F, 0x43)]);
    quiet_plan(&plans[0]);
    assert_eq!(grids[0][3], "      G4      ");
}

#[test]
fn a_pick_above_a_note_slot_writes_the_note_the_play_sounds() {
    let (plans, grids, _) = tick_by_tick(
        Grid::with_shape(14, 3),
        &[".=0101", "      #pC4io04", "!>007F"],
        1,
    );
    assert_eq!(plans[0].play_commands, vec![raw(0, 0x7F, 0x43)]);
    quiet_plan(&plans[0]);
    assert_eq!(&grids[0][2][..8], "!>007FG4");
}

#[test]
fn pick_in_root_and_stack_in_set_sound_a_chord_on_a_scale_degree() {
    // Degree 4 of C major: Pick answers G4 and Stack answers 07, so the Chord
    // Play sounds G4 B4 D5 F5.
    let (plans, grids, _) = tick_by_tick(
        Grid::with_shape(26, 4),
        &[".=0101", "", "!:0064#pC4io04#sio0404CL00", ""],
        1,
    );
    assert_eq!(
        plans[0].play_commands,
        vec![
            raw(0, 0x64, 0x43),
            raw(0, 0x64, 0x47),
            raw(0, 0x64, 0x4A),
            raw(0, 0x64, 0x4D),
        ]
    );
    quiet_plan(&plans[0]);
    // Each nested Function writes its own answer south of itself as well.
    assert_eq!(grids[0][3], "      G4      07          ");
}

#[test]
fn pick_and_stack_compose_in_a_timed_chord_play() {
    // Degree 4 of C major, three tones: G4 B4 D5.
    let (plans, _, _) = tick_by_tick(
        Grid::with_shape(28, 4),
        &[".=0101", "", "!#0064#pC4io04#sio0403CL0004", ""],
        1,
    );
    let timed = |note: u8| PlayCommand::Timed {
        channel: MidiChannel::try_from(0).unwrap(),
        velocity: Velocity::try_from(0x64).unwrap(),
        note: Note::try_from(note).unwrap(),
        length: Length::from(4),
    };
    assert_eq!(
        plans[0].play_commands,
        vec![timed(0x43), timed(0x47), timed(0x4A)]
    );
    quiet_plan(&plans[0]);
}

#[test]
fn a_nested_stack_no_name_fits_sends_no_chord() {
    let (plans, _, _) = tick_by_tick(
        Grid::with_shape(20, 4),
        &[".=0101", "", "!:0064C4#spM0003CL00", ""],
        1,
    );
    assert_eq!(plans[0].play_commands, vec![]);
    assert_eq!(
        messages(&plans[0]),
        [
            "#s finds no Interval Set for pM from degree 00 with size 03",
            "nested computation at column 8, row 2 returned nothing"
        ]
    );
}
