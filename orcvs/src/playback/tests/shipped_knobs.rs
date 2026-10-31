//! The knobs Source File in `examples/`, loaded exactly as shipped and run
//! through Source/Tick and Playback while its preset changes.
//!
//! Every Tick the bassline plays its next step, and each of the eight knobs
//! sends one Control Change to its Built-in Synth effect, whose value moves
//! toward the knob's value in the selected preset by at most [`RATE`]. The
//! bassline and presets are written out here rather than read from the file,
//! so a change to either shows up.

use super::{HandDrivenRun, note_on, stop};
use crate::playback::{InMemoryOutputAdapter, OutputCommand};
use crate::source::{
    ControlValue, Controller, LanguageUnitKind, MidiChannel, SourceCommander, file,
};

/// The bassline's sixteen steps, one a Tick, as MIDI notes: A1 A1 A2 A1 C2 A1
/// G2 A1 A1 E2 A1 A2 C3 A1 G2 E2.
const BASS: [u8; 16] = [
    33, 33, 45, 33, 36, 33, 43, 33, 33, 40, 33, 45, 48, 33, 43, 40,
];

/// The bassline's MIDI channel, the Built-in Synth's saw, and its velocity.
const BASS_CHANNEL: u8 = 0x02;
const BASS_VELOCITY: u8 = 0x64;

/// Each knob's value in each preset: `PRESETS[preset][knob]`. Knob `k` sets
/// effect `k + 1`: `MD`, `OF`, `LN`, `DL`, `CH`, `EQ`, `R1` and `R2`.
const PRESETS: [[u8; 8]; 6] = [
    [0x30, 0x00, 0x7F, 0x00, 0x00, 0x40, 0x00, 0x00],
    [0x00, 0x00, 0x7F, 0x00, 0x00, 0x40, 0x00, 0x00],
    [0x10, 0x20, 0x40, 0x30, 0x00, 0x30, 0x30, 0x00],
    [0x7F, 0x00, 0x7F, 0x00, 0x40, 0x40, 0x00, 0x00],
    [0x20, 0x00, 0x60, 0x40, 0x00, 0x60, 0x00, 0x40],
    [0x50, 0x10, 0x7F, 0x20, 0x30, 0x40, 0x20, 0x40],
];

/// How far a knob's value moves in one Tick, its Interpolation's `rate`.
const RATE: u8 = 0x08;

/// The Cell holding the preset's low digit: column 23 of row 4.
const PRESET_DIGIT: (usize, usize) = (23, 4);

///
/// The Control Change knob `knob` sends with `value`: channel `00`,
/// controller `01` onward, the Built-in Synth's effects in order.
///
fn knob_change(knob: u8, value: u8) -> OutputCommand {
    OutputCommand::ControlChange {
        channel: MidiChannel::try_from(0).expect("a MIDI channel"),
        controller: Controller::try_from(0x01 + knob).expect("a MIDI data byte"),
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
fn the_knobs_glide_the_effects_to_each_preset_over_the_bassline() {
    let source = SourceCommander::with_source(
        file::read(include_bytes!("../../../../examples/knobs.orcvs"))
            .expect("the shipped file is a Source File"),
    );
    // The bassline's steps, the preset and every knob's presets are Comment
    // text the Reads select, so the Language Map diagnoses none of them.
    // Until the first Tick each Read's `n` waits on its Add, so the Map
    // diagnoses the bassline's Timed Play and each Control Change.
    source.read_source(|source| assert_eq!(source.language_map().lexical_diagnostics().count(), 0));
    let mapped = |source: &SourceCommander| {
        let mut mapped = Vec::new();
        source.read_source(|source| {
            mapped = source
                .language_map()
                .diagnostics()
                .map(|d| (d.anchor().x(), d.anchor().y()))
                .collect();
        });
        mapped
    };
    assert_eq!(
        mapped(&source),
        std::iter::once((4, 2))
            .chain((0..8).map(|knob| (4, 6 + 3 * knob)))
            .collect::<Vec<_>>()
    );
    // Each Comment as its anchor's column and row and its last Cell's column:
    // the bassline's steps, the preset and the eight knobs' presets.
    let mut comments = Vec::new();
    source.read_source(|source| {
        comments = source
            .language_map()
            .units()
            .filter(|unit| matches!(unit.kind(), LanguageUnitKind::Comment(_)))
            .map(|unit| {
                let end = unit.span().positions().last().expect("a Span holds a Cell");
                (unit.anchor().x(), unit.anchor().y(), end.x())
            })
            .collect::<Vec<_>>();
    });
    assert_eq!(comments.len(), 10);
    let adapter = InMemoryOutputAdapter::default();
    let mut run = HandDrivenRun::new(source.clone(), adapter.clone());
    run.begin_run();
    let mut values = [0u8; 8];

    // Preset 00 as shipped, then 04, then 02: long enough for every knob to
    // reach its value from anywhere in the byte and hold it.
    let mut tick = 0;
    for preset in [0, 4, 2] {
        source
            .set(
                super::cell(
                    source.grid(),
                    PRESET_DIGIT.1 * source.grid().columns() + PRESET_DIGIT.0,
                ),
                &preset.to_string(),
            )
            .expect("a digit is one Cell");
        for _ in 0..24 {
            let plan = run.run_tick(tick);
            assert!(
                plan.diagnostics.is_empty(),
                "Tick {tick}: {:?}",
                plan.diagnostics
            );
            // The Cells the nested Functions write below themselves are
            // diagnosed as they are in the tracker; the Comments never are.
            assert!(
                mapped(&source).iter().all(|&(column, row)| {
                    !comments
                        .iter()
                        .any(|&(start, at, end)| at == row && (start..=end).contains(&column))
                }),
                "Tick {tick}'s Language Map"
            );
            let mut delivery = Vec::new();
            let step = tick as usize % BASS.len();
            if tick > 0 {
                delivery.push(stop(
                    BASS_CHANNEL,
                    BASS[(step + BASS.len() - 1) % BASS.len()],
                ));
            }
            delivery.push(note_on(BASS_CHANNEL, BASS_VELOCITY, BASS[step]));
            for (knob, value) in values.iter_mut().enumerate() {
                *value = glide(*value, PRESETS[preset][knob]);
                delivery.push(knob_change(knob as u8, *value));
            }
            assert_eq!(
                adapter.command_lists().last(),
                Some(&delivery),
                "Tick {tick}'s delivery"
            );
            tick += 1;
        }
        assert_eq!(values, PRESETS[preset], "preset {preset} is reached");
    }
}
