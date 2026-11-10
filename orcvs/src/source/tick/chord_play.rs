//! Chord Play over Operand Literals, as one Tick runs it: a Bang activates the
//! root, its tones reach the Tick Plan lowest first as ordinary Play Commands,
//! and a chord that cannot be built whole sends nothing.
//!
//! Equality at (0, 0) Bangs the Play at (2, 1) on every Tick.

use lang::{Length, MidiChannel, Note, PlayCommand, Tick, Velocity};

use super::observed::{raw, source_of};
use crate::grid::Grid;

fn timed(channel: u8, velocity: u8, note: u8, length: u8) -> PlayCommand {
    PlayCommand::Timed {
        channel: MidiChannel::try_from(channel).expect("a MIDI channel"),
        velocity: Velocity::try_from(velocity).expect("a MIDI data byte"),
        note: Note::try_from(note).expect("a MIDI note"),
        length: Length::from(length),
    }
}

/// The Play Commands and diagnostic anchors the first Tick of `rows` plans.
fn tick(rows: &[&str]) -> (Vec<PlayCommand>, Vec<(usize, usize)>) {
    let mut source = source_of(Grid::with_shape(20, 3), rows);
    let plan = source.execute(Tick::ZERO);
    let anchors = plan
        .diagnostics
        .iter()
        .map(|diagnostic| (diagnostic.anchor().x(), diagnostic.anchor().y()))
        .collect();
    (plan.play_commands, anchors)
}

/// The first Tick of `play` standing on row 1, beneath the Equality that
/// Bangs it.
fn banged(play: &str) -> (Vec<PlayCommand>, Vec<(usize, usize)>) {
    tick(&[".=0101", play])
}

#[test]
fn a_bang_activates_a_raw_chord_play_once_with_its_tones_in_order() {
    let (commands, diagnostics) = banged("  !:017FC4M7CL00");

    assert_eq!(
        commands,
        vec![
            raw(1, 0x7F, 0x3C),
            raw(1, 0x7F, 0x40),
            raw(1, 0x7F, 0x43),
            raw(1, 0x7F, 0x47),
        ]
    );
    assert_eq!(diagnostics, vec![]);
}

#[test]
fn a_bang_activates_a_timed_chord_play_once_with_its_tones_in_order() {
    let (commands, diagnostics) = banged("  !#017FC4M_CL0003");

    assert_eq!(
        commands,
        vec![
            timed(1, 0x7F, 0x3C, 3),
            timed(1, 0x7F, 0x40, 3),
            timed(1, 0x7F, 0x43, 3),
        ]
    );
    assert_eq!(diagnostics, vec![]);
}

#[test]
fn an_unactivated_chord_play_sends_nothing_and_reports_nothing() {
    for play in ["!:017FC4M7CL00", "!#017FC4M7CL0003"] {
        assert_eq!(tick(&[play]), (vec![], vec![]), "{play}");
    }
}

#[test]
fn a_chord_that_cannot_be_built_whole_diagnoses_and_sends_nothing() {
    for play in [
        // G9 is 7F, so every tone above the root leaves the Note domain.
        "  !:017FG9M_CL00",
        "  !#017FG9M_CL0003",
        // Too few tones for the Voicing.
        "  !:017FC4M_SH00",
        "  !#017FC4M_D60003",
        // A channel outside its domain.
        "  !:107FC4M_CL00",
    ] {
        let (commands, diagnostics) = banged(play);
        assert_eq!(commands, vec![], "{play}");
        assert_eq!(diagnostics, vec![(2, 1)], "{play}");
    }
}

#[test]
fn a_chord_play_is_invalid_where_a_value_is_required() {
    let row = "  .+!:007FC4M_CL0001";
    let source = source_of(Grid::with_shape(20, 3), &[".=0101", row]);
    assert_eq!(source.language_map().diagnostics().count(), 1);

    let (commands, _) = banged(row);
    assert_eq!(commands, vec![]);
}
