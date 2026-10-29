//! The tracker Source Files in `examples/`, loaded exactly as shipped and run
//! through Source/Tick and Playback for two complete loops.
//!
//! Each run is a hand-driven Playback run over the file's Source, so a Tick's
//! diagnostics are the Tick Plan's and its delivery is what the adapter is
//! handed. The expected notes are written out per voice beside the file they
//! describe, so a change to either shows up here.

use super::{HandDrivenRun, note_on, stop};
use crate::playback::{InMemoryOutputAdapter, OutputCommand};
use crate::source::{LanguageUnitKind, SourceCommander, Tick, TickPlan, file};

/// What one Tick delivers to the adapter, and its diagnostics as anchor
/// column, anchor row and message.
type Expected = (Vec<OutputCommand>, Vec<(usize, usize, String)>);

///
/// A shipped tracker as its file is laid out: how many Ticks a step lasts,
/// how many Ticks each Timed Play holds its note, the velocity every Play
/// writes, whether a Portal fills the note slots, and its voices, listed in
/// Source order.
///
struct Tracker<'a> {
    step_ticks: u64,
    length: u64,
    velocity: u8,
    slot_written: bool,
    voices: &'a [Voice],
}

/// One voice of a tracker: its MIDI channel, where its Timed Play is
/// anchored, and the Note each step holds, `None` for an empty step.
struct Voice {
    channel: u8,
    play: (usize, usize),
    steps: Vec<Option<u8>>,
}

impl Voice {
    ///
    /// The diagnostic at this voice's Timed Play on an empty step, if any. A
    /// Portal that fills the note slot clears it, so the Play finds its slot
    /// empty; a nested Read south answers a Clear, which rests the Play
    /// silently.
    ///
    fn rest(&self, slot_written: bool) -> Option<String> {
        slot_written.then(|| "expected a note, found \"  \"".to_owned())
    }
}

impl Tracker<'_> {
    /// Ticks per loop: every voice of a shipped tracker has the same steps.
    fn loop_ticks(&self) -> u64 {
        self.step_ticks * self.voices[0].steps.len() as u64
    }
}

///
/// The notes a run is sounding, each with the Tick its Timed lifetime ends,
/// in the order they started.
///
#[derive(Default)]
struct Sounding(Vec<(u8, u8, u64)>);

impl Sounding {
    ///
    /// What the run should deliver and diagnose at `tick` for `tracker`, and
    /// the notes it sounds afterwards.
    ///
    /// Each voice is banged on the first Tick of each step. Lifetimes that end
    /// at `tick` stop first, in the order they started. An occupied step then
    /// plays its Note for the tracker's length, stopping the same Note first
    /// if it is still sounding; an empty step's Timed Play rests, diagnosed
    /// only as [`Voice::rest`] describes.
    ///
    fn expect(&mut self, tracker: &Tracker<'_>, tick: u64) -> Expected {
        let mut delivery = Vec::new();
        let mut diagnostics = Vec::new();
        self.0.retain(|&(channel, note, due)| {
            let ends = due == tick;
            if ends {
                delivery.push(stop(channel, note));
            }
            !ends
        });
        if !tick.is_multiple_of(tracker.step_ticks) {
            return (delivery, diagnostics);
        }
        for voice in tracker.voices {
            let step = (tick / tracker.step_ticks) as usize % voice.steps.len();
            match voice.steps[step] {
                Some(note) => {
                    let before = self.0.len();
                    self.0
                        .retain(|&(channel, held, _)| (channel, held) != (voice.channel, note));
                    if self.0.len() < before {
                        delivery.push(stop(voice.channel, note));
                    }
                    delivery.push(note_on(voice.channel, tracker.velocity, note));
                    self.0.push((voice.channel, note, tick + tracker.length));
                }
                None => diagnostics.extend(
                    voice
                        .rest(tracker.slot_written)
                        .map(|message| (voice.play.0, voice.play.1, message)),
                ),
            }
        }
        (delivery, diagnostics)
    }
}

/// A Tick's diagnostics as anchor column, anchor row and message.
fn tick_diagnostics(plan: &TickPlan) -> Vec<(usize, usize, String)> {
    plan.diagnostics
        .iter()
        .map(|d| (d.anchor().x(), d.anchor().y(), d.message.clone()))
        .collect()
}

///
/// Runs the shipped Source File `text` for two complete loops and asserts
/// every Tick's delivery and diagnostics against `tracker`.
///
/// Where a Portal fills a voice's note slot, `slot_written` is true: while the
/// slot is empty the Language Map diagnoses the Timed Play as well. A nested
/// Read in the slot leaves the Map nothing to diagnose, so only the Tick does.
///
fn assert_two_loops(text: &[u8], tracker: &Tracker<'_>) {
    let source =
        SourceCommander::with_source(file::read(text).expect("the shipped file is a Source File"));
    let adapter = InMemoryOutputAdapter::default();
    let mut run = HandDrivenRun::new(source.clone(), adapter.clone());
    run.begin_run();
    let mut sounding = Sounding::default();

    for tick in 0..2 * tracker.loop_ticks() {
        let plan = run.run_tick(tick);
        let (commands, diagnostics) = sounding.expect(tracker, tick);
        assert_eq!(
            tick_diagnostics(&plan),
            diagnostics,
            "Tick {tick}'s diagnostics"
        );
        assert_eq!(
            adapter.command_lists().last(),
            Some(&commands),
            "Tick {tick}'s delivery"
        );

        let mut mapped = Vec::new();
        source.read_source(|source| {
            mapped = source
                .language_map()
                .diagnostics()
                .map(|d| (d.anchor().x(), d.anchor().y()))
                .collect();
        });
        for voice in tracker.voices {
            let step = (tick / tracker.step_ticks) as usize % voice.steps.len();
            assert_eq!(
                mapped.contains(&voice.play),
                tracker.slot_written && voice.steps[step].is_none(),
                "Tick {tick}: the Language Map's view of the Play at {:?}",
                voice.play
            );
        }
    }
    assert_eq!(
        adapter.command_lists().len() as u64,
        2 * tracker.loop_ticks()
    );
}

///
/// The cell tracker's first loop and the Tick that starts its second, written
/// out Tick by Tick rather than derived, so a misreading shared by
/// [`Sounding::expect`] and Playback cannot pass both.
///
#[test]
fn the_cell_tracker_delivers_its_first_loop_as_written_out() {
    let rest = || vec![(2, 3, "expected a note, found \"  \"".to_owned())];
    let expected: [Expected; 17] = [
        (vec![note_on(0, 0x64, 60)], vec![]),
        (vec![], vec![]),
        (vec![note_on(0, 0x64, 62)], vec![]),
        (vec![], vec![]),
        (vec![stop(0, 60)], rest()),
        (vec![], vec![]),
        (vec![stop(0, 62), note_on(0, 0x64, 67)], vec![]),
        (vec![], vec![]),
        (vec![], rest()),
        (vec![], vec![]),
        (vec![stop(0, 67), note_on(0, 0x64, 69)], vec![]),
        (vec![], vec![]),
        (vec![note_on(0, 0x64, 72)], vec![]),
        (vec![], vec![]),
        (vec![stop(0, 69)], rest()),
        (vec![], vec![]),
        (vec![stop(0, 72), note_on(0, 0x64, 60)], vec![]),
    ];
    let source = SourceCommander::with_source(
        file::read(include_bytes!("../../../../examples/cell-tracker.orcvs"))
            .expect("the shipped file is a Source File"),
    );
    let adapter = InMemoryOutputAdapter::default();
    let mut run = HandDrivenRun::new(source, adapter.clone());
    run.begin_run();

    for (tick, (delivery, diagnostics)) in expected.into_iter().enumerate() {
        let plan = run.run_tick(tick as u64);
        assert_eq!(
            tick_diagnostics(&plan),
            diagnostics,
            "Tick {tick}'s diagnostics"
        );
        assert_eq!(
            adapter.command_lists().last(),
            Some(&delivery),
            "Tick {tick}'s delivery"
        );
    }
}

#[test]
fn the_cell_tracker_plays_two_loops_with_its_empty_pairs_resting() {
    // C4 D4 __ G4 __ A4 C5 __ at `~.0208`, each held `04`, as
    // `examples/cell-tracker.md` describes.
    assert_two_loops(
        include_bytes!("../../../../examples/cell-tracker.orcvs"),
        &Tracker {
            step_ticks: 2,
            length: 4,
            velocity: 0x64,
            slot_written: true,
            voices: &[Voice {
                channel: 0,
                play: (2, 3),
                steps: vec![
                    Some(60),
                    Some(62),
                    None,
                    Some(67),
                    None,
                    Some(69),
                    Some(72),
                    None,
                ],
            }],
        },
    );
}

///
/// A Learn Orca patch's voices as Orca plays them: every voice steps once a
/// Tick and holds its note for one, at velocity `77`, the byte Orca's default
/// `f` sends, and a nested Read fills each note slot.
///
fn orca_patch(voices: &[Voice]) -> Tracker<'_> {
    Tracker {
        step_ticks: 1,
        length: 1,
        velocity: 0x77,
        slot_written: false,
        voices,
    }
}

/// `count` steps for `channel`, holding each of `notes` placed on it.
fn steps(notes: &[(u8, usize, u8)], channel: u8, count: usize) -> Vec<Option<u8>> {
    let mut steps = vec![None; count];
    for &(on, step, note) in notes {
        if on == channel {
            steps[step] = Some(note);
        }
    }
    steps
}

/// `note` spelled as the Source writes it: its pitch letter, lowercase for a
/// sharp, then its octave, from MIDI note 12 upward.
fn spelling(note: u8) -> String {
    let letter =
        ["C", "c", "D", "d", "E", "F", "f", "G", "g", "A", "a", "B"][usize::from(note % 12)];
    format!("{letter}{}", note / 12 - 1)
}

///
/// Each Comment the Language Map finds in `source` from `row` down, as its
/// anchor's column and row and its last Cell's column.
///
fn comments_from(source: &SourceCommander, row: usize) -> Vec<(usize, usize, usize)> {
    let mut comments = Vec::new();
    source.read_source(|source| {
        comments = source
            .language_map()
            .units()
            .filter(|unit| matches!(unit.kind(), LanguageUnitKind::Comment(_)))
            .filter(|unit| unit.anchor().y() >= row)
            .map(|unit| {
                let end = unit.span().positions().last().expect("a Span holds a Cell");
                (unit.anchor().x(), unit.anchor().y(), end.x())
            })
            .collect();
    });
    comments
}

///
/// The Learn Orca tracker's voices in Source order: two staircases of four,
/// voice `v` of group `g` on channel `4g + v` with its Timed Play anchored at
/// column `2 + 28g + 7v + 2` on row `13 - 3v`. `notes` places a Note at a
/// channel's step; every other step is empty.
///
fn orca_tracker_voices(notes: &[(u8, usize, u8)]) -> Vec<Voice> {
    let mut voices = Vec::new();
    for v in (0..4).rev() {
        for group in 0..2 {
            let channel = 4 * group + v;
            voices.push(Voice {
                channel,
                play: (
                    4 + 28 * usize::from(group) + 7 * usize::from(v),
                    13 - 3 * usize::from(v),
                ),
                steps: steps(notes, channel, 16),
            });
        }
    }
    voices
}

#[test]
fn the_orca_tracker_plays_two_bars_of_its_one_note() {
    // C4 on channel 1 at step 0 and nothing else, as Learn Orca's tracker
    // ships its `3C`: every voice steps once a Tick through sixteen steps, at
    // velocity `77`, the byte Orca's default `f` sends.
    assert_two_loops(
        include_bytes!("../../../../examples/tracker.orcvs"),
        &orca_patch(&orca_tracker_voices(&[(1, 0, 60)])),
    );
}

#[test]
fn the_orca_tracker_frames_its_pattern_rows_in_comments() {
    // Each pattern row is one closed Comment from column 10 to 64, its `||`
    // beside the first and last voices' steps, so the Notes of every voice's
    // step, and the single `|` marking the others, are text the Reads select
    // rather than standalone units the Language Map diagnoses.
    let source = SourceCommander::with_source(
        file::read(include_bytes!("../../../../examples/tracker.orcvs"))
            .expect("the shipped file is a Source File"),
    );
    assert_eq!(
        comments_from(&source, 15),
        (15..31).map(|row| (10, row, 64)).collect::<Vec<_>>()
    );
    source.read_source(|source| {
        let map = source.language_map();
        // The Reads' operands wait on their Adds, and nothing in the pattern
        // is diagnosed.
        assert_eq!(map.lexical_diagnostics().count(), 0);
        assert!(
            map.diagnostics()
                .all(|diagnostic| diagnostic.anchor().y() < 15)
        );
    });
}

#[test]
fn every_orca_tracker_voice_plays_its_own_column_on_its_own_channel() {
    // One Note per voice, each on a different step, so a voice that read
    // another's column, or played on another's channel, is caught.
    let notes: Vec<(u8, usize, u8)> = (0..8)
        .map(|channel| (channel, 2 * usize::from(channel), 60 + channel))
        .collect();
    // The Notes are written into the shipped pattern rows' Comments, over
    // their blank steps once the shipped C4 is blanked too.
    let text = include_str!("../../../../examples/tracker.orcvs").replace("C4", "  ");
    let mut rows: Vec<String> = text.lines().map(str::to_owned).collect();
    for &(channel, step, note) in &notes {
        let column = 2 + 28 * usize::from(channel / 4) + 7 * usize::from(channel % 4) + 10;
        let row = &mut rows[15 + step];
        assert_eq!(
            &row[column - 1..column + 3],
            "|  |",
            "step {step} is marked"
        );
        row.replace_range(column..column + 2, &spelling(note));
    }
    assert_two_loops(
        rows.join("\n").as_bytes(),
        &orca_patch(&orca_tracker_voices(&notes)),
    );
}

#[test]
fn a_half_typed_step_silences_only_its_own_voice() {
    // Clearing the `4` of channel 1's step-0 `C4` leaves `C `, partial input
    // for its Read. The Read is diagnosed, its Timed Play is invalid with it
    // and sends nothing, and every other voice rests silently.
    let source = SourceCommander::with_source(
        file::read(include_bytes!("../../../../examples/tracker.orcvs"))
            .expect("the shipped file is a Source File"),
    );
    let grid = source.grid();
    source.unset(super::cell(grid, 15 * grid.columns() + 20));

    let plan = source.execute(Tick::ZERO);

    let half_typed = plan
        .diagnostics
        .iter()
        .map(|d| (d.anchor().x(), d.anchor().y(), d.message.as_str()))
        .filter(|&(_, row, _)| row == 10)
        .collect::<Vec<_>>();
    assert_eq!(
        half_typed,
        [
            (17, 10, "&v has partial or invalid input"),
            (
                11,
                10,
                "nested computation at column 17, row 10 returned nothing"
            ),
        ]
    );
    assert_eq!(plan.diagnostics.len(), 2, "{:?}", plan.diagnostics);
    assert!(plan.play_commands.is_empty(), "{:?}", plan.play_commands);
}

///
/// The Learn Orca sequencer's ten voices in Source order: voice `v` on
/// channel `v`, its Timed Play anchored at column 4 on row `4 + 3v`, reading
/// thirty-two steps along its own row. `notes` places a Note at a channel's
/// step; every other step is empty.
///
fn sequencer_voices(notes: &[(u8, usize, u8)]) -> Vec<Voice> {
    (0..10)
        .map(|channel| Voice {
            channel,
            play: (4, 4 + 3 * usize::from(channel)),
            steps: steps(notes, channel, 32),
        })
        .collect()
}

#[test]
fn the_sequencer_ships_with_every_step_resting() {
    // Learn Orca's sequencer ships with no notes, so every voice rests on
    // every step, silently.
    assert_two_loops(
        include_bytes!("../../../../examples/sequencer.orcvs"),
        &orca_patch(&sequencer_voices(&[])),
    );
}

#[test]
fn every_sequencer_voice_plays_its_own_row_on_its_own_channel() {
    // One Note per voice, each on a different step, and channel 0's first
    // and last steps, so a voice that read another's row, played on another's
    // channel or read past either end of its steps, is caught.
    let mut notes: Vec<(u8, usize, u8)> = (0..10)
        .map(|channel| (channel, 3 * usize::from(channel) + 1, 60 + channel))
        .collect();
    notes.extend([(0, 0, 72), (0, 31, 48)]);
    let text = include_str!("../../../../examples/sequencer.orcvs");
    let mut rows: Vec<String> = text.lines().map(str::to_owned).collect();
    for &(channel, step, note) in &notes {
        let row = &mut rows[4 + 3 * usize::from(channel)];
        let column = 20 + 2 * step;
        assert_eq!(&row[column..column + 2], "  ", "step {step} is empty");
        row.replace_range(column..column + 2, &spelling(note));
    }
    assert_two_loops(
        rows.join("\n").as_bytes(),
        &orca_patch(&sequencer_voices(&notes)),
    );
}

#[test]
fn the_sequencer_frames_each_voices_steps_in_a_comment() {
    // Each voice's steps are one closed Comment from column 18 to 85, so the
    // Notes typed into them are text its Read selects rather than standalone
    // units the Language Map diagnoses.
    let source = SourceCommander::with_source(
        file::read(include_bytes!("../../../../examples/sequencer.orcvs"))
            .expect("the shipped file is a Source File"),
    );
    assert_eq!(
        comments_from(&source, 4),
        (0..10).map(|v| (18, 4 + 3 * v, 85)).collect::<Vec<_>>()
    );
    source.read_source(|source| assert_eq!(source.language_map().lexical_diagnostics().count(), 0));
}

///
/// The 808 beat's first bar and the Tick that starts its second: kick `C2` on
/// every beat, snare `D2` on beats 2 and 4 and closed hat `f2` on every
/// eighth, all on channel `08` and each held one Tick, with nothing diagnosed
/// before or during the run.
///
#[test]
fn the_808_stacks_its_instruments_twelve_columns_wide() {
    // Each instrument's label, then its machine, so a phone shows all three.
    let text = include_str!("../../../../examples/808.orcvs");
    let labels: Vec<usize> = text
        .lines()
        .enumerate()
        .filter_map(|(row, line)| line.starts_with("|| ").then_some(row))
        .collect();
    assert_eq!(labels, [1, 5, 10]);
    assert!(text.lines().all(|line| line.len() <= 12), "{text}");
}

#[test]
fn the_808_beat_plays_one_bar_with_nothing_diagnosed() {
    let (kick, snare, hat) = (36, 38, 42);
    let source = SourceCommander::with_source(
        file::read(include_bytes!("../../../../examples/808.orcvs"))
            .expect("the shipped file is a Source File"),
    );
    source.read_source(|source| assert_eq!(source.language_map().diagnostics().count(), 0));
    let adapter = InMemoryOutputAdapter::default();
    let mut run = HandDrivenRun::new(source, adapter.clone());
    run.begin_run();

    for tick in 0..17u64 {
        let plan = run.run_tick(tick);
        let mut delivery = Vec::new();
        // Each hit stops on the Tick after it starts, in the order they started.
        if tick % 4 == 1 {
            delivery.push(stop(8, kick));
        }
        if tick % 8 == 5 {
            delivery.push(stop(8, snare));
        }
        if tick % 2 == 1 {
            delivery.push(stop(8, hat));
        }
        if tick % 4 == 0 {
            delivery.push(note_on(8, 0x7F, kick));
        }
        if tick % 8 == 4 {
            delivery.push(note_on(8, 0x64, snare));
        }
        if tick % 2 == 0 {
            delivery.push(note_on(8, 0x40, hat));
        }
        assert_eq!(tick_diagnostics(&plan), vec![], "Tick {tick}'s diagnostics");
        assert_eq!(
            adapter.command_lists().last(),
            Some(&delivery),
            "Tick {tick}'s delivery"
        );
    }
}
