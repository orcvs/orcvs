//! The knobs Source File in `examples/`, loaded exactly as shipped and run
//! through Source/Tick and Playback.
//!
//! Every Tick the bassline plays its next step, and each knob sends one
//! Control Change to its Built-in Synth effect. A knob's value glides by at
//! most [`RATE`] a Tick toward its Clock, which sweeps up a step at a time and
//! wraps to `00`. The bassline and knobs are written out here rather than read
//! from the file, so a change to either shows up.

use super::{HandDrivenRun, note_on, stop};
use crate::playback::{InMemoryOutputAdapter, OutputCommand};
use crate::source::{ControlValue, Controller, MidiChannel, SourceCommander, file};

/// The bassline's sixteen steps, one a Tick, as MIDI notes: A1 A1 A2 A1 C2 A1
/// G2 A1 A1 E2 A1 A2 C3 A1 G2 E2.
const BASS: [u8; 16] = [
    33, 33, 45, 33, 36, 33, 43, 33, 33, 40, 33, 45, 48, 33, 43, 40,
];

/// The bassline's MIDI channel, the Built-in Synth's saw, and its velocity.
const BASS_CHANNEL: u8 = 0x02;
const BASS_VELOCITY: u8 = 0x64;

/// Each knob in Source order as its controller and its Clock's `rate` and
/// `modulus`: `MD`, `OF`, `DL`, `CH`, `R1` and `R2`.
const KNOBS: [(u8, u64, u64); 6] = [
    (0x01, 0x01, 0x80),
    (0x02, 0x03, 0x20),
    (0x04, 0x02, 0x40),
    (0x05, 0x03, 0x40),
    (0x07, 0x02, 0x30),
    (0x08, 0x04, 0x40),
];

/// How far a knob's value moves in one Tick, its Interpolation's `rate`.
const RATE: u8 = 0x08;

/// The Control Change a knob sends to `controller` with `value` on channel `00`.
fn knob_change(controller: u8, value: u8) -> OutputCommand {
    OutputCommand::ControlChange {
        channel: MidiChannel::try_from(0).expect("a MIDI channel"),
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
fn the_knobs_sweep_the_effects_over_the_bassline() {
    let source = SourceCommander::with_source(
        file::read(include_bytes!("../../../../examples/knobs.orcvs"))
            .expect("the shipped file is a Source File"),
    );
    // The bassline's steps and the labels are Comments, so the Language Map
    // diagnoses none of them. Until the first Tick the bassline's Read east
    // waits on its Add, so the Map diagnoses its Timed Play.
    source.read_source(|source| {
        assert_eq!(source.language_map().lexical_diagnostics().count(), 0);
        assert_eq!(
            source
                .language_map()
                .diagnostics()
                .map(|d| (d.anchor().x(), d.anchor().y()))
                .collect::<Vec<_>>(),
            [(4, 3)]
        );
    });
    let adapter = InMemoryOutputAdapter::default();
    let mut run = HandDrivenRun::new(source, adapter.clone());
    run.begin_run();
    let mut values = [0u8; KNOBS.len()];

    // Long enough for `MD`'s Clock to wrap at Tick `80` and its value to
    // glide back down.
    for tick in 0..0x90 {
        let plan = run.run_tick(tick);
        assert!(
            plan.diagnostics.is_empty(),
            "Tick {tick}: {:?}",
            plan.diagnostics
        );
        let step = tick as usize % BASS.len();
        let mut delivery = Vec::new();
        if tick > 0 {
            delivery.push(stop(
                BASS_CHANNEL,
                BASS[(step + BASS.len() - 1) % BASS.len()],
            ));
        }
        delivery.push(note_on(BASS_CHANNEL, BASS_VELOCITY, BASS[step]));
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
