//! The knobs Source File in `examples/`, loaded exactly as shipped and run
//! through Source/Tick and Playback.
//!
//! Every Tick the bassline plays its next step on the Built-in Synth's acid
//! channel, and each knob sends one Control Change to the acid voice. A knob's
//! value glides by at most [`RATE`] a Tick toward its Clock, which sweeps up a
//! step at a time and wraps to `00`. The bassline and knobs are written out
//! here rather than read from the file, so a change to either shows up.

use super::{HandDrivenRun, note_on, stop};
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
const ACID: u8 = 0x09;

/// Each knob in Source order as its controller and its Clock's `rate` and
/// `modulus`: cutoff, resonance and decay.
const KNOBS: [(u8, u64, u64); 3] = [(0x4A, 0x01, 0x80), (0x47, 0x02, 0x80), (0x4B, 0x03, 0x40)];

/// How far a knob's value moves in one Tick, its Interpolation's `rate`.
const RATE: u8 = 0x08;

/// The Control Change a knob sends to `controller` with `value` on the acid
/// channel.
fn knob_change(controller: u8, value: u8) -> OutputCommand {
    OutputCommand::ControlChange {
        channel: MidiChannel::try_from(ACID).expect("a MIDI channel"),
        controller: Controller::try_from(controller).expect("a MIDI data byte"),
        value: ControlValue::try_from(value).expect("a MIDI data byte"),
    }
}

/// `value` moved toward `target` by at most [`RATE`], never past it.
fn glide(value: u8, target: u8) -> u8 {
    if value < target {
        target.min(value + RATE)
    } else {
        target.max(value.saturating_sub(RATE))
    }
}

#[test]
fn the_knobs_turn_the_acid_voice_under_an_accented_sliding_bassline() {
    let source = SourceCommander::with_source(
        file::read(include_bytes!("../../../../examples/knobs.orcvs"))
            .expect("the shipped file is a Source File"),
    );
    // The steps and the labels are Comments, so the Language Map diagnoses
    // none of them. Until the first Tick each row's Read east waits on its
    // Add, so the Map diagnoses its Timed Play.
    source.read_source(|source| {
        assert_eq!(source.language_map().lexical_diagnostics().count(), 0);
        assert_eq!(
            source
                .language_map()
                .diagnostics()
                .map(|d| (d.anchor().x(), d.anchor().y()))
                .collect::<Vec<_>>(),
            [(4, 3), (4, 7), (4, 11)]
        );
    });
    let adapter = InMemoryOutputAdapter::default();
    let mut run = HandDrivenRun::new(source, adapter.clone());
    run.begin_run();
    let mut values = [0u8; KNOBS.len()];
    // Each sounding note and the Tick its Timed Play ends, in the order they
    // started.
    let mut sounding: Vec<(u8, u64)> = Vec::new();

    // Long enough for the cutoff's Clock to wrap at Tick `80` and its value
    // to glide back down.
    for tick in 0..0x90 {
        let plan = run.run_tick(tick);
        assert!(
            plan.diagnostics.is_empty(),
            "Tick {tick}: {:?}",
            plan.diagnostics
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
        for (value, &(controller, rate, modulus)) in values.iter_mut().zip(&KNOBS) {
            *value = glide(*value, (tick / rate % modulus) as u8);
            delivery.push(knob_change(controller, *value));
        }
        assert_eq!(
            adapter.command_lists().last(),
            Some(&delivery),
            "Tick {tick}'s delivery"
        );
    }
}
