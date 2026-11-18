//!
//! Live editing of the shipped trackers through the console's own input and
//! file workflows.
//!
//! Each test opens a copy of `examples/melody.orcvs`,
//! `examples/tracker.orcvs` or a one-Play Source through the Open path, or
//! opens the Kombine, the Presets and the Tracker from `Help → Examples`,
//! edits it with the keys, clipboard events and menu items a viewer uses, and
//! runs Ticks against the Source the console owns. A Tick here is
//! `SourceCommander::execute`, the call Playback makes for each Tick it runs,
//! so the assertions read the Play Commands and diagnostics Playback would be
//! handed without waiting on its clock. Playback stays stopped, which keeps the
//! test the only thing that runs a Tick.
//!
//! The Melody's layout, as `examples/melody.md` describes it: Track at
//! (8, 2) reads pair `n` at column `14 + 2n` of row 2, the Clock selects step
//! `(Tick / 2) % 8`, the Every bangs every even Tick, and the Timed Play at
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

/// The first even Tick that selects pair `n`, so the Every bangs on it.
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
    timed_plays(plan)
        .into_iter()
        .map(|(channel, note, _)| (channel, note))
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
/// Asserts that `plan`, a Tick that banged the Timed Play on an empty step,
/// sent nothing and reported nothing, and that the Language Map does not
/// diagnose the Play either: a blank note is quiet everywhere.
///
fn assert_quiet(harness: &Harness<'_, Console>, plan: &TickPlan, context: &str) {
    assert!(
        plan.play_commands.is_empty(),
        "{context}: {:?}",
        plan.play_commands
    );
    assert_eq!(diagnostics(plan), [], "{context}: the Tick");
    assert!(
        !map_diagnoses_the_play(harness),
        "{context}: the Language Map"
    );
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

/// An Every banging a Timed Play on every Tick, the Play's note slot at
/// (8, 1) left blank, so nothing but the note is missing.
const BLANK_NOTE: &str = "~*0101\n  !~0064  04\n";

/// The Language Map's diagnostics as anchor and message.
fn mapped(harness: &Harness<'_, Console>) -> Vec<((usize, usize), String)> {
    let mut mapped = Vec::new();
    harness.state().orcvs.source().read_source(|source| {
        mapped = source
            .language_map()
            .diagnostics()
            .map(|d| ((d.anchor().x(), d.anchor().y()), d.message.clone()))
            .collect();
    });
    mapped
}

///
/// A Note typed into a Play's blank note slot goes from blank, which sends
/// and reports nothing, through half written, which the Language Map
/// diagnoses at the Play, to whole, which plays. No step of the typing leaves
/// a written note silent and undiagnosed.
///
#[tokio::test]
async fn a_note_typed_into_a_blank_note_slot_is_diagnosed_until_it_plays() {
    let dir = TempDir::new();
    let (mut harness, _) = console_with(&dir, "blank-note.orcvs", BLANK_NOTE);
    let play = (2, 1);

    assert_eq!(mapped(&harness), [], "a blank note is not diagnosed");
    let blank = tick(&harness, 0);
    assert_eq!(diagnostics(&blank), [], "a blank note reports nothing");
    assert_eq!(notes(&blank), [], "a blank note plays nothing");

    go_to(&mut harness, 8, 1);
    type_text(&mut harness, "E");
    assert_eq!(text(&harness, 1, 8..10), "E ");
    assert_eq!(
        mapped(&harness),
        [
            (play, "expected a note, found \"E \"".to_owned()),
            ((8, 1), "invalid Language Unit character 'E'".to_owned())
        ],
        "the Language Map"
    );
    // The Language Map diagnoses the Play before any Turn, so the Tick
    // leaves it out rather than report it twice.
    let half = tick(&harness, 1);
    assert_eq!(diagnostics(&half), [], "the Tick");
    assert_eq!(notes(&half), [], "a half-written note plays nothing");

    type_text(&mut harness, "5");
    assert_eq!(mapped(&harness), [], "a whole note is not diagnosed");
    let whole = tick(&harness, 2);
    assert_eq!(diagnostics(&whole), [], "a whole note reports nothing");
    assert_eq!(notes(&whole), [76], "E5");
}

///
/// Clearing a pair empties its step without moving the pairs after it, and
/// typing its Note back restores the step.
///
#[tokio::test]
async fn a_cleared_pair_empties_its_step_in_place_and_typing_it_back_restores_it() {
    let dir = TempDir::new();
    let (mut harness, _) = console_with_the_tracker(&dir);

    // Backspace empties the Cell under the Cursor and steps left, so two
    // presses from the pair's second Cell empty both.
    go_to(&mut harness, pair_column(1) + 1, PAIR_ROW);
    press(&mut harness, Key::Backspace, 2);

    assert_eq!(pairs(&harness), "C4    G4  A4C5  ");
    tick(&harness, banged_tick_of(0));
    let cleared = tick(&harness, banged_tick_of(1));
    assert_quiet(&harness, &cleared, "the cleared step");
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
/// back writes those spaces, so a pasted pattern keeps its empty steps.
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
    let copied = clipboard(&harness).expect("Copy put the Region on the clipboard");
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
    let empty = tick(&harness, banged_tick_of(2));
    assert_quiet(&harness, &empty, "the pasted empty step");
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
/// Saving the edited tracker writes the pattern as typed, empty steps included, and
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
            None => assert_quiet(&reopened, &plan, &format!("step {step}")),
        }
        let held = tick(&reopened, banged_tick_of(step) + 1);
        assert!(held.play_commands.is_empty(), "step {step}'s unbanged Tick");
        assert!(held.diagnostics.is_empty(), "step {step}'s unbanged Tick");
    }
}

///
/// The Tracker edits as the Melody does. Channel 4's voice, which
/// ships empty, reads the column at x 40 from row 15, one step per row and
/// one step per Tick: a typed step plays, clearing it silences that voice and
/// reports nothing, typing it back restores it, and
/// Save and a reopen keep the edits and start every voice at step 0.
///
#[tokio::test]
async fn the_tracker_edits_saves_and_reopens_a_voices_steps() {
    const COLUMN: usize = 40;
    const CHANNEL: u8 = 4;
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
    let cleared = tick(&harness, 0);
    assert!(
        notes_on(&cleared, CHANNEL).is_empty(),
        "{:?}",
        cleared.play_commands
    );
    assert!(
        cleared.diagnostics.is_empty(),
        "a cleared step reports nothing: {:?}",
        cleared.diagnostics
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

///
/// The Kombine guide's transfer, followed with the console's own menus, keys
/// and clipboard events: copy the accent Region from (0, 9) to (17, 11), paste
/// it into the Tracker at (2, 32), make the pasted Clock's `division` `02`,
/// move the closed hat's Every four columns west with a cut and a paste, and
/// retype its Play four columns west with `&$0222` for its velocity. The hat
/// then plays `70`, `30`, `50` and `40` on its hits at steps 0, 2, 4 and 6,
/// every other voice plays the shipped Tracker's channels, notes, steps and
/// velocity `77`, and no Tick diagnoses anything.
///
#[tokio::test]
async fn the_kombine_accent_region_pasted_into_the_tracker_accents_the_closed_hat() {
    let mut harness = running_console(Vec2::from(DEFAULT_VIEW_SIZE));
    harness.run_steps(2);

    super::choose_example(&mut harness, "Kombine");
    go_to(&mut harness, 0, 9);
    for _ in 0..17 {
        harness.key_press_modifiers(Modifiers::SHIFT, Key::ArrowRight);
    }
    for _ in 0..2 {
        harness.key_press_modifiers(Modifiers::SHIFT, Key::ArrowDown);
    }
    harness.event(Event::Copy);
    harness.step();
    let copied = clipboard(&harness).expect("Copy put the Region on the clipboard");
    harness.run_steps(1);

    super::choose_example(&mut harness, "Tracker");
    go_to(&mut harness, 2, 32);
    harness.event(Event::Paste(copied));
    harness.step();
    harness.run_steps(1);
    go_to(&mut harness, 7, 33);
    type_text(&mut harness, "2");
    // Space asks for Playback rather than typing an empty Cell, so the guide
    // moves the Every with a cut and a paste, and empties the Play's Cells
    // with Delete before retyping it.
    go_to(&mut harness, 16, 6);
    for _ in 0..5 {
        harness.key_press_modifiers(Modifiers::SHIFT, Key::ArrowRight);
    }
    harness.event(Event::Cut);
    harness.step();
    let cut = clipboard(&harness).expect("Cut put the Every on the clipboard");
    assert_eq!(cut, "~*0101", "the Cut takes the hat's Every");
    harness.run_steps(1);
    go_to(&mut harness, 12, 6);
    harness.event(Event::Paste(cut));
    harness.step();
    harness.run_steps(1);
    go_to(&mut harness, 18, 7);
    for _ in 0..11 {
        harness.key_press_modifiers(Modifiers::SHIFT, Key::ArrowRight);
    }
    press(&mut harness, Key::Delete, 1);
    go_to(&mut harness, 14, 7);
    type_text(&mut harness, "!~09&$0222&v");
    press(&mut harness, Key::ArrowRight, 2);
    type_text(&mut harness, "01");

    let rows: Vec<String> = [6, 7, 32, 33, 34]
        .into_iter()
        .map(|row| text(&harness, row, 0..30).trim_end().to_owned())
        .collect();
    assert_eq!(
        rows,
        [
            "            ~*0101        .+08",
            "              !~09&$0222&v  01",
            "  || Accent ||",
            "  &t~.02040470305040",
            "",
        ]
    );

    let mut hat = Vec::new();
    let mut neighbours = Vec::new();
    for step in 0..16 {
        let plan = tick(&harness, step);
        assert_eq!(diagnostics(&plan), [], "Tick {step}");
        let (hits, others): (Vec<_>, Vec<_>) = timed_plays(&plan)
            .into_iter()
            .partition(|&(channel, note, _)| (channel, note) == (9, 42));
        hat.extend(hits.into_iter().map(|(_, _, velocity)| (step, velocity)));
        neighbours.push(others);
    }
    assert_eq!(
        neighbours,
        shipped_tracker_neighbours(),
        "every other voice plays as in the shipped Tracker"
    );
    assert_eq!(
        hat,
        [
            (0, 0x70),
            (2, 0x30),
            (4, 0x50),
            (6, 0x40),
            (8, 0x70),
            (10, 0x30),
            (12, 0x50),
            (14, 0x40),
        ]
    );
}

///
/// The text the frame's Copy or Cut put on the clipboard, if it put any.
///
fn clipboard(harness: &Harness<'_, Console>) -> Option<String> {
    harness
        .output()
        .platform_output
        .commands
        .iter()
        .find_map(|command| match command {
            egui::OutputCommand::CopyText(copied) => Some(copied.clone()),
            _ => None,
        })
}

/// The channel, MIDI note and velocity of each note a Tick's Timed Plays
/// start. Any other play command is a failure.
fn timed_plays(plan: &TickPlan) -> Vec<(u8, u8, u8)> {
    timed_plays_skipping(plan, |_| false)
}

/// The channel, MIDI note and velocity of each note a Tick's Timed Plays
/// start, leaving out the Control Changes. Any other play command is a
/// failure.
fn notes_beside_controls(plan: &TickPlan) -> Vec<(u8, u8, u8)> {
    timed_plays_skipping(plan, |command| {
        matches!(command, PlayCommand::ControlChange { .. })
    })
}

/// The channel, MIDI note and velocity of each Timed Play in `plan`, passing
/// over each command `skipped` answers true for.
fn timed_plays_skipping(
    plan: &TickPlan,
    skipped: impl Fn(&PlayCommand) -> bool,
) -> Vec<(u8, u8, u8)> {
    plan.play_commands
        .iter()
        .filter(|command| !skipped(command))
        .map(|command| match command {
            PlayCommand::Timed {
                channel,
                note,
                velocity,
                ..
            } => (channel.value(), note.value(), velocity.value()),
            other => panic!("expected only Timed notes, not {other:?}"),
        })
        .collect()
}

///
/// The shipped Tracker's first sixteen Ticks in a console of its own, each
/// Tick's notes without the closed hat's: what the transfer must leave every
/// other voice playing. Each voice plays velocity `77`, and the kick, snare
/// and bassline each play.
///
fn shipped_tracker_neighbours() -> Vec<Vec<(u8, u8, u8)>> {
    let mut harness = running_console(Vec2::from(DEFAULT_VIEW_SIZE));
    harness.run_steps(2);
    super::choose_example(&mut harness, "Tracker");
    let neighbours: Vec<Vec<(u8, u8, u8)>> = (0..16)
        .map(|step| {
            timed_plays(&tick(&harness, step))
                .into_iter()
                .filter(|&(channel, note, _)| (channel, note) != (9, 42))
                .collect()
        })
        .collect();
    let plays: Vec<&(u8, u8, u8)> = neighbours.iter().flatten().collect();
    assert!(plays.iter().all(|&&(_, _, velocity)| velocity == 0x77));
    // The kick `C2` and snare `D2` on channel `09`, and the bassline on `08`.
    assert!(
        plays
            .iter()
            .any(|&&(channel, note, _)| (channel, note) == (9, 36))
    );
    assert!(
        plays
            .iter()
            .any(|&&(channel, note, _)| (channel, note) == (9, 38))
    );
    assert!(plays.iter().any(|&&(channel, _, _)| channel == 8));
    neighbours
}

///
/// The Kombine guide's three accent steps, typed as the guide gives them:
/// `3` over the accent Clock's `steps` at (7, 10) and over the Track's
/// `count` at (9, 10), then Backspace twice from (17, 10) to clear the fourth
/// velocity. The loud `70` then plays on every C4.
///
#[tokio::test]
async fn three_kombine_accent_steps_typed_in_the_console_keep_the_loud_accent_on_c() {
    let mut harness = running_console(Vec2::from(DEFAULT_VIEW_SIZE));
    harness.run_steps(2);
    super::choose_example(&mut harness, "Kombine");

    go_to(&mut harness, 7, 10);
    type_text(&mut harness, "3");
    go_to(&mut harness, 9, 10);
    type_text(&mut harness, "3");
    go_to(&mut harness, 17, 10);
    press(&mut harness, Key::Backspace, 2);
    assert_eq!(
        text(&harness, 10, 0..18).trim_end(),
        "&t~.010303703050",
        "the accent Track"
    );

    let played: Vec<(u8, u8)> = (0..6)
        .map(|step| {
            let plan = tick(&harness, step);
            assert_eq!(diagnostics(&plan), [], "Tick {step}");
            match plan.play_commands.as_slice() {
                [PlayCommand::Timed { velocity, note, .. }] => (note.value(), velocity.value()),
                other => panic!("Tick {step} plays one Timed note, not {other:?}"),
            }
        })
        .collect();
    assert_eq!(
        played,
        [
            (60, 0x70),
            (64, 0x30),
            (67, 0x50),
            (60, 0x70),
            (64, 0x30),
            (67, 0x50),
        ]
    );
}

/// The acid channel's cutoff, resonance and decay controllers.
const CUTOFF: u8 = 0x4A;
const RESONANCE: u8 = 0x47;
const DECAY: u8 = 0x4B;

///
/// The controller and value of each Control Change a Tick sends, all on the
/// acid channel `08`.
///
fn control_changes(plan: &TickPlan) -> Vec<(u8, u8)> {
    plan.play_commands
        .iter()
        .filter_map(|command| match command {
            PlayCommand::ControlChange {
                channel,
                controller,
                value,
            } => {
                assert_eq!(channel.value(), 8, "a control sent off the acid channel");
                Some((controller.value(), value.value()))
            }
            _ => None,
        })
        .collect()
}

///
/// Runs `ticks` on the console's Source and answers each Tick's
/// Control Changes, asserting the bassline plays its step at velocity `64`
/// on the acid channel and nothing is diagnosed.
///
fn presets_controls(
    harness: &Harness<'_, Console>,
    ticks: std::ops::Range<u64>,
) -> Vec<Vec<(u8, u8)>> {
    const BASS: [u8; 8] = [33, 33, 45, 33, 36, 33, 43, 40];
    ticks
        .map(|step| {
            let plan = tick(harness, step);
            assert_eq!(diagnostics(&plan), [], "Tick {step}");
            assert_eq!(
                notes_beside_controls(&plan),
                [(8, BASS[(step % 8) as usize], 0x64)],
                "Tick {step}'s bassline"
            );
            control_changes(&plan)
        })
        .collect()
}

/// Each Tick's cutoff and resonance, as `presets_controls` answers them.
fn cutoff_and_resonance(cutoffs: &[u8], resonances: &[u8]) -> Vec<Vec<(u8, u8)>> {
    cutoffs
        .iter()
        .zip(resonances)
        .map(|(&cutoff, &resonance)| vec![(CUTOFF, cutoff), (RESONANCE, resonance)])
        .collect()
}

///
/// The Presets guide's preset change, typed in the console: `1` over the
/// selector's `0` at (5, 15) before Tick 4 turns cutoff and resonance toward
/// preset `01` from Tick 4, each at its own `amount`.
///
#[tokio::test]
async fn a_preset_typed_in_the_console_turns_both_presets_controls() {
    let mut harness = running_console(Vec2::from(DEFAULT_VIEW_SIZE));
    harness.run_steps(2);
    super::choose_example(&mut harness, "Presets");

    assert_eq!(
        presets_controls(&harness, 0..4),
        cutoff_and_resonance(&[0x20; 4], &[0x20; 4])
    );
    go_to(&mut harness, 5, 15);
    type_text(&mut harness, "1");
    assert_eq!(text(&harness, 15, 0..8), "  .%0102");
    assert_eq!(
        presets_controls(&harness, 4..15),
        cutoff_and_resonance(
            &[
                0x28, 0x30, 0x38, 0x40, 0x48, 0x50, 0x58, 0x60, 0x60, 0x60, 0x60
            ],
            &[
                0x25, 0x2A, 0x2F, 0x34, 0x39, 0x3E, 0x43, 0x48, 0x4D, 0x50, 0x50
            ],
        )
    );
}

///
/// Opening the Presets again from `Help → Examples` after a preset change
/// asks before discarding it and, confirmed, restores the file as shipped:
/// preset `00`, with both controls sending `20` from the first Tick.
///
#[tokio::test]
async fn reopening_the_presets_restores_preset_00() {
    let mut harness = running_console(Vec2::from(DEFAULT_VIEW_SIZE));
    harness.run_steps(2);
    super::choose_example(&mut harness, "Presets");
    go_to(&mut harness, 5, 15);
    type_text(&mut harness, "1");
    presets_controls(&harness, 0..6);
    assert_eq!(text(&harness, 20, 0..22), "      ||50  6003  01||");

    super::choose_example(&mut harness, "Presets");
    assert!(
        super::asking(&harness),
        "the preset change was discarded unasked"
    );
    super::answer(&mut harness, super::DISCARD);
    harness.run_steps(1);
    let presets = crate::examples::EXAMPLES
        .iter()
        .find(|example| example.name == "Presets")
        .expect("Presets is registered");
    assert_eq!(
        harness.state().orcvs.source().snapshot(),
        presets.source().snapshot()
    );
    assert_eq!(
        presets_controls(&harness, 0..3),
        cutoff_and_resonance(&[0x20; 3], &[0x20; 3])
    );
}

///
/// The Presets guide's decay control, followed with the console's keys and
/// clipboard: copy the `CUT` Region from (0, 18) to (23, 22), paste it at
/// (0, 30), and type `DEC`, `B`, `10`, `1` and `7` where the guide says.
/// Decay then follows the selector beside cutoff and resonance, which send as
/// they did before.
///
#[tokio::test]
async fn the_presets_decay_control_pasted_in_the_console_follows_the_selector() {
    let mut harness = running_console(Vec2::from(DEFAULT_VIEW_SIZE));
    harness.run_steps(2);
    super::choose_example(&mut harness, "Presets");

    go_to(&mut harness, 0, 18);
    for _ in 0..23 {
        harness.key_press_modifiers(Modifiers::SHIFT, Key::ArrowRight);
    }
    for _ in 0..4 {
        harness.key_press_modifiers(Modifiers::SHIFT, Key::ArrowDown);
    }
    harness.event(Event::Copy);
    harness.step();
    let copied = clipboard(&harness).expect("Copy put the Region on the clipboard");
    harness.run_steps(1);
    go_to(&mut harness, 0, 30);
    harness.event(Event::Paste(copied));
    harness.step();
    harness.run_steps(1);
    for (x, y, typed) in [
        (11, 30, "DEC"),
        (7, 31, "B"),
        (10, 31, "10"),
        (14, 33, "1"),
        (14, 34, "7"),
    ] {
        go_to(&mut harness, x, y);
        type_text(&mut harness, typed);
    }
    let rows: Vec<String> = (30..35)
        .map(|row| text(&harness, row, 0..24).trim_end().to_owned())
        .collect();
    assert_eq!(
        rows,
        [
            "~*0101  || DEC ||",
            "  !c084B~>10&v.+02&$0210",
            "      ||20  2002  00||",
            "||preset 00   10||",
            "||preset 01   70||",
        ]
    );

    let mut controls = presets_controls(&harness, 0..4);
    go_to(&mut harness, 5, 15);
    type_text(&mut harness, "1");
    controls.extend(presets_controls(&harness, 4..11));
    let expected: Vec<Vec<(u8, u8)>> = [
        (0x20, 0x20, 0x10),
        (0x20, 0x20, 0x10),
        (0x20, 0x20, 0x10),
        (0x20, 0x20, 0x10),
        (0x28, 0x25, 0x20),
        (0x30, 0x2A, 0x30),
        (0x38, 0x2F, 0x40),
        (0x40, 0x34, 0x50),
        (0x48, 0x39, 0x60),
        (0x50, 0x3E, 0x70),
        (0x58, 0x43, 0x70),
    ]
    .into_iter()
    .map(|(cutoff, resonance, decay)| {
        vec![(CUTOFF, cutoff), (RESONANCE, resonance), (DECAY, decay)]
    })
    .collect();
    assert_eq!(controls, expected);
}

///
/// The Presets guide's move into the Tracker, followed with the console's
/// menus, keys and clipboard: copy (0, 14) to (23, 28) in the Presets, paste
/// it into the Tracker at (0, 36) and type `26` over each control's `10` at
/// (22, 41) and (22, 47). Every voice then plays the shipped Tracker's Notes,
/// and the selector at (5, 37) turns both controls toward preset `01`.
///
#[tokio::test]
async fn the_presets_controls_pasted_into_the_tracker_slew_its_bassline() {
    let mut harness = running_console(Vec2::from(DEFAULT_VIEW_SIZE));
    harness.run_steps(2);
    super::choose_example(&mut harness, "Presets");
    go_to(&mut harness, 0, 14);
    for _ in 0..23 {
        harness.key_press_modifiers(Modifiers::SHIFT, Key::ArrowRight);
    }
    for _ in 0..14 {
        harness.key_press_modifiers(Modifiers::SHIFT, Key::ArrowDown);
    }
    harness.event(Event::Copy);
    harness.step();
    let copied = clipboard(&harness).expect("Copy put the Region on the clipboard");
    harness.run_steps(1);

    super::choose_example(&mut harness, "Tracker");
    go_to(&mut harness, 0, 36);
    harness.event(Event::Paste(copied));
    harness.step();
    harness.run_steps(1);
    for row in [41, 47] {
        go_to(&mut harness, 22, row);
        type_text(&mut harness, "26");
    }
    assert_eq!(text(&harness, 41, 0..24), "  !c084A~>08&v.+02&$0226");
    assert_eq!(text(&harness, 47, 0..24), "  !c0847~>05&v.+02&$0226");

    let mut plays = Vec::new();
    let mut controls = Vec::new();
    for step in 0..16 {
        if step == 4 {
            go_to(&mut harness, 5, 37);
            type_text(&mut harness, "1");
        }
        let plan = tick(&harness, step);
        assert_eq!(diagnostics(&plan), [], "Tick {step}");
        plays.push(notes_beside_controls(&plan));
        controls.push(control_changes(&plan));
    }
    assert_eq!(plays, shipped_tracker_plays());
    assert_eq!(
        controls,
        cutoff_and_resonance(
            &[
                0x20, 0x20, 0x20, 0x20, 0x28, 0x30, 0x38, 0x40, 0x48, 0x50, 0x58, 0x60, 0x60, 0x60,
                0x60, 0x60
            ],
            &[
                0x20, 0x20, 0x20, 0x20, 0x25, 0x2A, 0x2F, 0x34, 0x39, 0x3E, 0x43, 0x48, 0x4D, 0x50,
                0x50, 0x50
            ],
        )
    );
}

///
/// The shipped Tracker's first sixteen Ticks in a console of its own, every
/// voice's Notes on each Tick.
///
fn shipped_tracker_plays() -> Vec<Vec<(u8, u8, u8)>> {
    let mut harness = running_console(Vec2::from(DEFAULT_VIEW_SIZE));
    harness.run_steps(2);
    super::choose_example(&mut harness, "Tracker");
    let plays: Vec<Vec<(u8, u8, u8)>> = (0..16)
        .map(|step| timed_plays(&tick(&harness, step)))
        .collect();
    // The bassline, on the acid channel, plays on each of the sixteen Ticks.
    let bassline = plays
        .iter()
        .filter(|tick| tick.iter().any(|&(channel, _, _)| channel == 8))
        .count();
    assert_eq!(bassline, 16);
    plays
}
