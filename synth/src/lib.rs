//! The Built-in Synth's sound (ADR 0087): MIDI messages in, mono samples out.
//!
//! The crate does no audio I/O. A caller schedules each message at the frame
//! it should sound on and asks for samples; the frame clock is the caller's.

use std::collections::VecDeque;

mod acid;
mod drum;
mod tone;
mod vibrato;
#[cfg(target_arch = "wasm32")]
mod web;

use acid::{Acid, Controls};
use drum::{Drum, Instrument};
use tone::Tone;
use vibrato::Vibrato;

/// How many voices sound at once.
pub const VOICES: usize = 16;

/// The scale a voice's velocity and its own envelope multiply into the mix.
/// Tones are quieter than drums, which spend most of their life decaying.
const TONE_LEVEL: f32 = 0.2;
const DRUM_LEVEL: f32 = 0.35;
const ACID_LEVEL: f32 = 0.3;

/// The synth: sixteen voices, a frame clock, and the messages still to come.
///
/// Channels `0`–`7` play tones, `9` the acid voice (ADR 0097), and `8` and
/// `10`–`15` the 808-style kit. Velocity is the only level. Control Change 1 on
/// any channel sets `MD`, vibrato on the whole mix. All Sound Off and All Notes
/// Off release a channel's tones and its acid voice. The acid channel's own
/// Control Changes and Reset All Controllers set its [`Controls`]; every other
/// Control Change, Pitch Bend and Program Change is ignored.
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
    acid: Controls,
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
            let mut mix = 0.0;
            for slot in &mut self.voices {
                let Some(voice) = slot else { continue };
                let (value, scale) = match &mut voice.sound {
                    Sound::Tone(tone) => (tone.sample(dt), TONE_LEVEL),
                    Sound::Drum(drum) => (drum.sample(dt), DRUM_LEVEL),
                    Sound::Acid(acid) => (acid.sample(dt, &self.acid), ACID_LEVEL),
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
            0xB0 if data1 == 1 => self.vibrato.set(data2),
            // All Sound Off and All Notes Off.
            0xB0 if data1 == 120 || data1 == 123 => self.release_channel(channel),
            0xB0 if channel == acid::CHANNEL => self.acid.set(data1, data2),
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
        if channel == acid::CHANNEL && self.play_acid(note, velocity) {
            return;
        }
        let sound = if channel < 8 {
            Sound::Tone(Tone::new(channel, note))
        } else if channel == acid::CHANNEL {
            Sound::Acid(Acid::new(note, velocity))
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

    /// Plays `note` on the acid voice if it is still sounding, and answers
    /// whether it was. A note struck afresh counts as the newest voice.
    fn play_acid(&mut self, note: u8, velocity: u8) -> bool {
        let Some(voice) = self
            .voices
            .iter_mut()
            .flatten()
            .find(|voice| matches!(voice.sound, Sound::Acid(_)))
        else {
            return false;
        };
        let Sound::Acid(acid) = &mut voice.sound else {
            unreachable!("the voice was found by its acid sound");
        };
        if !acid.is_held() {
            self.started += 1;
            voice.started = self.started;
            voice.level = f32::from(velocity) / 127.0;
        }
        acid.play(note, velocity);
        voice.note = note;
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
        synth.schedule(0, [0xB0, 1, 0x7F]);
        synth.schedule(seconds(0.25) as u64, [0xB0, 1, 0x00]);
        synth.schedule(seconds(0.35) as u64, [0xB0, 1, 0x7F]);

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
        // Control Change 2 to 8 set effects whose sound has not landed yet, and
        // the acid controls act on the acid channel alone.
        for controller in (2..=8).chain([70, 71, 74, 75, 79, 121]) {
            synth.schedule(0, [0xB0, controller, 0x7F]);
        }
        synth.schedule(0, [0xC0, 5, 0]);

        assert_eq!(render(&mut synth, 4_800), render(&mut plain, 4_800));
    }

    #[test]
    fn every_drum_sounds_ignores_note_off_and_falls_silent() {
        for note in 36..48 {
            let mut synth = Synth::new(RATE);
            synth.schedule(0, note_on(10, note, 127));
            synth.schedule(1, note_off(10, note));
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
        synth.schedule(0, note_on(10, 46, 100));
        render(&mut synth, seconds(0.1));

        assert_eq!(synth.sounding(), vec![(10, 46), (8, 42)]);
    }

    /// The acid channel, with its filter opened all the way and no resonance,
    /// so the voice's pitch is plain to count.
    fn open_acid(synth: &mut Synth) {
        synth.schedule(0, [0xB9, 74, 0x7F]);
        synth.schedule(0, [0xB9, 71, 0x00]);
    }

    #[test]
    fn channel_nine_holds_one_note_at_its_pitch() {
        let mut synth = Synth::new(RATE);
        open_acid(&mut synth);
        synth.schedule(0, note_on(9, 45, 100));

        let out = render(&mut synth, seconds(1.0));

        assert_eq!(synth.sounding(), vec![(9, 45)]);
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
            synth.schedule(0, note_on(9, note, 100));
        }
        render(&mut synth, 10);

        assert_eq!(synth.sounding(), vec![(9, 52)]);
    }

    #[test]
    fn a_note_struck_while_the_last_is_held_glides_to_its_pitch() {
        let mut synth = Synth::new(RATE);
        open_acid(&mut synth);
        synth.schedule(0, note_on(9, 45, 100));
        synth.schedule(seconds(0.5) as u64, note_on(9, 57, 100));

        let out = render(&mut synth, seconds(1.0));

        let pitch = |from: f32, to: f32| {
            rising_zero_crossings(&out[seconds(from)..seconds(to)]) as f32 / (to - from)
        };
        // Partway through the glide, between 110 Hz and 220 Hz.
        let gliding = pitch(0.5, 0.53);
        assert!(gliding > 120.0 && gliding < 210.0, "{gliding} Hz");
        let arrived = pitch(0.75, 1.0);
        assert!((216.0..=224.0).contains(&arrived), "{arrived} Hz");
    }

    #[test]
    fn a_late_note_off_for_the_note_slid_from_changes_nothing() {
        let slide = |late_note_off: bool| {
            let mut synth = Synth::new(RATE);
            synth.schedule(0, note_on(9, 45, 100));
            synth.schedule(seconds(0.2) as u64, note_on(9, 57, 100));
            if late_note_off {
                synth.schedule(seconds(0.25) as u64, note_off(9, 45));
            }
            render(&mut synth, seconds(0.5))
        };

        assert_eq!(slide(true), slide(false));
    }

    #[test]
    fn the_note_off_of_the_note_sounding_silences_the_acid_voice() {
        let mut synth = Synth::new(RATE);
        synth.schedule(0, note_on(9, 45, 100));
        synth.schedule(seconds(0.2) as u64, note_off(9, 45));

        let out = render(&mut synth, seconds(0.5));

        assert!(synth.sounding().is_empty());
        assert_eq!(rms(&out[seconds(0.3)..]), 0.0);
    }

    /// The acid voice on note `45` with its cutoff at `0`, below the note,
    /// after `controls`, so its loudness shows how far the envelope opens the
    /// filter.
    fn closed_acid(controls: &[(u8, u8)]) -> Synth {
        let mut synth = Synth::new(RATE);
        synth.schedule(0, [0xB9, 74, 0x00]);
        for &(controller, value) in controls {
            synth.schedule(0, [0xB9, controller, value]);
        }
        synth.schedule(0, note_on(9, 45, 100));
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
        synth.schedule(seconds(1.0) as u64, note_on(9, 45, 100));
        synth.schedule(seconds(1.5) as u64, note_off(9, 45));
        synth.schedule(seconds(1.6) as u64, note_on(9, 45, 100));

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
            synth.schedule(0, [0xB9, 74, 0x00]);
            synth.schedule(0, [0xB9, 79, 0x00]);
            synth.schedule(0, note_on(9, 45, velocity));
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
            synth.schedule(0, [0xB9, 70, waveform]);
            synth.schedule(0, note_on(9, 45, 100));
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
        synth.schedule(0, note_on(9, 45, 100));
        render(&mut synth, seconds(0.2))
    }

    #[test]
    fn reset_all_controllers_returns_the_acid_controls_and_leaves_the_effects() {
        let turned: Vec<[u8; 3]> = [70, 71, 74, 75, 79]
            .into_iter()
            .map(|controller| [0xB9, controller, 0x7F])
            .collect();
        let plain = acid_after(&[]);

        for message in &turned {
            assert_ne!(acid_after(&[*message]), plain, "{message:?}");
        }
        assert_eq!(
            acid_after(&[turned.as_slice(), &[[0xB9, 121, 0]]].concat()),
            plain
        );
        assert_eq!(
            acid_after(&[[0xB9, 1, 0x7F], [0xB9, 121, 0]]),
            acid_after(&[[0xB9, 1, 0x7F]])
        );
    }

    #[test]
    fn the_acid_controls_on_another_channel_change_nothing() {
        let plain = acid_after(&[]);

        for controller in [70, 71, 74, 75, 79, 121] {
            assert_eq!(acid_after(&[[0xB0, controller, 0x7F]]), plain);
            assert_eq!(acid_after(&[[0xBA, controller, 0x7F]]), plain);
        }
    }

    #[test]
    fn all_notes_off_silences_the_acid_voice() {
        for controller in [120, 123] {
            let mut synth = Synth::new(RATE);
            synth.schedule(0, note_on(9, 45, 100));
            synth.schedule(10, [0xB9, controller, 0]);
            render(&mut synth, seconds(0.2));

            assert!(synth.sounding().is_empty(), "controller {controller}");
        }
    }

    #[test]
    fn the_acid_voice_stays_finite_and_falls_silent_at_any_setting() {
        for value in [0x00, 0x7F] {
            let mut synth = Synth::new(RATE);
            for controller in [70, 71, 74, 75, 79] {
                synth.schedule(0, [0xB9, controller, value]);
            }
            for step in 0..16u64 {
                let note = 24 + 6 * step as u8;
                synth.schedule(step * 6_000, note_on(9, note, 0x7F));
                synth.schedule(step * 6_000 + 5_000, note_off(9, note));
            }
            let out = render(&mut synth, seconds(3.0));

            assert!(
                out.iter().all(|s| s.is_finite() && s.abs() <= 1.0),
                "{value}"
            );
            assert!(synth.sounding().is_empty(), "{value}");
            assert_eq!(rms(&out[seconds(2.5)..]), 0.0, "{value}");
        }
    }

    #[test]
    fn resonance_keeps_the_note_about_as_loud() {
        let loudness = |resonance| rms(&acid_after(&[[0xB9, 71, resonance]]));

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
        let loudness = |cutoff| rms(&acid_after(&[[0xB9, 79, 0x00], [0xB9, 74, cutoff]]));

        assert!(loudness(0x7F) > loudness(0x00) * 4.0);
    }

    #[test]
    fn raising_the_resonance_sharpens_the_peak_at_the_cutoff() {
        // Cutoff `40` is about 330 Hz, the sixth harmonic of A1's 55 Hz.
        let peak = |resonance| {
            let out = acid_after(&[[0xB9, 79, 0x00], [0xB9, 74, 0x40], [0xB9, 71, resonance]]);
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
            synth.schedule(0, note_off(9, 45));
            synth.schedule(1, note_on(9, 45, velocity));
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
        synth.schedule(0, [0xB9, 74, 0x7F]);
        synth.schedule(0, note_on(9, 96, 100));
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
        synth.schedule(100, note_on(9, 45, 100));
        render(&mut synth, 200);

        let sounding = synth.sounding();
        assert_eq!(sounding.len(), VOICES);
        assert!(!sounding.contains(&(1, 40)));
        assert_eq!(sounding.last(), Some(&(9, 45)));
    }
}
