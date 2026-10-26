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
    let config = device
        .default_output_config()
        .map_err(|e| error(&e))?
        .config();
    let channels = usize::from(config.channels);
    let sample_rate = config.sample_rate as f32;
    let mut synth = Synth::new(sample_rate);
    let mut stats = Stats::default();
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
                // two buffers, so a message sent just after a callback still
                // lands inside the buffer that plays at its time.
                let delay_ms = 2.0 * buffer_ms + 20.0;

                let mut messages = std::mem::take(
                    &mut *inbox
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner),
                );
                let mut scheduled: Vec<(usize, [u8; 3])> = Vec::with_capacity(messages.len());
                for message in messages.drain(..) {
                    let frame = if message.timestamped {
                        let offset_ms = message.sent_ms + delay_ms - play_start_ms;
                        if offset_ms < 0.0 {
                            stats.late_messages += 1;
                        }
                        ((offset_ms.max(0.0) * f64::from(sample_rate) / 1000.0) as usize)
                            .min(frames.saturating_sub(1))
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
    noise: u32,
}

#[derive(Clone, Copy)]
enum Drum {
    Kick,
    Hat,
}

/// Channels 0–7 are tones (sine, triangle, saw, square, repeating); channels
/// 8–15 are drums, where notes below C3 are a kick and the rest a hat.
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
        let slot = self
            .voices
            .iter()
            .position(|voice| !voice.active)
            .unwrap_or_else(|| {
                (0..VOICES)
                    .min_by_key(|&index| self.voices[index].age)
                    .unwrap_or(0)
            });
        let drum = (channel >= 8).then_some(if note < 48 { Drum::Kick } else { Drum::Hat });
        self.voices[slot] = Voice {
            active: true,
            gate: true,
            channel,
            note,
            age: self.clock,
            phase: 0.0,
            frequency: 440.0 * 2f32.powf((f32::from(note) - 69.0) / 12.0),
            level: f32::from(velocity) / 127.0,
            envelope: if drum.is_some() { 1.0 } else { 0.0 },
            drum,
            noise: 0x1234_5678 ^ u32::from(note),
        };
    }

    fn sample(&mut self) -> f32 {
        let dt = 1.0 / self.sample_rate;
        let mut mix = 0.0;
        for voice in &mut self.voices {
            if !voice.active {
                continue;
            }
            let value = match voice.drum {
                Some(Drum::Kick) => {
                    voice.envelope -= dt / 0.25;
                    let frequency = 50.0 + 120.0 * voice.envelope.max(0.0).powi(3);
                    voice.phase = (voice.phase + frequency * dt).fract();
                    (voice.phase * std::f32::consts::TAU).sin()
                }
                Some(Drum::Hat) => {
                    voice.envelope -= dt / 0.06;
                    voice.noise ^= voice.noise << 13;
                    voice.noise ^= voice.noise >> 17;
                    voice.noise ^= voice.noise << 5;
                    (voice.noise as f32 / u32::MAX as f32) * 2.0 - 1.0
                }
                None => {
                    voice.envelope = if voice.gate {
                        (voice.envelope + dt / 0.005).min(1.0)
                    } else {
                        voice.envelope - dt / 0.15
                    };
                    voice.phase = (voice.phase + voice.frequency * dt).fract();
                    let p = voice.phase;
                    match voice.channel % 4 {
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
                    }
                }
            };
            if voice.envelope <= 0.0 {
                voice.active = false;
                continue;
            }
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
