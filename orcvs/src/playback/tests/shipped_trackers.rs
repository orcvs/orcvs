//! The tracker Source Files in `examples/`, loaded exactly as shipped and run
//! through Source/Tick and Playback for two complete loops.
//!
//! Each run is a hand-driven Playback run over the file's Source, so a Tick's
//! diagnostics are the Tick Plan's and its delivery is what the adapter is
//! handed. The expected notes are written out per voice beside the file they
//! describe, so a change to either shows up here.

use super::{HandDrivenRun, note_on, stop};
use crate::playback::{InMemoryOutputAdapter, OutputCommand};
use crate::source::{
    Length, MidiChannel, Note, PlayCommand, SourceCommander, Tick, TickPlan, Velocity, file,
};

/// Ticks per step: both trackers' Clocks are `~.02 08`.
const STEP_TICKS: u64 = 2;
/// Steps per loop: both Clocks wrap at `08`.
const STEPS: u64 = 8;
/// Every Timed Play in both trackers has length `04`.
const LENGTH: u64 = 4;

/// What one Tick delivers to the adapter, and its diagnostics as anchor
/// column, anchor row and message.
type Expected = (Vec<OutputCommand>, Vec<(usize, usize, String)>);

/// One voice of a tracker: its MIDI channel, where its Timed Play is
/// anchored, and the Note each step holds, `None` for a deliberately empty
/// step.
struct Voice {
    channel: u8,
    play: (usize, usize),
    steps: [Option<u8>; STEPS as usize],
}

impl Voice {
    ///
    /// The diagnostic at this voice's Timed Play on an empty step. A Portal
    /// that fills the note slot clears it, so the Play finds its slot empty; a
    /// nested Read south, six columns east of the anchor, answers a Clear, and
    /// the Play names it.
    ///
    fn rest(&self, slot_written: bool) -> String {
        if slot_written {
            "expected a note, found \"  \"".to_owned()
        } else {
            format!(
                "nested computation at column {}, row {} cleared its operand",
                self.play.0 + 6,
                self.play.1
            )
        }
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
    /// What the run should deliver and diagnose at `tick` for `voices`, and
    /// the notes it sounds afterwards.
    ///
    /// A Delay bangs on the first Tick of each step. Lifetimes that end at
    /// `tick` stop first, in the order they started. An occupied step then
    /// plays its Note for [`LENGTH`] Ticks, stopping the same Note first if it
    /// is still sounding; an empty step's Timed Play is diagnosed at its
    /// anchor, as [`Voice::rest`] describes.
    ///
    fn expect(&mut self, voices: &[Voice], slot_written: bool, tick: u64) -> Expected {
        let mut delivery = Vec::new();
        let mut diagnostics = Vec::new();
        self.0.retain(|&(channel, note, due)| {
            let ends = due == tick;
            if ends {
                delivery.push(stop(channel, note));
            }
            !ends
        });
        if !tick.is_multiple_of(STEP_TICKS) {
            return (delivery, diagnostics);
        }
        let step = ((tick / STEP_TICKS) % STEPS) as usize;
        for voice in voices {
            match voice.steps[step] {
                Some(note) => {
                    let before = self.0.len();
                    self.0
                        .retain(|&(channel, held, _)| (channel, held) != (voice.channel, note));
                    if self.0.len() < before {
                        delivery.push(stop(voice.channel, note));
                    }
                    delivery.push(note_on(voice.channel, 0x64, note));
                    self.0.push((voice.channel, note, tick + LENGTH));
                }
                None => {
                    diagnostics.push((voice.play.0, voice.play.1, voice.rest(slot_written)));
                }
            }
        }
        (delivery, diagnostics)
    }
}

/// The Play Command a tracker's Timed Play sends for `note` on `channel`.
fn timed(channel: u8, note: u8) -> PlayCommand {
    PlayCommand::Timed {
        channel: MidiChannel::try_from(channel).expect("a MIDI channel"),
        velocity: Velocity::try_from(0x64).expect("a MIDI data byte"),
        note: Note::try_from(note).expect("a MIDI note"),
        length: Length::from(0x04),
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
/// every Tick's delivery and diagnostics against `voices`.
///
/// Where a Portal fills a voice's note slot, `slot_written` is true: while the
/// slot is empty the Language Map diagnoses the Timed Play as well. A nested
/// Read in the slot leaves the Map nothing to diagnose, so only the Tick does.
///
fn assert_two_loops(text: &[u8], voices: &[Voice], slot_written: bool) {
    let source =
        SourceCommander::with_source(file::read(text).expect("the shipped file is a Source File"));
    let adapter = InMemoryOutputAdapter::default();
    let mut run = HandDrivenRun::new(source.clone(), adapter.clone());
    run.begin_run();
    let mut sounding = Sounding::default();

    for tick in 0..2 * STEPS * STEP_TICKS {
        let plan = run.run_tick(tick);
        let (commands, diagnostics) = sounding.expect(voices, slot_written, tick);
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
        for voice in voices {
            let step = ((tick / STEP_TICKS) % STEPS) as usize;
            assert_eq!(
                mapped.contains(&voice.play),
                slot_written && voice.steps[step].is_none(),
                "Tick {tick}: the Language Map's view of the Play at {:?}",
                voice.play
            );
        }
    }
    assert_eq!(adapter.command_lists().len() as u64, 2 * STEPS * STEP_TICKS);
}

///
/// The cell tracker's first loop and the Tick that starts its second, written
/// out Tick by Tick rather than derived, so a misreading shared by
/// [`Sounding::expect`] and Playback cannot pass both.
///
#[test]
fn the_cell_tracker_delivers_its_first_loop_as_written_out() {
    let rest = || vec![(2, 2, "expected a note, found \"  \"".to_owned())];
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
    // C4 D4 __ G4 __ A4 C5 __, as `examples/cell-tracker.md` describes.
    assert_two_loops(
        include_bytes!("../../../../examples/cell-tracker.orcvs"),
        &[Voice {
            channel: 0,
            play: (2, 2),
            steps: [
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
        true,
    );
}

#[test]
fn the_four_voice_tracker_plays_two_loops_of_every_voice() {
    // As `examples/tracker.md` describes, one column per voice.
    assert_two_loops(
        include_bytes!("../../../../examples/tracker.orcvs"),
        &[
            Voice {
                channel: 0,
                play: (2, 3),
                steps: [
                    Some(48),
                    None,
                    Some(48),
                    None,
                    Some(43),
                    None,
                    Some(45),
                    Some(48),
                ],
            },
            Voice {
                channel: 1,
                play: (22, 3),
                steps: [
                    None,
                    Some(64),
                    None,
                    Some(64),
                    None,
                    Some(62),
                    None,
                    Some(62),
                ],
            },
            Voice {
                channel: 2,
                play: (42, 3),
                steps: [
                    Some(72),
                    None,
                    Some(76),
                    Some(79),
                    None,
                    Some(76),
                    Some(74),
                    None,
                ],
            },
            Voice {
                channel: 3,
                play: (62, 3),
                steps: [
                    Some(79),
                    Some(81),
                    None,
                    Some(83),
                    Some(84),
                    None,
                    None,
                    Some(83),
                ],
            },
        ],
        false,
    );
}

#[test]
fn a_half_typed_step_silences_only_its_own_voice() {
    // Clearing the `3` of voice 1's step-0 `C3` leaves `C `, partial input
    // for its Read. The Read is diagnosed, its Timed Play is invalid with it
    // and sends nothing, and the other voices play step 0 as before.
    let source = SourceCommander::with_source(
        file::read(include_bytes!("../../../../examples/tracker.orcvs"))
            .expect("the shipped file is a Source File"),
    );
    let grid = source.grid();
    source.unset(super::cell(grid, 4 * grid.columns() + 11));

    let plan = source.execute(Tick::ZERO);

    assert_eq!(
        plan.diagnostics
            .iter()
            .map(|d| (d.anchor().x(), d.anchor().y(), d.message.as_str()))
            .collect::<Vec<_>>(),
        [
            (8, 3, "&v has partial or invalid input"),
            (
                2,
                3,
                "nested computation at column 8, row 3 returned nothing"
            ),
            (
                22,
                3,
                "nested computation at column 28, row 3 cleared its operand"
            ),
        ]
    );
    assert_eq!(plan.play_commands, [timed(2, 72), timed(3, 79)]);
}
