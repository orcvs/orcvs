//!
//! End-to-end MIDI: a running Orcvs plays a Source through the console's MIDI
//! backend, a listener receives what it sent, and each test cross-checks the
//! two ledgers message by message.
//!
//! The sent ledger records every message the destination accepted, at the
//! [`MidiConnection`] the Playback Engine writes to. The received ledger is
//! filled by the listener. The listener decodes what it received with its own
//! reading of the wire format rather than the output adapter's encoder, so a
//! byte the adapter gets wrong is a difference these tests can see.
//!
//! The tests in `virtual_port` need the platform's MIDI service, a sequencer
//! on Linux (`/dev/snd/seq`) or CoreMIDI on macOS, and are ignored by the
//! ordinary suite so a host without one does not fail them.
//! `mise run test_midi_loopback` runs them.
//!

use std::sync::{Arc, Mutex};
use std::time::Duration;

use tokio::time::Instant;

use orcvs::app::{InputEvent, InputKey, Orcvs};
use orcvs::grid::Grid;
use orcvs::midi::{MidiConnection, MidiDestinationId, MidiError, MidiOutputAdapter};
use orcvs::playback::{PlaybackDiagnostic, PlaybackObservationWatch};
use orcvs::source::{Source, file};

type Ledger = Arc<Mutex<Vec<Vec<u8>>>>;

/// Each message the destination accepted, with the instant it was sent and
/// the absolute Tick the engine had last executed when it was.
type SentLedger = Arc<Mutex<Vec<(Instant, u64, Vec<u8>)>>>;

///
/// The program the `midi_loopback` tests play. Each `.=0101` bangs the Cell
/// two rows below it on every Tick, so each Tick sends a Raw Play, a Control
/// Change, a Pitch Bend and a Timed Play, each on its own channel.
///
const PROGRAM: [&str; 3] = [
    ".=0101    .=0101    .=0101    .=0101",
    "",
    "!>007FC4  !c010740  !b020040  !~030AD401",
];

const PROGRAM_WIDTH: usize = 40;

///
/// A MIDI message as the listener reads it off the wire.
///
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Heard {
    NoteOn {
        channel: u8,
        note: u8,
        velocity: u8,
    },
    NoteOff {
        channel: u8,
        note: u8,
    },
    ControlChange {
        channel: u8,
        controller: u8,
        value: u8,
    },
    PitchBend {
        channel: u8,
        value: u16,
    },
}

///
/// Reads one complete channel message.
///
/// Only the four messages Orcvs sends are accepted, and each must be exactly
/// its own length: a stray byte, a missing one or a data byte with its high
/// bit set is a wire error and is answered as one rather than read around.
/// Orcvs sends every stop as a Note Off at release velocity `00`, so a Note On
/// at velocity `00` and a Note Off with a release velocity are refused too.
///
fn decode(bytes: &[u8]) -> Result<Heard, String> {
    let [status, data @ ..] = bytes else {
        return Err("an empty message".to_owned());
    };
    if status & 0x80 == 0 {
        return Err(format!("{bytes:02x?} does not start with a status byte"));
    }
    if data.iter().any(|byte| byte & 0x80 != 0) {
        return Err(format!(
            "{bytes:02x?} has a data byte with its high bit set"
        ));
    }
    let channel = status & 0x0f;
    match (status & 0xf0, data) {
        (0x90, &[_, 0]) => Err(format!(
            "{bytes:02x?} is a Note On at velocity 00, which Orcvs sends as a Note Off"
        )),
        (0x90, &[note, velocity]) => Ok(Heard::NoteOn {
            channel,
            note,
            velocity,
        }),
        (0x80, &[note, 0]) => Ok(Heard::NoteOff { channel, note }),
        (0xb0, &[controller, value]) => Ok(Heard::ControlChange {
            channel,
            controller,
            value,
        }),
        (0xe0, &[lsb, msb]) => Ok(Heard::PitchBend {
            channel,
            value: (u16::from(msb) << 7) | u16::from(lsb),
        }),
        _ => Err(format!("{bytes:02x?} is not a message Orcvs sends")),
    }
}

///
/// Asserts the listener received exactly what was sent, in order, and
/// answers the received messages decoded.
///
fn cross_check(sent: &[Vec<u8>], received: &[Vec<u8>]) -> Vec<Heard> {
    if let Some(index) = sent
        .iter()
        .zip(received)
        .position(|(sent, received)| sent != received)
    {
        panic!(
            "message {index} was sent as {:02x?} and received as {:02x?}",
            sent[index], received[index]
        );
    }
    assert_eq!(
        sent.len(),
        received.len(),
        "sent {} messages and received {}; the first {} agree",
        sent.len(),
        received.len(),
        sent.len().min(received.len())
    );
    received
        .iter()
        .enumerate()
        .map(|(index, bytes)| {
            decode(bytes).unwrap_or_else(|error| panic!("message {index}: {error}"))
        })
        .collect()
}

///
/// Asserts the decoded stream is the program's: every message it plays
/// arrived, including the Note Off its Timed Play's expiry owes, and the
/// stream ends with every channel silenced and reset.
///
fn assert_plays_the_program(heard: &[Heard]) {
    let played = [
        Heard::NoteOn {
            channel: 0,
            note: 60,
            velocity: 0x7f,
        },
        Heard::ControlChange {
            channel: 1,
            controller: 0x07,
            value: 0x40,
        },
        Heard::PitchBend {
            channel: 2,
            value: 0x2000,
        },
        Heard::NoteOn {
            channel: 3,
            note: 62,
            velocity: 0x0a,
        },
        Heard::NoteOff {
            channel: 3,
            note: 62,
        },
    ];
    for message in played {
        assert!(
            heard.contains(&message),
            "the listener never heard {message:?}; it heard {heard:?}"
        );
    }

    assert!(
        heard.ends_with(&safety_reset()),
        "the stream does not end with the safety reset on all sixteen channels: {heard:?}"
    );
}

///
/// What a stop sends: every channel silenced, its controllers reset and its
/// bend centred.
///
fn safety_reset() -> Vec<Heard> {
    (0..16)
        .flat_map(|channel| {
            [
                Heard::ControlChange {
                    channel,
                    controller: 123,
                    value: 0,
                },
                Heard::ControlChange {
                    channel,
                    controller: 121,
                    value: 0,
                },
                Heard::PitchBend {
                    channel,
                    value: 0x2000,
                },
            ]
        })
        .collect()
}

fn program() -> Source {
    let grid = Grid::with_shape(PROGRAM_WIDTH, PROGRAM.len());
    let mut source = Source::new(grid);
    for (y, row) in PROGRAM.iter().enumerate() {
        for (x, content) in row.chars().enumerate().filter(|(_, c)| *c != ' ') {
            let cell = grid
                .cell_index(y * PROGRAM_WIDTH + x)
                .expect("the program fits its Grid");
            source
                .set(cell, &content.to_string())
                .expect("the program is valid Source");
        }
    }
    source
}

///
/// Forwards each message to the destination and records the ones it accepted.
///
/// The engine publishes a Tick before it delivers that Tick's messages, so the
/// observation read at each send names the Tick that sent it.
///
struct Recording {
    destination: Box<dyn MidiConnection>,
    observation: PlaybackObservationWatch,
    sent: SentLedger,
}

impl MidiConnection for Recording {
    fn send(&mut self, message: &[u8]) -> Result<(), MidiError> {
        self.destination.send(message)?;
        let tick = self.observation.borrow().tick.get();
        self.sent
            .lock()
            .unwrap()
            .push((Instant::now(), tick, message.to_vec()));
        Ok(())
    }
}

///
/// A listener inside the test's own process: what is sent to it is what it
/// receives, so a test over it checks the program's bytes and the
/// cross-check itself, and not a transport.
///
struct InProcessListener {
    received: Ledger,
}

impl MidiConnection for InProcessListener {
    fn send(&mut self, message: &[u8]) -> Result<(), MidiError> {
        self.received.lock().unwrap().push(message.to_vec());
        Ok(())
    }
}

fn space(orcvs: &mut Orcvs) {
    orcvs.event_handler(vec![InputEvent::KeyPressed(InputKey::Space)]);
}

/// The last message of the safety reset: channel 15's bend, centred.
const LAST_SAFETY_RESET_MESSAGE: [u8; 3] = [0xef, 0x00, 0x40];

///
/// What one run sent, from the Space that started it to the safety reset its
/// stop sent.
///
struct Run {
    started_at: Instant,
    tick_period: Duration,
    /// Every message the destination accepted, with the instant it was sent
    /// and the Tick that sent it.
    sent: Vec<(Instant, u64, Vec<u8>)>,
    /// How many of `sent` the run's Ticks sent; the rest are the stop's.
    ticked: usize,
    /// The last Tick the run executed.
    last_tick: u64,
    /// What the engine reported over the run.
    diagnostics: Vec<PlaybackDiagnostic>,
}

impl Run {
    fn bytes(&self) -> Vec<Vec<u8>> {
        self.sent
            .iter()
            .map(|(_, _, bytes)| bytes.clone())
            .collect()
    }

    ///
    /// The messages the run's Ticks sent, `heard` as the listener decoded
    /// them, grouped by the Tick that sent each.
    ///
    /// A run's Tick `k` is due `k` periods after its start, and the start
    /// follows the Space within a period, so the whole periods between the
    /// Space and a message name the Tick that sent it.
    ///
    fn by_tick(&self, heard: &[Heard], ticks: usize) -> Vec<Vec<Heard>> {
        let mut grouped = vec![Vec::new(); ticks];
        for (index, ((at, _, _), message)) in self.sent[..self.ticked].iter().zip(heard).enumerate()
        {
            let tick = (at.duration_since(self.started_at).as_nanos() / self.tick_period.as_nanos())
                as usize;
            assert!(
                tick < ticks,
                "message {index}, {message:?}, was sent at Tick {tick} of a run of {ticks}"
            );
            grouped[tick].push(*message);
        }
        grouped
    }
}

///
/// Plays `source` into `connection` for exactly `ticks` Ticks, stops, and
/// answers what the run sent.
///
/// The run is stopped half a period after its last Tick, so the Tick after it
/// is not yet due and every earlier one has been sent. The key press only
/// queues the stop for the engine's task, and yielding once does not promise
/// that task a turn, so the wait is for the reset's last message to reach the
/// sent ledger. Neither program bends channel 15, so nothing a Tick sends can
/// end the ledger with that message.
///
async fn play(
    source: Source,
    destination: MidiDestinationId,
    connection: Box<dyn MidiConnection>,
    ticks: u32,
) -> Run {
    let sent = SentLedger::default();
    let mut orcvs = Orcvs::with_source_and_midi_output_adapter(source, MidiOutputAdapter::new())
        .expect("the test runtime");
    orcvs
        .midi_selection_handle()
        .install(
            destination,
            Box::new(Recording {
                destination: connection,
                observation: orcvs.playback_observation_watch(),
                sent: sent.clone(),
            }),
        )
        .expect("a running Orcvs");
    let tick_period = orcvs.bpm().tick_period();

    let started_at = Instant::now();
    space(&mut orcvs);
    tokio::time::sleep(tick_period * ticks - tick_period / 2).await;
    let ticked = sent.lock().unwrap().len();
    space(&mut orcvs);
    let deadline = Instant::now() + Duration::from_secs(2);
    while sent
        .lock()
        .unwrap()
        .last()
        .map(|(_, _, bytes)| bytes.as_slice())
        != Some(&LAST_SAFETY_RESET_MESSAGE[..])
    {
        assert!(
            Instant::now() < deadline,
            "the engine never sent the safety reset after the stop"
        );
        tokio::time::sleep(Duration::from_millis(1)).await;
    }
    let last_tick = orcvs.playback_observation().tick.get();
    let diagnostics = orcvs.drain_playback_diagnostics();
    drop(orcvs);

    let sent = std::mem::take(&mut *sent.lock().unwrap());
    Run {
        started_at,
        tick_period,
        sent,
        ticked,
        last_tick,
        diagnostics,
    }
}

#[tokio::test(start_paused = true)]
async fn midi_loopback_in_process() {
    let received = Ledger::default();
    let run = play(
        program(),
        MidiDestinationId::new("in-process"),
        Box::new(InProcessListener {
            received: received.clone(),
        }),
        3,
    )
    .await;

    let heard = cross_check(&run.bytes(), &received.lock().unwrap());
    assert_plays_the_program(&heard);
}

///
/// The Learn Orca tracker as shipped, its pattern holding `notes` as
/// (channel, step, Note spelling) in place of its one `C4`.
///
/// Channel `c`'s steps are the column `2 + 28(c / 4) + 7(c % 4) + 10` of rows
/// 14 to 29, each pair marked `|  |` inside the `||` Comment that spans the
/// row.
///
fn tracker(notes: &[(u8, usize, &str)]) -> Source {
    let text = include_str!("../../../examples/tracker.orcvs").replace("C4", "  ");
    let mut rows: Vec<String> = text.lines().map(str::to_owned).collect();
    for &(channel, step, note) in notes {
        let column = 2 + 28 * usize::from(channel / 4) + 7 * usize::from(channel % 4) + 10;
        let row = &mut rows[14 + step];
        assert_eq!(
            &row[column - 1..column + 3],
            "|  |",
            "step {step} is marked"
        );
        row.replace_range(column..column + 2, note);
    }
    file::read(rows.join("\n").as_bytes()).expect("the tracker is a Source File")
}

///
/// The tracker's pattern for the loopback: a chord across both groups of
/// voices on step 0, a lone voice on step 1, the same voice again with
/// another on step 2, a second chord on step 8, and lone voices on steps 12
/// and 15. Every other step rests, and so does every voice a step leaves
/// empty.
///
const TRACKER_NOTES: [(u8, usize, &str); 11] = [
    (0, 0, "C4"),
    (2, 0, "E4"),
    (5, 0, "G4"),
    (3, 1, "D4"),
    (3, 2, "F4"),
    (6, 2, "B4"),
    (1, 8, "G4"),
    (4, 8, "C4"),
    (7, 8, "E4"),
    (0, 12, "A4"),
    (6, 15, "C5"),
];

/// Ticks the tracker loopback plays: one bar of sixteen steps, then the first
/// step of the next.
const TRACKER_TICKS: u32 = 17;

///
/// What each Tick of the tracker run plays.
///
/// Within a Tick, the Note Offs of the Notes whose one-Tick length ends there
/// come first, in the order those Notes started; the Tick's Note Ons follow
/// in Source order, row by row from the top and left to right within a row,
/// which puts channels 3 and 7 first and 0 and 4 last. All on velocity
/// `0x77`, as each voice's `!~cc77` writes it.
///
fn tracker_plays() -> [Vec<Heard>; TRACKER_TICKS as usize] {
    let on = |channel, note| Heard::NoteOn {
        channel,
        note,
        velocity: 0x77,
    };
    let off = |channel, note| Heard::NoteOff { channel, note };
    let step_0 = [on(2, 64), on(5, 67), on(0, 60)];
    [
        step_0.to_vec(),
        vec![off(2, 64), off(5, 67), off(0, 60), on(3, 62)],
        vec![off(3, 62), on(3, 65), on(6, 71)],
        vec![off(3, 65), off(6, 71)],
        vec![],
        vec![],
        vec![],
        vec![],
        vec![on(7, 64), on(1, 67), on(4, 60)],
        vec![off(7, 64), off(1, 67), off(4, 60)],
        vec![],
        vec![],
        vec![on(0, 69)],
        vec![off(0, 69)],
        vec![],
        vec![on(6, 72)],
        [vec![off(6, 72)], step_0.to_vec()].concat(),
    ]
}

///
/// Asserts the tracker run executed every deadline and sent, Tick by Tick,
/// exactly what its pattern plays, and then the stop's safety reset.
///
fn assert_plays_the_tracker(run: &Run, heard: &[Heard]) {
    assert_eq!(run.diagnostics, [], "the engine reported nothing");
    assert_eq!(
        run.last_tick,
        u64::from(TRACKER_TICKS) - 1,
        "the run executed every Tick"
    );
    let played = run.by_tick(heard, TRACKER_TICKS as usize);
    for (tick, (played, expected)) in played.iter().zip(&tracker_plays()).enumerate() {
        assert_eq!(played, expected, "Tick {tick}");
    }
    assert_eq!(
        &heard[run.ticked..],
        safety_reset(),
        "the stop sends the safety reset and nothing else"
    );
}

#[tokio::test(start_paused = true)]
async fn the_tracker_loopback_in_process() {
    let received = Ledger::default();
    let run = play(
        tracker(&TRACKER_NOTES),
        MidiDestinationId::new("in-process"),
        Box::new(InProcessListener {
            received: received.clone(),
        }),
        TRACKER_TICKS,
    )
    .await;

    let heard = cross_check(&run.bytes(), &received.lock().unwrap());
    assert_plays_the_tracker(&run, &heard);
}

#[test]
fn the_listener_refuses_what_is_not_a_complete_channel_message() {
    for (bytes, error) in [
        (&[][..], "an empty message"),
        (
            &[0x40, 0x00][..],
            "[40, 00] does not start with a status byte",
        ),
        (
            &[0x90, 0x80, 0x00][..],
            "[90, 80, 00] has a data byte with its high bit set",
        ),
        (&[0x90, 0x3c][..], "[90, 3c] is not a message Orcvs sends"),
        (
            &[0x90, 0x3c, 0x7f, 0x00][..],
            "[90, 3c, 7f, 00] is not a message Orcvs sends",
        ),
        (&[0xc0, 0x01][..], "[c0, 01] is not a message Orcvs sends"),
        (
            &[0x93, 0x3e, 0x00][..],
            "[93, 3e, 00] is a Note On at velocity 00, which Orcvs sends as a Note Off",
        ),
        (
            &[0x83, 0x3e, 0x40][..],
            "[83, 3e, 40] is not a message Orcvs sends",
        ),
    ] {
        assert_eq!(decode(bytes), Err(error.to_owned()));
    }
}

#[test]
fn the_listener_reads_a_note_off_at_release_velocity_00() {
    assert_eq!(
        decode(&[0x83, 0x3e, 0x00]),
        Ok(Heard::NoteOff {
            channel: 3,
            note: 62,
        })
    );
}

#[test]
fn the_listener_reads_a_bend_low_half_first() {
    assert_eq!(
        decode(&[0xe5, 0x01, 0x40]),
        Ok(Heard::PitchBend {
            channel: 5,
            value: 0x2001,
        })
    );
}

#[test]
#[should_panic(expected = "message 1 was sent as [90, 3c, 7f] and received as [90, 3c, 00]")]
fn the_cross_check_names_the_first_message_that_differs() {
    let sent = [vec![0xb0, 7, 0x40], vec![0x90, 60, 0x7f]];
    let received = [vec![0xb0, 7, 0x40], vec![0x90, 60, 0x00]];
    cross_check(&sent, &received);
}

#[test]
#[should_panic(expected = "sent 2 messages and received 1; the first 1 agree")]
fn the_cross_check_counts_a_lost_message() {
    let sent = [vec![0xb0, 7, 0x40], vec![0x90, 60, 0x7f]];
    let received = [vec![0xb0, 7, 0x40]];
    cross_check(&sent, &received);
}

///
/// The transport half: a listener on a port the platform's MIDI service
/// offers. Virtual ports exist on every native target but Windows.
///
#[cfg(any(target_os = "macos", target_os = "linux"))]
mod virtual_port {
    use std::sync::PoisonError;
    use std::time::Duration;

    use midir::os::unix::VirtualInput;
    use midir::{Ignore, MidiInput, MidiInputConnection};

    use orcvs::midi::{MidiBackend, MidiConnection, MidiConnectionRequest};
    use orcvs::source::Source;

    use super::super::ConsoleMidiBackend;
    use super::{
        Heard, Ledger, Run, TRACKER_NOTES, TRACKER_TICKS, assert_plays_the_program, cross_check,
        play, program, tracker,
    };
    #[cfg(target_os = "macos")]
    use super::{PlaybackDiagnostic, safety_reset, tracker_plays};

    ///
    /// A listener on a virtual input port, which the platform's MIDI service
    /// offers to every output on the host, Orcvs's own included.
    ///
    /// The connection is held for the listener's lifetime, because dropping it
    /// closes the port.
    ///
    struct VirtualListener {
        name: String,
        received: Ledger,
        _connection: MidiInputConnection<()>,
    }

    impl VirtualListener {
        ///
        /// Opens a port named after `test`, so tests running at once in one
        /// process each find their own.
        ///
        fn open(test: &str) -> Result<Self, String> {
            // The process id keeps two concurrent runs on one host from finding
            // each other's port.
            let name = format!("Orcvs loopback listener {test} {}", std::process::id());
            let mut input = MidiInput::new(&name).map_err(|error| error.to_string())?;
            input.ignore(Ignore::None);
            let received = Ledger::default();
            let ledger = received.clone();
            let connection = input
                .create_virtual(
                    &name,
                    // A test that panics while holding the ledger poisons it; this
                    // thread keeps recording rather than panicking inside the
                    // platform's MIDI service.
                    move |_timestamp, message, ()| {
                        ledger
                            .lock()
                            .unwrap_or_else(PoisonError::into_inner)
                            .push(message.to_vec());
                    },
                    (),
                )
                .map_err(|error| error.to_string())?;
            Ok(Self {
                name,
                received,
                _connection: connection,
            })
        }
    }

    fn ready(request: MidiConnectionRequest) -> Box<dyn MidiConnection> {
        match request {
            MidiConnectionRequest::Ready(connection) => connection,
            MidiConnectionRequest::Pending(_) => panic!("a native MIDI port opens synchronously"),
        }
    }

    ///
    /// Plays `source` for `ticks` Ticks into a listener on a virtual port
    /// named after `test`, found through the console's own backend, and
    /// answers the run with the messages the listener decoded.
    ///
    async fn play_through_a_virtual_port(
        test: &str,
        source: Source,
        ticks: u32,
    ) -> (Run, Vec<Heard>) {
        let listener = VirtualListener::open(test).expect("a virtual input port on this host");
        let mut backend = ConsoleMidiBackend::new();
        let destinations = backend
            .destinations()
            .expect("the platform's MIDI destinations");
        // A platform may decorate a port's name (ALSA prefixes its client), so
        // the listener is found by containment, and only a single match is
        // trusted: a second would be a port left behind by another run.
        let [destination] = &destinations
            .iter()
            .filter(|destination| destination.name.contains(&listener.name))
            .collect::<Vec<_>>()[..]
        else {
            panic!(
                "expected one destination named after {:?}, found {destinations:?}",
                listener.name
            );
        };
        let id = destination.id.clone();
        let connection = ready(backend.connect(&id).expect("the listener's port opens"));
        let run = play(source, id, connection, ticks).await;

        // Delivery crosses the platform's MIDI service on its own thread, so the
        // received ledger catches up some time after the last send. Two seconds
        // is far beyond any local delivery and bounds a run whose messages were
        // lost.
        let deadline = tokio::time::Instant::now() + Duration::from_secs(2);
        while listener.received.lock().unwrap().len() < run.sent.len()
            && tokio::time::Instant::now() < deadline
        {
            tokio::time::sleep(Duration::from_millis(10)).await;
        }

        let heard = cross_check(&run.bytes(), &listener.received.lock().unwrap());
        (run, heard)
    }

    #[tokio::test]
    #[ignore = "needs the platform's MIDI service; run with `mise run test_midi_loopback`"]
    async fn midi_loopback_through_a_virtual_port() {
        let (_, heard) = play_through_a_virtual_port("program", program(), 3).await;
        assert_plays_the_program(&heard);
    }

    #[tokio::test]
    #[ignore = "needs the platform's MIDI service; run with `mise run test_midi_loopback`"]
    async fn the_tracker_loopback_through_a_virtual_port() {
        let (run, heard) =
            play_through_a_virtual_port("tracker", tracker(&TRACKER_NOTES), TRACKER_TICKS).await;
        #[cfg(not(target_os = "macos"))]
        super::assert_plays_the_tracker(&run, &heard);
        #[cfg(target_os = "macos")]
        assert_plays_the_tracker_through_overruns(&run, &heard);
    }

    ///
    /// Asserts the tracker run played its pattern exactly on every Tick it
    /// executed, and lost a deadline only where the engine reported an
    /// Overrun, then sent the stop's safety reset.
    ///
    /// The hosted macOS runner stalls the engine's task for longer than a
    /// period, and ADR 0037 declines a Tick that late rather than replaying
    /// it. A declined deadline plays nothing and consumes no absolute Tick,
    /// so the step it was due to play sounds at the next deadline and the
    /// Ticks the run executes are a prefix of the pattern. Each message is
    /// grouped by the Tick that sent it rather than by the period it was sent
    /// in, so a Tick that ran late keeps its own messages. Every other host
    /// holds the run to every deadline with `assert_plays_the_tracker`.
    ///
    #[cfg(target_os = "macos")]
    fn assert_plays_the_tracker_through_overruns(run: &Run, heard: &[Heard]) {
        let overruns: Vec<_> = run
            .diagnostics
            .iter()
            .filter_map(|diagnostic| match diagnostic {
                PlaybackDiagnostic::Overrun {
                    scheduled_at,
                    observed_at,
                } => Some((*scheduled_at, *observed_at - *scheduled_at)),
                _ => None,
            })
            .collect();
        assert_eq!(
            overruns.len(),
            run.diagnostics.len(),
            "the engine reported only Overruns: {:?}",
            run.diagnostics
        );
        let executed = usize::try_from(run.last_tick).expect("a short run") + 1;
        let expected = tracker_plays();
        assert!(
            executed == expected.len() || !overruns.is_empty(),
            "the run executed {executed} of {} Ticks and the engine reported no Overrun",
            expected.len()
        );
        assert!(
            executed <= expected.len(),
            "the run executed {executed} Ticks"
        );

        let ticked = heard.len() - safety_reset().len();
        let mut played = vec![Vec::new(); executed];
        for ((_, tick, _), message) in run.sent[..ticked].iter().zip(heard) {
            played[usize::try_from(*tick).expect("a short run")].push(*message);
        }
        for (tick, (played, expected)) in played.iter().zip(&expected).enumerate() {
            assert_eq!(
                played, expected,
                "Tick {tick}, with Overruns (deadline, lateness) {overruns:?}"
            );
        }
        assert_eq!(
            &heard[ticked..],
            safety_reset(),
            "the stop sends the safety reset and nothing else"
        );
        // TEMPORARY: report what the runner did, on every run.
        panic!(
            "diagnostic: executed {executed} of {} Ticks; period {:?}; Overruns (deadline, lateness) {overruns:?}",
            expected.len(),
            run.tick_period
        );
    }
}
