//! `MD`, Control Change 1 (Modulation): vibrato on one channel.

use std::f32::consts::TAU;

/// How fast the pitch wavers, in Hz.
const RATE: f32 = 5.0;
/// The delay swing at `7F`, in seconds: a delay that swings by `A` at `RATE`
/// bends the pitch by up to `TAU * RATE * A`, here a semitone either way:
/// a semitone up is `2^(1/12)`, about `1.059463`, times the pitch.
const MAX_SWING: f32 = 0.059_463 / (TAU * RATE);
/// How long the swing takes to move from none to `MAX_SWING`, in seconds. A
/// swing that jumped would step the delay, and click, whenever the depth
/// changes mid-note.
const SMOOTHING: f32 = 0.05;

/// Vibrato as a delay line whose delay swings between zero and twice the
/// depth. At depth zero it reads the sample it was just given, so the channel
/// passes unchanged.
///
/// A depth change moves there over `SMOOTHING` at most, so the delay never
/// steps, and reaches the new depth exactly.
pub(crate) struct Vibrato {
    sample_rate: f32,
    line: Vec<f32>,
    write: usize,
    phase: f32,
    swing: f32,
    target: f32,
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
            target: 0.0,
        }
    }

    /// Sets the depth from a Control Change value, `00`–`7F`.
    pub(crate) fn set(&mut self, value: u8) {
        self.target = MAX_SWING * f32::from(value) / 127.0;
    }

    /// Whether its depth is zero and staying there, so it passes the
    /// channel unchanged.
    pub(crate) fn is_still(&self) -> bool {
        self.swing == 0.0 && self.target == 0.0
    }

    /// The vibrato's output for the channel's next sample.
    pub(crate) fn process(&mut self, input: f32) -> f32 {
        let length = self.line.len();
        self.line[self.write] = input;
        let step = MAX_SWING / (SMOOTHING * self.sample_rate);
        let gap = self.target - self.swing;
        self.swing = if gap.abs() <= step {
            self.target
        } else {
            self.swing + step.copysign(gap)
        };
        // At no swing the delay is zero whatever the phase, so the sample
        // just written is the output; the line still takes every sample, so a
        // depth turned up later swings over what the channel played.
        if self.swing == 0.0 {
            self.phase = (self.phase + RATE / self.sample_rate).fract();
            self.write = (self.write + 1) % length;
            return input;
        }
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
