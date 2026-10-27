//! `MD`, Control Change 1: vibrato on the whole mix.

use std::f32::consts::TAU;

/// How fast the pitch wavers, in Hz.
const RATE: f32 = 5.0;
/// The delay swing at `7F`, in seconds: a delay that swings by `A` at `RATE`
/// bends the pitch by up to `TAU * RATE * A`, here a semitone either way:
/// a semitone up is `2^(1/12)`, about `1.059463`, times the pitch.
const MAX_SWING: f32 = 0.059_463 / (TAU * RATE);

/// Vibrato as a delay line whose delay swings between zero and twice the
/// depth. At depth zero it reads the sample it was just given, so the mix
/// passes unchanged.
pub(crate) struct Vibrato {
    sample_rate: f32,
    line: Vec<f32>,
    write: usize,
    phase: f32,
    swing: f32,
}

impl Vibrato {
    pub(crate) fn new(sample_rate: f32) -> Self {
        Self {
            sample_rate,
            // The longest delay, and the sample either side to interpolate.
            line: vec![0.0; (2.0 * MAX_SWING * sample_rate).ceil() as usize + 2],
            write: 0,
            phase: 0.0,
            swing: 0.0,
        }
    }

    /// Sets the depth from a Control Change value, `00`–`7F`.
    pub(crate) fn set(&mut self, value: u8) {
        self.swing = MAX_SWING * f32::from(value) / 127.0;
    }

    /// The vibrato's output for the mix's next sample.
    pub(crate) fn process(&mut self, input: f32) -> f32 {
        let length = self.line.len();
        self.line[self.write] = input;
        let delay = self.swing * (1.0 + (self.phase * TAU).sin()) * self.sample_rate;
        self.phase = (self.phase + RATE / self.sample_rate).fract();
        let whole = delay.floor();
        let fraction = delay - whole;
        let newer = self.line[(self.write + length - whole as usize) % length];
        let older = self.line[(self.write + length - whole as usize - 1) % length];
        self.write = (self.write + 1) % length;
        newer + (older - newer) * fraction
    }
}
