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
use std::time::Duration;

use orcvs::app::{InputEvent, InputKey, Orcvs};
use orcvs::network::NetworkDestination;
use orcvs::source::{Source, file};

///
/// One OSC argument as the listener reads it off the wire. Orcvs sends only
/// these three types.
///
#[derive(Debug, PartialEq)]
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
fn decode(packet: &[u8]) -> Result<Message, String> {
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
/// below it on every Tick. Each Clock `~.0140` answers the Tick modulo `40` (64) and
/// writes it into a Payload, so every send carries the Tick that sent it.
///
const PROGRAM: &str = "\
.=0101    .=0101
    ~.0140     ~.0140
!ut   !   !o/t   !
";

fn program() -> Source {
    file::read(PROGRAM.as_bytes()).expect("the program is a Source File")
}

/// One OSC message as the listener reads it: its address and arguments.
type Message = (String, Vec<Heard>);

/// What Tick `tick` of the program sends, in Tick Plan order: its UDP
/// datagram, then its OSC message.
fn tick_sends(tick: u8) -> (Vec<u8>, Message) {
    (
        format!("t {tick:02X}").into_bytes(),
        ("/t".to_owned(), vec![Heard::Int(i32::from(tick))]),
    )
}

fn space(orcvs: &mut Orcvs) {
    orcvs.event_handler(vec![InputEvent::KeyPressed(InputKey::Space)]);
}

///
/// A socket on a loopback port that both the UDP and the OSC port name, so the
/// order between a Tick's two sends is the order they arrive in.
///
fn listener() -> UdpSocket {
    let socket = UdpSocket::bind("127.0.0.1:0").expect("a loopback port");
    // Long enough for a datagram still crossing loopback on a slow runner.
    socket
        .set_read_timeout(Some(Duration::from_millis(200)))
        .expect("a read timeout");
    socket
}

fn destination(listener: &UdpSocket) -> NetworkDestination {
    let port = listener.local_addr().unwrap().port();
    NetworkDestination {
        host: Ipv4Addr::LOCALHOST.into(),
        udp_port: port,
        osc_port: port,
    }
}

///
/// Every datagram `listener` holds, read as one UDP datagram then one OSC
/// message per Tick.
///
fn received(listener: &UdpSocket) -> Vec<(Vec<u8>, Message)> {
    let mut packets = Vec::new();
    let mut buffer = [0; 512];
    while let Ok(length) = listener.recv(&mut buffer) {
        packets.push(buffer[..length].to_vec());
    }
    assert!(
        packets.len().is_multiple_of(2),
        "a Tick sent one datagram: {packets:?}"
    );
    packets
        .chunks(2)
        .map(|pair| {
            let osc = decode(&pair[1]).expect("a well-formed OSC message");
            (pair[0].clone(), osc)
        })
        .collect()
}

const TICKS: u8 = 8;

///
/// Plays the program for `TICKS` Ticks, stops half a period after the last,
/// and waits four more periods, so a send after the stop would arrive as one
/// datagram too many.
///
#[tokio::test(start_paused = true)]
async fn udp_and_osc_loopback_through_a_running_orcvs() {
    let socket = listener();
    let mut orcvs = Orcvs::with_source(program()).expect("the test runtime");
    orcvs.set_network_destination(destination(&socket));
    let tick_period = orcvs.bpm().tick_period();

    space(&mut orcvs);
    tokio::time::sleep(tick_period * u32::from(TICKS) - tick_period / 2).await;
    space(&mut orcvs);
    tokio::time::sleep(tick_period * 4).await;

    let sent: Vec<_> = (0..TICKS).map(tick_sends).collect();
    assert_eq!(received(&socket), sent);
    assert_eq!(orcvs.drain_playback_diagnostics(), vec![]);
}

///
/// Tick 0 goes to a port nothing listens on and Tick 1 to the listener. On
/// Windows Tick 0's last datagram comes back as `ConnectionReset` on the
/// socket's next send, Tick 1's first, which is still made and reports
/// nothing.
///
#[tokio::test(start_paused = true)]
async fn a_send_after_one_nothing_heard_is_made_and_reports_nothing() {
    let socket = listener();
    // Bound and dropped at once, so nothing listens on this port.
    let unheard = destination(&listener());
    let mut orcvs = Orcvs::with_source(program()).expect("the test runtime");
    orcvs.set_network_destination(unheard);
    let tick_period = orcvs.bpm().tick_period();

    space(&mut orcvs);
    tokio::time::sleep(tick_period / 2).await;
    orcvs.set_network_destination(destination(&socket));
    tokio::time::sleep(tick_period).await;
    space(&mut orcvs);
    tokio::time::sleep(tick_period * 4).await;

    assert_eq!(received(&socket), [tick_sends(1)]);
    assert_eq!(orcvs.drain_playback_diagnostics(), vec![]);
}
