//! PROTOTYPE, throwaway: does a mono, sliding, accented saw through a resonant
//! 4-pole lowpass with its own envelope sound like a 303 next to channel `02`?

use std::f32::consts::PI;

const SR: f32 = 48000.0;

struct Acid {
    phase: f32,
    freq: f32,
    target: f32,
    gate: bool,
    amp: f32,
    env: f32,
    accent: f32,
    stages: [f32; 4],
    // Control Changes, 0..=127.
    cutoff: f32,
    resonance: f32,
    env_mod: f32,
    decay: f32,
    square: bool,
}

fn note_hz(n: u8) -> f32 {
    440.0 * 2f32.powf((n as f32 - 69.0) / 12.0)
}

fn poly_blep(t: f32, dt: f32) -> f32 {
    if t < dt {
        let t = t / dt;
        t + t - t * t - 1.0
    } else if t > 1.0 - dt {
        let t = (t - 1.0) / dt;
        t * t + t + t + 1.0
    } else {
        0.0
    }
}

impl Acid {
    fn new() -> Self {
        Acid { phase: 0.0, freq: 110.0, target: 110.0, gate: false, amp: 0.0, env: 0.0, accent: 0.0,
            stages: [0.0; 4], cutoff: 40.0, resonance: 90.0, env_mod: 70.0, decay: 50.0, square: false }
    }
    /// A Note On: legato (previous still gated) slides without retriggering.
    fn note_on(&mut self, note: u8, velocity: u8) {
        self.target = note_hz(note);
        let accented = velocity >= 0x60;
        if !self.gate {
            self.freq = self.target;
            self.env = 1.0;
            self.accent = if accented { 1.0 } else { 0.0 };
        }
        self.gate = true;
    }
    fn note_off(&mut self) {
        self.gate = false;
    }
    fn sample(&mut self) -> f32 {
        let dt = 1.0 / SR;
        // Slide: about 60 ms toward the target pitch.
        self.freq += (self.target - self.freq) * (1.0 - (-dt / 0.02).exp());
        // VCA: fast gate.
        let goal = if self.gate { 1.0 } else { 0.0 };
        self.amp += (goal - self.amp) * (1.0 - (-dt / 0.003).exp());
        // Filter envelope: decay 200 ms .. 2 s; accent forces the shortest.
        let decay = if self.accent > 0.5 { 0.2 } else { 0.2 * 10f32.powf(self.decay / 127.0) };
        self.env *= (-dt / decay).exp();
        self.accent *= (-dt / 0.25).exp();

        let inc = self.freq * dt;
        self.phase = (self.phase + inc).fract();
        let mut osc = 2.0 * self.phase - 1.0 - poly_blep(self.phase, inc);
        if self.square {
            let p2 = (self.phase + 0.5).fract();
            osc = osc - (2.0 * p2 - 1.0 - poly_blep(p2, inc));
        }

        let base = 40.0 * 2f32.powf(self.cutoff / 127.0 * 6.0); // 40 Hz .. 2.6 kHz
        let octaves = self.env * (self.env_mod / 127.0 * 4.5 + self.accent * 1.5);
        let fc = (base * 2f32.powf(octaves)).min(8000.0);
        let g = 1.0 - (-2.0 * PI * fc / SR).exp();
        let k = self.resonance / 127.0 * 3.3;
        let mut x = (osc - k * self.stages[3]).tanh();
        for s in &mut self.stages {
            *s += g * (x - s.tanh());
            x = *s;
        }
        let level = 0.5 + 0.5 * self.accent;
        (self.stages[3] * self.amp * level * 1.6).tanh() * 0.6
    }
}

// A step: note (0 rests), accent, slide into the next.
type Step = (u8, bool, bool);

fn pattern() -> [Step; 16] {
    let (a1, a2, c2, c3, d2, e2, g2) = (33, 45, 36, 48, 38, 40, 43);
    [
        (a1, true, false), (a1, false, false), (a2, false, true), (a1, false, false),
        (c2, true, false), (0, false, false), (g2, false, true), (a1, false, false),
        (a1, false, false), (e2, true, false), (a1, false, true), (a2, false, false),
        (c3, true, true), (d2, false, false), (g2, false, false), (e2, true, false),
    ]
}

fn wav(path: &str, out: &[f32]) {
    let mut w = Vec::new();
    let n = out.len() as u32;
    w.extend(b"RIFF"); w.extend((36 + n * 2).to_le_bytes()); w.extend(b"WAVEfmt ");
    w.extend(16u32.to_le_bytes()); w.extend(1u16.to_le_bytes()); w.extend(1u16.to_le_bytes());
    w.extend(48000u32.to_le_bytes()); w.extend(96000u32.to_le_bytes()); w.extend(2u16.to_le_bytes()); w.extend(16u16.to_le_bytes());
    w.extend(b"data"); w.extend((n * 2).to_le_bytes());
    for x in out { w.extend(((x.clamp(-1.0, 1.0) * 32767.0) as i16).to_le_bytes()); }
    std::fs::write(path, w).unwrap();
}

fn main() {
    let dir = std::env::args().nth(1).unwrap();
    let bpm = 130.0;
    let step = (SR * 60.0 / bpm / 4.0) as usize;
    let bars = 8;
    let steps = 16 * bars;
    let pat = pattern();

    // The acid voice, with cutoff, resonance and env mod swept per step the way
    // a Source sweeps a Control Change every Tick.
    let mut acid = Acid::new();
    let mut out = Vec::with_capacity(steps * step);
    for i in 0..steps {
        let t = i as f32 / steps as f32;
        acid.cutoff = 20.0 + 70.0 * (t * PI).sin();
        acid.resonance = if i >= 64 { 115.0 } else { 80.0 };
        acid.env_mod = 50.0 + 60.0 * t;
        acid.decay = 40.0;
        acid.square = i >= 96;
        let (note, accent, slide) = pat[i % 16];
        if note != 0 {
            acid.note_on(note, if accent { 0x7F } else { 0x50 });
        }
        for s in 0..step {
            // Gate length: 60% of the step, or the whole step when sliding.
            if s == step * 6 / 10 && !(slide && note != 0) {
                acid.note_off();
            }
            out.push(acid.sample());
        }
        if note != 0 && slide {
            // Held into the next step: the next Note On is legato.
        } else {
            acid.note_off();
        }
    }
    wav(&format!("{dir}/acid-prototype.wav"), &out);

    // The same line on channel `02` today, for comparison.
    let mut synth = synth::Synth::new(SR);
    let mut held: Option<u8> = None;
    for i in 0..steps {
        let f = (i * step) as u64;
        if let Some(n) = held.take() { synth.schedule(f, [0x82, n, 0]); }
        let (note, accent, _) = pat[i % 16];
        if note != 0 {
            synth.schedule(f, [0x92, note, if accent { 0x7F } else { 0x50 }]);
            held = Some(note);
        }
    }
    let mut now = vec![0f32; steps * step];
    synth.render(&mut now);
    wav(&format!("{dir}/channel-02-today.wav"), &now);
}
