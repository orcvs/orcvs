//! `EQ`, Control Change 74 (Brightness): one tilt on one channel.

use std::f32::consts::TAU;

use crate::smoothing::Smoothing;

/// The frequency the tilt pivots on, in Hz.
const PIVOT: f32 = 800.0;
/// The gain above the pivot at full tilt, and its inverse below: 12 dB.
const RANGE: f32 = 4.0;

/// The channel split at `PIVOT` by a one-pole lowpass, each side raised or
/// lowered by as much as the other is lowered or raised. At `40` the gains
/// are exactly one and the channel passes unchanged.
pub(crate) struct Tilt {
    coefficient: f32,
    low: f32,
    /// From `-1`, darkest, through `0`, flat, to just under `1`, brightest.
    tilt: Smoothing,
}

/// The tilt at a Control Change value, from `-1` to just under `1`: `40` is
/// flat, lower is darker and higher is brighter.
pub(crate) const fn amount(value: u8) -> f32 {
    (value as f32 - 64.0) / 64.0
}

impl Tilt {
    /// A tilt at Control Change `value` from the start.
    pub(crate) fn new(sample_rate: f32, value: u8) -> Self {
        Self {
            coefficient: 1.0 - (-TAU * PIVOT / sample_rate).exp(),
            low: 0.0,
            tilt: Smoothing::new(sample_rate, amount(value)),
        }
    }

    /// Sets the tilt from a Control Change value.
    pub(crate) fn set(&mut self, value: u8) {
        self.tilt.set(amount(value));
    }

    /// Whether it is flat and staying there, so it passes the channel
    /// unchanged.
    pub(crate) fn rests(&self) -> bool {
        self.tilt.rests_at(0.0)
    }

    /// Forgets what the channel played, as the filter settles in silence.
    pub(crate) fn clear(&mut self) {
        self.low = 0.0;
    }

    /// The channel's next sample, tilted.
    pub(crate) fn process(&mut self, input: f32) -> f32 {
        self.low += (input - self.low) * self.coefficient;
        let tilt = self.tilt.next();
        // Flat, both gains are one.
        if tilt == 0.0 {
            return input;
        }
        let high = input - self.low;
        let bright = RANGE.powf(tilt);
        input + (bright - 1.0) * high + (1.0 / bright - 1.0) * self.low
    }
}
