//! Chord Play over Operand Literals, as one Tick runs it: a Bang activates the
//! root, its tones reach the Tick Plan lowest first as ordinary Play Commands,
//! and a chord that cannot be built whole sends nothing.
//!
//! Equality at (0, 0) Bangs the Play at (2, 1) on every Tick.

use lang::PlayCommand;

use super::observed::{first_tick, raw, source_of, timed};
use crate::grid::Grid;

/// The first Tick of `rows` on the Grid every test here shares.
fn tick(rows: &[&str]) -> (Vec<PlayCommand>, Vec<(usize, usize, String)>) {
    first_tick(Grid::with_shape(20, 3), rows)
}

/// The first Tick of `play` standing on row 1, beneath the Equality that
/// Bangs it.
fn banged(play: &str) -> (Vec<PlayCommand>, Vec<(usize, usize, String)>) {
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
    for (play, refusal) in [
        // G9 is 7F, so every tone above the root leaves the Note domain.
        (
            "  !:017FG9M_CL00",
            "!: cannot sound chord tone 131, outside the MIDI note range 0–127",
        ),
        (
            "  !#017FG9M_CL0003",
            "!# cannot sound chord tone 131, outside the MIDI note range 0–127",
        ),
        // Too few tones for the Voicing.
        (
            "  !:017FC4M_SH00",
            "!: cannot voice Interval Set M_ as SH: the Voicing needs 4 tones and the set has 3, \
             so no tone is sent",
        ),
        (
            "  !#017FC4M_D60003",
            "!# cannot voice Interval Set M_ as D6: the Voicing needs 4 tones and the set has 3, \
             so no tone is sent",
        ),
        // A channel outside its domain.
        (
            "  !:107FC4M_CL00",
            "MIDI channel 10 is outside the range 00–0F",
        ),
    ] {
        let (commands, diagnostics) = banged(play);
        assert_eq!(commands, vec![], "{play}");
        assert_eq!(diagnostics, vec![(2, 1, refusal.to_owned())], "{play}");
    }
}

#[test]
fn a_chord_play_is_invalid_where_a_value_is_required() {
    let row = "  .+!:007FC4M_CL0001";
    let source = source_of(Grid::with_shape(20, 3), &[".=0101", row]);
    let reported: Vec<_> = source
        .language_map()
        .diagnostics()
        .map(|diagnostic| {
            (
                diagnostic.message.clone(),
                diagnostic.anchor().x(),
                diagnostic.anchor().y(),
            )
        })
        .collect();
    // The refusal is the nesting rule, raised from Source alone and anchored
    // at the Expression the Chord Play stands in.
    assert_eq!(
        reported,
        vec![(lang::SyntaxError::NestedEffectFunction.to_string(), 2, 1)]
    );

    let (commands, _) = banged(row);
    assert_eq!(commands, vec![]);
}
