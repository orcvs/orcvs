//! The tone channels, `0`–`7`.

use std::f32::consts::TAU;

const ATTACK: f32 = 0.005;
const RELEASE: f32 = 0.15;

/// One tone: a waveform its channel picks, under one shared envelope that
/// rises over 5 ms, holds while the note is held, and falls over 150 ms once
/// it is released.
pub(crate) struct Tone {
    shape: Shape,
    frequency: f32,
    phase: f32,
    modulator: f32,
    time: f32,
    envelope: f32,
    held: bool,
}

#[derive(Clone, Copy)]
enum Shape {
    Sine,
    Triangle,
    Saw,
    Square,
    /// Two-operator FM: the modulator runs at `ratio` times the carrier and
    /// deepens the carrier's phase by `index`, which decays with the time
    /// constant `decay` in seconds, so a bell starts bright and mellows.
    Fm {
        ratio: f32,
        index: f32,
        decay: f32,
    },
    /// AM: the carrier's loudness follows a sine at `harmonicity` times its
    /// pitch.
    Am {
        harmonicity: f32,
    },
}

impl Shape {
    fn of(channel: u8) -> Self {
        match channel {
            0 => Self::Sine,
            1 => Self::Triangle,
            2 => Self::Saw,
            3 => Self::Square,
            4 => Self::Fm {
                ratio: 3.5,
                index: 2.5,
                decay: 0.4,
            },
            5 => Self::Fm {
                ratio: 1.0,
                index: 1.5,
                decay: f32::INFINITY,
            },
            6 => Self::Am { harmonicity: 1.5 },
            _ => Self::Am { harmonicity: 2.0 },
        }
    }
}

impl Tone {
    pub(crate) fn new(channel: u8, note: u8) -> Self {
        Self {
            shape: Shape::of(channel),
            frequency: 440.0 * 2f32.powf((f32::from(note) - 69.0) / 12.0),
            phase: 0.0,
            modulator: 0.0,
            time: 0.0,
            envelope: 0.0,
            held: true,
        }
    }

    pub(crate) fn release(&mut self) {
        self.held = false;
    }

    /// The next sample, or `None` once the release has finished.
    pub(crate) fn sample(&mut self, dt: f32) -> Option<f32> {
        self.envelope = if self.held {
            (self.envelope + dt / ATTACK).min(1.0)
        } else {
            self.envelope - dt / RELEASE
        };
        if self.envelope <= 0.0 && !self.held {
            return None;
        }
        self.time += dt;
        let phase = self.phase;
        self.phase = (self.phase + self.frequency * dt).fract();
        let value = match self.shape {
            Shape::Sine => (phase * TAU).sin(),
            Shape::Triangle => 1.0 - 4.0 * (phase - 0.5).abs(),
            Shape::Saw => 2.0 * phase - 1.0,
            Shape::Square => {
                if phase < 0.5 {
                    1.0
                } else {
                    -1.0
                }
            }
            Shape::Fm {
                ratio,
                index,
                decay,
            } => {
                let modulator = self.advance_modulator(ratio, dt);
                let depth = index * (-self.time / decay).exp();
                (phase * TAU + depth * modulator).sin()
            }
            Shape::Am { harmonicity } => {
                let modulator = self.advance_modulator(harmonicity, dt);
                (phase * TAU).sin() * 0.5 * (1.0 + modulator)
            }
        };
        Some(value * self.envelope.max(0.0))
    }

    fn advance_modulator(&mut self, ratio: f32, dt: f32) -> f32 {
        let value = (self.modulator * TAU).sin();
        self.modulator = (self.modulator + ratio * self.frequency * dt).fract();
        value
    }
}
