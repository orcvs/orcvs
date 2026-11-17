//! `CH`, Control Change 93 (Chorus Send): one chorus every channel sends to.

use std::f32::consts::TAU;

/// The two taps' middle delay, in seconds.
const CENTRE: f32 = 0.012;
/// How far each tap's delay swings either side of `CENTRE`, in seconds.
const SWING: f32 = 0.004;
/// How fast the taps swing, in Hz.
const RATE: f32 = 0.7;

/// Two delay taps that swing in opposite phase. It answers their average
/// alone; a channel sending at its full level mixes half itself with half
/// that, and at level zero passes unchanged.
pub(crate) struct Chorus {
    sample_rate: f32,
    line: Vec<f32>,
    write: usize,
    phase: f32,
}

impl Chorus {
    pub(crate) fn new(sample_rate: f32) -> Self {
        Self {
            sample_rate,
            // The longest delay, and the sample either side to interpolate.
            line: vec![0.0; ((CENTRE + SWING) * sample_rate).ceil() as usize + 2],
            write: 0,
            phase: 0.0,
        }
    }

    /// The chorus for the next sample the channels send it.
    pub(crate) fn process(&mut self, input: f32) -> f32 {
        self.line[self.write] = input;
        let swing = (self.phase * TAU).sin();
        let wet = 0.5 * (self.tap(CENTRE + SWING * swing) + self.tap(CENTRE - SWING * swing));
        self.phase = (self.phase + RATE / self.sample_rate).fract();
        self.write = (self.write + 1) % self.line.len();
        wet
    }

    /// The line `delay` seconds before the sample just written, interpolated.
    fn tap(&self, delay: f32) -> f32 {
        let length = self.line.len();
        let delay = delay * self.sample_rate;
        let whole = delay.floor() as usize;
        let fraction = delay - delay.floor();
        let newer = self.line[(self.write + length - whole) % length];
        let older = self.line[(self.write + length - whole - 1) % length];
        newer + (older - newer) * fraction
    }
}
