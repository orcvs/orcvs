//! `R1` and `R2`, Control Change 7 and 8: two reverbs on the whole mix.

use crate::glide::Glide;

/// The comb filters' delays, in samples at 44.1 kHz, at size `1`. Their
/// lengths share no factor, so their echoes do not pile up on one beat.
const COMBS: [f32; 4] = [1116.0, 1188.0, 1277.0, 1356.0];
/// The allpass filters' delays, in samples at 44.1 kHz, at every size.
const ALLPASSES: [f32; 2] = [556.0, 441.0];
/// How much of the mix each comb takes in.
const INPUT: f32 = 0.25;
/// The lowpass inside each comb: the share of each echo's change it keeps,
/// so the room's tail darkens as it fades.
const DAMPING: f32 = 0.7;

/// One room: how long its combs are, against [`COMBS`], and how much of
/// each echo they return.
#[derive(Clone, Copy)]
pub(crate) struct Room {
    pub(crate) size: f32,
    pub(crate) feedback: f32,
}

/// A Schroeder reverb: four damped combs side by side, then two allpasses in
/// a row, added to the mix at its level. At level zero the mix passes
/// unchanged.
pub(crate) struct Reverb {
    combs: [Comb; 4],
    allpasses: [Allpass; 2],
    level: Glide,
}

impl Reverb {
    pub(crate) fn new(sample_rate: f32, room: Room) -> Self {
        let samples = |delay: f32| (delay * sample_rate / 44_100.0) as usize;
        Self {
            combs: COMBS.map(|delay| Comb::new(samples(delay * room.size), room.feedback)),
            allpasses: ALLPASSES.map(|delay| Allpass::new(samples(delay))),
            level: Glide::new(sample_rate, 0.0),
        }
    }

    /// Sets the reverb's level from a Control Change value, `00`–`7F`.
    pub(crate) fn set(&mut self, value: u8) {
        self.level.set(f32::from(value) / 127.0);
    }

    /// The mix's next sample with the room's ring.
    pub(crate) fn process(&mut self, input: f32) -> f32 {
        let mut ring: f32 = self
            .combs
            .iter_mut()
            .map(|comb| comb.process(INPUT * input))
            .sum();
        for allpass in &mut self.allpasses {
            ring = allpass.process(ring);
        }
        input + self.level.next() * ring
    }
}

/// A feedback comb with a lowpass in its loop.
struct Comb {
    line: Vec<f32>,
    position: usize,
    feedback: f32,
    damped: f32,
}

impl Comb {
    fn new(length: usize, feedback: f32) -> Self {
        Self {
            line: vec![0.0; length],
            position: 0,
            feedback,
            damped: 0.0,
        }
    }

    fn process(&mut self, input: f32) -> f32 {
        let out = self.line[self.position];
        self.damped += (out - self.damped) * DAMPING;
        self.line[self.position] = input + self.feedback * self.damped;
        self.position = (self.position + 1) % self.line.len();
        out
    }
}

/// An allpass that smears each echo without colouring it.
struct Allpass {
    line: Vec<f32>,
    position: usize,
}

impl Allpass {
    /// How much each pass returns.
    const GAIN: f32 = 0.5;

    fn new(length: usize) -> Self {
        Self {
            line: vec![0.0; length],
            position: 0,
        }
    }

    fn process(&mut self, input: f32) -> f32 {
        let delayed = self.line[self.position];
        let fed = input + Self::GAIN * delayed;
        self.line[self.position] = fed;
        self.position = (self.position + 1) % self.line.len();
        delayed - Self::GAIN * fed
    }
}
