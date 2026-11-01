//! `CH`, Control Change 5: a chorus on the whole mix.

use std::f32::consts::TAU;

use crate::glide::Glide;

/// The two taps' middle delay, in seconds.
const CENTRE: f32 = 0.012;
/// How far each tap's delay swings either side of `CENTRE`, in seconds.
const SWING: f32 = 0.004;
/// How fast the taps swing, in Hz.
const RATE: f32 = 0.7;

/// Two delay taps that swing in opposite phase, mixed in with the dry mix up
/// to half and half at the chorus's level. At level zero the mix passes
/// unchanged.
pub(crate) struct Chorus {
    sample_rate: f32,
    line: Vec<f32>,
    write: usize,
    phase: f32,
    level: Glide,
}

impl Chorus {
    pub(crate) fn new(sample_rate: f32) -> Self {
        Self {
            sample_rate,
            // The longest delay, and the sample either side to interpolate.
            line: vec![0.0; ((CENTRE + SWING) * sample_rate).ceil() as usize + 2],
            write: 0,
            phase: 0.0,
            level: Glide::new(sample_rate, 0.0),
        }
    }

    /// Sets the chorus's level from a Control Change value, `00`–`7F`.
    pub(crate) fn set(&mut self, value: u8) {
        self.level.set(f32::from(value) / 127.0);
    }

    /// The mix's next sample through the chorus.
    pub(crate) fn process(&mut self, input: f32) -> f32 {
        self.line[self.write] = input;
        let swing = (self.phase * TAU).sin();
        let wet = 0.5 * (self.tap(CENTRE + SWING * swing) + self.tap(CENTRE - SWING * swing));
        self.phase = (self.phase + RATE / self.sample_rate).fract();
        self.write = (self.write + 1) % self.line.len();
        input + 0.5 * self.level.next() * (wet - input)
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
