//!
//! Live editing of the shipped trackers through the console's own input and
//! file workflows.
//!
//! Each test opens a copy of `examples/melody.orcvs` or
//! `examples/tracker.orcvs` through the Open path, edits
//! it with the keys, clipboard events and menu items a viewer uses, and runs
//! Ticks against the Source the console owns. A Tick here is
//! `SourceCommander::execute`, the call Playback makes for each Tick it runs,
//! so the assertions read the Play Commands and diagnostics Playback would be
//! handed without waiting on its clock. Playback stays stopped, which keeps the
//! test the only thing that runs a Tick.
//!
//! The Melody's layout, as `examples/melody.md` describes it: Track at
//! (8, 2) reads pair `n` at column `14 + 2n` of row 2, the Clock selects step
//! `(Tick / 2) % 8`, the Delay bangs every even Tick, and the Timed Play at
//! (2, 3) plays what Track writes into its note slot at (8, 3).
//!

use egui::{Event, Key, Modifiers, Vec2};
use egui_kittest::Harness;
use orcvs::source::{PlayCommand, SourcePaint, Tick, TickPlan};

use super::super::{Console, DEFAULT_VIEW_SIZE};
use super::{choose_in_menu, cursor, open_picked, running_console};
use crate::theme_registry::tests_support::TempDir;

/// The shipped tracker, byte for byte.
const CELL_TRACKER: &str = include_str!("../../../../examples/melody.orcvs");

/// The shipped Tracker, byte for byte.
const TRACKER: &str = include_str!("../../../../examples/tracker.orcvs");

/// The row holding Track and the pairs it reads.
const PAIR_ROW: usize = 2;

/// The column of pair `n`'s first Cell.
fn pair_column(n: usize) -> usize {
    14 + 2 * n
}

/// The Timed Play's anchor, where its diagnostics are anchored.
const PLAY: (usize, usize) = (2, 3);

/// The first even Tick that selects pair `n`, so the Delay bangs on it.
fn banged_tick_of(n: usize) -> u64 {
    2 * n as u64
}

///
/// A running console that opened `file`, a copy of the shipped `contents`
/// written into `dir`, with the Cursor where the Open left it.
///
fn console_with(
    dir: &TempDir,
    file: &str,
    contents: &str,
) -> (Harness<'static, Console>, std::path::PathBuf) {
    let path = dir.write(file, contents);
    (reopen(&path), path)
}

/// The shipped Melody, opened as [`console_with`] opens a file.
fn console_with_the_tracker(dir: &TempDir) -> (Harness<'static, Console>, std::path::PathBuf) {
    console_with(dir, "melody.orcvs", CELL_TRACKER)
}

/// A new running console that opened the Source File at `path`.
fn reopen(path: &std::path::Path) -> Harness<'static, Console> {
    let mut harness = running_console(Vec2::from(DEFAULT_VIEW_SIZE));
    harness.run_steps(2);
    open_picked(&mut harness, Some(path.to_path_buf()));
    harness
}

///
/// The text of `columns` on `row` of the console's Source, an empty Cell as a
/// space.
///
fn text(harness: &Harness<'_, Console>, row: usize, columns: std::ops::Range<usize>) -> String {
    let frame = harness.state().orcvs.render_frame();
    let grid = frame.grid();
    columns
        .map(|x| {
            let position = grid.position(x, row).expect("inside the Grid");
            frame.at(position).content().unwrap_or(' ')
        })
        .collect()
}

/// The eight pairs, as the row holds them.
fn pairs(harness: &Harness<'_, Console>) -> String {
    text(harness, PAIR_ROW, pair_column(0)..pair_column(8))
}

/// Runs Tick `tick` on the console's Source, as Playback runs one.
fn tick(harness: &Harness<'_, Console>, tick: u64) -> TickPlan {
    harness.state().orcvs.source().execute(Tick::new(tick))
}

/// The channel and MIDI note of each note a Tick's Timed Plays start.
fn timed_notes(plan: &TickPlan) -> Vec<(u8, u8)> {
    plan.play_commands
        .iter()
        .map(|command| match command {
            PlayCommand::Timed { channel, note, .. } => (channel.value(), note.value()),
            other => panic!("the trackers play only Timed notes, not {other:?}"),
        })
        .collect()
}

/// The MIDI notes a Tick's Timed Plays start, on any channel.
fn notes(plan: &TickPlan) -> Vec<u8> {
    timed_notes(plan)
        .into_iter()
        .map(|(_, note)| note)
        .collect()
}

/// The MIDI notes a Tick's Timed Plays start on `channel`.
fn notes_on(plan: &TickPlan, channel: u8) -> Vec<u8> {
    timed_notes(plan)
        .into_iter()
        .filter_map(|(on, note)| (on == channel).then_some(note))
        .collect()
}

/// A Tick's diagnostics as anchor and message.
fn diagnostics(plan: &TickPlan) -> Vec<((usize, usize), String)> {
    plan.diagnostics
        .iter()
        .map(|d| ((d.anchor().x(), d.anchor().y()), d.message.clone()))
        .collect()
}

///
/// What the Tick that empties the Timed Play's note slot diagnoses it with.
/// While the slot stays empty the Language Map diagnoses the Play instead,
/// and the Tick does not repeat it.
///
fn empty_note_slot() -> Vec<((usize, usize), String)> {
    vec![(PLAY, "expected a note, found \"  \"".to_owned())]
}

/// Whether the Language Map diagnoses the Timed Play.
fn map_diagnoses_the_play(harness: &Harness<'_, Console>) -> bool {
    let mut diagnosed = false;
    harness.state().orcvs.source().read_source(|source| {
        diagnosed = source
            .language_map()
            .diagnostics()
            .any(|d| (d.anchor().x(), d.anchor().y()) == PLAY);
    });
    diagnosed
}

///
/// Moves the Cursor to (`x`, `y`) with the arrow keys and checks it arrived.
///
fn go_to(harness: &mut Harness<'_, Console>, x: usize, y: usize) {
    let (from_x, from_y) = cursor(harness.state());
    let presses = [
        (Key::ArrowRight, x.saturating_sub(from_x)),
        (Key::ArrowLeft, from_x.saturating_sub(x)),
        (Key::ArrowDown, y.saturating_sub(from_y)),
        (Key::ArrowUp, from_y.saturating_sub(y)),
    ];
    for (key, count) in presses {
        for _ in 0..count {
            harness.key_press(key);
        }
    }
    harness.step();
    harness.run_steps(1);
    assert_eq!(
        cursor(harness.state()),
        (x, y),
        "the arrows missed the Cell"
    );
}

/// Types `typed` at the Cursor, one character per event as a keyboard does.
fn type_text(harness: &mut Harness<'_, Console>, typed: &str) {
    for character in typed.chars() {
        harness.event(Event::Text(character.to_string()));
    }
    harness.step();
    harness.run_steps(1);
}

/// Presses `key` `times` times at the Cursor.
fn press(harness: &mut Harness<'_, Console>, key: Key, times: usize) {
    for _ in 0..times {
        harness.key_press(key);
    }
    harness.step();
    harness.run_steps(1);
}

///
/// A Note typed over a pair Track selects is what the next Tick that selects
/// it plays: the edit reaches the Source, not a copy of the pattern.
///
#[tokio::test]
async fn a_note_typed_over_a_pair_plays_on_the_next_tick_that_selects_it() {
    let dir = TempDir::new();
    let (mut harness, _) = console_with_the_tracker(&dir);
    assert_eq!(notes(&tick(&harness, banged_tick_of(3))), [67], "G4");

    go_to(&mut harness, pair_column(3), PAIR_ROW);
    type_text(&mut harness, "B");

    assert_eq!(pairs(&harness), "C4D4  B4  A4C5  ");
    let plan = tick(&harness, banged_tick_of(3));
    assert_eq!(notes(&plan), [71], "B4");
    assert!(plan.diagnostics.is_empty(), "{:?}", plan.diagnostics);
}

///
/// A Note typed into an empty pair passes through a half-written pair. That
/// Tick Track diagnoses it once and writes nothing, so the note slot keeps
/// the Note an earlier step left and the Bang plays that Note again. The
/// Tick after the Note is finished plays it and diagnoses nothing.
///
#[tokio::test]
async fn a_half_typed_note_is_diagnosed_once_and_finishing_it_plays_it() {
    let dir = TempDir::new();
    let (mut harness, _) = console_with_the_tracker(&dir);
    assert_eq!(notes(&tick(&harness, banged_tick_of(1))), [62], "D4");

    go_to(&mut harness, pair_column(2), PAIR_ROW);
    type_text(&mut harness, "E");
    assert_eq!(pairs(&harness), "C4D4E G4  A4C5  ");

    let half = tick(&harness, banged_tick_of(2));
    assert_eq!(
        diagnostics(&half),
        [((8, PAIR_ROW), "&t has partial or invalid input".to_owned())]
    );
    assert_eq!(notes(&half), [62], "the slot's last Note, D4, plays again");

    type_text(&mut harness, "5");
    let finished = tick(&harness, banged_tick_of(2));
    assert_eq!(notes(&finished), [76], "E5");
    assert!(
        finished.diagnostics.is_empty(),
        "{:?}",
        finished.diagnostics
    );
}

///
/// Clearing a pair makes it a rest without moving the pairs after it, and
/// typing its Note back restores the step.
///
#[tokio::test]
async fn a_cleared_pair_rests_in_place_and_typing_it_back_restores_it() {
    let dir = TempDir::new();
    let (mut harness, _) = console_with_the_tracker(&dir);

    // Backspace empties the Cell under the Cursor and steps left, so two
    // presses from the pair's second Cell empty both.
    go_to(&mut harness, pair_column(1) + 1, PAIR_ROW);
    press(&mut harness, Key::Backspace, 2);

    assert_eq!(pairs(&harness), "C4    G4  A4C5  ");
    tick(&harness, banged_tick_of(0));
    let rest = tick(&harness, banged_tick_of(1));
    assert!(rest.play_commands.is_empty(), "{:?}", rest.play_commands);
    assert_eq!(diagnostics(&rest), empty_note_slot());
    assert_eq!(
        notes(&tick(&harness, banged_tick_of(3))),
        [67],
        "G4 stays step 3"
    );

    go_to(&mut harness, pair_column(1), PAIR_ROW);
    type_text(&mut harness, "D4");

    assert_eq!(pairs(&harness), "C4D4  G4  A4C5  ");
    let restored = tick(&harness, banged_tick_of(1));
    assert_eq!(notes(&restored), [62], "D4");
    assert!(
        restored.diagnostics.is_empty(),
        "{:?}",
        restored.diagnostics
    );
}

///
/// A whole Note pasted into an empty pair arrives in one edit, so the next
/// Tick that selects it plays it with no half-written pair between.
///
#[tokio::test]
async fn a_note_pasted_into_an_empty_pair_plays_on_the_next_tick_that_selects_it() {
    let dir = TempDir::new();
    let (mut harness, _) = console_with_the_tracker(&dir);
    tick(&harness, banged_tick_of(1));

    go_to(&mut harness, pair_column(2), PAIR_ROW);
    harness.event(Event::Paste("E5".to_owned()));
    harness.step();
    harness.run_steps(1);

    assert_eq!(pairs(&harness), "C4D4E5G4  A4C5  ");
    let plan = tick(&harness, banged_tick_of(2));
    assert_eq!(notes(&plan), [76], "E5");
    assert!(plan.diagnostics.is_empty(), "{:?}", plan.diagnostics);
}
///
/// Copying the pairs carries their empty pairs as spaces, and pasting them
/// back writes those spaces, so a pasted pattern keeps its rests.
///
#[tokio::test]
async fn copied_and_pasted_pairs_keep_their_empty_pairs() {
    let dir = TempDir::new();
    let (mut harness, _) = console_with_the_tracker(&dir);

    go_to(&mut harness, pair_column(0), PAIR_ROW);
    for _ in 1..pair_column(8) - pair_column(0) {
        harness.key_press_modifiers(Modifiers::SHIFT, Key::ArrowRight);
    }
    harness.event(Event::Copy);
    harness.step();
    let copied = harness
        .output()
        .platform_output
        .commands
        .iter()
        .find_map(|command| match command {
            egui::OutputCommand::CopyText(copied) => Some(copied.clone()),
            _ => None,
        })
        .expect("Copy put the Region on the clipboard");
    harness.run_steps(1);
    assert_eq!(copied, "C4D4  G4  A4C5  ");

    // Fill the first empty pair, then paste the copy back over the row.
    go_to(&mut harness, pair_column(2), PAIR_ROW);
    type_text(&mut harness, "E5");
    assert_eq!(pairs(&harness), "C4D4E5G4  A4C5  ");
    go_to(&mut harness, pair_column(0), PAIR_ROW);
    harness.event(Event::Paste(copied));
    harness.step();
    harness.run_steps(1);

    assert_eq!(pairs(&harness), "C4D4  G4  A4C5  ");
    tick(&harness, banged_tick_of(1));
    let rest = tick(&harness, banged_tick_of(2));
    assert!(rest.play_commands.is_empty(), "{:?}", rest.play_commands);
    assert_eq!(diagnostics(&rest), empty_note_slot());
}

///
/// The pairs east of Track are ordinary Source: Track claims only its
/// operands, so the pairs paint exactly as the same Cells do once Track
/// is gone.
///
#[tokio::test]
async fn the_pairs_east_of_track_paint_as_they_do_without_track() {
    let dir = TempDir::new();
    let (mut harness, _) = console_with_the_tracker(&dir);
    let paints = |harness: &Harness<'_, Console>| {
        let frame = harness.state().orcvs.render_frame();
        let grid = frame.grid();
        (pair_column(0)..pair_column(8))
            .map(|x| {
                frame
                    .at(grid.position(x, PAIR_ROW).expect("inside the Grid"))
                    .source_paint()
            })
            .collect::<Vec<_>>()
    };
    let with_track = paints(&harness);
    assert!(
        !with_track
            .iter()
            .any(|paint| matches!(paint, SourcePaint::Operand { .. })),
        "Track claimed a pair as an operand: {with_track:?}"
    );

    go_to(&mut harness, 9, PAIR_ROW);
    press(&mut harness, Key::Backspace, 2);
    assert_eq!(
        text(&harness, PAIR_ROW, 8..10),
        "  ",
        "Track was not removed"
    );

    assert_eq!(paints(&harness), with_track);
}

///
/// Saving the edited tracker writes the pattern as typed, rests included, and
/// reopening it in a new console plays that pattern from step 0 when its
/// Ticks start again at 0, whatever Tick the Source was saved at. That a new
/// Playback run starts at Tick 0 is Playback's own contract, pinned in
/// `orcvs::playback`'s tests; this test runs the Ticks itself.
///
#[tokio::test]
async fn a_saved_and_reopened_tracker_plays_the_edited_pattern_from_tick_zero() {
    let dir = TempDir::new();
    let (mut harness, path) = console_with_the_tracker(&dir);
    for executed in 0..=banged_tick_of(3) + 1 {
        tick(&harness, executed);
    }
    go_to(&mut harness, pair_column(4), PAIR_ROW);
    type_text(&mut harness, "F4");
    go_to(&mut harness, pair_column(6) + 1, PAIR_ROW);
    press(&mut harness, Key::Backspace, 2);
    assert_eq!(pairs(&harness), "C4D4  G4F4A4    ");

    choose_in_menu(&mut harness, "File", "Save");
    let saved = std::fs::read_to_string(&path).expect("Save wrote the file");
    assert!(
        saved.contains("C4D4  G4F4A4"),
        "the saved file lost the edits: {saved:?}"
    );
    drop(harness);

    let reopened = reopen(&path);
    assert_eq!(pairs(&reopened), "C4D4  G4F4A4    ");

    let pattern = [
        Some(60),
        Some(62),
        None,
        Some(67),
        Some(65),
        Some(69),
        None,
        None,
    ];
    for (step, expected) in pattern.into_iter().enumerate() {
        let plan = tick(&reopened, banged_tick_of(step));
        match expected {
            Some(note) => {
                assert_eq!(notes(&plan), [note], "step {step}");
                assert!(
                    plan.diagnostics.is_empty(),
                    "step {step}: {:?}",
                    plan.diagnostics
                );
            }
            // Step 6 empties the slot step 5 filled; step 7 finds it empty
            // already, so the Language Map diagnoses the Play and the Tick
            // does not.
            None if step == 7 => {
                assert!(plan.play_commands.is_empty(), "step {step}");
                assert!(
                    plan.diagnostics.is_empty(),
                    "step {step}: {:?}",
                    plan.diagnostics
                );
                assert!(map_diagnoses_the_play(&reopened), "step {step}");
            }
            None => {
                assert!(plan.play_commands.is_empty(), "step {step}");
                assert_eq!(diagnostics(&plan), empty_note_slot(), "step {step}");
                assert!(map_diagnoses_the_play(&reopened), "step {step}");
            }
        }
        let held = tick(&reopened, banged_tick_of(step) + 1);
        assert!(held.play_commands.is_empty(), "step {step}'s unbanged Tick");
        assert!(held.diagnostics.is_empty(), "step {step}'s unbanged Tick");
    }
}

///
/// The Tracker edits as the Melody does. Channel 3's voice, which
/// ships empty, reads the column at x 33 from row 15, one step per row and
/// one step per Tick: a typed step plays, clearing it rests that voice with
/// no diagnostic at its Timed Play at (25, 4), typing it back restores it,
/// and Save and a reopen keep the edits and start every voice at step 0.
///
#[tokio::test]
async fn the_tracker_edits_saves_and_reopens_a_voices_steps() {
    const COLUMN: usize = 33;
    const CHANNEL: u8 = 3;
    let step_row = |step: usize| 15 + step;
    let dir = TempDir::new();
    let (mut harness, path) = console_with(&dir, "tracker.orcvs", TRACKER);
    assert!(
        notes_on(&tick(&harness, 0), CHANNEL).is_empty(),
        "the voice ships empty"
    );

    go_to(&mut harness, COLUMN, step_row(0));
    type_text(&mut harness, "C4");
    assert_eq!(notes_on(&tick(&harness, 0), CHANNEL), [60], "C4");

    go_to(&mut harness, COLUMN, step_row(1));
    type_text(&mut harness, "E4");
    assert_eq!(notes_on(&tick(&harness, 1), CHANNEL), [64], "E4");

    go_to(&mut harness, COLUMN + 1, step_row(0));
    press(&mut harness, Key::Backspace, 2);
    assert_eq!(text(&harness, step_row(0), COLUMN..COLUMN + 2), "  ");
    let rest = tick(&harness, 0);
    assert!(
        notes_on(&rest, CHANNEL).is_empty(),
        "{:?}",
        rest.play_commands
    );
    assert!(
        diagnostics(&rest)
            .iter()
            .all(|(anchor, _)| *anchor != (25, 4)),
        "a cleared step is a silent rest: {:?}",
        rest.diagnostics
    );

    go_to(&mut harness, COLUMN, step_row(0));
    type_text(&mut harness, "C5");
    assert_eq!(notes_on(&tick(&harness, 0), CHANNEL), [72], "C5");

    choose_in_menu(&mut harness, "File", "Save");
    drop(harness);
    let reopened = reopen(&path);

    let played: Vec<Vec<u8>> = (0..3)
        .map(|step| notes_on(&tick(&reopened, step), CHANNEL))
        .collect();
    assert_eq!(played, [vec![72], vec![64], vec![]]);
}
