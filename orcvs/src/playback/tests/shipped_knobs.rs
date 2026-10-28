//! The knobs Source File in `examples/`, loaded exactly as shipped and run
//! through Source/Tick and Playback while its preset changes.
//!
//! Every Tick each of the eight knobs sends one Control Change, whose value
//! moves toward the knob's value in the selected preset by at most
//! [`RATE`]. The presets are written out here rather than read from the file,
//! so a change to either shows up.

use super::HandDrivenRun;
use crate::playback::{InMemoryOutputAdapter, OutputCommand};
use crate::source::{ControlValue, Controller, MidiChannel, SourceCommander, file};

/// Each knob's value in each preset: `PRESETS[preset][knob]`.
const PRESETS: [[u8; 8]; 6] = [
    [0x00, 0x0B, 0x00, 0x00, 0x00, 0x00, 0x00, 0x42],
    [0x00, 0x37, 0x00, 0x21, 0x0B, 0x16, 0x16, 0x42],
    [0x0F, 0x0F, 0x08, 0x13, 0x00, 0x37, 0x37, 0x42],
    [0x37, 0x37, 0x00, 0x21, 0x0B, 0x16, 0x16, 0x21],
    [0x37, 0x0F, 0x04, 0x13, 0x3B, 0x37, 0x37, 0x4D],
    [0x37, 0x37, 0x00, 0x21, 0x0B, 0x16, 0x16, 0x42],
];

/// How far a knob's value moves in one Tick, its Interpolation's `rate`.
const RATE: u8 = 0x04;

/// The Cell holding the preset's low digit: column 23, row 0.
const PRESET_DIGIT: usize = 23;

///
/// The Control Change knob `knob` sends with `value`: channel `00`,
/// controller `41` onward, the controllers Orca's `!` sends knobs `1` to `8`
/// to.
///
fn knob_change(knob: u8, value: u8) -> OutputCommand {
    OutputCommand::ControlChange {
        channel: MidiChannel::try_from(0).expect("a MIDI channel"),
        controller: Controller::try_from(0x41 + knob).expect("a MIDI data byte"),
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
fn the_knobs_glide_to_each_preset_selected_and_hold_it() {
    let source = SourceCommander::with_source(
        file::read(include_bytes!("../../../../examples/knobs.orcvs"))
            .expect("the shipped file is a Source File"),
    );
    // The preset and every knob's presets are Comment text the Reads select,
    // so the Language Map diagnoses none of them. Until the first Tick each
    // Read's `n` waits on its Add, so the Map diagnoses each Control Change.
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
        (0..8).map(|knob| (4, 2 + 3 * knob)).collect::<Vec<_>>()
    );
    let adapter = InMemoryOutputAdapter::default();
    let mut run = HandDrivenRun::new(source.clone(), adapter.clone());
    run.begin_run();
    let mut values = [0u8; 8];

    // Preset 00 as shipped, then 04, then 02: long enough for every knob to
    // reach its value from anywhere in the byte's lower half and hold it.
    let mut tick = 0;
    for preset in [0, 4, 2] {
        source
            .set(
                super::cell(source.grid(), PRESET_DIGIT),
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
                mapped(&source).iter().all(|&(column, _)| column < 22),
                "Tick {tick}'s Language Map"
            );
            let mut delivery = Vec::new();
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
