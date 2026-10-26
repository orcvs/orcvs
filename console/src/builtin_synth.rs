//! PROTOTYPE, not for merge: a built-in synth MIDI destination that measures
//! its own timing.
//!
//! Two destinations share one cpal output stream. "Built-in synth" starts each
//! message at the start of the next buffer it renders, so a note's start moves
//! with where the message fell between buffers. "Built-in synth (timestamped)"
//! starts each message a fixed delay after it was sent, at its frame within
//! the buffer, so the jitter becomes a constant latency.
//!
//! Every two seconds of audio it logs: buffers rendered, buffers that started
//! late (an audible gap), the largest gap between callbacks, the spread of
//! send-to-sound latency over the Note Ons it played, and the spread of the
//! intervals between Note Ons as Playback sent them.

use std::cell::RefCell;
use std::sync::{Arc, Mutex};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use orcvs::midi::{MidiConnection, MidiConnectionRequest, MidiDestination, MidiError};

pub const IMMEDIATE_ID: &str = "orcvs-built-in-synth";
pub const TIMESTAMPED_ID: &str = "orcvs-built-in-synth-timestamped";

pub fn destinations() -> [MidiDestination; 2] {
    [
        MidiDestination::new(IMMEDIATE_ID, "Built-in synth"),
        MidiDestination::new(TIMESTAMPED_ID, "Built-in synth (timestamped)"),
    ]
}

pub fn is_synth(id: &str) -> bool {
    id == IMMEDIATE_ID || id == TIMESTAMPED_ID
}

fn now_ms() -> f64 {
    #[cfg(target_arch = "wasm32")]
    {
        web_sys::window()
            .and_then(|window| window.performance())
            .map_or(0.0, |performance| performance.now())
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        use std::sync::OnceLock;
        use std::time::Instant;
        static START: OnceLock<Instant> = OnceLock::new();
        START.get_or_init(Instant::now).elapsed().as_secs_f64() * 1000.0
    }
}

fn report(line: &str) {
    #[cfg(target_arch = "wasm32")]
    log::info!("{line}");
    #[cfg(not(target_arch = "wasm32"))]
    eprintln!("{line}");
}

struct Message {
    sent_ms: f64,
    timestamped: bool,
    bytes: [u8; 3],
}

type Inbox = Arc<Mutex<Vec<Message>>>;

thread_local! {
    static STREAM: RefCell<Option<(cpal::Stream, Inbox)>> = const { RefCell::new(None) };
}

pub fn connect(id: &str) -> Result<MidiConnectionRequest, MidiError> {
    let inbox = STREAM.with(|slot| -> Result<Inbox, MidiError> {
        let mut slot = slot.borrow_mut();
        if let Some((stream, inbox)) = slot.as_ref() {
            stream
                .play()
                .map_err(|error| MidiError::new(error.to_string()))?;
            return Ok(inbox.clone());
        }
        let inbox: Inbox = Arc::default();
        let stream = open_stream(inbox.clone())?;
        *slot = Some((stream, inbox.clone()));
        Ok(inbox)
    })?;
    Ok(MidiConnectionRequest::Ready(Box::new(SynthConnection {
        inbox,
        timestamped: id == TIMESTAMPED_ID || timestamped_by_url(),
    })))
}

/// `?synth=timestamped` makes the first destination timestamp too, so a run
/// that selects it automatically can measure either mode.
fn timestamped_by_url() -> bool {
    #[cfg(target_arch = "wasm32")]
    {
        web_sys::window()
            .and_then(|window| window.location().search().ok())
            .is_some_and(|search| search.contains("synth=timestamped"))
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        std::env::var_os("ORCVS_SYNTH_TIMESTAMPED").is_some()
    }
}

/// `?frames=512` asks for a smaller audio buffer than the host's default.
fn buffer_frames_by_url() -> Option<u32> {
    #[cfg(target_arch = "wasm32")]
    {
        let search = web_sys::window()?.location().search().ok()?;
        let (_, rest) = search.split_once("frames=")?;
        rest.split('&').next()?.parse().ok()
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        std::env::var("ORCVS_SYNTH_FRAMES").ok()?.parse().ok()
    }
}

struct SynthConnection {
    inbox: Inbox,
    timestamped: bool,
}

impl MidiConnection for SynthConnection {
    fn send(&mut self, message: &[u8]) -> Result<(), MidiError> {
        let mut bytes = [0; 3];
        let len = message.len().min(3);
        bytes[..len].copy_from_slice(&message[..len]);
        self.inbox
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .push(Message {
                sent_ms: now_ms(),
                timestamped: self.timestamped,
                bytes,
            });
        Ok(())
    }
}

fn open_stream(inbox: Inbox) -> Result<cpal::Stream, MidiError> {
    let error = |error: &dyn std::fmt::Display| MidiError::new(error.to_string());
    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .ok_or_else(|| MidiError::new("no audio output device"))?;
    let mut config = device
        .default_output_config()
        .map_err(|e| error(&e))?
        .config();
    if let Some(frames) = buffer_frames_by_url() {
        config.buffer_size = cpal::BufferSize::Fixed(frames);
    }
    let channels = usize::from(config.channels);
    let sample_rate = config.sample_rate as f32;
    let mut synth = Synth::new(sample_rate);
    let mut stats = Stats::default();
    let mut waiting: Vec<Message> = Vec::new();
    let stream = device
        .build_output_stream::<f32, _, _>(
            config,
            move |data, info| {
                let callback_ms = now_ms();
                let timestamp = info.timestamp();
                let lead_ms = timestamp
                    .playback
                    .checked_duration_since(timestamp.callback)
                    .map_or(0.0, |lead| lead.as_secs_f64() * 1000.0);
                let frames = data.len() / channels;
                let buffer_ms = frames as f64 * 1000.0 / f64::from(sample_rate);
                let play_start_ms = callback_ms + lead_ms;
                // The fixed delay a timestamped message sounds after it was sent:
                // the lead the browser schedules ahead (about 70 ms) plus a
                // buffer and a margin for a late callback.
                let delay_ms = 3.0 * buffer_ms + 30.0;

                let mut messages = std::mem::take(&mut waiting);
                messages.append(
                    &mut inbox
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner),
                );
                let mut scheduled: Vec<(usize, [u8; 3])> = Vec::with_capacity(messages.len());
                for message in messages {
                    let frame = if message.timestamped {
                        let offset_ms = message.sent_ms + delay_ms - play_start_ms;
                        if offset_ms < 0.0 {
                            stats.late_messages += 1;
                        }
                        let frame = (offset_ms.max(0.0) * f64::from(sample_rate) / 1000.0) as usize;
                        if frame >= frames {
                            // Its time falls in a later buffer.
                            waiting.push(message);
                            continue;
                        }
                        frame
                    } else {
                        0
                    };
                    if message.bytes[0] & 0xF0 == 0x90 && message.bytes[2] > 0 {
                        let sound_ms =
                            play_start_ms + frame as f64 * 1000.0 / f64::from(sample_rate);
                        stats.note(message.sent_ms, sound_ms - message.sent_ms);
                    }
                    scheduled.push((frame, message.bytes));
                }
                scheduled.sort_by_key(|(frame, _)| *frame);

                let mut next = 0;
                for frame in 0..frames {
                    while next < scheduled.len() && scheduled[next].0 <= frame {
                        synth.message(scheduled[next].1);
                        next += 1;
                    }
                    let sample = synth.sample();
                    for out in &mut data[frame * channels..(frame + 1) * channels] {
                        *out = sample;
                    }
                }
                while next < scheduled.len() {
                    synth.message(scheduled[next].1);
                    next += 1;
                }

                stats.buffer(callback_ms, lead_ms, buffer_ms, frames, sample_rate);
            },
            |error| report(&format!("built-in synth stream error: {error}")),
            None,
        )
        .map_err(|e| error(&e))?;
    stream.play().map_err(|e| error(&e))?;
    report(&format!(
        "built-in synth: {} Hz, {} channels, stream started",
        sample_rate, channels
    ));
    Ok(stream)
}

#[derive(Default)]
struct Stats {
    frames: usize,
    buffers: usize,
    late_buffers: usize,
    late_messages: usize,
    last_callback_ms: Option<f64>,
    max_callback_gap_ms: f64,
    min_lead_ms: Option<f64>,
    latencies: Vec<f64>,
    last_note_on_ms: Option<f64>,
    note_on_intervals: Vec<f64>,
}

impl Stats {
    fn note(&mut self, sent_ms: f64, latency_ms: f64) {
        self.latencies.push(latency_ms);
        if let Some(last) = self.last_note_on_ms {
            // Ignore Note Ons sent in the same tick (chords, several channels).
            if sent_ms - last > 1.0 {
                self.note_on_intervals.push(sent_ms - last);
                self.last_note_on_ms = Some(sent_ms);
            }
        } else {
            self.last_note_on_ms = Some(sent_ms);
        }
    }

    fn buffer(
        &mut self,
        callback_ms: f64,
        lead_ms: f64,
        buffer_ms: f64,
        frames: usize,
        sample_rate: f32,
    ) {
        self.buffers += 1;
        self.frames += frames;
        if lead_ms <= 0.0 {
            self.late_buffers += 1;
        }
        self.min_lead_ms = Some(self.min_lead_ms.map_or(lead_ms, |min| min.min(lead_ms)));
        if let Some(last) = self.last_callback_ms {
            self.max_callback_gap_ms = self.max_callback_gap_ms.max(callback_ms - last);
        }
        self.last_callback_ms = Some(callback_ms);
        if self.frames as f32 >= 2.0 * sample_rate {
            report(&format!(
                "built-in synth: {} buffers of {:.1} ms, {} late, max callback gap {:.1} ms, \
                 min lead {:.1} ms | {} notes, send-to-sound {} | Note On intervals {} | \
                 {} timestamped messages too late",
                self.buffers,
                buffer_ms,
                self.late_buffers,
                self.max_callback_gap_ms,
                self.min_lead_ms.unwrap_or(0.0),
                self.latencies.len(),
                spread(&self.latencies),
                spread(&self.note_on_intervals),
                self.late_messages,
            ));
            let last_note_on_ms = self.last_note_on_ms;
            *self = Self {
                last_callback_ms: Some(callback_ms),
                last_note_on_ms,
                ..Self::default()
            };
        }
    }
}

fn spread(values: &[f64]) -> String {
    if values.is_empty() {
        return "-".to_owned();
    }
    let min = values.iter().copied().fold(f64::INFINITY, f64::min);
    let max = values.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let mean = values.iter().sum::<f64>() / values.len() as f64;
    format!(
        "min {min:.1} / mean {mean:.1} / max {max:.1} ms (spread {:.1})",
        max - min
    )
}

const VOICES: usize = 16;

#[derive(Clone, Copy, Default)]
struct Voice {
    active: bool,
    gate: bool,
    channel: u8,
    note: u8,
    age: u64,
    phase: f32,
    frequency: f32,
    level: f32,
    envelope: f32,
    drum: Option<Drum>,
}

///
/// An 808-style kit, rebuilt from what the circuits do rather than from
/// recordings. A drum channel picks the instrument by pitch class, in General
/// MIDI's order from C: kick, rim, snare, clap, cowbell, low tom, closed hat,
/// mid tom, cymbal, high tom, open hat, conga.
///
#[derive(Clone, Copy)]
enum Instrument {
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
    fn of(note: u8) -> Self {
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

/// The six square oscillators, at clashing pitches, that make the 808's
/// hats and cymbal sound metallic.
const METAL: [f32; 6] = [205.3, 304.4, 369.6, 522.7, 540.0, 800.0];

#[derive(Clone, Copy)]
struct Drum {
    instrument: Instrument,
    time: f32,
    phases: [f32; 6],
    noise: u32,
    first: Svf,
    second: Svf,
}

impl Drum {
    fn new(instrument: Instrument, sample_rate: f32) -> Self {
        let (first, second) = match instrument {
            Instrument::Snare => (Svf::new(1800.0, 0.7, sample_rate), Svf::default()),
            Instrument::Clap => (Svf::new(1100.0, 1.5, sample_rate), Svf::default()),
            Instrument::Rim => (Svf::new(2500.0, 2.0, sample_rate), Svf::default()),
            Instrument::Cowbell => (Svf::new(800.0, 2.5, sample_rate), Svf::default()),
            Instrument::ClosedHat | Instrument::OpenHat => (
                Svf::new(7100.0, 1.0, sample_rate),
                Svf::new(6000.0, 0.7, sample_rate),
            ),
            Instrument::Cymbal => (
                Svf::new(4500.0, 0.8, sample_rate),
                Svf::new(3000.0, 0.7, sample_rate),
            ),
            _ => (Svf::default(), Svf::default()),
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

    fn noise(&mut self) -> f32 {
        self.noise ^= self.noise << 13;
        self.noise ^= self.noise >> 17;
        self.noise ^= self.noise << 5;
        (self.noise as f32 / u32::MAX as f32) * 2.0 - 1.0
    }

    fn sine(&mut self, index: usize, frequency: f32, dt: f32) -> f32 {
        self.phases[index] = (self.phases[index] + frequency * dt).fract();
        (self.phases[index] * std::f32::consts::TAU).sin()
    }

    fn square(&mut self, index: usize, frequency: f32, dt: f32) -> f32 {
        self.phases[index] = (self.phases[index] + frequency * dt).fract();
        if self.phases[index] < 0.5 { 1.0 } else { -1.0 }
    }

    fn metal(&mut self, dt: f32) -> f32 {
        (0..6)
            .map(|index| self.square(index, METAL[index], dt))
            .sum::<f32>()
            / 6.0
    }

    /// A tom or conga: a sine that starts a little sharp and settles.
    fn tom(&mut self, pitch: f32, decay: f32, dt: f32) -> (f32, f32) {
        let frequency = pitch * (1.0 + 0.3 * (-self.time / 0.02).exp());
        (self.sine(0, frequency, dt), (-self.time / decay).exp())
    }

    /// The next sample and its loudness envelope.
    fn sample(&mut self, dt: f32) -> (f32, f32) {
        let t = self.time;
        self.time += dt;
        match self.instrument {
            Instrument::Kick => {
                let frequency = 52.0 + 90.0 * (-t / 0.012).exp();
                let body = (1.8 * self.sine(0, frequency, dt)).tanh();
                let click = if t < 0.002 { self.noise() * 0.5 } else { 0.0 };
                (body + click, (-t / 0.45).exp())
            }
            Instrument::Snare => {
                let tone = 0.6 * self.sine(0, 180.0, dt) * (-t / 0.05).exp()
                    + 0.4 * self.sine(1, 330.0, dt) * (-t / 0.035).exp();
                let noise = self.noise();
                let snappy = self.first.high(noise) * (-t / 0.11).exp();
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
                let metal = self.metal(dt);
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
}

/// A state-variable filter (topology-preserving transform) with one cutoff.
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
        let g = (std::f32::consts::PI * cutoff / sample_rate).tan();
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

    /// Advances one sample and answers (low, band, high).
    fn tick(&mut self, input: f32) -> (f32, f32, f32) {
        let v3 = input - self.ic2;
        let v1 = self.a1 * self.ic1 + self.a2 * v3;
        let v2 = self.ic2 + self.a2 * self.ic1 + self.a3 * v3;
        self.ic1 = 2.0 * v1 - self.ic1;
        self.ic2 = 2.0 * v2 - self.ic2;
        (v2, v1, input - self.k * v1 - v2)
    }

    fn band(&mut self, input: f32) -> f32 {
        self.tick(input).1
    }

    fn high(&mut self, input: f32) -> f32 {
        self.tick(input).2
    }
}

/// Channels 0–7 are tones (sine, triangle, saw, square, repeating); channels
/// 8–15 are the 808-style kit (`Instrument`).
struct Synth {
    sample_rate: f32,
    voices: [Voice; VOICES],
    clock: u64,
}

impl Synth {
    fn new(sample_rate: f32) -> Self {
        Self {
            sample_rate,
            voices: [Voice::default(); VOICES],
            clock: 0,
        }
    }

    fn message(&mut self, [status, data1, data2]: [u8; 3]) {
        let channel = status & 0x0F;
        match status & 0xF0 {
            0x90 if data2 > 0 => self.note_on(channel, data1, data2),
            0x80 | 0x90 => {
                for voice in &mut self.voices {
                    if voice.active && voice.channel == channel && voice.note == data1 {
                        voice.gate = false;
                    }
                }
            }
            0xB0 if data1 == 120 || data1 == 123 => {
                for voice in &mut self.voices {
                    if voice.channel == channel {
                        voice.gate = false;
                    }
                }
            }
            _ => {}
        }
    }

    fn note_on(&mut self, channel: u8, note: u8, velocity: u8) {
        self.clock += 1;
        let drum = (channel >= 8).then(|| Drum::new(Instrument::of(note), self.sample_rate));
        // A closed hat chokes an open one, as on the 808.
        if let Some(Drum {
            instrument: Instrument::ClosedHat,
            ..
        }) = drum
        {
            for voice in &mut self.voices {
                if voice.channel == channel
                    && matches!(
                        voice.drum,
                        Some(Drum {
                            instrument: Instrument::OpenHat,
                            ..
                        })
                    )
                {
                    voice.active = false;
                }
            }
        }
        let slot = self
            .voices
            .iter()
            .position(|voice| !voice.active)
            .unwrap_or_else(|| {
                (0..VOICES)
                    .min_by_key(|&index| self.voices[index].age)
                    .unwrap_or(0)
            });
        self.voices[slot] = Voice {
            active: true,
            gate: true,
            channel,
            note,
            age: self.clock,
            phase: 0.0,
            frequency: 440.0 * 2f32.powf((f32::from(note) - 69.0) / 12.0),
            level: f32::from(velocity) / 127.0,
            envelope: 0.0,
            drum,
        };
    }

    fn sample(&mut self) -> f32 {
        let dt = 1.0 / self.sample_rate;
        let mut mix = 0.0;
        for voice in &mut self.voices {
            if !voice.active {
                continue;
            }
            if let Some(drum) = voice.drum.as_mut() {
                let (value, envelope) = drum.sample(dt);
                if envelope < 0.001 {
                    voice.active = false;
                    continue;
                }
                mix += value * envelope * voice.level * 0.35;
                continue;
            }
            voice.envelope = if voice.gate {
                (voice.envelope + dt / 0.005).min(1.0)
            } else {
                voice.envelope - dt / 0.15
            };
            if voice.envelope <= 0.0 {
                voice.active = false;
                continue;
            }
            voice.phase = (voice.phase + voice.frequency * dt).fract();
            let p = voice.phase;
            let value = match voice.channel % 4 {
                0 => (p * std::f32::consts::TAU).sin(),
                1 => 1.0 - 4.0 * (p - 0.5).abs(),
                2 => 2.0 * p - 1.0,
                _ => {
                    if p < 0.5 {
                        1.0
                    } else {
                        -1.0
                    }
                }
            };
            mix += value * voice.envelope * voice.level * 0.2;
        }
        mix.tanh()
    }
}

/// Resumes the shared stream, if one is open.
#[cfg(target_arch = "wasm32")]
pub fn resume() {
    STREAM.with(|slot| {
        if let Some((stream, _)) = slot.borrow().as_ref() {
            let _ = stream.play();
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    const RATE: f32 = 48_000.0;

    fn render(synth: &mut Synth, seconds: f32, out: &mut Vec<f32>) {
        for _ in 0..(seconds * RATE) as usize {
            out.push(synth.sample());
        }
    }

    #[test]
    fn every_drum_sounds_and_falls_silent() {
        for note in 36..48 {
            let mut synth = Synth::new(RATE);
            synth.message([0x99, note, 127]);
            let mut out = Vec::new();
            render(&mut synth, 8.0, &mut out);
            let peak = out.iter().fold(0.0f32, |peak, s| peak.max(s.abs()));
            assert!(out.iter().all(|s| s.is_finite()), "note {note}");
            assert!(peak > 0.05, "note {note} peak {peak}");
            assert!(
                synth.voices.iter().all(|voice| !voice.active),
                "note {note}"
            );
        }
    }

    /// Writes the kit and a beat as WAV files when `ORCVS_SYNTH_WAV_DIR` is set.
    #[test]
    fn render_the_kit_to_wav() {
        let Some(dir) = std::env::var_os("ORCVS_SYNTH_WAV_DIR") else {
            return;
        };
        let dir = std::path::PathBuf::from(dir);

        let mut synth = Synth::new(RATE);
        let mut kit = Vec::new();
        for note in 36..48 {
            synth.message([0x99, note, 120]);
            render(&mut synth, 0.6, &mut kit);
        }
        write_wav(&dir.join("808-kit.wav"), &kit);

        // Two bars at 120 BPM in 16ths, four times: kick, snare, clap,
        // closed and open hats, cowbell, and a bass on channel 0.
        let mut synth = Synth::new(RATE);
        let mut beat = Vec::new();
        let step = 0.125;
        for bar in 0..8 {
            for sixteenth in 0..16 {
                let mut hit = |note: u8, velocity: u8| synth.message([0x99, note, velocity]);
                if [0, 6, 10].contains(&sixteenth) {
                    hit(36, 127);
                }
                if [4, 12].contains(&sixteenth) {
                    hit(38, 110);
                }
                if sixteenth == 12 && bar % 2 == 1 {
                    hit(39, 110);
                }
                if sixteenth == 14 {
                    hit(46, 90);
                } else if sixteenth % 2 == 0 {
                    hit(42, 90);
                } else {
                    hit(42, 50);
                }
                if bar >= 4 && [3, 11].contains(&sixteenth) {
                    hit(40, 80);
                }
                if [0, 3, 8, 11].contains(&sixteenth) {
                    let bass = [36u8, 36, 39, 34][(bar % 4) as usize];
                    synth.message([0x90, bass, 100]);
                    render(&mut synth, step * 0.8, &mut beat);
                    synth.message([0x80, bass, 0]);
                    render(&mut synth, step * 0.2, &mut beat);
                } else {
                    render(&mut synth, step, &mut beat);
                }
            }
        }
        render(&mut synth, 1.0, &mut beat);
        write_wav(&dir.join("808-beat.wav"), &beat);
    }

    fn write_wav(path: &std::path::Path, samples: &[f32]) {
        let data_len = (samples.len() * 2) as u32;
        let mut bytes = Vec::with_capacity(44 + samples.len() * 2);
        bytes.extend_from_slice(b"RIFF");
        bytes.extend_from_slice(&(36 + data_len).to_le_bytes());
        bytes.extend_from_slice(b"WAVEfmt ");
        bytes.extend_from_slice(&16u32.to_le_bytes());
        bytes.extend_from_slice(&1u16.to_le_bytes());
        bytes.extend_from_slice(&1u16.to_le_bytes());
        bytes.extend_from_slice(&(RATE as u32).to_le_bytes());
        bytes.extend_from_slice(&(RATE as u32 * 2).to_le_bytes());
        bytes.extend_from_slice(&2u16.to_le_bytes());
        bytes.extend_from_slice(&16u16.to_le_bytes());
        bytes.extend_from_slice(b"data");
        bytes.extend_from_slice(&data_len.to_le_bytes());
        for sample in samples {
            let value = (sample.clamp(-1.0, 1.0) * f32::from(i16::MAX)) as i16;
            bytes.extend_from_slice(&value.to_le_bytes());
        }
        std::fs::write(path, bytes).expect("the WAV is written");
    }
}
