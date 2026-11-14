//! The Built-in Synth's sound (ADR 0087): MIDI messages in, mono samples out.
//!
//! The crate does no audio I/O. A caller schedules each message at the frame
//! it should sound on and asks for samples; the frame clock is the caller's.

use std::collections::VecDeque;

mod acid;
mod chorus;
mod controller;
mod delay;
mod drum;
mod reverb;
mod smoothing;
mod tilt;
mod tone;
mod vibrato;
#[cfg(target_arch = "wasm32")]
mod web;

use acid::{Acid, Controls};
use chorus::Chorus;
use controller::{CH, DL, EQ, LN, MD, OF, R1, R2, RESET};
use delay::Delay;
use drum::{Drum, Instrument};
use reverb::{Reverb, Room};
use smoothing::Smoothing;
use tilt::Tilt;
use tone::Tone;
use vibrato::Vibrato;

/// How many voices sound at once.
pub const VOICES: usize = 16;

/// How many MIDI channels the synth answers.
const CHANNELS: usize = 16;

/// The scale a voice's velocity and its own envelope multiply into the mix.
/// Tones are quieter than drums, which spend most of their life decaying.
const TONE_LEVEL: f32 = 0.2;
const DRUM_LEVEL: f32 = 0.35;
const ACID_LEVEL: f32 = 0.3;

/// How far into its course `OF` starts each new sound at `7F`, in seconds.
const MAX_OFFSET: f32 = 0.1;
/// `R1`'s room: about two thirds of a second to fall 60 dB.
const SMALL: Room = Room {
    size: 0.6,
    feedback: 0.85,
};
/// `R2`'s room: about two seconds to fall 60 dB.
const LARGE: Room = Room {
    size: 1.0,
    feedback: 0.92,
};

/// The share of its own length a sound keeps at `LN` `00`; `7F` keeps all of
/// it.
const SHORTEST: f32 = 0.02;
/// How long an offset sound fades in over, in seconds, so a sound joined
/// at full loudness does not click.
const FADE: f32 = 0.005;

/// The synth: sixteen voices, a frame clock, and the messages still to come.
///
/// Channels `0`–`7` play tones, `8` the acid voice (ADR 0099), and `9`–`15`
/// the 808-style kit, which answers General MIDI drum keys, so MIDI channel
/// 10 plays drums as on a General MIDI module.
/// Velocity is the only level. Each channel's effects are its own, set by
/// Control Changes on that channel (ADR 0088): `MD` on CC 1 (Modulation),
/// vibrato; `EQ` on CC 74 (Brightness), one tilt; `LN` on CC 75 (Decay
/// Time), how long each new sound decays; `OF` on CC 85, an undefined
/// controller, how far into its course each new sound starts; and its sends
/// to the shared effects, `R1` on CC 91 and `R2` on CC 92, a small and a
/// large reverb, `CH` on CC 93, a chorus, and `DL` on CC 94, an echo. On the
/// acid channel CC 74 and 75 are the acid voice's cutoff and decay instead,
/// so neither the tilt nor the length applies there. All Sound Off and All
/// Notes Off release a channel's tones and its acid voice. Reset All
/// Controllers clears the channel's Modulation and leaves its sound and every
/// other effect alone (ADR 0099). The acid channel's own Control Changes set
/// its controls; every other Control Change, Pitch Bend and Program Change is
/// ignored.
pub struct Synth {
    sample_rate: f32,
    frame: u64,
    voices: [Option<Voice>; VOICES],
    /// Counts Note Ons, so the voice with the lowest `started` is the oldest.
    started: u64,
    /// Messages not yet applied, in frame order; equal frames keep the order
    /// they were scheduled in.
    queue: VecDeque<(u64, [u8; 3])>,
    channels: [Channel; CHANNELS],
    chorus: Chorus,
    delay: Delay,
    small: Reverb,
    large: Reverb,
    acid: Controls,
}

/// One channel's effects: its vibrato and tilt on everything it plays, how
/// each new sound on it starts and decays, and how much of it reaches each
/// shared effect.
struct Channel {
    vibrato: Vibrato,
    tilt: Tilt,
    /// `OF`, `00`–`7F`.
    offset: u8,
    /// `LN`, `00`–`7F`.
    length: u8,
    /// `CH`, `DL`, `R1` and `R2`, each from `0` to `1`.
    chorus: Smoothing,
    delay: Smoothing,
    small: Smoothing,
    large: Smoothing,
    /// Whether the render loop skips the channel, silent with its effects
    /// at rest, and has cleared what its effects held.
    idle: bool,
}

impl Channel {
    fn new(sample_rate: f32) -> Self {
        Self {
            vibrato: Vibrato::new(sample_rate),
            tilt: Tilt::new(sample_rate),
            offset: 0,
            length: 0x7F,
            chorus: Smoothing::new(sample_rate, 0.0),
            delay: Smoothing::new(sample_rate, 0.0),
            small: Smoothing::new(sample_rate, 0.0),
            large: Smoothing::new(sample_rate, 0.0),
            idle: false,
        }
    }

    /// Sets the effect on `controller` from a Control Change value. Reset All
    /// Controllers clears Modulation, a performance controller, and leaves
    /// the rest, the Sound and Effect Controllers and `OF` beside them, as
    /// RP-015 has it. Any other controller changes nothing.
    fn set(&mut self, controller: u8, value: u8) {
        let level = f32::from(value) / 127.0;
        match controller {
            MD => self.vibrato.set(value),
            EQ => self.tilt.set(value),
            LN => self.length = value,
            OF => self.offset = value,
            R1 => self.small.set(level),
            R2 => self.large.set(level),
            CH => self.chorus.set(level),
            DL => self.delay.set(level),
            RESET => self.vibrato.set(0),
            _ => {}
        }
    }

    /// Whether every effect is at its load value and staying there, so that
    /// silence on the channel adds nothing to the mix. Its offset and length
    /// shape only sounds still to start.
    fn rests(&self) -> bool {
        self.vibrato.is_still()
            && self.tilt.is_flat()
            && self.chorus.rests_at(0.0)
            && self.delay.rests_at(0.0)
            && self.small.rests_at(0.0)
            && self.large.rests_at(0.0)
    }
}

/// One sample of the mix ahead of the shared effects: every channel's dry
/// signal, and what the channels send each shared effect.
#[derive(Default)]
struct Mix {
    dry: f32,
    chorus: f32,
    delay: f32,
    small: f32,
    large: f32,
}

impl Mix {
    /// Adds `channel`'s next sample, `input`, through its own effects.
    fn add(&mut self, channel: &mut Channel, input: f32) {
        let out = channel.tilt.process(channel.vibrato.process(input));
        let chorus = channel.chorus.next();
        // Up to half the channel goes to the chorus in place of itself.
        self.dry += out * (1.0 - 0.5 * chorus);
        self.chorus += out * chorus;
        self.delay += out * channel.delay.next();
        self.small += out * channel.small.next();
        self.large += out * channel.large.next();
    }
}

/// What a channel plays. The channel layout (ADR 0099) lives here alone, so
/// every route from a channel to a sound asks [`Part::of`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Part {
    /// Channels `0`–`7`: a tone, its waveform picked by its channel.
    Tone,
    /// Channel `8`: the one 303-style acid voice.
    Acid,
    /// Channels `9`–`15`: the 808-style kit, so MIDI channel 10 plays drums as
    /// in General MIDI.
    Kit,
}

impl Part {
    fn of(channel: u8) -> Self {
        match channel {
            0..=7 => Self::Tone,
            8 => Self::Acid,
            _ => Self::Kit,
        }
    }
}

struct Voice {
    channel: u8,
    note: u8,
    started: u64,
    level: f32,
    /// Rises from `0` to `1` over `FADE` once an offset sound starts, and is
    /// `1` from the start otherwise.
    fade: f32,
    sound: Sound,
}

enum Sound {
    Tone(Tone),
    Drum(Drum),
    Acid(Acid),
}

impl Sound {
    /// Whether a Note Off would release it. A drum ignores Note Off.
    fn is_held(&self) -> bool {
        match self {
            Self::Tone(tone) => tone.is_held(),
            Self::Drum(_) => false,
            Self::Acid(acid) => acid.is_held(),
        }
    }

    fn release(&mut self) {
        match self {
            Self::Tone(tone) => tone.release(),
            Self::Drum(_) => {}
            Self::Acid(acid) => acid.release(),
        }
    }

    /// Jumps `seconds` into its course under `acid`, at a cost that does not
    /// grow with `seconds`, and answers whether it still sounds there.
    fn jump(&mut self, seconds: f32, acid: &Controls) -> bool {
        match self {
            Self::Tone(tone) => {
                tone.jump(seconds);
                true
            }
            Self::Drum(drum) => drum.jump(seconds),
            Self::Acid(sound) => {
                sound.jump(seconds, acid);
                true
            }
        }
    }

    /// The next sample at its voice's level, or `None` once it has finished.
    fn sample(&mut self, dt: f32, acid: &Controls) -> Option<f32> {
        match self {
            Self::Tone(tone) => tone.sample(dt).map(|value| value * TONE_LEVEL),
            Self::Drum(drum) => drum.sample(dt).map(|value| value * DRUM_LEVEL),
            Self::Acid(sound) => sound.sample(dt, acid).map(|value| value * ACID_LEVEL),
        }
    }
}

impl Synth {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            sample_rate,
            frame: 0,
            voices: Default::default(),
            started: 0,
            // Room for several Ticks of messages, so scheduling on the
            // audio thread rarely allocates.
            queue: VecDeque::with_capacity(64),
            channels: std::array::from_fn(|_| Channel::new(sample_rate)),
            chorus: Chorus::new(sample_rate),
            delay: Delay::new(sample_rate),
            small: Reverb::new(sample_rate, SMALL),
            large: Reverb::new(sample_rate, LARGE),
            acid: Controls::default(),
        }
    }

    /// The frame the next rendered sample is.
    #[cfg(test)]
    fn frame(&self) -> u64 {
        self.frame
    }

    /// Queues `message` to take effect at `frame`. A frame already rendered
    /// takes effect at the next one.
    pub fn schedule(&mut self, frame: u64, message: [u8; 3]) {
        let at = self.queue.partition_point(|(queued, _)| *queued <= frame);
        self.queue.insert(at, (frame, message));
    }

    /// Fills `out` with the next samples and advances the frame clock by its
    /// length.
    pub fn render(&mut self, out: &mut [f32]) {
        let dt = 1.0 / self.sample_rate;
        for sample in out {
            while let Some(&(at, message)) = self.queue.front()
                && at <= self.frame
            {
                self.queue.pop_front();
                self.apply(message);
            }
            let mut parts = [0.0; CHANNELS];
            let mut sounding = [false; CHANNELS];
            for slot in &mut self.voices {
                let Some(voice) = slot else { continue };
                match voice.sound.sample(dt, &self.acid) {
                    Some(value) => {
                        voice.fade = (voice.fade + dt / FADE).min(1.0);
                        let channel = usize::from(voice.channel);
                        parts[channel] += value * voice.level * voice.fade;
                        sounding[channel] = true;
                    }
                    None => *slot = None,
                }
            }
            let mut mix = Mix::default();
            for ((channel, part), sounding) in self.channels.iter_mut().zip(parts).zip(sounding) {
                // A silent channel at rest would add exactly nothing, so it is
                // skipped. Its effects' history is cleared once as it goes
                // idle, as the silence they would have taken in clears it, so
                // a later turn of an effect does not replay the voice a steal
                // cut off.
                if sounding || !channel.rests() {
                    channel.idle = false;
                    mix.add(channel, part);
                } else if !channel.idle {
                    channel.vibrato.clear();
                    channel.tilt.clear();
                    channel.idle = true;
                }
            }
            // The shared effects side by side, each returning only what the
            // channels sent it.
            let out = mix.dry
                + 0.5 * self.chorus.process(mix.chorus)
                + self.delay.process(mix.delay)
                + self.small.process(mix.small)
                + self.large.process(mix.large);
            *sample = out.tanh();
            self.frame += 1;
        }
    }

    fn apply(&mut self, [status, data1, data2]: [u8; 3]) {
        let channel = status & 0x0F;
        match status & 0xF0 {
            0x90 if data2 > 0 => self.note_on(channel, data1, data2),
            0x80 | 0x90 => self.release_note(channel, data1),
            // All Sound Off and All Notes Off.
            0xB0 if data1 == 120 || data1 == 123 => self.release_channel(channel),
            // The acid voice's own controllers come first, so on its channel
            // CC 74 and 75 are its cutoff and decay.
            0xB0 => {
                let taken = if Part::of(channel) == Part::Acid {
                    self.acid.set(data1, data2)
                } else {
                    false
                };
                if !taken {
                    self.channels[usize::from(channel)].set(data1, data2);
                }
            }
            _ => {}
        }
    }

    /// Releases the tones and the acid voice on `channel`. A drum rings out
    /// its own decay.
    fn release_channel(&mut self, channel: u8) {
        for voice in self.voices.iter_mut().flatten() {
            if voice.channel == channel {
                voice.sound.release();
            }
        }
    }

    /// Releases the oldest held tone of `note` on `channel`, so a note struck
    /// again before its Note Off arrives keeps sounding until its own. The acid
    /// voice is released only by the Note Off of the note it is playing.
    fn release_note(&mut self, channel: u8, note: u8) {
        let oldest = self
            .voices
            .iter_mut()
            .flatten()
            .filter(|voice| voice.channel == channel && voice.note == note && voice.sound.is_held())
            .min_by_key(|voice| voice.started);
        if let Some(voice) = oldest {
            voice.sound.release();
        }
    }

    fn note_on(&mut self, channel: u8, note: u8, velocity: u8) {
        self.started += 1;
        let effects = &self.channels[usize::from(channel)];
        // `SHORTEST` at `00`, rising exponentially to exactly `1` at `7F`.
        let length = SHORTEST.powf(1.0 - f32::from(effects.length) / 127.0);
        let offset = MAX_OFFSET * f32::from(effects.offset) / 127.0;
        let mut sound = match Part::of(channel) {
            Part::Tone => Sound::Tone(Tone::new(channel, note, length)),
            Part::Acid => {
                if self.play_acid(note, velocity) {
                    return;
                }
                Sound::Acid(Acid::new(note, velocity))
            }
            Part::Kit => {
                let Some(instrument) = Instrument::of(note) else {
                    return;
                };
                // The closed and pedal hats choke the open hat, as General
                // MIDI's exclusive class does.
                if instrument == Instrument::ClosedHat {
                    self.choke_open_hats(channel);
                }
                Sound::Drum(Drum::new(instrument, self.sample_rate, length))
            }
        };
        // `OF` jumps the sound into its course. A sound that ends there never
        // sounds.
        if !sound.jump(offset, &self.acid) {
            return;
        }
        // A free voice ranks first, then the oldest.
        let slot = (0..VOICES)
            .min_by_key(|&index| self.voices[index].as_ref().map_or(0, |voice| voice.started))
            .unwrap_or_default();
        self.voices[slot] = Some(Voice {
            channel,
            note,
            started: self.started,
            level: f32::from(velocity) / 127.0,
            fade: if offset > 0.0 { 0.0 } else { 1.0 },
            sound,
        });
    }

    /// Plays `note` on the acid voice if it is still sounding, and answers
    /// whether it was. The note counts as the newest voice and takes its own
    /// velocity, slid into or struck afresh.
    fn play_acid(&mut self, note: u8, velocity: u8) -> bool {
        let Some(Voice {
            note: playing,
            started,
            level,
            sound: Sound::Acid(acid),
            ..
        }) = self
            .voices
            .iter_mut()
            .flatten()
            .find(|voice| matches!(voice.sound, Sound::Acid(_)))
        else {
            return false;
        };
        acid.play(note, velocity);
        *playing = note;
        *started = self.started;
        *level = f32::from(velocity) / 127.0;
        true
    }

    fn choke_open_hats(&mut self, channel: u8) {
        for slot in &mut self.voices {
            if slot.as_ref().is_some_and(|voice| {
                voice.channel == channel
                    && matches!(&voice.sound, Sound::Drum(drum) if drum.instrument() == Instrument::OpenHat)
            }) {
                *slot = None;
            }
        }
    }

    /// The (channel, note) of every voice still sounding, oldest first.
    #[cfg(test)]
    fn sounding(&self) -> Vec<(u8, u8)> {
        let mut voices: Vec<&Voice> = self.voices.iter().flatten().collect();
        voices.sort_by_key(|voice| voice.started);
        voices
            .iter()
            .map(|voice| (voice.channel, voice.note))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const RATE: f32 = 48_000.0;

    /// The acid voice's channel, `08`: MIDI channel 9.
    const ACID: u8 = 8;
    /// The kit's first channel, `09`: MIDI channel 10, General MIDI's drum
    /// channel.
    const KIT: u8 = 9;

    fn note_on(channel: u8, note: u8, velocity: u8) -> [u8; 3] {
        [0x90 | channel, note, velocity]
    }

    fn note_off(channel: u8, note: u8) -> [u8; 3] {
        [0x80 | channel, note, 0]
    }

    fn control_change(channel: u8, controller: u8, value: u8) -> [u8; 3] {
        [0xB0 | channel, controller, value]
    }

    fn seconds(seconds: f32) -> usize {
        (seconds * RATE) as usize
    }

    fn render(synth: &mut Synth, frames: usize) -> Vec<f32> {
        let mut out = vec![0.0; frames];
        synth.render(&mut out);
        out
    }

    fn rms(samples: &[f32]) -> f32 {
        (samples.iter().map(|s| s * s).sum::<f32>() / samples.len() as f32).sqrt()
    }

    fn rising_zero_crossings(samples: &[f32]) -> usize {
        samples
            .windows(2)
            .filter(|pair| pair[0] < 0.0 && pair[1] >= 0.0)
            .count()
    }

    #[test]
    fn channels_zero_to_seven_play_tones_eight_the_acid_voice_and_nine_to_fifteen_the_kit() {
        for channel in 0..ACID {
            assert_eq!(Part::of(channel), Part::Tone, "channel {channel}");
        }
        assert_eq!(Part::of(ACID), Part::Acid);
        for channel in KIT..16 {
            assert_eq!(Part::of(channel), Part::Kit, "channel {channel}");
        }
    }

    #[test]
    fn a_message_sounds_from_the_frame_it_is_scheduled_at() {
        let mut synth = Synth::new(RATE);
        synth.schedule(1_000, note_on(0, 69, 100));

        let out = render(&mut synth, 2_000);

        assert!(out[..1_000].iter().all(|&s| s == 0.0));
        assert!(out[1_000..1_200].iter().any(|&s| s != 0.0));
        assert_eq!(synth.frame(), 2_000);
    }

    #[test]
    fn a_message_for_a_frame_already_rendered_sounds_at_once() {
        let mut synth = Synth::new(RATE);
        render(&mut synth, 500);
        synth.schedule(100, note_on(0, 69, 100));

        let out = render(&mut synth, 200);

        assert!(out[..20].iter().any(|&s| s != 0.0));
    }

    #[test]
    fn messages_apply_in_frame_order_whatever_order_they_were_scheduled_in() {
        let mut synth = Synth::new(RATE);
        synth.schedule(seconds(0.1) as u64, note_off(0, 69));
        synth.schedule(0, note_on(0, 69, 100));

        let out = render(&mut synth, seconds(0.5));

        assert!(rms(&out[..seconds(0.1)]) > 0.01);
        assert_eq!(rms(&out[seconds(0.3)..]), 0.0);
    }

    #[test]
    fn channel_zero_plays_a_sine_at_the_notes_pitch() {
        let mut synth = Synth::new(RATE);
        synth.schedule(0, note_on(0, 69, 100));

        let out = render(&mut synth, seconds(1.0));

        let crossings = rising_zero_crossings(&out);
        assert!((438..=442).contains(&crossings), "{crossings} Hz");
    }

    #[test]
    fn each_tone_channel_sounds_and_each_differs_from_the_others() {
        let mut shapes = Vec::new();
        for channel in 0..8 {
            let mut synth = Synth::new(RATE);
            synth.schedule(0, note_on(channel, 57, 100));
            let out = render(&mut synth, seconds(0.2));
            assert!(rms(&out) > 0.01, "channel {channel}");
            shapes.push(out);
        }
        for (a, first) in shapes.iter().enumerate() {
            for (b, second) in shapes.iter().enumerate().skip(a + 1) {
                assert_ne!(first, second, "channels {a} and {b}");
            }
        }
    }

    #[test]
    fn velocity_sets_loudness() {
        let loudness = |velocity| {
            let mut synth = Synth::new(RATE);
            synth.schedule(0, note_on(0, 60, velocity));
            rms(&render(&mut synth, seconds(0.2)))
        };

        assert!(loudness(127) > loudness(40) * 2.0);
    }

    #[test]
    fn a_tone_releases_over_150_ms_after_its_note_off() {
        let mut synth = Synth::new(RATE);
        synth.schedule(0, note_on(0, 60, 100));
        synth.schedule(seconds(0.2) as u64, note_off(0, 60));

        let out = render(&mut synth, seconds(0.5));

        assert!(rms(&out[seconds(0.2)..seconds(0.25)]) > 0.005);
        assert_eq!(rms(&out[seconds(0.36)..]), 0.0);
    }

    #[test]
    fn a_note_off_releases_only_the_oldest_held_voice_of_its_note() {
        let mut synth = Synth::new(RATE);
        synth.schedule(0, note_on(0, 60, 100));
        synth.schedule(seconds(0.1) as u64, note_on(0, 60, 100));
        synth.schedule(seconds(0.1) as u64, note_off(0, 60));

        render(&mut synth, seconds(0.5));

        assert_eq!(synth.sounding(), vec![(0, 60)]);
    }

    #[test]
    fn a_channel_plays_a_chord() {
        let mut synth = Synth::new(RATE);
        for note in [60, 64, 67] {
            synth.schedule(0, note_on(0, note, 100));
        }
        render(&mut synth, 10);

        assert_eq!(synth.sounding(), vec![(0, 60), (0, 64), (0, 67)]);
    }

    #[test]
    fn a_seventeenth_note_takes_the_oldest_voice() {
        let mut synth = Synth::new(RATE);
        for note in 40..56 {
            synth.schedule(u64::from(note), note_on(1, note, 100));
        }
        synth.schedule(100, note_on(1, 80, 100));
        render(&mut synth, 200);

        let sounding = synth.sounding();
        assert_eq!(sounding.len(), VOICES);
        assert!(!sounding.contains(&(1, 40)));
        assert!(sounding.contains(&(1, 41)));
        assert!(sounding.contains(&(1, 80)));
    }

    #[test]
    fn all_notes_off_releases_tones_and_leaves_drums_ringing() {
        for controller in [120, 123] {
            let mut synth = Synth::new(RATE);
            synth.schedule(0, note_on(0, 60, 100));
            synth.schedule(0, note_on(KIT, 36, 100));
            synth.schedule(10, [0xB0, controller, 0]);
            synth.schedule(10, control_change(KIT, controller, 0));
            render(&mut synth, seconds(0.2));

            assert_eq!(synth.sounding(), vec![(KIT, 36)], "controller {controller}");
        }
    }

    /// The ratio of the longest to the shortest whole period of `samples`,
    /// between rising zero crossings.
    fn period_spread(samples: &[f32]) -> f32 {
        let crossings: Vec<usize> = samples
            .windows(2)
            .enumerate()
            .filter(|(_, pair)| pair[0] < 0.0 && pair[1] >= 0.0)
            .map(|(index, _)| index)
            .collect();
        let periods = crossings.windows(2).map(|pair| (pair[1] - pair[0]) as f32);
        let longest = periods.clone().fold(0.0, f32::max);
        let shortest = periods.fold(f32::INFINITY, f32::min);
        longest / shortest
    }

    #[test]
    fn modulation_makes_a_steady_tone_on_its_channel_waver() {
        let tone = |modulation: Option<[u8; 3]>| {
            let mut synth = Synth::new(RATE);
            if let Some(message) = modulation {
                synth.schedule(0, message);
            }
            synth.schedule(0, note_on(0, 69, 100));
            render(&mut synth, seconds(1.0))
        };

        assert!(period_spread(&tone(None)) < 1.03);
        assert!(period_spread(&tone(Some(control_change(0, MD, 0x7F)))) > 1.08);
        assert_eq!(tone(Some(control_change(0, MD, 0x00))), tone(None));
        assert_eq!(tone(Some(control_change(5, MD, 0x7F))), tone(None));
    }

    #[test]
    fn changing_modulation_mid_note_moves_the_tone_without_a_step() {
        let largest_step = |samples: &[f32]| {
            samples
                .windows(2)
                .map(|pair| (pair[1] - pair[0]).abs())
                .fold(0.0, f32::max)
        };
        let mut plain = Synth::new(RATE);
        plain.schedule(0, note_on(0, 69, 100));
        let steady = largest_step(&render(&mut plain, seconds(0.5)));
        let mut synth = Synth::new(RATE);
        synth.schedule(0, note_on(0, 69, 100));
        // A quarter of the way through the vibrato's cycle, where its delay is
        // at its longest.
        synth.schedule(0, control_change(0, MD, 0x7F));
        synth.schedule(seconds(0.25) as u64, control_change(0, MD, 0x00));
        synth.schedule(seconds(0.35) as u64, control_change(0, MD, 0x7F));

        let out = render(&mut synth, seconds(0.5));

        let step = largest_step(&out);
        assert!(step < steady * 1.3, "{step} against {steady}");
    }

    #[test]
    fn other_messages_change_nothing() {
        let mut synth = Synth::new(RATE);
        synth.schedule(0, note_on(0, 69, 100));
        let mut plain = Synth::new(RATE);
        plain.schedule(0, note_on(0, 69, 100));
        synth.schedule(0, [0xE0, 0x00, 0x7F]);
        // The acid controls act on the acid channel alone, and no effect
        // answers controllers 2 to 8.
        for controller in [2, 3, 4, 5, 6, 7, 8, 70, 71, 79, 121] {
            synth.schedule(0, [0xB0, controller, 0x7F]);
        }
        synth.schedule(0, [0xC0, 5, 0]);

        assert_eq!(render(&mut synth, 4_800), render(&mut plain, 4_800));
    }

    /// `note` on `channel` after `messages`, all at frame zero.
    fn after(messages: &[[u8; 3]], channel: u8, note: u8, frames: usize) -> Vec<f32> {
        let mut synth = Synth::new(RATE);
        for &message in messages {
            synth.schedule(0, message);
        }
        synth.schedule(0, note_on(channel, note, 100));
        render(&mut synth, frames)
    }

    #[test]
    fn offset_at_its_load_value_changes_nothing() {
        for (channel, note) in [(0, 69), (KIT, 36), (ACID, 45)] {
            assert_eq!(
                after(
                    &[control_change(channel, OF, 0x00)],
                    channel,
                    note,
                    seconds(0.3)
                ),
                after(&[], channel, note, seconds(0.3)),
                "channel {channel}"
            );
        }
    }

    #[test]
    fn offset_starts_a_kick_past_its_punch() {
        let plain = after(&[], KIT, 36, seconds(0.5));
        let offset = after(&[control_change(KIT, OF, 0x7F)], KIT, 36, seconds(0.5));

        assert!(offset.iter().all(|s| s.is_finite()));
        assert!(rms(&offset[..seconds(0.02)]) < 0.9 * rms(&plain[..seconds(0.02)]));
        assert!(rms(&offset[..seconds(0.02)]) > 0.01);
    }

    #[test]
    fn an_offset_tone_fades_in_rather_than_clicks() {
        let out = after(&[control_change(0, OF, 0x7F)], 0, 69, seconds(0.1));

        assert!(out[0].abs() < 0.01, "{}", out[0]);
        assert!(rms(&out[seconds(0.05)..]) > 0.05);
    }

    #[test]
    fn a_sound_that_ends_within_the_offset_takes_no_voice() {
        let mut synth = Synth::new(RATE);
        synth.schedule(0, control_change(KIT, OF, 0x7F));
        // The rim shot decays to silence well inside 100 ms.
        synth.schedule(0, note_on(KIT, 37, 100));
        render(&mut synth, 10);

        assert!(synth.sounding().is_empty());
    }

    #[test]
    fn length_at_its_load_value_changes_nothing() {
        for (channel, note) in [(0, 69), (KIT, 36)] {
            assert_eq!(
                after(
                    &[control_change(channel, LN, 0x7F)],
                    channel,
                    note,
                    seconds(0.3)
                ),
                after(&[], channel, note, seconds(0.3)),
                "channel {channel}"
            );
        }
    }

    #[test]
    fn a_short_length_cuts_a_kick_to_a_blip() {
        let mut synth = Synth::new(RATE);
        synth.schedule(0, control_change(KIT, LN, 0x00));
        synth.schedule(0, note_on(KIT, 36, 100));
        let out = render(&mut synth, seconds(0.1));

        assert!(rms(&out[..seconds(0.005)]) > 0.01);
        assert!(synth.sounding().is_empty());
        let full = after(&[], KIT, 36, seconds(0.1));
        assert!(rms(&full[seconds(0.05)..]) > 0.05);
    }

    #[test]
    fn a_short_length_shortens_a_tones_release() {
        let mut synth = Synth::new(RATE);
        synth.schedule(0, control_change(0, LN, 0x00));
        synth.schedule(0, note_on(0, 60, 100));
        synth.schedule(seconds(0.1) as u64, note_off(0, 60));
        let out = render(&mut synth, seconds(0.2));

        assert!(rms(&out[..seconds(0.1)]) > 0.05);
        assert_eq!(rms(&out[seconds(0.11)..]), 0.0);
    }

    /// A 50 ms tone on channel `00` after `messages`, then `frames` in all.
    fn blip(messages: &[[u8; 3]], frames: usize) -> Vec<f32> {
        let mut synth = Synth::new(RATE);
        for &message in messages {
            synth.schedule(0, message);
        }
        synth.schedule(0, note_on(0, 69, 100));
        synth.schedule(seconds(0.05) as u64, note_off(0, 69));
        render(&mut synth, frames)
    }

    #[test]
    fn delay_at_its_load_value_changes_nothing() {
        assert_eq!(
            blip(&[control_change(0, DL, 0x00)], seconds(1.0)),
            blip(&[], seconds(1.0))
        );
    }

    #[test]
    fn delay_echoes_the_mix_and_the_echoes_die_away() {
        let dry = blip(&[], seconds(1.0));
        let wet = blip(&[control_change(0, DL, 0x7F)], seconds(6.0));

        assert!(wet.iter().all(|s| s.is_finite()));
        let echo = seconds(0.375)..seconds(0.425);
        assert_eq!(rms(&dry[echo.clone()]), 0.0);
        assert!(rms(&wet[echo]) > 0.02);
        assert!(rms(&wet[seconds(5.5)..]) < 1e-4);
    }

    /// The ratio of the loudest to the quietest 20 ms of `samples`.
    fn loudness_spread(samples: &[f32]) -> f32 {
        let windows: Vec<f32> = samples.chunks(seconds(0.02)).map(rms).collect();
        let loudest = windows.iter().copied().fold(0.0, f32::max);
        let quietest = windows.iter().copied().fold(f32::INFINITY, f32::min);
        loudest / quietest
    }

    #[test]
    fn chorus_at_its_load_value_changes_nothing() {
        assert_eq!(
            after(&[control_change(0, CH, 0x00)], 0, 69, seconds(0.5)),
            after(&[], 0, 69, seconds(0.5))
        );
    }

    #[test]
    fn chorus_makes_a_steady_tone_shimmer() {
        let steady = 0.1..;
        let dry = after(&[], 0, 69, seconds(2.0));
        let wet = after(&[control_change(0, CH, 0x7F)], 0, 69, seconds(2.0));

        assert!(wet.iter().all(|s| s.is_finite()));
        assert!(loudness_spread(&dry[seconds(steady.start)..]) < 1.05);
        assert!(loudness_spread(&wet[seconds(steady.start)..]) > 1.3);
    }

    /// How bright `samples` are: the loudness of their change from sample to
    /// sample against their own loudness.
    fn brightness(samples: &[f32]) -> f32 {
        let changes: Vec<f32> = samples.windows(2).map(|pair| pair[1] - pair[0]).collect();
        rms(&changes) / rms(samples)
    }

    #[test]
    fn eq_at_its_load_value_changes_nothing() {
        assert_eq!(
            after(&[control_change(2, EQ, 0x40)], 2, 45, seconds(0.5)),
            after(&[], 2, 45, seconds(0.5))
        );
    }

    #[test]
    fn eq_tilts_the_mix_darker_below_40_and_brighter_above() {
        let tilted = |value| {
            let out = after(&[control_change(2, EQ, value)], 2, 45, seconds(0.5));
            assert!(out.iter().all(|s| s.is_finite()));
            brightness(&out[seconds(0.1)..])
        };
        let flat = tilted(0x40);

        assert!(tilted(0x00) < 0.7 * flat, "{} against {flat}", tilted(0x00));
        assert!(tilted(0x7F) > 1.3 * flat, "{} against {flat}", tilted(0x7F));
    }

    #[test]
    fn the_small_reverb_at_its_load_value_changes_nothing() {
        assert_eq!(
            blip(&[control_change(0, R1, 0x00)], seconds(1.0)),
            blip(&[], seconds(1.0))
        );
    }

    #[test]
    fn the_small_reverb_rings_after_the_mix_and_dies_away() {
        let dry = blip(&[], seconds(1.0));
        let wet = blip(&[control_change(0, R1, 0x7F)], seconds(4.0));

        assert!(wet.iter().all(|s| s.is_finite()));
        let tone = rms(&dry[..seconds(0.05)]);
        let tail = seconds(0.25)..seconds(0.35);
        assert_eq!(rms(&dry[tail.clone()]), 0.0);
        assert!(rms(&wet[tail]) > 0.01 * tone);
        assert!(rms(&wet[seconds(3.5)..]) < 1e-5 * tone);
    }

    #[test]
    fn the_large_reverb_at_its_load_value_changes_nothing() {
        assert_eq!(
            blip(&[control_change(0, R2, 0x00)], seconds(1.0)),
            blip(&[], seconds(1.0))
        );
    }

    #[test]
    fn the_large_reverb_rings_longer_than_the_small_and_dies_away() {
        let tone = rms(&blip(&[], seconds(0.05)));
        let small = blip(&[control_change(0, R1, 0x7F)], seconds(1.2));
        let large = blip(&[control_change(0, R2, 0x7F)], seconds(10.0));

        assert!(large.iter().all(|s| s.is_finite()));
        let late = seconds(1.0)..seconds(1.2);
        assert!(rms(&large[late.clone()]) > 0.01 * tone);
        assert!(rms(&large[late.clone()]) > 100.0 * rms(&small[late]));
        assert!(rms(&large[seconds(9.5)..]) < 1e-5 * tone);
    }

    /// Every effect's controller at a value away from its load value.
    const TURNED: [(u8, u8); 8] = [
        (MD, 0x7F),
        (EQ, 0x00),
        (LN, 0x00),
        (OF, 0x7F),
        (R1, 0x7F),
        (R2, 0x7F),
        (CH, 0x7F),
        (DL, 0x7F),
    ];

    #[test]
    fn each_effect_changes_the_channel_it_is_set_on() {
        let plain = blip(&[], seconds(1.0));

        for (controller, value) in TURNED {
            assert_ne!(
                blip(&[control_change(0, controller, value)], seconds(1.0)),
                plain,
                "controller {controller}"
            );
        }
    }

    #[test]
    fn an_effect_turned_on_a_channel_whose_voice_was_stolen_replays_nothing() {
        // Channel 0's A4 is stolen mid-note by sixteen notes on channel 1,
        // released a frame later. Its effects rest, so it is skipped from
        // then on, and turning one afterwards must not sound what the
        // effect held when the voice was cut.
        for (controller, value) in [(MD, 0x7F), (EQ, 0x00)] {
            let mut synth = Synth::new(RATE);
            synth.schedule(0, note_on(0, 69, 100));
            for note in 0..16 {
                synth.schedule(1000, note_on(1, 40 + note, 100));
                synth.schedule(1001, note_off(1, 40 + note));
            }
            render(&mut synth, seconds(1.0));
            let peak = |out: &[f32]| out.iter().fold(0.0f32, |peak, s| peak.max(s.abs()));
            assert_eq!(
                peak(&render(&mut synth, seconds(0.5))),
                0.0,
                "before {controller}"
            );
            synth.schedule(synth.frame, control_change(0, controller, value));
            assert_eq!(
                peak(&render(&mut synth, seconds(0.5))),
                0.0,
                "after {controller}"
            );
        }
    }

    #[test]
    fn an_effect_set_on_one_channel_leaves_another_alone() {
        let plain = blip(&[], seconds(1.0));

        for (controller, value) in TURNED {
            assert_eq!(
                blip(&[control_change(1, controller, value)], seconds(1.0)),
                plain,
                "controller {controller}"
            );
        }
    }

    #[test]
    fn a_send_returns_only_what_its_own_channel_plays() {
        // A 50 ms tone on channel `01` sent to `send`, alone or beside the
        // same tone on channel `00`, which sends nothing.
        let tail = |send: u8, beside: bool| {
            let mut synth = Synth::new(RATE);
            synth.schedule(0, control_change(1, send, 0x7F));
            for channel in [0, 1] {
                if channel == 1 || beside {
                    synth.schedule(0, note_on(channel, 69, 100));
                    synth.schedule(seconds(0.05) as u64, note_off(channel, 69));
                }
            }
            render(&mut synth, seconds(1.0))[seconds(0.3)..].to_vec()
        };

        for send in [R1, R2, DL] {
            assert!(rms(&tail(send, false)) > 0.0, "controller {send}");
            assert_eq!(tail(send, true), tail(send, false), "controller {send}");
        }
    }

    #[test]
    fn controllers_74_and_75_on_the_acid_channel_set_its_cutoff_and_decay() {
        let plain = acid_after(&[]);

        // `40` is the tilt's load value and `7F` the length's, so either
        // effect would leave the note as it is: only the voice hears them.
        assert_ne!(acid_after(&[control_change(ACID, EQ, 0x40)]), plain);
        assert_ne!(acid_after(&[control_change(ACID, LN, 0x7F)]), plain);
        // `30` is the voice's cutoff and decay at load, and would darken and
        // shorten the note as a tilt and a length: neither reaches them.
        assert_eq!(
            acid_after(&[
                control_change(ACID, EQ, 0x30),
                control_change(ACID, LN, 0x30)
            ]),
            plain
        );
    }

    #[test]
    fn every_drum_sounds_ignores_note_off_and_falls_silent() {
        for note in KIT_KEYS.iter().flat_map(|family| family.iter().copied()) {
            let mut synth = Synth::new(RATE);
            synth.schedule(0, note_on(KIT, note, 127));
            synth.schedule(1, note_off(KIT, note));
            let out = render(&mut synth, seconds(8.0));

            assert!(out.iter().all(|s| s.is_finite()), "note {note}");
            assert!(rms(&out[..seconds(0.04)]) > 0.02, "note {note}");
            assert!(synth.sounding().is_empty(), "note {note}");
        }
    }

    /// The kit's General MIDI keys, one family to a row: kick, side stick,
    /// snare, hand clap, closed and pedal hats, open hat, low, mid and high
    /// toms, crash and ride cymbals, cowbell, and congas.
    const KIT_KEYS: [&[u8]; 12] = [
        &[35, 36],
        &[37],
        &[38, 40],
        &[39],
        &[42, 44],
        &[46],
        &[41, 43],
        &[45, 47],
        &[48, 50],
        &[49, 51, 57],
        &[56],
        &[62, 63, 64],
    ];

    fn drum_hit(note: u8) -> Vec<f32> {
        let mut synth = Synth::new(RATE);
        synth.schedule(0, note_on(KIT, note, 100));
        render(&mut synth, seconds(0.5))
    }

    #[test]
    fn the_keys_of_a_family_sound_alike() {
        for family in KIT_KEYS {
            let first = drum_hit(family[0]);
            assert!(rms(&first) > 0.0, "key {}", family[0]);
            for &key in &family[1..] {
                assert_eq!(drum_hit(key), first, "key {key} against {}", family[0]);
            }
        }
    }

    #[test]
    fn each_family_sounds_different() {
        let hits: Vec<Vec<f32>> = KIT_KEYS.iter().map(|family| drum_hit(family[0])).collect();
        for (index, hit) in hits.iter().enumerate() {
            for (other, against) in hits.iter().enumerate().skip(index + 1) {
                assert_ne!(
                    hit, against,
                    "keys {} and {}",
                    KIT_KEYS[index][0], KIT_KEYS[other][0]
                );
            }
        }
    }

    #[test]
    fn a_key_outside_the_kit_is_silent() {
        for key in [0, 34, 52, 60, 65, 127] {
            let mut synth = Synth::new(RATE);
            synth.schedule(0, note_on(KIT, key, 100));
            let out = render(&mut synth, seconds(0.1));

            assert!(out.iter().all(|&s| s == 0.0), "key {key}");
            assert!(synth.sounding().is_empty(), "key {key}");
        }
    }

    #[test]
    fn the_kick_is_low_and_the_closed_hat_is_bright() {
        let hit = |note| {
            let mut synth = Synth::new(RATE);
            synth.schedule(0, note_on(KIT, note, 100));
            render(&mut synth, seconds(0.04))
        };

        let kick = hit(36);
        // Silence has no zero crossings, so the pitch check alone would pass
        // on a kick that never sounded.
        assert!(rms(&kick) > 0.02);
        let kick = rising_zero_crossings(&kick) as f32 / 0.04;
        let hat = rising_zero_crossings(&hit(42)) as f32 / 0.04;
        assert!(kick < 200.0, "{kick} Hz");
        assert!(hat > 3_000.0, "{hat} Hz");
    }

    #[test]
    fn a_closed_or_pedal_hat_chokes_an_open_hat_on_its_channel_only() {
        for hat in [42, 44] {
            let mut synth = Synth::new(RATE);
            synth.schedule(0, note_on(KIT, 46, 100));
            synth.schedule(seconds(0.05) as u64, note_on(KIT, hat, 100));
            synth.schedule(0, note_on(KIT + 1, 46, 100));
            render(&mut synth, seconds(0.1));

            assert_eq!(
                synth.sounding(),
                vec![(KIT + 1, 46), (KIT, hat)],
                "key {hat}"
            );
        }
    }

    #[test]
    fn a_drum_on_the_kit_channel_rings_out_past_its_note_off() {
        let mut synth = Synth::new(RATE);
        synth.schedule(0, note_on(KIT, 36, 100));
        synth.schedule(10, note_off(KIT, 36));
        render(&mut synth, seconds(0.05));

        assert_eq!(synth.sounding(), vec![(KIT, 36)]);
    }

    /// The acid channel, with its filter opened all the way and no resonance,
    /// so the voice's pitch is plain to count.
    fn open_acid(synth: &mut Synth) {
        synth.schedule(0, control_change(ACID, 74, 0x7F));
        synth.schedule(0, control_change(ACID, 71, 0x00));
    }

    #[test]
    fn the_acid_channel_holds_one_note_at_its_pitch() {
        let mut synth = Synth::new(RATE);
        open_acid(&mut synth);
        synth.schedule(0, note_on(ACID, 45, 100));

        let out = render(&mut synth, seconds(1.0));

        assert_eq!(synth.sounding(), vec![(ACID, 45)]);
        let crossings = rising_zero_crossings(&out[seconds(0.5)..]);
        assert!(
            (53..=57).contains(&crossings),
            "{crossings} in half a second"
        );
        assert!(rms(&out[seconds(0.9)..]) > 0.01);
    }

    #[test]
    fn a_chord_on_the_acid_channel_sounds_as_its_last_note_in_one_voice() {
        let mut synth = Synth::new(RATE);
        for note in [45, 48, 52] {
            synth.schedule(0, note_on(ACID, note, 100));
        }
        render(&mut synth, 10);

        assert_eq!(synth.sounding(), vec![(ACID, 52)]);
    }

    #[test]
    fn a_note_struck_while_the_last_is_held_slides_to_its_pitch() {
        let mut synth = Synth::new(RATE);
        open_acid(&mut synth);
        synth.schedule(0, note_on(ACID, 45, 100));
        synth.schedule(seconds(0.5) as u64, note_on(ACID, 57, 100));

        let out = render(&mut synth, seconds(1.0));

        let pitch = |from: f32, to: f32| {
            rising_zero_crossings(&out[seconds(from)..seconds(to)]) as f32 / (to - from)
        };
        // Partway through the slide, between 110 Hz and 220 Hz.
        let sliding = pitch(0.5, 0.53);
        assert!(sliding > 120.0 && sliding < 210.0, "{sliding} Hz");
        let arrived = pitch(0.75, 1.0);
        assert!((216.0..=224.0).contains(&arrived), "{arrived} Hz");
    }

    #[test]
    fn a_late_note_off_for_the_note_slid_from_changes_nothing() {
        let slide = |late_note_off: bool| {
            let mut synth = Synth::new(RATE);
            synth.schedule(0, note_on(ACID, 45, 100));
            synth.schedule(seconds(0.2) as u64, note_on(ACID, 57, 100));
            if late_note_off {
                synth.schedule(seconds(0.25) as u64, note_off(ACID, 45));
            }
            render(&mut synth, seconds(0.5))
        };

        assert_eq!(slide(true), slide(false));
    }

    #[test]
    fn the_note_off_of_the_note_sounding_silences_the_acid_voice() {
        let mut synth = Synth::new(RATE);
        synth.schedule(0, note_on(ACID, 45, 100));
        synth.schedule(seconds(0.2) as u64, note_off(ACID, 45));

        let out = render(&mut synth, seconds(0.5));

        assert!(synth.sounding().is_empty());
        assert_eq!(rms(&out[seconds(0.3)..]), 0.0);
    }

    /// The acid voice on note `45` with its cutoff at `0`, below the note,
    /// after `controls`, so its loudness shows how far the envelope opens the
    /// filter.
    fn closed_acid(controls: &[(u8, u8)]) -> Synth {
        let mut synth = Synth::new(RATE);
        synth.schedule(0, control_change(ACID, 74, 0x00));
        for &(controller, value) in controls {
            synth.schedule(0, control_change(ACID, controller, value));
        }
        synth.schedule(0, note_on(ACID, 45, 100));
        synth
    }

    #[test]
    fn each_struck_note_opens_the_filter_and_it_closes_as_the_envelope_decays() {
        let swept = render(&mut closed_acid(&[(79, 0x7F)]), seconds(1.0));
        let flat = render(&mut closed_acid(&[(79, 0x00)]), seconds(1.0));

        let opening = rms(&swept[seconds(0.05)..seconds(0.1)]);
        let closed = rms(&swept[seconds(0.9)..]);
        assert!(opening > closed * 4.0, "{opening} against {closed}");
        let start = rms(&flat[seconds(0.1)..seconds(0.2)]);
        let end = rms(&flat[seconds(0.9)..]);
        assert!(start < end * 1.1, "{start} against {end}");
    }

    #[test]
    fn a_longer_decay_keeps_the_filter_open_longer() {
        let at_half_a_second = |decay| {
            let out = render(&mut closed_acid(&[(75, decay)]), seconds(0.6));
            rms(&out[seconds(0.5)..])
        };

        assert!(at_half_a_second(0x7F) > at_half_a_second(0x00) * 4.0);
    }

    #[test]
    fn a_slide_leaves_the_envelope_decaying_and_a_fresh_note_restarts_it() {
        let mut synth = closed_acid(&[]);
        synth.schedule(seconds(1.0) as u64, note_on(ACID, 45, 100));
        synth.schedule(seconds(1.5) as u64, note_off(ACID, 45));
        synth.schedule(seconds(1.6) as u64, note_on(ACID, 45, 100));

        let out = render(&mut synth, seconds(2.0));

        let slid = rms(&out[seconds(1.05)..seconds(1.1)]);
        let struck = rms(&out[seconds(1.65)..seconds(1.7)]);
        assert!(struck > slid * 4.0, "{struck} against {slid}");
    }

    #[test]
    fn velocity_from_78_accents_the_note_louder_and_brighter() {
        // With no envelope depth, only accent opens the closed filter.
        let early = |velocity| {
            let mut synth = Synth::new(RATE);
            synth.schedule(0, control_change(ACID, 74, 0x00));
            synth.schedule(0, control_change(ACID, 79, 0x00));
            synth.schedule(0, note_on(ACID, 45, velocity));
            rms(&render(&mut synth, seconds(0.1))[seconds(0.05)..])
        };

        assert!(early(0x78) > early(0x77) * 3.0);
        assert!(early(0x77) > early(0x3C) * 1.8);
    }

    #[test]
    fn control_change_70_from_40_plays_a_square() {
        let note = |waveform| {
            let mut synth = Synth::new(RATE);
            open_acid(&mut synth);
            synth.schedule(0, control_change(ACID, 70, waveform));
            synth.schedule(0, note_on(ACID, 45, 100));
            render(&mut synth, seconds(0.2))
        };
        let saw = note(0x00);

        assert_eq!(note(0x3F), saw);
        assert_ne!(note(0x40), saw);
    }

    /// The acid voice's note `45` for a fifth of a second after `messages`.
    fn acid_after(messages: &[[u8; 3]]) -> Vec<f32> {
        let mut synth = Synth::new(RATE);
        for &message in messages {
            synth.schedule(0, message);
        }
        synth.schedule(0, note_on(ACID, 45, 100));
        render(&mut synth, seconds(0.2))
    }

    #[test]
    fn reset_all_controllers_leaves_the_acid_voices_sound_alone() {
        let turned: Vec<[u8; 3]> = [70, 71, 74, 75, 79]
            .into_iter()
            .map(|controller| control_change(ACID, controller, 0x7F))
            .collect();
        let plain = acid_after(&[]);

        for message in &turned {
            assert_ne!(acid_after(&[*message]), plain, "{message:?}");
        }
        assert_eq!(
            acid_after(&[turned.as_slice(), &[control_change(ACID, RESET, 0)]].concat()),
            acid_after(&turned)
        );
    }

    #[test]
    fn reset_all_controllers_leaves_every_effect_but_modulation_alone() {
        for (controller, value) in TURNED
            .into_iter()
            .filter(|&(controller, _)| controller != MD)
        {
            let turned = control_change(0, controller, value);
            assert_eq!(
                blip(&[turned, control_change(0, RESET, 0)], seconds(1.0)),
                blip(&[turned], seconds(1.0)),
                "controller {controller}"
            );
        }
    }

    #[test]
    fn reset_all_controllers_clears_modulation_on_its_channel_only() {
        let reset = |channel| {
            let mut synth = Synth::new(RATE);
            synth.schedule(0, control_change(0, MD, 0x7F));
            synth.schedule(0, control_change(channel, RESET, 0));
            synth.schedule(0, note_on(0, 69, 100));
            render(&mut synth, seconds(1.0))
        };

        assert!(period_spread(&reset(0)) < 1.03);
        assert!(period_spread(&reset(1)) > 1.08);
    }

    #[test]
    fn the_acid_controls_on_another_channel_change_nothing() {
        let plain = acid_after(&[]);

        for controller in [70, 71, 74, 75, 79, 121] {
            assert_eq!(acid_after(&[[0xB0, controller, 0x7F]]), plain);
            assert_eq!(acid_after(&[control_change(KIT, controller, 0x7F)]), plain);
        }
    }

    #[test]
    fn all_notes_off_silences_the_acid_voice() {
        for controller in [120, 123] {
            let mut synth = Synth::new(RATE);
            synth.schedule(0, note_on(ACID, 45, 100));
            synth.schedule(10, control_change(ACID, controller, 0));
            render(&mut synth, seconds(0.2));

            assert!(synth.sounding().is_empty(), "controller {controller}");
        }
    }

    #[test]
    fn the_acid_voice_stays_finite_and_falls_silent_at_any_setting() {
        for value in [0x00, 0x7F] {
            let mut synth = Synth::new(RATE);
            for controller in [70, 71, 74, 75, 79] {
                synth.schedule(0, control_change(ACID, controller, value));
            }
            for step in 0..16u64 {
                let note = 24 + 6 * step as u8;
                synth.schedule(step * 6_000, note_on(ACID, note, 0x7F));
                synth.schedule(step * 6_000 + 5_000, note_off(ACID, note));
            }
            let out = render(&mut synth, seconds(3.0));

            // The mix's own saturation holds every sample under 1; the voice
            // alone leaves it room for the others.
            assert!(out.iter().all(|s| s.is_finite()), "{value}");
            let peak = out.iter().fold(0.0f32, |peak, s| peak.max(s.abs()));
            assert!(peak < 0.5, "{value}: {peak}");
            assert!(synth.sounding().is_empty(), "{value}");
            assert_eq!(rms(&out[seconds(2.5)..]), 0.0, "{value}");
        }
    }

    #[test]
    fn resonance_keeps_the_note_about_as_loud() {
        let loudness = |resonance| rms(&acid_after(&[control_change(ACID, 71, resonance)]));

        let ratio = loudness(0x7F) / loudness(0x00);
        assert!((0.7..1.4).contains(&ratio), "{ratio}");
    }

    /// The amplitude of the `frequency` component of `samples`, by Goertzel.
    fn amplitude_at(samples: &[f32], frequency: f32) -> f32 {
        let coefficient = 2.0 * (std::f32::consts::TAU * frequency / RATE).cos();
        let (mut previous, mut before) = (0.0f32, 0.0f32);
        for &sample in samples {
            let next = sample + coefficient * previous - before;
            before = previous;
            previous = next;
        }
        let power = previous * previous + before * before - coefficient * previous * before;
        2.0 * power.max(0.0).sqrt() / samples.len() as f32
    }

    #[test]
    fn raising_the_cutoff_opens_the_filter() {
        let loudness = |cutoff| {
            rms(&acid_after(&[
                control_change(ACID, 79, 0x00),
                control_change(ACID, 74, cutoff),
            ]))
        };

        assert!(loudness(0x7F) > loudness(0x00) * 4.0);
    }

    #[test]
    fn raising_the_resonance_sharpens_the_peak_at_the_cutoff() {
        // Cutoff `40` is about 330 Hz, the sixth harmonic of A1's 55 Hz.
        let peak = |resonance| {
            let out = acid_after(&[
                control_change(ACID, 79, 0x00),
                control_change(ACID, 74, 0x40),
                control_change(ACID, 71, resonance),
            ]);
            let harmonic = |n: f32| amplitude_at(&out[seconds(0.05)..], 55.0 * n);
            harmonic(6.0) / harmonic(1.0)
        };

        assert!(
            peak(0x7F) > peak(0x00) * 3.0,
            "{} against {}",
            peak(0x7F),
            peak(0x00)
        );
    }

    #[test]
    fn an_accented_note_takes_the_shortest_decay() {
        let closing = |velocity| {
            let mut synth = closed_acid(&[(79, 0x7F), (75, 0x7F)]);
            synth.schedule(0, note_off(ACID, 45));
            synth.schedule(1, note_on(ACID, 45, velocity));
            let out = render(&mut synth, seconds(0.6));
            rms(&out[seconds(0.5)..]) / rms(&out[seconds(0.05)..seconds(0.1)])
        };

        assert!(closing(0x7F) < closing(0x64) / 4.0);
    }

    #[test]
    fn a_high_saw_folds_back_almost_nothing() {
        // C7's twenty-third harmonic, 48 139 Hz, would fold back to 139 Hz,
        // below the note and inside the filter's passband.
        let mut synth = Synth::new(RATE);
        synth.schedule(0, control_change(ACID, 74, 0x7F));
        synth.schedule(0, note_on(ACID, 96, 100));
        let out = render(&mut synth, seconds(1.0));

        let note = 440.0 * 2f32.powf(27.0 / 12.0);
        let folded = amplitude_at(&out, 23.0 * note - RATE);
        let fundamental = amplitude_at(&out, note);
        assert!(
            folded < fundamental * 1e-3,
            "{folded} against {fundamental}"
        );
    }

    #[test]
    fn the_acid_voice_takes_one_of_the_sixteen_voices() {
        let mut synth = Synth::new(RATE);
        for note in 40..56 {
            synth.schedule(u64::from(note), note_on(1, note, 100));
        }
        synth.schedule(100, note_on(ACID, 45, 100));
        render(&mut synth, 200);

        let sounding = synth.sounding();
        assert_eq!(sounding.len(), VOICES);
        assert!(!sounding.contains(&(1, 40)));
        assert_eq!(sounding.last(), Some(&(ACID, 45)));
    }

    #[test]
    fn a_slide_keeps_the_acid_voice_from_being_the_oldest() {
        let mut synth = Synth::new(RATE);
        synth.schedule(0, note_on(ACID, 45, 100));
        for note in 40..55 {
            synth.schedule(u64::from(note), note_on(1, note, 100));
        }
        synth.schedule(100, note_on(ACID, 48, 100));
        synth.schedule(110, note_on(1, 80, 100));
        render(&mut synth, 200);

        let sounding = synth.sounding();
        assert!(sounding.contains(&(ACID, 48)), "{sounding:?}");
        assert!(!sounding.contains(&(1, 40)), "{sounding:?}");
    }

    #[test]
    fn a_note_slid_into_takes_its_own_velocity() {
        let slid_into = |velocity| {
            let mut synth = Synth::new(RATE);
            synth.schedule(0, control_change(ACID, 79, 0x00));
            synth.schedule(0, note_on(ACID, 45, 0x40));
            synth.schedule(seconds(0.2) as u64, note_on(ACID, 47, velocity));
            rms(&render(&mut synth, seconds(0.4))[seconds(0.3)..])
        };

        assert!(slid_into(0x7F) > slid_into(0x40) * 1.5);
    }
}
