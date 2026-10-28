#![cfg(not(target_arch = "wasm32"))]
//!
//! End-to-end UDP and OSC: a running Orcvs plays a Source of `!u` and `!o`
//! rows to a listener on a loopback socket, and each test checks what arrived
//! against what the Source asked for, Tick by Tick.
//!
//! The listener reads OSC with its own reading of OSC 1.0 rather than the
//! output adapter's encoder, so a byte the adapter gets wrong is a difference
//! these tests can see. A loopback socket needs no platform service, so these
//! run in the ordinary suite on every native platform.
//!

use std::net::{Ipv4Addr, UdpSocket};

use orcvs::app::{InputEvent, InputKey, Orcvs};
use orcvs::grid::Grid;
use orcvs::network::NetworkDestination;
use orcvs::source::Source;

///
/// One OSC argument as the listener reads it off the wire. Orcvs sends only
/// these three types.
///
#[derive(Clone, Debug, PartialEq)]
enum Heard {
    Int(i32),
    Float(f32),
    String(String),
}

///
/// Reads one OSC 1.0 message: a padded address, a padded type tag string and
/// one big-endian argument per tag.
///
/// Anything else is refused rather than read around: a packet whose length is
/// not a multiple of four, a string without its NUL and zero padding, a type
/// tag string that does not start with `,`, a tag Orcvs never sends, a
/// truncated argument, or bytes left over after the last argument.
///
fn decode(packet: &[u8]) -> Result<(String, Vec<Heard>), String> {
    if !packet.len().is_multiple_of(4) {
        return Err(format!("a packet of {} bytes is not padded", packet.len()));
    }
    let mut rest = packet;
    let address = string(&mut rest)?;
    let tags = string(&mut rest)?;
    let tags = tags
        .strip_prefix(',')
        .ok_or_else(|| format!("type tag string {tags:?} has no comma"))?;
    let mut arguments = Vec::new();
    for tag in tags.chars() {
        arguments.push(match tag {
            'i' => Heard::Int(i32::from_be_bytes(word(&mut rest)?)),
            'f' => Heard::Float(f32::from_be_bytes(word(&mut rest)?)),
            's' => Heard::String(string(&mut rest)?),
            other => return Err(format!("Orcvs never sends tag {other:?}")),
        });
    }
    if !rest.is_empty() {
        return Err(format!("{} bytes after the last argument", rest.len()));
    }
    Ok((address, arguments))
}

/// Reads one NUL-terminated string and the zero padding that ends it on a
/// four-byte boundary.
fn string(rest: &mut &[u8]) -> Result<String, String> {
    let end = rest
        .iter()
        .position(|byte| *byte == 0)
        .ok_or("a string with no NUL")?;
    let padded = (end / 4 + 1) * 4;
    if rest.len() < padded || rest[end..padded].iter().any(|byte| *byte != 0) {
        return Err(format!("a string not padded with zeros: {rest:?}"));
    }
    let text = String::from_utf8(rest[..end].to_vec()).map_err(|error| error.to_string())?;
    *rest = &rest[padded..];
    Ok(text)
}

fn word(rest: &mut &[u8]) -> Result<[u8; 4], String> {
    let (bytes, after) = rest
        .split_first_chunk::<4>()
        .ok_or("a truncated argument")?;
    *rest = after;
    Ok(*bytes)
}

#[test]
fn the_listener_reads_the_osc_specifications_example() {
    // OSC 1.0, "Examples": `/oscillator/4/frequency ,f 440.0`.
    let packet: Vec<u8> = [
        b"/osc".as_slice(),
        b"illa",
        b"tor/",
        b"4/fr",
        b"eque",
        b"ncy\0",
        b",f\0\0",
        &[0x43, 0xdc, 0x00, 0x00],
    ]
    .concat();
    assert_eq!(
        decode(&packet),
        Ok((
            "/oscillator/4/frequency".to_owned(),
            vec![Heard::Float(440.0)]
        ))
    );
}

#[test]
fn the_listener_reads_an_int_and_a_string() {
    let packet: Vec<u8> = [
        b"/t\0\0".as_slice(),
        b",is\0",
        &[0xff, 0xff, 0xff, 0xff],
        b"hi\0\0",
    ]
    .concat();
    assert_eq!(
        decode(&packet),
        Ok((
            "/t".to_owned(),
            vec![Heard::Int(-1), Heard::String("hi".to_owned())]
        ))
    );
}

#[test]
fn the_listener_refuses_what_is_not_a_complete_padded_message() {
    for (case, packet) in [
        ("an unpadded length", b"/t\0\0,\0\0".as_slice()),
        ("an address without its NUL", b"/abc,\0\0\0".as_slice()),
        ("padding that is not zero", b"/t\0x,\0\0\0".as_slice()),
        (
            "a type tag string without its comma",
            b"/t\0\0i\0\0\0\0\0\0\x01".as_slice(),
        ),
        ("a tag Orcvs never sends", b"/t\0\0,T\0\0".as_slice()),
        ("a truncated argument", b"/t\0\0,i\0\0".as_slice()),
        (
            "bytes after the last argument",
            b"/t\0\0,\0\0\0\0\0\0\0".as_slice(),
        ),
    ] {
        assert!(decode(packet).is_err(), "{case} was read: {packet:?}");
    }
}

///
/// The program the loopback plays. Each `.=0101` bangs the Function two rows
/// below it on every Tick. Each Clock `~.0140` answers the Tick, under 40, and
/// writes it into a Payload, so every send carries the Tick that sent it.
///
const PROGRAM: [&str; 3] = [
    ".=0101    .=0101",
    "    ~.0140     ~.0140",
    "!ut   !   !o/t   !",
];

const PROGRAM_WIDTH: usize = 24;

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

/// One datagram as the listener reads it.
#[derive(Debug, PartialEq)]
enum Received {
    Udp(Vec<u8>),
    Osc(String, Vec<Heard>),
}

/// What Tick `tick` of the program sends: its UDP datagram, then its OSC
/// message.
fn tick_sends(tick: u8) -> (Received, Received) {
    (
        Received::Udp(format!("t {tick:02X}").into_bytes()),
        Received::Osc("/t".to_owned(), vec![Heard::Int(i32::from(tick))]),
    )
}

fn space(orcvs: &mut Orcvs) {
    orcvs.event_handler(vec![InputEvent::KeyPressed(InputKey::Space)]);
}

///
/// A UDP and an OSC listener on loopback ports.
///
struct Listeners {
    udp: UdpSocket,
    osc: UdpSocket,
}

impl Listeners {
    fn bind() -> Self {
        Self {
            udp: listener(),
            osc: listener(),
        }
    }

    fn destination(&self) -> NetworkDestination {
        NetworkDestination {
            host: Ipv4Addr::LOCALHOST.into(),
            udp_port: self.udp.local_addr().unwrap().port(),
            osc_port: self.osc.local_addr().unwrap().port(),
        }
    }

    /// Every datagram waiting, as each port's sends in the order they arrived.
    fn received(&self) -> (Vec<Received>, Vec<Received>) {
        let udp = drain(&self.udp).into_iter().map(Received::Udp).collect();
        let osc = drain(&self.osc)
            .iter()
            .map(|packet| {
                let (address, arguments) = decode(packet).expect("a well-formed OSC message");
                Received::Osc(address, arguments)
            })
            .collect();
        (udp, osc)
    }
}

fn listener() -> UdpSocket {
    let socket = UdpSocket::bind("127.0.0.1:0").expect("a loopback port");
    socket
        .set_nonblocking(true)
        .expect("a non-blocking listener");
    socket
}

fn drain(socket: &UdpSocket) -> Vec<Vec<u8>> {
    let mut packets = Vec::new();
    let mut buffer = [0; 512];
    while let Ok(length) = socket.recv(&mut buffer) {
        packets.push(buffer[..length].to_vec());
    }
    packets
}

const TICKS: u8 = 8;

///
/// Plays the program for `TICKS` Ticks, stops half a period after the last,
/// and waits four more periods, so a send after the stop would arrive as one
/// datagram too many on either port.
///
#[tokio::test(start_paused = true)]
async fn udp_and_osc_loopback_through_a_running_orcvs() {
    let listeners = Listeners::bind();
    let mut orcvs = Orcvs::with_source(program()).expect("the test runtime");
    orcvs.set_network_destination(listeners.destination());
    let tick_period = orcvs.bpm().tick_period();

    space(&mut orcvs);
    tokio::time::sleep(tick_period * u32::from(TICKS) - tick_period / 2).await;
    space(&mut orcvs);
    tokio::time::sleep(tick_period * 4).await;

    let sent: (Vec<Received>, Vec<Received>) = (0..TICKS).map(tick_sends).unzip();
    assert_eq!(listeners.received(), sent);
    assert_eq!(orcvs.drain_playback_diagnostics(), vec![]);
}

///
/// Tick 0 goes to ports nothing listens on and Tick 1 to the listeners. On
/// Windows the first Tick's datagrams come back as `ConnectionReset` on the
/// second Tick's sends, which are still made and report nothing.
///
#[tokio::test(start_paused = true)]
async fn a_send_after_one_nothing_heard_is_made_and_reports_nothing() {
    let listeners = Listeners::bind();
    // Bound and dropped at once, so nothing listens on these ports.
    let unheard = Listeners::bind().destination();
    let mut orcvs = Orcvs::with_source(program()).expect("the test runtime");
    orcvs.set_network_destination(unheard);
    let tick_period = orcvs.bpm().tick_period();

    space(&mut orcvs);
    tokio::time::sleep(tick_period / 2).await;
    orcvs.set_network_destination(listeners.destination());
    tokio::time::sleep(tick_period).await;
    space(&mut orcvs);
    tokio::time::sleep(tick_period * 4).await;

    let (udp, osc) = tick_sends(1);
    assert_eq!(listeners.received(), (vec![udp], vec![osc]));
    assert_eq!(orcvs.drain_playback_diagnostics(), vec![]);
}
