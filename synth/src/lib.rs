//! The Built-in Synth's sound (ADR 0087): MIDI messages in, mono samples out.
//!
//! The crate does no audio I/O. A caller schedules each message at the frame
//! it should sound on and asks for samples; the frame clock is the caller's.

use std::collections::VecDeque;

mod drum;
mod tone;
mod vibrato;
#[cfg(target_arch = "wasm32")]
mod web;

use drum::{Drum, Instrument};
use tone::Tone;
use vibrato::Vibrato;

/// How many voices sound at once.
pub const VOICES: usize = 16;

/// The scale a voice's velocity and its own envelope multiply into the mix.
/// Tones are quieter than drums, which spend most of their life decaying.
const TONE_LEVEL: f32 = 0.2;
const DRUM_LEVEL: f32 = 0.35;

/// The synth: sixteen voices, a frame clock, and the messages still to come.
///
/// Channels `0`–`7` play tones and `8`–`15` the 808-style kit. Velocity is the
/// only level. Control Change 1 on any channel sets `MD`, vibrato on the whole
/// mix. All Sound Off and All Notes Off release a channel's tones;
/// every other Control Change, Pitch Bend and Program Change is ignored.
pub struct Synth {
    sample_rate: f32,
    frame: u64,
    voices: [Option<Voice>; VOICES],
    /// Counts Note Ons, so the voice with the lowest `started` is the oldest.
    started: u64,
    /// Messages not yet applied, in frame order; equal frames keep the order
    /// they were scheduled in.
    queue: VecDeque<(u64, [u8; 3])>,
    vibrato: Vibrato,
}

struct Voice {
    channel: u8,
    note: u8,
    started: u64,
    level: f32,
    sound: Sound,
}

enum Sound {
    Tone(Tone),
    Drum(Drum),
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
            vibrato: Vibrato::new(sample_rate),
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
            let mut mix = 0.0;
            for slot in &mut self.voices {
                let Some(voice) = slot else { continue };
                let (value, scale) = match &mut voice.sound {
                    Sound::Tone(tone) => (tone.sample(dt), TONE_LEVEL),
                    Sound::Drum(drum) => (drum.sample(dt), DRUM_LEVEL),
                };
                match value {
                    Some(value) => mix += value * voice.level * scale,
                    None => *slot = None,
                }
            }
            *sample = self.vibrato.process(mix).tanh();
            self.frame += 1;
        }
    }

    fn apply(&mut self, [status, data1, data2]: [u8; 3]) {
        let channel = status & 0x0F;
        match status & 0xF0 {
            0x90 if data2 > 0 => self.note_on(channel, data1, data2),
            0x80 | 0x90 => self.release_note(channel, data1),
            // All Sound Off and All Notes Off.
            0xB0 if data1 == 1 => self.vibrato.set(data2),
            0xB0 if data1 == 120 || data1 == 123 => self.release_channel(channel),
            _ => {}
        }
    }

    /// Releases the tones on `channel`. A drum rings out its own decay.
    fn release_channel(&mut self, channel: u8) {
        for voice in self.voices.iter_mut().flatten() {
            if voice.channel == channel
                && let Sound::Tone(tone) = &mut voice.sound
            {
                tone.release();
            }
        }
    }

    /// Releases the oldest held tone of `note` on `channel`, so a note struck
    /// again before its Note Off arrives keeps sounding until its own.
    fn release_note(&mut self, channel: u8, note: u8) {
        let oldest = self
            .voices
            .iter_mut()
            .flatten()
            .filter(|voice| voice.channel == channel && voice.note == note)
            .filter_map(|voice| match &mut voice.sound {
                Sound::Tone(tone) if tone.is_held() => Some((voice.started, tone)),
                _ => None,
            })
            .min_by_key(|(started, _)| *started);
        if let Some((_, tone)) = oldest {
            tone.release();
        }
    }

    fn note_on(&mut self, channel: u8, note: u8, velocity: u8) {
        let sound = if channel < 8 {
            Sound::Tone(Tone::new(channel, note))
        } else {
            let instrument = Instrument::of(note);
            if instrument == Instrument::ClosedHat {
                self.choke_open_hats(channel);
            }
            Sound::Drum(Drum::new(instrument, self.sample_rate))
        };
        self.started += 1;
        // A free voice ranks first, then the oldest.
        let slot = (0..VOICES)
            .min_by_key(|&index| self.voices[index].as_ref().map_or(0, |voice| voice.started))
            .unwrap_or_default();
        self.voices[slot] = Some(Voice {
            channel,
            note,
            started: self.started,
            level: f32::from(velocity) / 127.0,
            sound,
        });
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

    fn note_on(channel: u8, note: u8, velocity: u8) -> [u8; 3] {
        [0x90 | channel, note, velocity]
    }

    fn note_off(channel: u8, note: u8) -> [u8; 3] {
        [0x80 | channel, note, 0]
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
            synth.schedule(0, note_on(8, 36, 100));
            synth.schedule(10, [0xB0, controller, 0]);
            synth.schedule(10, [0xB8, controller, 0]);
            render(&mut synth, seconds(0.2));

            assert_eq!(synth.sounding(), vec![(8, 36)], "controller {controller}");
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
    fn modulation_on_any_channel_makes_a_steady_tone_waver() {
        let tone = |modulation: Option<[u8; 3]>| {
            let mut synth = Synth::new(RATE);
            if let Some(message) = modulation {
                synth.schedule(0, message);
            }
            synth.schedule(0, note_on(0, 69, 100));
            render(&mut synth, seconds(1.0))
        };

        assert!(period_spread(&tone(None)) < 1.03);
        assert!(period_spread(&tone(Some([0xB5, 1, 0x7F]))) > 1.08);
        assert_eq!(tone(Some([0xB0, 1, 0x00])), tone(None));
    }

    #[test]
    fn other_messages_change_nothing() {
        let mut synth = Synth::new(RATE);
        synth.schedule(0, note_on(0, 69, 100));
        let mut plain = Synth::new(RATE);
        plain.schedule(0, note_on(0, 69, 100));
        synth.schedule(0, [0xE0, 0x00, 0x7F]);
        // Control Change 2 to 8 set effects whose sound has not landed yet.
        for controller in 2..=8 {
            synth.schedule(0, [0xB0, controller, 0x7F]);
        }
        synth.schedule(0, [0xC0, 5, 0]);

        assert_eq!(render(&mut synth, 4_800), render(&mut plain, 4_800));
    }

    #[test]
    fn every_drum_sounds_ignores_note_off_and_falls_silent() {
        for note in 36..48 {
            let mut synth = Synth::new(RATE);
            synth.schedule(0, note_on(9, note, 127));
            synth.schedule(1, note_off(9, note));
            let out = render(&mut synth, seconds(8.0));

            assert!(out.iter().all(|s| s.is_finite()), "note {note}");
            assert!(rms(&out[..seconds(0.04)]) > 0.02, "note {note}");
            assert!(synth.sounding().is_empty(), "note {note}");
        }
    }

    #[test]
    fn a_drum_is_picked_by_pitch_class_whatever_the_octave() {
        let hit = |note| {
            let mut synth = Synth::new(RATE);
            synth.schedule(0, note_on(8, note, 100));
            render(&mut synth, seconds(0.5))
        };

        assert_eq!(hit(36), hit(60));
        assert_ne!(hit(36), hit(38));
    }

    #[test]
    fn the_kick_is_low_and_the_closed_hat_is_bright() {
        let hit = |note| {
            let mut synth = Synth::new(RATE);
            synth.schedule(0, note_on(8, note, 100));
            render(&mut synth, seconds(0.04))
        };

        let kick = rising_zero_crossings(&hit(36)) as f32 / 0.04;
        let hat = rising_zero_crossings(&hit(42)) as f32 / 0.04;
        assert!(kick < 200.0, "{kick} Hz");
        assert!(hat > 3_000.0, "{hat} Hz");
    }

    #[test]
    fn a_closed_hat_chokes_an_open_hat_on_its_channel() {
        let mut synth = Synth::new(RATE);
        synth.schedule(0, note_on(8, 46, 100));
        synth.schedule(seconds(0.05) as u64, note_on(8, 42, 100));
        synth.schedule(0, note_on(9, 46, 100));
        render(&mut synth, seconds(0.1));

        assert_eq!(synth.sounding(), vec![(9, 46), (8, 42)]);
    }
}
