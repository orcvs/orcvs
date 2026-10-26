//! The 808-style kit on channels `8`–`15`, rebuilt from what the 808's
//! circuits do rather than from recordings.

use std::f32::consts::{PI, TAU};

/// A drum is silent, and its voice free, once its envelope falls below this.
const SILENT: f32 = 0.001;

/// The six square oscillators, at clashing pitches, that make the 808's hats
/// and cymbal sound metallic.
const METAL: [f32; 6] = [205.3, 304.4, 369.6, 522.7, 540.0, 800.0];

/// The kit, in General MIDI's order from C: the Note's pitch class picks the
/// instrument and its octave is ignored.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Instrument {
    Kick,
    Rim,
    Snare,
    Clap,
    Cowbell,
    LowTom,
    ClosedHat,
    MidTom,
    Cymbal,
    HighTom,
    OpenHat,
    Conga,
}

impl Instrument {
    pub(crate) fn of(note: u8) -> Self {
        match note % 12 {
            0 => Self::Kick,
            1 => Self::Rim,
            2 => Self::Snare,
            3 => Self::Clap,
            4 => Self::Cowbell,
            5 => Self::LowTom,
            6 => Self::ClosedHat,
            7 => Self::MidTom,
            8 => Self::Cymbal,
            9 => Self::HighTom,
            10 => Self::OpenHat,
            _ => Self::Conga,
        }
    }
}

/// One drum hit. It ignores Note Off and rings out its own decay.
pub(crate) struct Drum {
    instrument: Instrument,
    time: f32,
    phases: [f32; 6],
    noise: u32,
    first: Svf,
    second: Svf,
}

impl Drum {
    pub(crate) fn new(instrument: Instrument, sample_rate: f32) -> Self {
        let filter = |cutoff, q| Svf::new(cutoff, q, sample_rate);
        let (first, second) = match instrument {
            Instrument::Snare => (filter(1800.0, 0.7), Svf::default()),
            Instrument::Clap => (filter(1100.0, 1.5), Svf::default()),
            Instrument::Rim => (filter(2500.0, 2.0), Svf::default()),
            Instrument::Cowbell => (filter(800.0, 2.5), Svf::default()),
            Instrument::ClosedHat | Instrument::OpenHat => {
                (filter(7100.0, 1.0), filter(6000.0, 0.7))
            }
            Instrument::Cymbal => (filter(4500.0, 0.8), filter(3000.0, 0.7)),
            Instrument::Kick
            | Instrument::LowTom
            | Instrument::MidTom
            | Instrument::HighTom
            | Instrument::Conga => (Svf::default(), Svf::default()),
        };
        Self {
            instrument,
            time: 0.0,
            phases: [0.0; 6],
            noise: 0x1234_5678,
            first,
            second,
        }
    }

    pub(crate) fn instrument(&self) -> Instrument {
        self.instrument
    }

    /// The next sample, or `None` once the hit has decayed to silence.
    pub(crate) fn sample(&mut self, dt: f32) -> Option<f32> {
        let (value, envelope) = self.next(dt);
        (envelope >= SILENT).then_some(value * envelope)
    }

    /// The next raw sample and the loudness envelope it plays under.
    fn next(&mut self, dt: f32) -> (f32, f32) {
        let t = self.time;
        self.time += dt;
        match self.instrument {
            Instrument::Kick => {
                let frequency = 52.0 + 90.0 * (-t / 0.012).exp();
                let body = (1.8 * self.sine(0, frequency, dt)).tanh();
                let click = if t < 0.002 { 0.5 * self.noise() } else { 0.0 };
                (body + click, (-t / 0.45).exp())
            }
            Instrument::Snare => {
                let tone = 0.6 * self.sine(0, 180.0, dt) * (-t / 0.05).exp()
                    + 0.4 * self.sine(1, 330.0, dt) * (-t / 0.035).exp();
                let noise = self.noise();
                let snappy = self.first.high(noise);
                (tone + 0.8 * snappy, (-t / 0.11).exp())
            }
            Instrument::Clap => {
                let noise = self.noise();
                let band = self.first.band(noise);
                let burst = [0.0, 0.011, 0.022]
                    .iter()
                    .filter(|&&start| t >= start)
                    .map(|&start| (-(t - start) / 0.004).exp())
                    .fold(0.0, f32::max);
                let tail = if t >= 0.033 {
                    (-(t - 0.033) / 0.12).exp()
                } else {
                    0.0
                };
                (3.0 * band, burst.max(tail))
            }
            Instrument::Rim => {
                let tone = self.sine(0, 1700.0, dt);
                let noise = self.noise();
                (
                    0.6 * tone + 0.6 * self.first.band(noise),
                    (-t / 0.012).exp(),
                )
            }
            Instrument::Cowbell => {
                let tone = 0.5 * self.square(0, 540.0, dt) + 0.5 * self.square(1, 800.0, dt);
                let envelope = 0.6 * (-t / 0.015).exp() + 0.4 * (-t / 0.18).exp();
                (0.6 * self.first.band(tone), envelope)
            }
            Instrument::ClosedHat | Instrument::OpenHat | Instrument::Cymbal => {
                let metal = (0..6)
                    .map(|index| self.square(index, METAL[index], dt))
                    .sum::<f32>()
                    / 6.0;
                let band = self.first.band(metal);
                let decay = match self.instrument {
                    Instrument::ClosedHat => 0.045,
                    Instrument::OpenHat => 0.35,
                    _ => 0.9,
                };
                (2.5 * self.second.high(band), (-t / decay).exp())
            }
            Instrument::LowTom => self.tom(90.0, 0.3, dt),
            Instrument::MidTom => self.tom(130.0, 0.25, dt),
            Instrument::HighTom => self.tom(175.0, 0.2, dt),
            Instrument::Conga => self.tom(330.0, 0.12, dt),
        }
    }

    /// A tom or conga: a sine that starts a little sharp and settles.
    fn tom(&mut self, pitch: f32, decay: f32, dt: f32) -> (f32, f32) {
        let frequency = pitch * (1.0 + 0.3 * (-self.time / 0.02).exp());
        (self.sine(0, frequency, dt), (-self.time / decay).exp())
    }

    fn sine(&mut self, index: usize, frequency: f32, dt: f32) -> f32 {
        let phase = self.advance(index, frequency, dt);
        (phase * TAU).sin()
    }

    fn square(&mut self, index: usize, frequency: f32, dt: f32) -> f32 {
        if self.advance(index, frequency, dt) < 0.5 {
            1.0
        } else {
            -1.0
        }
    }

    fn advance(&mut self, index: usize, frequency: f32, dt: f32) -> f32 {
        self.phases[index] = (self.phases[index] + frequency * dt).fract();
        self.phases[index]
    }

    /// White noise from a xorshift generator, so every hit is the same.
    fn noise(&mut self) -> f32 {
        self.noise ^= self.noise << 13;
        self.noise ^= self.noise >> 17;
        self.noise ^= self.noise << 5;
        (self.noise as f32 / u32::MAX as f32) * 2.0 - 1.0
    }
}

/// A state-variable filter (topology-preserving transform) at one cutoff.
#[derive(Clone, Copy, Default)]
struct Svf {
    a1: f32,
    a2: f32,
    a3: f32,
    k: f32,
    ic1: f32,
    ic2: f32,
}

impl Svf {
    fn new(cutoff: f32, q: f32, sample_rate: f32) -> Self {
        let g = (PI * cutoff / sample_rate).tan();
        let k = 1.0 / q;
        let a1 = 1.0 / (1.0 + g * (g + k));
        let a2 = g * a1;
        Self {
            a1,
            a2,
            a3: g * a2,
            k,
            ic1: 0.0,
            ic2: 0.0,
        }
    }

    /// Advances one sample and answers its band-pass and high-pass outputs.
    fn tick(&mut self, input: f32) -> (f32, f32) {
        let v3 = input - self.ic2;
        let v1 = self.a1 * self.ic1 + self.a2 * v3;
        let v2 = self.ic2 + self.a2 * self.ic1 + self.a3 * v3;
        self.ic1 = 2.0 * v1 - self.ic1;
        self.ic2 = 2.0 * v2 - self.ic2;
        (v1, input - self.k * v1 - v2)
    }

    fn band(&mut self, input: f32) -> f32 {
        self.tick(input).0
    }

    fn high(&mut self, input: f32) -> f32 {
        self.tick(input).1
    }
}
