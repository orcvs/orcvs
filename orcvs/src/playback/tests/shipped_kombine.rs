//! The Kombine Source File in `examples/`, loaded exactly as shipped and run
//! through Source/Tick and Playback, and again after the edit
//! `examples/kombine.md` advertises.
//!
//! One Timed Play is banged every Tick on channel `00`. Its note comes from a
//! three-step pitch Track and its velocity from a four-step accent Track, so
//! the loud accent moves across the triad and both Tracks start together again
//! every twelve Ticks. The expected deliveries are written out Tick by Tick
//! rather than derived, so a misreading of the Tracks shared by the test and
//! Playback cannot pass both.

use super::{
    HandDrivenRun, assert_guide_draws, functions_mapped, note_on, replace_at, stop,
    tick_diagnostics,
};
use crate::playback::{InMemoryOutputAdapter, OutputCommand};
use crate::source::{SourceCommander, file};

/// The shipped Kombine Source File's text.
const KOMBINE: &str = include_str!("../../../../examples/kombine.orcvs");

/// The guide beside it, whose diagrams the edited rows must match.
const GUIDE: &str = include_str!("../../../../examples/kombine.md");

/// The Play's channel: the Built-in Synth's sine voice.
const SINE: u8 = 0x00;

/// The pitch Track's Notes, C4, E4 and G4, as MIDI notes.
const C4: u8 = 60;
const E4: u8 = 64;
const G4: u8 = 67;

/// The row of the pitch Track, below its `Pitch` Comment.
const PITCH_ROW: usize = 6;

/// The row of the accent Track, below its `Accent` Comment.
const ACCENT_ROW: usize = 10;

///
/// Runs `text` from Tick 0 and asserts each Tick's delivery against
/// `expected`, in order, with no Tick diagnosing anything.
///
fn assert_delivers(text: &str, expected: &[Vec<OutputCommand>]) {
    let source = SourceCommander::with_source(
        file::read(text.as_bytes()).expect("the Kombine is a Source File"),
    );
    let adapter = InMemoryOutputAdapter::default();
    let mut run = HandDrivenRun::new(source, adapter.clone());
    run.begin_run();

    for (tick, delivery) in expected.iter().enumerate() {
        let plan = run.run_tick(tick as u64);
        assert_eq!(tick_diagnostics(&plan), vec![], "Tick {tick}'s diagnostics");
        assert_eq!(
            adapter.command_lists().last(),
            Some(delivery),
            "Tick {tick}'s delivery"
        );
    }
    assert_eq!(adapter.command_lists().len(), expected.len(), "deliveries");
}

///
/// `text` with each of `edits`, a row, a column, the Cells there and what
/// replaces them, applied in order: edits the guide documents, applied to the
/// shipped file.
///
fn edited(text: &str, edits: &[(usize, usize, &str, &str)]) -> String {
    let mut rows: Vec<String> = text.lines().map(str::to_owned).collect();
    for &(row, column, before, after) in edits {
        replace_at(&mut rows, row, column, before, after);
    }
    let mut text = rows.join("\n");
    text.push('\n');
    text
}

///
/// The Cells of the Tracks' data, as column and row in Grid order: the pairs
/// east of each Track's `count`, the pitch Track's six Cells and the accent
/// Track's eight.
///
fn track_data() -> Vec<(usize, usize)> {
    (10..16)
        .map(|column| (column, PITCH_ROW))
        .chain((10..18).map(|column| (column, ACCENT_ROW)))
        .collect()
}

///
/// Asserts the Language Map diagnoses `text` before its first Tick exactly
/// as it diagnoses the shipped file: no Function, and each Cell of the
/// Tracks' data twice, once as a character that starts no Language Unit and
/// once as an unknown Function.
///
fn assert_diagnoses_only_the_track_data(text: &str) {
    let source = SourceCommander::with_source(
        file::read(text.as_bytes()).expect("the Kombine is a Source File"),
    );
    assert_eq!(functions_mapped(&source), vec![]);
    source.read_source(|source| {
        let map = source.language_map();
        let in_grid_order = |mut anchors: Vec<(usize, usize)>| {
            anchors.sort_by_key(|&(column, row)| (row, column));
            anchors
        };
        let lexical = in_grid_order(
            map.lexical_diagnostics()
                .map(|d| (d.anchor().x(), d.anchor().y()))
                .collect(),
        );
        assert_eq!(lexical, track_data(), "the characters diagnosed");
        let unknown = in_grid_order(
            map.diagnostics()
                .filter(|d| d.message.starts_with("unknown function"))
                .map(|d| (d.anchor().x(), d.anchor().y()))
                .collect(),
        );
        assert_eq!(unknown, track_data(), "the unknown Functions diagnosed");
        assert_eq!(
            map.diagnostics().count(),
            2 * track_data().len(),
            "every diagnosis"
        );
    });
}

///
/// Before the first Tick the Language Map diagnoses no Function. It
/// diagnoses each Cell of the Tracks' data, the pairs east of each `count`,
/// as a character that starts no Language Unit and as an unknown Function: a
/// Track claims only its operands, so the Cells it reads are ordinary Source,
/// where a standalone Note or Number is invalid. The guide draws every row of
/// the file as shipped.
///
#[test]
fn kombine_opens_diagnosing_only_the_track_data_and_draws_as_shipped() {
    assert_diagnoses_only_the_track_data(KOMBINE);
    for (row, line) in KOMBINE.lines().enumerate() {
        assert_guide_draws(GUIDE, row, line);
    }
}

///
/// A Kombine whose Tracks hold no data is diagnosed differently, so the
/// assertion above cannot pass on a Map that diagnoses nothing.
///
#[test]
#[should_panic(expected = "the characters diagnosed")]
fn kombine_without_track_data_is_diagnosed_differently() {
    assert_diagnoses_only_the_track_data(&edited(
        KOMBINE,
        &[
            (PITCH_ROW, 10, "C4E4G4", ""),
            (ACCENT_ROW, 10, "70305040", ""),
        ],
    ));
}

///
/// The first twelve Ticks and the two that start them again: each
/// Tick's Note stops on the next as that Tick's Note starts, and the loud
/// `70` lands on C4 at Tick 0, E4 at 4, G4 at 8 and C4 again at 12.
///
#[test]
fn kombine_cycles_three_pitches_against_four_velocities_for_twelve_ticks() {
    let expected: [Vec<OutputCommand>; 14] = [
        vec![note_on(SINE, 0x70, C4)],
        vec![stop(SINE, C4), note_on(SINE, 0x30, E4)],
        vec![stop(SINE, E4), note_on(SINE, 0x50, G4)],
        vec![stop(SINE, G4), note_on(SINE, 0x40, C4)],
        vec![stop(SINE, C4), note_on(SINE, 0x70, E4)],
        vec![stop(SINE, E4), note_on(SINE, 0x30, G4)],
        vec![stop(SINE, G4), note_on(SINE, 0x50, C4)],
        vec![stop(SINE, C4), note_on(SINE, 0x40, E4)],
        vec![stop(SINE, E4), note_on(SINE, 0x70, G4)],
        vec![stop(SINE, G4), note_on(SINE, 0x30, C4)],
        vec![stop(SINE, C4), note_on(SINE, 0x50, E4)],
        vec![stop(SINE, E4), note_on(SINE, 0x40, G4)],
        vec![stop(SINE, G4), note_on(SINE, 0x70, C4)],
        vec![stop(SINE, C4), note_on(SINE, 0x30, E4)],
    ];
    assert_delivers(KOMBINE, &expected);
}

///
/// The guide's three-step edit: the accent Clock's `steps` and the Track's
/// `count` both become `03`, and the fourth velocity is deleted, so both
/// Tracks wrap together every three Ticks and the loud `70` always lands on C4.
///
#[test]
fn three_accent_steps_keep_the_loud_accent_on_c() {
    let text = edited(
        KOMBINE,
        &[
            (ACCENT_ROW, 7, "4", "3"),
            (ACCENT_ROW, 9, "4", "3"),
            (ACCENT_ROW, 16, "40", ""),
        ],
    );
    let accent = text.lines().nth(ACCENT_ROW).unwrap_or_default();
    assert_guide_draws(GUIDE, ACCENT_ROW, accent);
    let expected: [Vec<OutputCommand>; 10] = [
        vec![note_on(SINE, 0x70, C4)],
        vec![stop(SINE, C4), note_on(SINE, 0x30, E4)],
        vec![stop(SINE, E4), note_on(SINE, 0x50, G4)],
        vec![stop(SINE, G4), note_on(SINE, 0x70, C4)],
        vec![stop(SINE, C4), note_on(SINE, 0x30, E4)],
        vec![stop(SINE, E4), note_on(SINE, 0x50, G4)],
        vec![stop(SINE, G4), note_on(SINE, 0x70, C4)],
        vec![stop(SINE, C4), note_on(SINE, 0x30, E4)],
        vec![stop(SINE, E4), note_on(SINE, 0x50, G4)],
        vec![stop(SINE, G4), note_on(SINE, 0x70, C4)],
    ];
    assert_delivers(&text, &expected);
}

///
/// The guide's warning about changing only `count`: the Clock still counts
/// `00` to `03` and the Track plays pair `index % 03`, so the velocities run
/// `70 30 50 70`, the loud accent twice in a row, and drift across the triad.
///
#[test]
fn only_count_at_three_plays_the_loud_accent_twice_in_a_row() {
    let text = edited(KOMBINE, &[(ACCENT_ROW, 9, "4", "3")]);
    let expected: [Vec<OutputCommand>; 13] = [
        vec![note_on(SINE, 0x70, C4)],
        vec![stop(SINE, C4), note_on(SINE, 0x30, E4)],
        vec![stop(SINE, E4), note_on(SINE, 0x50, G4)],
        vec![stop(SINE, G4), note_on(SINE, 0x70, C4)],
        vec![stop(SINE, C4), note_on(SINE, 0x70, E4)],
        vec![stop(SINE, E4), note_on(SINE, 0x30, G4)],
        vec![stop(SINE, G4), note_on(SINE, 0x50, C4)],
        vec![stop(SINE, C4), note_on(SINE, 0x70, E4)],
        vec![stop(SINE, E4), note_on(SINE, 0x70, G4)],
        vec![stop(SINE, G4), note_on(SINE, 0x30, C4)],
        vec![stop(SINE, C4), note_on(SINE, 0x50, E4)],
        vec![stop(SINE, E4), note_on(SINE, 0x70, G4)],
        vec![stop(SINE, G4), note_on(SINE, 0x70, C4)],
    ];
    assert_delivers(&text, &expected);
}

///
/// The guide's note about changing only `steps`: the Clock counts `00` to
/// `02`, so the Track plays `70 30 50` and the loud accent stays on C4, and
/// the fourth pair `count` still names never plays.
///
#[test]
fn only_steps_at_three_keeps_the_loud_accent_on_c() {
    let text = edited(KOMBINE, &[(ACCENT_ROW, 7, "4", "3")]);
    let expected: [Vec<OutputCommand>; 7] = [
        vec![note_on(SINE, 0x70, C4)],
        vec![stop(SINE, C4), note_on(SINE, 0x30, E4)],
        vec![stop(SINE, E4), note_on(SINE, 0x50, G4)],
        vec![stop(SINE, G4), note_on(SINE, 0x70, C4)],
        vec![stop(SINE, C4), note_on(SINE, 0x30, E4)],
        vec![stop(SINE, E4), note_on(SINE, 0x50, G4)],
        vec![stop(SINE, G4), note_on(SINE, 0x70, C4)],
    ];
    assert_delivers(&text, &expected);
}
