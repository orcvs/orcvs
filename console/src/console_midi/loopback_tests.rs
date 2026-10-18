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
//! `midi_loopback_through_a_virtual_port` needs the platform's MIDI service,
//! a sequencer on Linux (`/dev/snd/seq`) or CoreMIDI on macOS, and is ignored
//! by the ordinary suite so a host without one does not fail it.
//! `mise run test_midi_loopback` runs it.
//!

use std::sync::{Arc, Mutex};
use std::time::Duration;

use midir::os::unix::VirtualInput;
use midir::{Ignore, MidiInput, MidiInputConnection};

use orcvs::app::{InputEvent, InputKey, Orcvs};
use orcvs::grid::Grid;
use orcvs::midi::{
    MidiBackend, MidiConnection, MidiConnectionRequest, MidiDestinationId, MidiError,
    MidiOutputAdapter,
};
use orcvs::source::Source;

use super::ConsoleMidiBackend;

type Ledger = Arc<Mutex<Vec<Vec<u8>>>>;

///
/// The program every test plays. Each `.=0101` bangs the Cell two rows below
/// it on every Tick, so each Tick sends a Raw Play, a Control Change, a Pitch
/// Bend and a Timed Play, each on its own channel.
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
/// Only the three messages Orcvs sends are accepted, and each must be exactly
/// its own length: a stray byte, a missing one or a data byte with its high
/// bit set is a wire error and is answered as one rather than read around.
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
        (0x90, &[note, velocity]) => Ok(Heard::NoteOn {
            channel,
            note,
            velocity,
        }),
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
/// arrived, and the stream ends with every channel silenced and reset.
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
    ];
    for message in played {
        assert!(
            heard.contains(&message),
            "the listener never heard {message:?}; it heard {heard:?}"
        );
    }

    let safety_reset: Vec<Heard> = (0..16)
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
        .collect();
    assert!(
        heard.ends_with(&safety_reset),
        "the stream does not end with the safety reset on all sixteen channels: {heard:?}"
    );
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
struct Recording {
    destination: Box<dyn MidiConnection>,
    sent: Ledger,
}

impl MidiConnection for Recording {
    fn send(&mut self, message: &[u8]) -> Result<(), MidiError> {
        self.destination.send(message)?;
        self.sent.lock().unwrap().push(message.to_vec());
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
    fn open() -> Result<Self, String> {
        // The process id keeps two concurrent runs on one host from finding
        // each other's port.
        let name = format!("Orcvs loopback listener {}", std::process::id());
        let mut input = MidiInput::new(&name).map_err(|error| error.to_string())?;
        input.ignore(Ignore::None);
        let received = Ledger::default();
        let ledger = received.clone();
        let connection = input
            .create_virtual(
                &name,
                move |_timestamp, message, ()| ledger.lock().unwrap().push(message.to_vec()),
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

fn space(orcvs: &mut Orcvs) {
    orcvs.event_handler(vec![InputEvent::KeyPressed(InputKey::Space)]);
}

fn ready(request: MidiConnectionRequest) -> Box<dyn MidiConnection> {
    match request {
        MidiConnectionRequest::Ready(connection) => connection,
        MidiConnectionRequest::Pending(_) => panic!("a native MIDI port opens synchronously"),
    }
}

#[tokio::test(start_paused = true)]
async fn midi_loopback_in_process() {
    let sent = Ledger::default();
    let received = Ledger::default();
    let mut orcvs = Orcvs::with_source_and_midi_output_adapter(program(), MidiOutputAdapter::new())
        .expect("the test runtime");
    orcvs
        .midi_selection_handle()
        .install(
            MidiDestinationId::new("in-process"),
            Box::new(Recording {
                destination: Box::new(InProcessListener {
                    received: received.clone(),
                }),
                sent: sent.clone(),
            }),
        )
        .expect("a running Orcvs");

    space(&mut orcvs);
    tokio::time::sleep(orcvs.bpm().tick_period() * 3).await;
    space(&mut orcvs);
    tokio::task::yield_now().await;
    drop(orcvs);

    let heard = cross_check(&sent.lock().unwrap(), &received.lock().unwrap());
    assert_plays_the_program(&heard);
}

#[tokio::test]
#[ignore = "needs the platform's MIDI service; run with `mise run test_midi_loopback`"]
async fn midi_loopback_through_a_virtual_port() {
    let listener = VirtualListener::open().expect("a virtual input port on this host");
    let mut backend = ConsoleMidiBackend::new();
    let destination = backend
        .destinations()
        .expect("the platform's MIDI destinations")
        .into_iter()
        .find(|destination| destination.name.contains(&listener.name))
        .unwrap_or_else(|| panic!("no destination is named after {:?}", listener.name));
    let sent = Ledger::default();
    let mut orcvs = Orcvs::with_source_and_midi_output_adapter(program(), MidiOutputAdapter::new())
        .expect("the test runtime");
    orcvs
        .midi_selection_handle()
        .install(
            destination.id.clone(),
            Box::new(Recording {
                destination: ready(
                    backend
                        .connect(&destination.id)
                        .expect("the listener's port opens"),
                ),
                sent: sent.clone(),
            }),
        )
        .expect("a running Orcvs");

    space(&mut orcvs);
    tokio::time::sleep(orcvs.bpm().tick_period() * 3).await;
    space(&mut orcvs);
    tokio::task::yield_now().await;
    drop(orcvs);

    // Delivery crosses the platform's MIDI service on its own thread, so the
    // received ledger catches up some time after the last send. Two seconds
    // is far beyond any local delivery and bounds a run whose messages were
    // lost.
    let deadline = tokio::time::Instant::now() + Duration::from_secs(2);
    while listener.received.lock().unwrap().len() < sent.lock().unwrap().len()
        && tokio::time::Instant::now() < deadline
    {
        tokio::time::sleep(Duration::from_millis(10)).await;
    }

    let heard = cross_check(&sent.lock().unwrap(), &listener.received.lock().unwrap());
    assert_plays_the_program(&heard);
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
    ] {
        assert_eq!(decode(bytes), Err(error.to_owned()));
    }
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
