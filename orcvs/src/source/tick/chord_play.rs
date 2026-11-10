//! Chord Plays in Source: a Bang activates one once and its tones reach the
//! Tick Plan as ordinary Play Commands, lowest first; an unactivated one, a
//! chord that cannot be built whole, and one nested where a value is
//! required send nothing.

use lang::{Length, MidiChannel, Note, PlayCommand, Velocity};

use crate::grid::Grid;
use crate::source::TickPlan;

use super::fixture::{messages, tick_by_tick};
use super::observed::raw;

fn timed(channel: u8, velocity: u8, note: u8, length: u8) -> PlayCommand {
    PlayCommand::Timed {
        channel: MidiChannel::try_from(channel).expect("a MIDI channel"),
        velocity: Velocity::try_from(velocity).expect("a MIDI data byte"),
        note: Note::try_from(note).expect("a MIDI note"),
        length: Length::from(length),
    }
}

/// Two Ticks of `chord` on row 2, with Equality's Bang on row 1 above it:
/// each Tick's plan and the Grid after it.
fn banged(chord: &str) -> (Vec<TickPlan>, Vec<Vec<String>>) {
    let width = chord.len().max(6);
    let (plans, grids, _) = tick_by_tick(Grid::with_shape(width, 4), &[".=0101", "", chord, ""], 2);
    (plans, grids)
}

#[test]
fn a_bang_activates_a_chord_play_once_with_its_tones_lowest_first() {
    // C4 major seventh with a Drop 2: G3 C4 E4 B4, sent lowest first though
    // the dropped G is the Interval Set's third tone.
    let (plans, grids) = banged("!:027FC4M7D200");
    for plan in &plans {
        assert_eq!(
            plan.play_commands,
            [
                raw(2, 0x7F, 55),
                raw(2, 0x7F, 60),
                raw(2, 0x7F, 64),
                raw(2, 0x7F, 71),
            ]
        );
        assert!(plan.diagnostics.is_empty(), "{:?}", plan.diagnostics);
    }
    // A Chord Play writes nothing to Source.
    assert_eq!(grids[0][3], "              ");

    let (plans, _) = banged("!#007FC4M_CL0104");
    assert_eq!(
        plans[0].play_commands,
        [
            timed(0, 0x7F, 64, 4),
            timed(0, 0x7F, 67, 4),
            timed(0, 0x7F, 72, 4),
        ]
    );
}

#[test]
fn an_unactivated_chord_play_sends_nothing() {
    for chord in ["!:007FC4M7CL00", "!#007FC4M7CL0004"] {
        let (plans, _, _) = tick_by_tick(Grid::with_shape(16, 2), &[chord, ""], 2);
        for plan in plans {
            assert!(plan.play_commands.is_empty(), "{chord}");
            assert!(plan.diagnostics.is_empty(), "{:?}", plan.diagnostics);
        }
    }
}

#[test]
fn a_twelve_tone_chord_reaches_the_tick_plan_whole() {
    let (plans, _) = banged("!#037FC4chCL0001");
    assert_eq!(
        plans[0].play_commands,
        (60..72)
            .map(|note| timed(3, 0x7F, note, 1))
            .collect::<Vec<_>>()
    );
}

#[test]
fn a_chord_play_beside_a_play_root_keeps_each_ones_commands_whole() {
    // A Bang reaches each root on row 2. Each root's commands stay together
    // in the plan, the chord's in its own order.
    let (plans, _, _) = tick_by_tick(
        Grid::with_shape(24, 4),
        &[".=0101         .=0101", "", "!:007FC4M_CL00 !>017FA5", ""],
        1,
    );
    let commands = &plans[0].play_commands;
    assert_eq!(commands.len(), 4, "{commands:?}");
    let chord: Vec<_> = commands
        .iter()
        .filter(|command| matches!(command, PlayCommand::Raw { channel, .. } if channel.value() == 0))
        .cloned()
        .collect();
    assert_eq!(
        chord,
        [raw(0, 0x7F, 60), raw(0, 0x7F, 64), raw(0, 0x7F, 67)]
    );
    assert!(commands.contains(&raw(1, 0x7F, 81)));
    let first = commands
        .iter()
        .position(|command| *command == raw(0, 0x7F, 60))
        .unwrap();
    assert_eq!(&commands[first..first + 3], chord.as_slice());
}

#[test]
fn a_chord_that_cannot_be_built_whole_sends_nothing_and_diagnoses() {
    for (chord, message) in [
        (
            "!:007FG9M_CL00",
            "!: would sound 83, outside the Note range 00–7F, so no tone is sent",
        ),
        (
            "!#007FC/M7D20004",
            "!# would sound -05, outside the Note range 00–7F, so no tone is sent",
        ),
        (
            "!:007FC4M_SH00",
            "Voicing SH needs 4 tones and Interval Set M_ has 3, so no tone is sent",
        ),
    ] {
        let (plans, _) = banged(chord);
        assert!(plans[0].play_commands.is_empty(), "{chord}");
        assert_eq!(messages(&plans[0]), [message], "{chord}");
    }
}

#[test]
fn a_chord_play_with_an_out_of_domain_operand_sends_nothing() {
    for chord in ["!:107FC4M7CL00", "!:0080C4M7CL00", "!#1080C4M7CL0004"] {
        let (plans, _) = banged(chord);
        assert!(plans[0].play_commands.is_empty(), "{chord}");
        assert_eq!(plans[0].diagnostics.len(), 1, "{chord}");
    }
}

#[test]
fn a_chord_play_is_invalid_where_a_value_is_required() {
    // Nested in Add's left operand, the Chord Play is refused from Source
    // alone: the Add answers nothing and no tone is sent.
    for nested in [".+!:007FC4M7CL0001", ".+!#007FC4M7CL000401"] {
        let (plans, grids) = banged(nested);
        for plan in &plans {
            assert!(plan.play_commands.is_empty(), "{nested}");
        }
        assert_eq!(grids[0][3].trim(), "", "{nested}");
    }
}
