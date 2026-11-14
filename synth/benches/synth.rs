//! Benchmarks for the Built-in Synth's audio path: `Synth::render` filling one
//! block of samples, the call the AudioWorklet makes for every render quantum.
//!
//! Run with `mise run bench`. The `--output-format bencher` flag it passes is not
//! cosmetic: CI parses the output with a regex that only matches that format.

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use std::hint::black_box;
use synth::Synth;

/// The rate the site's AudioWorklet runs the synth at.
const RATE: f32 = 48_000.0;

/// One Web Audio render quantum: the block the worklet asks for at a time.
const BLOCK: usize = 128;

/// Rendered once before measuring, long enough for every controller's smoothing
/// to settle and every drum to ring out, so each measured block is the steady
/// state rather than the onset.
const PREROLL: usize = 4 * RATE as usize;

/// Modulation, Brightness, and the four sends on CC 91–94: the per-channel
/// effects a sounding channel runs on every sample once any is turned.
const MD: u8 = 1;
const EQ: u8 = 74;
const SENDS: [u8; 4] = [91, 92, 93, 94];

/// A synth with one held note on each of `channels`, each channel's effects
/// turned when `effects` is set, rendered through its preroll.
///
/// Tones and the acid voice sound for as long as their note is held. A drum
/// rings out during the preroll, but a channel whose effects are turned runs
/// them on every sample whether or not it sounds, so the kit channels still
/// carry the per-channel cost the count is there to scale.
fn sounding(channels: u8, effects: bool) -> Synth {
    let mut synth = Synth::new(RATE);
    for channel in 0..channels {
        if effects {
            synth.schedule(0, [0xB0 | channel, MD, 0x40]);
            synth.schedule(0, [0xB0 | channel, EQ, 0x20]);
            for send in SENDS {
                synth.schedule(0, [0xB0 | channel, send, 0x40]);
            }
        }
        // Middle C on a tone or the acid voice; the bass drum on the kit.
        let note = if channel < 9 { 60 } else { 36 };
        synth.schedule(0, [0x90 | channel, note, 0x64]);
    }
    let mut preroll = vec![0.0; PREROLL];
    synth.render(&mut preroll);
    synth
}

/// Every channel silent and at rest: the floor every block pays, the shared
/// effects alone.
fn render_idle(c: &mut Criterion) {
    let mut synth = sounding(0, false);
    let mut block = [0.0; BLOCK];
    c.bench_function("synth_render_idle", |b| {
        b.iter(|| synth.render(black_box(&mut block)))
    });
}

/// One held tone with its channel's effects at rest.
fn render_at_rest(c: &mut Criterion) {
    let mut synth = sounding(1, false);
    let mut block = [0.0; BLOCK];
    c.bench_function("synth_render_at_rest", |b| {
        b.iter(|| synth.render(black_box(&mut block)))
    });
}

/// Channels sounding with vibrato, tilt, and every send turned, by how many.
fn render_effects(c: &mut Criterion) {
    let mut group = c.benchmark_group("synth_render_effects");
    for channels in [1, 4, 16] {
        let mut synth = sounding(channels, true);
        let mut block = [0.0; BLOCK];
        group.bench_function(BenchmarkId::from_parameter(channels), |b| {
            b.iter(|| synth.render(black_box(&mut block)))
        });
    }
    group.finish();
}

criterion_group!(benches, render_idle, render_at_rest, render_effects);
criterion_main!(benches);
