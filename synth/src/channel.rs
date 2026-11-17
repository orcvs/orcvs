//! One channel's strip: its own effects, how each new sound on it starts,
//! and what it sends the shared effects.

use crate::controller::Effect;
use crate::sends::{Mix, Sends};
use crate::smoothing::Smoothing;
use crate::tilt::Tilt;
use crate::vibrato::Vibrato;

/// How far into its course `OF` starts each new sound at `7F`, in seconds.
const MAX_OFFSET: f32 = 0.1;

/// The share of its own length a sound keeps at `LN` `00`; `7F` keeps all of
/// it.
const SHORTEST: f32 = 0.02;

/// How much of the channel a send takes at a Control Change value, from `0`
/// to `1`.
fn gain(value: u8) -> f32 {
    f32::from(value) / 127.0
}

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
            vibrato: Vibrato::new(sample_rate, Effect::Vibrato.load()),
            tilt: Tilt::new(sample_rate, Effect::Tilt.load()),
            offset: Effect::Offset.load(),
            length: Effect::Length.load(),
            sends: Sends::from_fn(|send| {
                Smoothing::new(sample_rate, gain(Effect::Send(send).load()))
            }),
            idle: false,
        }
    }

    /// Sets `effect` from a Control Change value.
    pub(crate) fn set(&mut self, effect: Effect, value: u8) {
        match effect {
            Effect::Vibrato => self.vibrato.set(value),
            Effect::Tilt => self.tilt.set(value),
            Effect::Length => self.length = value,
            Effect::Offset => self.offset = value,
            Effect::Send(send) => self.sends.of(send).set(gain(value)),
        }
    }

    /// Returns each effect Reset All Controllers clears to its load value.
    pub(crate) fn reset(&mut self) {
        for effect in Effect::CLEARED_BY_RESET {
            self.set(effect, effect.load());
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

    /// Whether every effect is at its pass-through value and staying there,
    /// so that silence on the channel adds nothing to the mix. Each effect
    /// loads at its pass-through value, so a fresh channel rests. Its offset
    /// and length shape only sounds still to start.
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
    use crate::controller::SharedEffect;

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
    fn every_effects_load_value_passes_the_channel_unchanged() {
        // A channel rests only at each effect's pass-through value, so a
        // load value that changed the sound would keep a fresh channel from
        // going idle; one that rested anywhere else would skip what it plays.
        assert!(Channel::new(RATE).rests());
        for effect in [
            Effect::Vibrato,
            Effect::Tilt,
            Effect::Length,
            Effect::Offset,
            Effect::Send(SharedEffect::Small),
            Effect::Send(SharedEffect::Large),
            Effect::Send(SharedEffect::Chorus),
            Effect::Send(SharedEffect::Delay),
        ] {
            let mut channel = Channel::new(RATE);
            for away in [0x00, 0x7F] {
                channel.set(effect, away);
                for n in 0..RATE as usize / 10 {
                    channel.process(Some(signal(n)));
                }
                channel.set(effect, effect.load());
                for n in 0..RATE as usize / 10 {
                    channel.process(Some(signal(n)));
                }
                assert!(channel.rests(), "{effect:?} from {away:#04x}");
            }
        }
    }

    #[test]
    fn a_channel_with_an_effect_turned_is_not_skipped_with_no_voice() {
        for effect in [
            Effect::Vibrato,
            Effect::Send(SharedEffect::Small),
            Effect::Send(SharedEffect::Large),
            Effect::Send(SharedEffect::Chorus),
            Effect::Send(SharedEffect::Delay),
        ] {
            let mut channel = Channel::new(RATE);
            channel.set(effect, 0x40);
            assert!(channel.process(None).is_some(), "{effect:?}");
        }
        let mut channel = Channel::new(RATE);
        channel.set(Effect::Tilt, 0x20);
        assert!(channel.process(None).is_some());
    }

    #[test]
    fn silence_through_a_channel_at_rest_is_silence_whatever_it_played() {
        // Skipping the channel adds `Mix::default()`, so a resting channel
        // given silence must add exactly that for the skip to change nothing,
        // including once an effect has been turned and returned to rest.
        for turned in [
            None,
            Some((Effect::Vibrato, 0x7F, 0x00)),
            Some((Effect::Tilt, 0x00, 0x40)),
        ] {
            let mut channel = Channel::new(RATE);
            if let Some((effect, away, _)) = turned {
                channel.set(effect, away);
            }
            for n in 0..RATE as usize / 2 {
                channel.process(Some(signal(n)));
            }
            if let Some((effect, _, back)) = turned {
                channel.set(effect, back);
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
        for (effect, value) in [(Effect::Vibrato, 0x7F), (Effect::Tilt, 0x00)] {
            let mut channel = Channel::new(RATE);
            for n in 0..1_000 {
                channel.process(Some(signal(n)));
            }
            assert_eq!(channel.process(None), None, "{effect:?}");
            channel.set(effect, value);
            for _ in 0..RATE as usize / 10 {
                assert_eq!(channel.process(None), Some(Mix::default()), "{effect:?}");
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

        channel.set(Effect::Length, 0x00);
        channel.set(Effect::Offset, 0x7F);
        assert_eq!(
            channel.onset(),
            Onset {
                length: SHORTEST,
                offset: MAX_OFFSET
            }
        );

        // Reset All Controllers leaves both alone.
        channel.reset();
        assert_eq!(channel.onset().offset, MAX_OFFSET);
        assert_eq!(channel.onset().length, SHORTEST);
    }
}
