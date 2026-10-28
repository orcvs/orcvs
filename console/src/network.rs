//!
//! Where UDP `!u` and OSC `!o` send: one host and two ports, a console setting
//! the Panel shows beside MIDI Output and the console's storage keeps. The
//! Source never names a destination.
//!

use orcvs::network::NetworkDestination;

///
/// The Storage key the destination lives under.
///
#[cfg(feature = "persistence")]
pub(crate) const NETWORK_KEY: &str = "orcvs_network_destination";

///
/// The destination the console was last stored with, or the default.
///
#[cfg(feature = "persistence")]
pub(crate) fn stored(storage: Option<&dyn eframe::Storage>) -> NetworkDestination {
    storage
        .and_then(|storage| eframe::get_value(storage, NETWORK_KEY))
        .unwrap_or_default()
}

///
/// Without the `persistence` feature nothing is stored, so the console always
/// starts on the default destination.
///
#[cfg(not(feature = "persistence"))]
pub(crate) fn stored(_storage: Option<&dyn eframe::Storage>) -> NetworkDestination {
    NetworkDestination::default()
}

#[cfg(all(feature = "persistence", not(target_arch = "wasm32")))]
pub(crate) fn store(storage: &mut dyn eframe::Storage, destination: &NetworkDestination) {
    eframe::set_value(storage, NETWORK_KEY, destination);
}

/// One of the three values the Panel lets a performer type. The web build
/// cannot send, so it shows none of them.
#[cfg(not(target_arch = "wasm32"))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum NetworkField {
    Host,
    UdpPort,
    OscPort,
}

///
/// The destination the console sends to, and the last value it refused.
///
pub(crate) struct NetworkSetting {
    destination: NetworkDestination,
    refused: Option<String>,
}

impl NetworkSetting {
    pub(crate) fn new(destination: NetworkDestination) -> Self {
        Self {
            destination,
            refused: None,
        }
    }

    pub(crate) fn destination(&self) -> NetworkDestination {
        self.destination
    }

    /// The text `field` shows.
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) fn text(&self, field: NetworkField) -> String {
        match field {
            NetworkField::Host => self.destination.host.to_string(),
            NetworkField::UdpPort => self.destination.udp_port.to_string(),
            NetworkField::OscPort => self.destination.osc_port.to_string(),
        }
    }

    ///
    /// Takes `text` as `field`'s new value, answering whether it is one. A
    /// value that is not is reported in the status and leaves the destination
    /// as it was, so nothing invalid is ever sent to or saved.
    ///
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) fn commit(&mut self, field: NetworkField, text: &str) -> bool {
        let text = text.trim();
        let committed = match field {
            NetworkField::Host => text
                .parse()
                .map(|host| self.destination.host = host)
                .map_err(|_| format!("host {text:?} is not an IP address")),
            NetworkField::UdpPort => port(text).map(|port| self.destination.udp_port = port),
            NetworkField::OscPort => port(text).map(|port| self.destination.osc_port = port),
        };
        self.refused = committed.as_ref().err().cloned();
        committed.is_ok()
    }

    /// What the Panel shows: the value it refused.
    pub(crate) fn status(&self) -> Option<&str> {
        self.refused.as_deref()
    }
}

/// A port a datagram can be sent to: 1 through 65535.
#[cfg(not(target_arch = "wasm32"))]
fn port(text: &str) -> Result<u16, String> {
    text.parse()
        .ok()
        .filter(|port| *port != 0)
        .ok_or_else(|| format!("port {text:?} is not a number from 1 to 65535"))
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use orcvs::network::NetworkDestination;

    use super::{NetworkField, NetworkSetting};

    #[test]
    fn a_valid_value_changes_the_destination() {
        let mut setting = NetworkSetting::new(NetworkDestination::default());
        assert!(setting.commit(NetworkField::Host, " 192.168.1.20 "));
        assert!(setting.commit(NetworkField::UdpPort, "9000"));
        assert!(setting.commit(NetworkField::OscPort, "57120"));
        assert!(setting.commit(NetworkField::Host, "::1"));
        assert_eq!(
            setting.destination(),
            NetworkDestination {
                host: "::1".parse().unwrap(),
                udp_port: 9000,
                osc_port: 57120,
            }
        );
        assert_eq!(setting.text(NetworkField::OscPort), "57120");
        assert_eq!(setting.status(), None);
    }

    #[test]
    fn an_invalid_value_is_reported_and_changes_nothing() {
        let mut setting = NetworkSetting::new(NetworkDestination::default());
        for (field, text, status) in [
            (
                NetworkField::Host,
                "localhost",
                "host \"localhost\" is not an IP address",
            ),
            (
                NetworkField::UdpPort,
                "0",
                "port \"0\" is not a number from 1 to 65535",
            ),
            (
                NetworkField::OscPort,
                "65536",
                "port \"65536\" is not a number from 1 to 65535",
            ),
            (
                NetworkField::UdpPort,
                "",
                "port \"\" is not a number from 1 to 65535",
            ),
        ] {
            assert!(!setting.commit(field, text), "{text:?}");
            assert_eq!(setting.status(), Some(status));
            assert_eq!(setting.destination(), NetworkDestination::default());
        }
        assert!(setting.commit(NetworkField::UdpPort, "49161"));
        assert_eq!(setting.status(), None, "a valid value clears the report");
    }

    #[cfg(feature = "persistence")]
    #[test]
    fn a_stored_destination_is_read_back() {
        let mut storage = crate::persistence::InMemoryStorage::default();
        assert_eq!(super::stored(Some(&storage)), NetworkDestination::default());
        let destination = NetworkDestination {
            host: "10.0.0.2".parse().unwrap(),
            udp_port: 7000,
            osc_port: 7001,
        };
        super::store(&mut storage, &destination);
        assert_eq!(super::stored(Some(&storage)), destination);
    }
}
