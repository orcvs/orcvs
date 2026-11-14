//! The knobs Source File in `examples/`, loaded exactly as shipped and run
//! through Source/Tick and Playback.
//!
//! Every Tick the bassline plays its next step on the Built-in Synth's acid
//! channel, and each knob sends one Control Change: three to the acid voice
//! and eight to the synth's effects. A knob's value slews by at most
//! [`RATE`] a Tick toward its Clock plus a floor, and the Clock sweeps up a
//! step at a time and wraps to `00`. The bassline and knobs are written out
//! here rather than read from the file, so a change to either shows up.

use super::{HandDrivenRun, functions_mapped, note_on, stop, tick_diagnostics};
use crate::playback::{InMemoryOutputAdapter, OutputCommand};
use crate::source::{ControlValue, Controller, MidiChannel, SourceCommander, file};

/// How a step is played: its velocity and how many Ticks it holds its note.
#[derive(Clone, Copy)]
enum Touch {
    /// Velocity `64`, one Tick.
    Plain,
    /// Velocity `7F`, which accents it, one Tick.
    Accent,
    /// Velocity `64`, two Ticks, so the next step's note slides from it.
    Slide,
}

impl Touch {
    fn velocity(self) -> u8 {
        match self {
            Self::Plain | Self::Slide => 0x64,
            Self::Accent => 0x7F,
        }
    }

    fn length(self) -> u64 {
        match self {
            Self::Plain | Self::Accent => 1,
            Self::Slide => 2,
        }
    }
}

/// The bassline's sixteen steps, one a Tick, as MIDI notes and touches: A1
/// A1 A2 A1 C2 A1 G2 A1 A1 E2 A1 A2 C3 A1 G2 E2, accented on steps 0, 4, 9,
/// 12 and 15 and sliding from steps 2, 6 and 10.
const BASS: [(u8, Touch); 16] = {
    use Touch::{Accent, Plain, Slide};
    [
        (33, Accent),
        (33, Plain),
        (45, Slide),
        (33, Plain),
        (36, Accent),
        (33, Plain),
        (43, Slide),
        (33, Plain),
        (33, Plain),
        (40, Accent),
        (33, Slide),
        (45, Plain),
        (48, Accent),
        (33, Plain),
        (43, Plain),
        (40, Accent),
    ]
};

/// The acid channel.
const ACID: u8 = 0x08;

/// The channel the effects' knobs send on. Each effect is global, so any
/// channel would do.
const EFFECTS: u8 = 0x00;

/// One knob: the channel and controller it sends, the floor its Add puts
/// under the Clock, and the Clock's `length` and `steps`.
struct Knob {
    channel: u8,
    controller: u8,
    floor: u8,
    length: u64,
    steps: u64,
}

impl Knob {
    const fn new(channel: u8, controller: u8, floor: u8, length: u64, steps: u64) -> Self {
        Self {
            channel,
            controller,
            floor,
            length,
            steps,
        }
    }

    /// Where the knob's value is gliding at `tick`.
    fn target(&self, tick: u64) -> u8 {
        self.floor + (tick / self.length % self.steps) as u8
    }
}

/// Each knob in Source order: the acid voice's cutoff, resonance and decay,
/// then `MD`, `OF`, `LN`, `DL`, `CH`, `EQ`, `R1` and `R2`.
const KNOBS: [Knob; 11] = [
    Knob::new(ACID, 0x4A, 0x00, 0x01, 0x80),
    Knob::new(ACID, 0x47, 0x00, 0x02, 0x80),
    Knob::new(ACID, 0x4B, 0x00, 0x03, 0x40),
    Knob::new(EFFECTS, 0x01, 0x00, 0x05, 0x40),
    Knob::new(EFFECTS, 0x02, 0x00, 0x07, 0x20),
    Knob::new(EFFECTS, 0x03, 0x40, 0x04, 0x40),
    Knob::new(EFFECTS, 0x04, 0x00, 0x03, 0x40),
    Knob::new(EFFECTS, 0x05, 0x00, 0x02, 0x40),
    Knob::new(EFFECTS, 0x06, 0x30, 0x05, 0x20),
    Knob::new(EFFECTS, 0x07, 0x00, 0x06, 0x30),
    Knob::new(EFFECTS, 0x08, 0x00, 0x08, 0x40),
];

/// How far a knob's value moves in one Tick, its Slew's `amount`.
const RATE: u8 = 0x08;

/// The Control Change a knob sends to `controller` with `value` on `channel`.
fn knob_change(channel: u8, controller: u8, value: u8) -> OutputCommand {
    OutputCommand::ControlChange {
        channel: MidiChannel::try_from(channel).expect("a MIDI channel"),
        controller: Controller::try_from(controller).expect("a MIDI data byte"),
        value: ControlValue::try_from(value).expect("a MIDI data byte"),
    }
}

/// `value` moved toward `target` by at most [`RATE`], never past it.
fn slew(value: u8, target: u8) -> u8 {
    if value < target {
        target.min(value + RATE)
    } else {
        target.max(value.saturating_sub(RATE))
    }
}

#[test]
fn the_knobs_turn_the_acid_voice_and_the_effects_under_an_accented_sliding_bassline() {
    let source = SourceCommander::with_source(
        file::read(include_bytes!("../../../../examples/knobs.orcvs"))
            .expect("the shipped file is a Source File"),
    );
    // The steps and the labels are Comments, and each row's Read south ships
    // with the offset its Add writes on Tick 0, so the Source loads with
    // nothing diagnosed: the web console opens it on a first visit.
    source.read_source(|source| assert_eq!(source.language_map().diagnostics().count(), 0));
    let adapter = InMemoryOutputAdapter::default();
    let mut run = HandDrivenRun::new(source.clone(), adapter.clone());
    run.begin_run();
    let mut values = [0u8; KNOBS.len()];
    // Each sounding note and the Tick its Timed Play ends, in the order they
    // started.
    let mut sounding: Vec<(u8, u64)> = Vec::new();

    // Long enough for the slowest Clock, `R2`'s, to wrap at Tick `200` and
    // its value to ramp back down.
    for tick in 0..0x210 {
        let plan = run.run_tick(tick);
        // The two bassline rows not holding the step read a blank note, which
        // sends nothing and reports nothing.
        assert_eq!(tick_diagnostics(&plan), vec![], "Tick {tick}");
        // Once each Add has written its row's Read offset, the Language Map
        // diagnoses no Function, the blank-note rows' Plays included.
        assert_eq!(
            functions_mapped(&source),
            vec![],
            "Tick {tick}'s Language Map"
        );
        let mut delivery = Vec::new();
        sounding.retain(|&(note, due)| {
            if due == tick {
                delivery.push(stop(ACID, note));
            }
            due != tick
        });
        let (note, touch) = BASS[tick as usize % BASS.len()];
        delivery.push(note_on(ACID, touch.velocity(), note));
        sounding.push((note, tick + touch.length()));
        for (value, knob) in values.iter_mut().zip(&KNOBS) {
            *value = slew(*value, knob.target(tick));
            delivery.push(knob_change(knob.channel, knob.controller, *value));
        }
        assert_eq!(
            adapter.command_lists().last(),
            Some(&delivery),
            "Tick {tick}'s delivery"
        );
    }
}

#[test]
fn the_knobs_stack_their_steps_down_the_rows_twenty_four_columns_wide() {
    // Each of the sixteen steps is a row of its own, its Note in the column of
    // the row that plays it, so no line outgrows a phone's view: 300 points at
    // 12 points a Cell.
    let text = include_str!("../../../../examples/knobs.orcvs");
    let steps: Vec<usize> = text
        .lines()
        .enumerate()
        .filter_map(|(row, line)| (line.matches('|').count() == 6).then_some(row))
        .collect();
    assert_eq!(steps, (16..16 + BASS.len()).collect::<Vec<_>>(), "{text}");
    assert!(text.lines().all(|line| line.len() <= 24), "{text}");
}
