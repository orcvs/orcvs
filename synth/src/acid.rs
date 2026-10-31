//! The acid channel, `9` (ADR 0097): one 303-style voice.

use std::f32::consts::TAU;

/// The channel the acid voice plays on.
pub(crate) const CHANNEL: u8 = 9;

/// The time constant of the gate's rise and fall, in seconds: short enough to
/// sound percussive, long enough not to click.
const GATE: f32 = 0.003;

/// The time constant of a slide, in seconds: the pitch is all but there after
/// about 60 ms.
const SLIDE: f32 = 0.02;

/// The lowest velocity that accents a note.
const ACCENT: u8 = 0x78;

/// The acid channel's Control Changes, each `0`–`127`. They hold between
/// notes and apply to the sounding note at once.
#[derive(Clone, Copy)]
pub(crate) struct Controls {
    waveform: u8,
    resonance: u8,
    cutoff: u8,
    decay: u8,
    env_mod: u8,
}

impl Default for Controls {
    /// The load values, which make an acid sound with no Control Change.
    fn default() -> Self {
        Self {
            waveform: 0x00,
            resonance: 0x50,
            cutoff: 0x30,
            decay: 0x30,
            env_mod: 0x50,
        }
    }
}

impl Controls {
    /// Sets `controller` to `value`. Reset All Controllers returns every
    /// control to its load value; any other controller changes nothing.
    pub(crate) fn set(&mut self, controller: u8, value: u8) {
        match controller {
            70 => self.waveform = value,
            71 => self.resonance = value,
            74 => self.cutoff = value,
            75 => self.decay = value,
            79 => self.env_mod = value,
            121 => *self = Self::default(),
            _ => {}
        }
    }
}

/// The voice: a saw, or a square from waveform `40`, through a resonant four-pole lowpass, under a gate. Each
/// struck note opens the filter by an envelope that decays; a note struck while
/// the last is held slides to its pitch and leaves the envelope decaying. An
/// accented note's envelope is at its shortest and also raises its loudness
/// and opens the filter further.
pub(crate) struct Acid {
    frequency: f32,
    target: f32,
    phase: f32,
    held: bool,
    gate: f32,
    envelope: f32,
    accented: bool,
    stages: [f32; 4],
}

impl Acid {
    pub(crate) fn new(note: u8, velocity: u8) -> Self {
        Self {
            frequency: pitch(note),
            target: pitch(note),
            phase: 0.0,
            held: true,
            gate: 0.0,
            envelope: 1.0,
            accented: velocity >= ACCENT,
            stages: [0.0; 4],
        }
    }

    /// Plays `note`: legato from the note held, or struck afresh once the
    /// last has been released.
    pub(crate) fn play(&mut self, note: u8, velocity: u8) {
        self.target = pitch(note);
        if !self.held {
            self.frequency = self.target;
            self.envelope = 1.0;
            self.accented = velocity >= ACCENT;
        }
        self.held = true;
    }

    pub(crate) fn is_held(&self) -> bool {
        self.held
    }

    pub(crate) fn release(&mut self) {
        self.held = false;
    }

    /// The next sample under `controls`, or `None` once the gate has closed.
    pub(crate) fn sample(&mut self, dt: f32, controls: &Controls) -> Option<f32> {
        let goal = if self.held { 1.0 } else { 0.0 };
        self.gate += (goal - self.gate) * (1.0 - (-dt / GATE).exp());
        if !self.held && self.gate < 1e-4 {
            return None;
        }
        self.frequency += (self.target - self.frequency) * (1.0 - (-dt / SLIDE).exp());
        let step = self.frequency * dt;
        self.phase = (self.phase + step).fract();
        let mut wave = saw(self.phase, step);
        if controls.waveform >= 0x40 {
            // A saw less the same saw half a period on is a square.
            wave -= saw((self.phase + 0.5).fract(), step);
        }

        // 200 ms at decay `0` to 2 s at `127`.
        let decay = if self.accented {
            0.2
        } else {
            0.2 * 10f32.powf(f32::from(controls.decay) / 127.0)
        };
        self.envelope *= (-dt / decay).exp();
        let accent = if self.accented { self.envelope } else { 0.0 };
        // 40 Hz at cutoff `0` to about 2.6 kHz at `127`, which the envelope
        // raises by up to four and a half octaves and accent by one and a half
        // more.
        let octaves = f32::from(controls.cutoff) / 127.0 * 6.0
            + self.envelope * f32::from(controls.env_mod) / 127.0 * 4.5
            + accent * 1.5;
        let cutoff = (40.0 * 2f32.powf(octaves)).min(8_000.0);
        let g = 1.0 - (-TAU * cutoff * dt).exp();
        // Below `4`, the feedback at which the ladder would self-oscillate.
        let feedback = f32::from(controls.resonance) / 127.0 * 3.3;
        // At half level the wave leaves the input's saturation room for the
        // feedback.
        let mut x = (0.5 * wave - feedback * self.stages[3]).tanh();
        for stage in &mut self.stages {
            *stage += g * (x - stage.tanh());
            x = *stage;
        }
        // The feedback takes as much from the passband as it adds at the
        // cutoff, so it is given back to keep the note about as loud.
        Some(self.stages[3] * (1.0 + feedback) * self.gate * (1.0 + accent))
    }
}

/// The frequency of MIDI `note`, in Hz.
fn pitch(note: u8) -> f32 {
    440.0 * 2f32.powf((f32::from(note) - 69.0) / 12.0)
}

/// A bandlimited saw at `phase`, rising from `-1` to `1`.
fn saw(phase: f32, step: f32) -> f32 {
    2.0 * phase - 1.0 - blep(phase, step)
}

/// The polyBLEP correction for a saw's step at phase `0`, which `step`, the
/// phase advance per sample, smooths over one sample either side.
fn blep(phase: f32, step: f32) -> f32 {
    if phase < step {
        let t = phase / step;
        2.0 * t - t * t - 1.0
    } else if phase > 1.0 - step {
        let t = (phase - 1.0) / step;
        t * t + 2.0 * t + 1.0
    } else {
        0.0
    }
}
