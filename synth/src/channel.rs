//! One channel's strip: its own effects, how each new sound on it starts,
//! and what it sends the shared effects.

use crate::controller::{EQ, LN, MD, OF, RESET};
use crate::sends::{Mix, Sends};
use crate::smoothing::Smoothing;
use crate::tilt::Tilt;
use crate::vibrato::Vibrato;

/// How far into its course `OF` starts each new sound at `7F`, in seconds.
const MAX_OFFSET: f32 = 0.1;

/// The share of its own length a sound keeps at `LN` `00`; `7F` keeps all of
/// it.
const SHORTEST: f32 = 0.02;

/// How a new sound on a channel starts and decays, from its `OF` and `LN`.
#[derive(Clone, Copy)]
#[cfg_attr(test, derive(Debug, PartialEq))]
pub(crate) struct Onset {
    /// The share of its own length the sound keeps, from `SHORTEST` to `1`.
    pub(crate) length: f32,
    /// How far into its course the sound starts, in seconds.
    pub(crate) offset: f32,
}

/// One channel's strip: its vibrato and tilt on everything it plays, how
/// each new sound on it starts and decays, and how much of it reaches each
/// shared effect.
///
/// A channel that is silent with every effect at rest adds exactly nothing
/// to the mix, so it goes idle and is skipped. That holds because a still
/// vibrato and a flat tilt each pass their input through unchanged, and
/// silence through them is silence. As it goes idle it clears what its
/// effects held, as the silence they would have taken in clears it, so a
/// later turn of an effect does not replay a voice a steal cut off.
pub(crate) struct Channel {
    vibrato: Vibrato,
    tilt: Tilt,
    /// `OF`, `00`–`7F`.
    offset: u8,
    /// `LN`, `00`–`7F`.
    length: u8,
    /// `CH`, `DL`, `R1` and `R2`, each from `0` to `1`.
    sends: Sends<Smoothing>,
    /// Whether the channel is skipped, silent with its effects at rest, and
    /// has cleared what its effects held.
    idle: bool,
}

impl Channel {
    pub(crate) fn new(sample_rate: f32) -> Self {
        Self {
            vibrato: Vibrato::new(sample_rate),
            tilt: Tilt::new(sample_rate),
            offset: 0,
            length: 0x7F,
            sends: Sends::from_fn(|| Smoothing::new(sample_rate, 0.0)),
            idle: false,
        }
    }

    /// Sets the effect on `controller` from a Control Change value. Reset All
    /// Controllers clears Modulation, a performance controller, and leaves
    /// the rest, the Sound and Effect Controllers and `OF` beside them, as
    /// RP-015 has it. Any other controller changes nothing.
    pub(crate) fn set(&mut self, controller: u8, value: u8) {
        match controller {
            MD => self.vibrato.set(value),
            EQ => self.tilt.set(value),
            LN => self.length = value,
            OF => self.offset = value,
            RESET => self.vibrato.set(0),
            _ => {
                if let Some(send) = self.sends.on(controller) {
                    send.set(f32::from(value) / 127.0);
                }
            }
        }
    }

    /// How a sound struck now on the channel starts and decays.
    pub(crate) fn onset(&self) -> Onset {
        Onset {
            // `SHORTEST` at `00`, rising exponentially to exactly `1` at `7F`.
            length: SHORTEST.powf(1.0 - f32::from(self.length) / 127.0),
            offset: MAX_OFFSET * f32::from(self.offset) / 127.0,
        }
    }

    /// What the channel adds to the mix for its next sample. `input` is what
    /// its voices played, `None` when no voice sounded on it. The answer is
    /// `None` while the channel is idle, so the caller adds nothing; the
    /// sample it goes idle on clears what its effects held.
    pub(crate) fn process(&mut self, input: Option<f32>) -> Option<Mix> {
        if input.is_some() || !self.rests() {
            self.idle = false;
            return Some(self.mix(input.unwrap_or(0.0)));
        }
        if !self.idle {
            self.vibrato.clear();
            self.tilt.clear();
            self.idle = true;
        }
        None
    }

    /// Whether every effect is at its load value and staying there, so that
    /// silence on the channel adds nothing to the mix. Its offset and length
    /// shape only sounds still to start.
    fn rests(&self) -> bool {
        self.vibrato.rests() && self.tilt.rests() && self.sends.all(|send| send.rests_at(0.0))
    }

    /// `input` through the channel's own effects: what it adds to the mix.
    fn mix(&mut self, input: f32) -> Mix {
        let out = self.tilt.process(self.vibrato.process(input));
        let gains = self.sends.map_mut(Smoothing::next);
        Mix {
            // Up to half the channel goes to the chorus in place of itself.
            dry: out * (1.0 - 0.5 * gains.chorus),
            sends: gains.map(|gain| out * gain),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::controller::{CH, DL, R1, R2};

    const RATE: f32 = 48_000.0;

    /// A tone-like input, never silent for long.
    fn signal(n: usize) -> f32 {
        (n as f32 * 0.05).sin()
    }

    #[test]
    fn a_channel_with_no_voice_and_its_effects_at_rest_adds_nothing() {
        let mut channel = Channel::new(RATE);
        assert_eq!(channel.process(None), None);
        for n in 0..100 {
            assert!(channel.process(Some(signal(n))).is_some());
        }
        assert_eq!(channel.process(None), None);
    }

    #[test]
    fn a_channel_with_an_effect_turned_is_not_skipped_with_no_voice() {
        for controller in [MD, R1, R2, CH, DL] {
            let mut channel = Channel::new(RATE);
            channel.set(controller, 0x40);
            assert!(channel.process(None).is_some(), "controller {controller}");
        }
        let mut channel = Channel::new(RATE);
        channel.set(EQ, 0x20);
        assert!(channel.process(None).is_some());
    }

    #[test]
    fn silence_through_a_channel_at_rest_is_silence_whatever_it_played() {
        // Skipping the channel adds `Mix::default()`, so a resting channel
        // given silence must add exactly that for the skip to change nothing,
        // including once an effect has been turned and returned to rest.
        for turned in [None, Some((MD, 0x7F, 0x00)), Some((EQ, 0x00, 0x40))] {
            let mut channel = Channel::new(RATE);
            if let Some((controller, away, _)) = turned {
                channel.set(controller, away);
            }
            for n in 0..RATE as usize / 2 {
                channel.process(Some(signal(n)));
            }
            if let Some((controller, _, back)) = turned {
                channel.set(controller, back);
            }
            let mut n = 0;
            while !channel.rests() {
                channel.process(Some(signal(n)));
                n += 1;
            }
            for _ in 0..1_000 {
                assert_eq!(
                    channel.process(Some(0.0)),
                    Some(Mix::default()),
                    "{turned:?}"
                );
            }
        }
    }

    #[test]
    fn going_idle_forgets_what_the_channel_played() {
        // A vibrato turned up reads back over its line, and a tilt turned
        // reads its low band: neither may bring back the voice it held.
        for (controller, value) in [(MD, 0x7F), (EQ, 0x00)] {
            let mut channel = Channel::new(RATE);
            for n in 0..1_000 {
                channel.process(Some(signal(n)));
            }
            assert_eq!(channel.process(None), None, "controller {controller}");
            channel.set(controller, value);
            for _ in 0..RATE as usize / 10 {
                assert_eq!(
                    channel.process(None),
                    Some(Mix::default()),
                    "controller {controller}"
                );
            }
        }
    }

    #[test]
    fn a_channel_at_rest_with_a_voice_passes_it_dry() {
        let mut channel = Channel::new(RATE);
        for n in 0..1_000 {
            let mix = channel.process(Some(signal(n))).unwrap();
            assert_eq!(mix.dry, signal(n));
            assert_eq!(mix.sends, Sends::default());
        }
    }

    #[test]
    fn length_and_offset_shape_each_new_sound() {
        let mut channel = Channel::new(RATE);
        assert_eq!(
            channel.onset(),
            Onset {
                length: 1.0,
                offset: 0.0
            }
        );

        channel.set(LN, 0x00);
        channel.set(OF, 0x7F);
        assert_eq!(
            channel.onset(),
            Onset {
                length: SHORTEST,
                offset: MAX_OFFSET
            }
        );

        // Reset All Controllers leaves both alone.
        channel.set(RESET, 0);
        assert_eq!(channel.onset().offset, MAX_OFFSET);
        assert_eq!(channel.onset().length, SHORTEST);
    }
}
