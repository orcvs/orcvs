//! `DL`, Control Change 4: an echo of the whole mix.

use crate::glide::Glide;

/// The time between echoes, in seconds: three Ticks at the default tempo.
const TIME: f32 = 0.375;
/// How much of each echo returns in the next one.
const FEEDBACK: f32 = 0.45;
/// The cutoff of the lowpass in the feedback path, in Hz, so each echo is
/// darker than the last.
const DAMPING: f32 = 3_000.0;

/// A feedback delay line whose echoes are added to the mix at its level. At
/// level zero the mix passes unchanged.
pub(crate) struct Delay {
    line: Vec<f32>,
    position: usize,
    damping: f32,
    damped: f32,
    level: Glide,
}

impl Delay {
    pub(crate) fn new(sample_rate: f32) -> Self {
        Self {
            line: vec![0.0; (TIME * sample_rate) as usize],
            position: 0,
            damping: 1.0 - (-std::f32::consts::TAU * DAMPING / sample_rate).exp(),
            damped: 0.0,
            level: Glide::new(sample_rate, 0.0),
        }
    }

    /// Sets the echo's level from a Control Change value, `00`–`7F`.
    pub(crate) fn set(&mut self, value: u8) {
        self.level.set(f32::from(value) / 127.0);
    }

    /// The mix's next sample with its echo.
    pub(crate) fn process(&mut self, input: f32) -> f32 {
        let echo = self.line[self.position];
        self.damped += (echo - self.damped) * self.damping;
        self.line[self.position] = input + FEEDBACK * self.damped;
        self.position = (self.position + 1) % self.line.len();
        input + self.level.next() * echo
    }
}
