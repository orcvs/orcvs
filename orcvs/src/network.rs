//!
//! The network output adapter: where UDP `!u` and OSC `!o` send, and the bytes
//! each sends.
//!
//! The Playback Engine hands it every network send of a Tick in Tick Plan
//! order. A native build sends each as one datagram from one
//! `std::net::UdpSocket`; a browser cannot open one, so the web build reports
//! every send as a failure instead. Orcvs only sends: nothing here receives.
//!

use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use lang::{NetworkSend, OscArgument, OscMessage};
use tokio::sync::watch;

use crate::playback::OutputAdapterError;

///
/// Where the network sends go: one host, a port for UDP and a port for OSC.
///
/// A console setting rather than Source, so a Source file means the same thing
/// wherever it is played. The defaults are Orca's.
///
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "persistence", derive(serde::Serialize, serde::Deserialize))]
pub struct NetworkDestination {
    pub host: IpAddr,
    pub udp_port: u16,
    pub osc_port: u16,
}

impl Default for NetworkDestination {
    fn default() -> Self {
        Self {
            host: IpAddr::V4(Ipv4Addr::LOCALHOST),
            udp_port: 49161,
            osc_port: 49162,
        }
    }
}

///
/// Where a Network Send leaves: the seam between the Playback Engine's network
/// output and a transport.
///
/// Each target supplies its own: a native build sends from a socket, and the
/// web build, which cannot open one, refuses every send.
///
pub(crate) trait NetworkAdapter: Send {
    ///
    /// Sends `send` to its port at `destination`, or answers why it did not
    /// leave.
    ///
    fn send(
        &mut self,
        send: &NetworkSend,
        destination: &NetworkDestination,
    ) -> Result<(), OutputAdapterError>;
}

/// The network adapter this build sends through.
pub(crate) fn platform_adapter() -> Box<dyn NetworkAdapter> {
    #[cfg(not(target_arch = "wasm32"))]
    return Box::new(SocketAdapter::default());
    #[cfg(target_arch = "wasm32")]
    return Box::new(RefusingAdapter);
}

///
/// Sends what `!u` and `!o` ask for to the destination last published on its
/// channel, which it reads at each send, so a change takes effect at the next
/// one.
///
pub(crate) struct NetworkOutput {
    destination: watch::Receiver<NetworkDestination>,
    adapter: Box<dyn NetworkAdapter>,
}

impl NetworkOutput {
    pub(crate) fn new(
        destination: watch::Receiver<NetworkDestination>,
        adapter: Box<dyn NetworkAdapter>,
    ) -> Self {
        Self {
            destination,
            adapter,
        }
    }

    ///
    /// Makes every one of `sends`, in their order, and answers one failure per
    /// send that failed. A failure stops nothing: the sends after it are still
    /// made.
    ///
    pub(crate) fn deliver(&mut self, sends: &[NetworkSend]) -> Vec<OutputAdapterError> {
        let destination = *self.destination.borrow();
        sends
            .iter()
            .filter_map(|send| self.adapter.send(send, &destination).err())
            .collect()
    }
}

///
/// The native adapter: one datagram per send from one `std::net::UdpSocket`,
/// bound at the first send and bound again only when the host changes address
/// family.
///
#[cfg(not(target_arch = "wasm32"))]
#[derive(Default)]
pub(crate) struct SocketAdapter {
    socket: Option<std::net::UdpSocket>,
}

#[cfg(not(target_arch = "wasm32"))]
impl NetworkAdapter for SocketAdapter {
    fn send(
        &mut self,
        send: &NetworkSend,
        destination: &NetworkDestination,
    ) -> Result<(), OutputAdapterError> {
        let (datagram, port) = match send {
            NetworkSend::Udp(bytes) => {
                (std::borrow::Cow::Borrowed(&bytes[..]), destination.udp_port)
            }
            NetworkSend::Osc(message) => (
                std::borrow::Cow::Owned(encode(message)),
                destination.osc_port,
            ),
        };
        let to = SocketAddr::new(destination.host, port);
        let failed = |error: std::io::Error| {
            OutputAdapterError::new(format!("network send to {to} failed: {error}"))
        };
        let socket = match self.socket.take() {
            Some(socket)
                if socket
                    .local_addr()
                    .is_ok_and(|local| local.is_ipv4() == to.is_ipv4()) =>
            {
                socket
            }
            _ => {
                let any = match to.ip() {
                    IpAddr::V4(_) => IpAddr::V4(Ipv4Addr::UNSPECIFIED),
                    IpAddr::V6(_) => IpAddr::V6(std::net::Ipv6Addr::UNSPECIFIED),
                };
                let socket = std::net::UdpSocket::bind((any, 0)).map_err(failed)?;
                // A broadcast host is a destination like any other. A send
                // that cannot go at once fails rather than holding the Tick.
                socket.set_broadcast(true).map_err(failed)?;
                socket.set_nonblocking(true).map_err(failed)?;
                socket
            }
        };
        let sent = socket.send_to(&datagram, to);
        self.socket = Some(socket);
        sent.map(drop).or_else(|error| match reported(error) {
            Some(error) => Err(failed(error)),
            None => Ok(()),
        })
    }
}

///
/// The web adapter: a browser cannot open a UDP socket, so every send is
/// refused, before anything is encoded.
///
#[cfg(any(target_arch = "wasm32", test))]
pub(crate) struct RefusingAdapter;

#[cfg(any(target_arch = "wasm32", test))]
impl NetworkAdapter for RefusingAdapter {
    fn send(
        &mut self,
        _send: &NetworkSend,
        _destination: &NetworkDestination,
    ) -> Result<(), OutputAdapterError> {
        Err(OutputAdapterError::new("this build cannot send UDP or OSC"))
    }
}

///
/// A test adapter that records every send and its destination, and refuses
/// them while it is told to.
///
#[cfg(test)]
#[derive(Clone, Default)]
pub(crate) struct RecordingAdapter {
    state: std::sync::Arc<std::sync::Mutex<Recorded>>,
}

#[cfg(test)]
#[derive(Default)]
struct Recorded {
    sends: Vec<(NetworkSend, NetworkDestination)>,
    refusal: Option<String>,
}

#[cfg(test)]
impl RecordingAdapter {
    /// Every send made so far, with the destination it was made to.
    pub(crate) fn sends(&self) -> Vec<(NetworkSend, NetworkDestination)> {
        self.state.lock().unwrap().sends.clone()
    }

    /// Refuses every send from now on with `message`, or accepts them again.
    pub(crate) fn refuse(&self, message: Option<&str>) {
        self.state.lock().unwrap().refusal = message.map(str::to_owned);
    }
}

#[cfg(test)]
impl NetworkAdapter for RecordingAdapter {
    fn send(
        &mut self,
        send: &NetworkSend,
        destination: &NetworkDestination,
    ) -> Result<(), OutputAdapterError> {
        let mut state = self.state.lock().unwrap();
        if let Some(refusal) = &state.refusal {
            return Err(OutputAdapterError::new(refusal.clone()));
        }
        state.sends.push((send.clone(), *destination));
        Ok(())
    }
}

///
/// The send error worth reporting, or `None` for `ConnectionReset`.
///
/// On Windows a datagram sent where nothing listens comes back as
/// `ConnectionReset` on a later send. It reports that earlier datagram, not
/// the one in hand, which was sent, so it is ignored.
///
#[cfg(not(target_arch = "wasm32"))]
fn reported(error: std::io::Error) -> Option<std::io::Error> {
    (error.kind() != std::io::ErrorKind::ConnectionReset).then_some(error)
}

///
/// One OSC 1.0 message's bytes: the address, the type tag string, then each
/// argument, with every string null-terminated and padded with nulls to a
/// multiple of four bytes and every number big-endian ("OSC Syntax").
///
pub(crate) fn encode(message: &OscMessage) -> Vec<u8> {
    let mut bytes = Vec::new();
    string(&mut bytes, &message.address);
    let tags: String = std::iter::once(',')
        .chain(message.arguments.iter().map(|argument| match argument {
            OscArgument::Int(_) => 'i',
            OscArgument::Float(_) => 'f',
            OscArgument::String(_) => 's',
        }))
        .collect();
    string(&mut bytes, &tags);
    for argument in &message.arguments {
        match argument {
            OscArgument::Int(int) => bytes.extend(int.to_be_bytes()),
            OscArgument::Float(float) => bytes.extend(float.to_be_bytes()),
            OscArgument::String(text) => string(&mut bytes, text),
        }
    }
    bytes
}

/// Appends `text` as an OSC-string: its bytes, then one to four nulls, so
/// the total is a multiple of four.
fn string(bytes: &mut Vec<u8>, text: &str) {
    bytes.extend(text.as_bytes());
    let nulls = 4 - text.len() % 4;
    bytes.extend(std::iter::repeat_n(0, nulls));
}

#[cfg(test)]
mod tests {
    use super::{NetworkDestination, encode};
    use lang::{OscArgument, OscMessage};

    fn message(address: &str, arguments: Vec<OscArgument>) -> OscMessage {
        OscMessage {
            address: address.to_owned(),
            arguments,
        }
    }

    #[test]
    fn the_default_destination_is_orcas() {
        assert_eq!(
            NetworkDestination::default(),
            NetworkDestination {
                host: "127.0.0.1".parse().unwrap(),
                udp_port: 49161,
                osc_port: 49162,
            }
        );
    }

    #[test]
    fn encodes_the_osc_specifications_first_example() {
        // OSC 1.0, "Examples": `/oscillator/4/frequency ,f 440.0`.
        let expected: Vec<u8> = [
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
            encode(&message(
                "/oscillator/4/frequency",
                vec![OscArgument::Float(440.0)]
            )),
            expected
        );
    }

    #[test]
    fn encodes_the_osc_specifications_second_example() {
        // OSC 1.0, "Examples": `/foo ,iisff 1000 -1 "hello" 1.234 5.678`.
        let expected: Vec<u8> = [
            b"/foo".as_slice(),
            b"\0\0\0\0",
            b",iis",
            b"ff\0\0",
            &[0x00, 0x00, 0x03, 0xe8],
            &[0xff, 0xff, 0xff, 0xff],
            b"hell",
            b"o\0\0\0",
            &[0x3f, 0x9d, 0xf3, 0xb6],
            &[0x40, 0xb5, 0xb2, 0x2d],
        ]
        .concat();
        assert_eq!(
            encode(&message(
                "/foo",
                vec![
                    OscArgument::Int(1000),
                    OscArgument::Int(-1),
                    OscArgument::String("hello".to_owned()),
                    OscArgument::Float(1.234),
                    OscArgument::Float(5.678),
                ]
            )),
            expected
        );
    }

    #[test]
    fn a_message_with_no_arguments_still_sends_its_type_tag_string() {
        assert_eq!(encode(&message("/go", vec![])), b"/go\0,\0\0\0");
    }

    mod output {
        use super::super::{NetworkDestination, NetworkOutput, RecordingAdapter, RefusingAdapter};
        use super::message;
        use lang::NetworkSend;
        use tokio::sync::watch;

        fn udp(bytes: &[u8]) -> NetworkSend {
            NetworkSend::Udp(bytes.into())
        }

        fn recording() -> (
            RecordingAdapter,
            NetworkOutput,
            watch::Sender<NetworkDestination>,
        ) {
            let adapter = RecordingAdapter::default();
            let (sender, receiver) = watch::channel(NetworkDestination::default());
            let output = NetworkOutput::new(receiver, Box::new(adapter.clone()));
            (adapter, output, sender)
        }

        #[test]
        fn sends_keep_their_order() {
            let (adapter, mut output, _sender) = recording();
            let osc = NetworkSend::Osc(message("/amp", vec![]));
            assert!(output.deliver(&[udp(b"first"), osc.clone()]).is_empty());
            let sent: Vec<NetworkSend> =
                adapter.sends().into_iter().map(|(send, _)| send).collect();
            assert_eq!(sent, [udp(b"first"), osc]);
        }

        #[test]
        fn a_destination_change_takes_effect_at_the_next_send() {
            let (adapter, mut output, sender) = recording();
            output.deliver(&[udp(b"one")]);
            sender.send_modify(|destination| destination.udp_port = 9000);
            output.deliver(&[udp(b"two")]);
            let ports: Vec<u16> = adapter.sends().iter().map(|(_, to)| to.udp_port).collect();
            assert_eq!(ports, [49161, 9000]);
        }

        #[test]
        fn each_failed_send_is_answered_and_the_sends_after_it_are_made() {
            let (adapter, mut output, _sender) = recording();
            adapter.refuse(Some("refused"));
            assert_eq!(output.deliver(&[udp(b"a"), udp(b"b")]).len(), 2);
            adapter.refuse(None);
            assert!(output.deliver(&[udp(b"c")]).is_empty());
            assert_eq!(adapter.sends().len(), 1);
        }

        #[test]
        fn the_web_adapter_refuses_every_send() {
            let (_sender, receiver) = watch::channel(NetworkDestination::default());
            let mut output = NetworkOutput::new(receiver, Box::new(RefusingAdapter));
            let failures = output.deliver(&[udp(b"hi"), NetworkSend::Osc(message("/x", vec![]))]);
            assert_eq!(failures.len(), 2);
            assert!(
                failures
                    .iter()
                    .all(|failure| failure.message == "this build cannot send UDP or OSC")
            );
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    mod native {
        use super::super::{NetworkAdapter, NetworkDestination, SocketAdapter, encode, reported};
        use super::message;
        use lang::{NetworkSend, OscArgument};
        use std::net::UdpSocket;
        use std::time::Duration;

        /// A loopback socket to receive on, and a destination whose UDP and
        /// OSC ports are that socket's.
        fn listening() -> (UdpSocket, NetworkDestination) {
            let listener = UdpSocket::bind("127.0.0.1:0").unwrap();
            listener
                .set_read_timeout(Some(Duration::from_secs(5)))
                .unwrap();
            let port = listener.local_addr().unwrap().port();
            let destination = NetworkDestination {
                host: "127.0.0.1".parse().unwrap(),
                udp_port: port,
                osc_port: port,
            };
            (listener, destination)
        }

        fn received(listener: &UdpSocket) -> Vec<u8> {
            let mut buffer = [0; 512];
            let (length, _) = listener.recv_from(&mut buffer).unwrap();
            buffer[..length].to_vec()
        }

        #[test]
        fn a_udp_send_arrives_as_its_exact_bytes() {
            let (listener, destination) = listening();
            let mut adapter = SocketAdapter::default();
            let send = NetworkSend::Udp(b"hi there ".as_slice().into());
            adapter.send(&send, &destination).unwrap();
            assert_eq!(received(&listener), b"hi there ");
        }

        #[test]
        fn an_osc_send_arrives_encoded() {
            let (listener, destination) = listening();
            let mut adapter = SocketAdapter::default();
            let osc = message("/amp", vec![OscArgument::Float(0.5)]);
            adapter
                .send(&NetworkSend::Osc(osc.clone()), &destination)
                .unwrap();
            assert_eq!(received(&listener), encode(&osc));
        }

        #[test]
        fn only_connection_reset_goes_unreported() {
            use std::io::{Error, ErrorKind};
            assert!(reported(Error::from(ErrorKind::ConnectionReset)).is_none());
            assert!(reported(Error::from(ErrorKind::PermissionDenied)).is_some());
        }

        #[test]
        fn a_refused_send_names_where_it_went() {
            let (_listener, mut destination) = listening();
            // Port zero is no destination, so the send is refused.
            destination.osc_port = 0;
            let failure = SocketAdapter::default()
                .send(&NetworkSend::Osc(message("/x", vec![])), &destination)
                .unwrap_err();
            assert!(
                failure
                    .message
                    .starts_with("network send to 127.0.0.1:0 failed"),
                "{failure:?}"
            );
        }
    }
}
