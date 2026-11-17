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

use super::{HandDrivenRun, functions_mapped, note_on, stop, tick_diagnostics};
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
}

///
/// `text` with `row`'s Cells from `column` replaced by `after`, which must
/// replace `before` there: an edit the guide documents, applied to the shipped
/// file, so a file that moved its Cells fails here rather than editing the
/// wrong ones.
///
fn edited(text: &str, row: usize, column: usize, before: &str, after: &str) -> String {
    let mut rows: Vec<String> = text.lines().map(str::to_owned).collect();
    let line = &mut rows[row];
    assert_eq!(
        line.get(column..column + before.len()),
        Some(before),
        "row {row} holds {before:?} at column {column}"
    );
    line.replace_range(column..column + before.len(), after);
    let mut text = rows.join("\n");
    text.push('\n');
    text
}

///
/// Asserts the guide draws `row` of `text` as its diagrams draw every row:
/// the row number right-aligned in three columns, two spaces, then the row.
///
fn assert_guide_draws(text: &str, row: usize) {
    let line = format!("{row:>3}  {}", text.lines().nth(row).unwrap_or_default());
    let line = line.trim_end();
    assert!(
        GUIDE.lines().any(|drawn| drawn.trim_end() == line),
        "the guide does not draw {line:?}"
    );
}

///
/// Before the first Tick the Language Map diagnoses no Function: what it
/// diagnoses is only the Tracks' data, the pairs east of each `count`, which
/// are Ordinary Source the Tracks read rather than Functions. The guide draws
/// every row of the file as shipped.
///
#[test]
fn kombine_opens_with_no_function_diagnosed_and_draws_as_shipped() {
    let source = SourceCommander::with_source(
        file::read(KOMBINE.as_bytes()).expect("the Kombine is a Source File"),
    );
    assert_eq!(functions_mapped(&source), vec![]);
    source.read_source(|source| {
        assert!(
            source
                .language_map()
                .diagnostics()
                .all(|d| [PITCH_ROW, ACCENT_ROW].contains(&d.anchor().y()) && d.anchor().x() >= 10),
            "a diagnosis outside the Tracks' data"
        );
    });
    for row in 0..KOMBINE.lines().count() {
        assert_guide_draws(KOMBINE, row);
    }
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
    let text = edited(KOMBINE, ACCENT_ROW, 7, "4", "3");
    let text = edited(&text, ACCENT_ROW, 9, "4", "3");
    let text = edited(&text, ACCENT_ROW, 16, "40", "");
    assert_guide_draws(&text, ACCENT_ROW);
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
