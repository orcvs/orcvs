//! Native and browser MIDI discovery for the console.
//!
//! Device enumeration and port opening happen on the console's thread. Playback
//! receives an already-open connection through [`MidiSelectionHandle::install`].
//!
//! [`MidiSelectionHandle::install`]: orcvs::midi::MidiSelectionHandle::install

pub use backend::NativeMidiBackend;
#[cfg(target_arch = "wasm32")]
pub use backend::request_access_within;

///
/// Whether this build can present MIDI device selection.
///
pub const AVAILABLE: bool = backend::AVAILABLE;

#[cfg(all(
    not(target_arch = "wasm32"),
    any(target_os = "macos", target_os = "windows", target_os = "linux")
))]
mod backend {
    use std::sync::Mutex;

    use midir::{MidiOutput, MidiOutputConnection};

    use orcvs::midi::{MidiBackend, MidiConnection, MidiDestination, MidiDestinationId, MidiError};

    pub const AVAILABLE: bool = true;

    const MIDI_CLIENT_NAME: &str = "Orcvs";

    ///
    /// The platform MIDI service, reached through `midir` on the console thread.
    ///
    /// One `MidiOutput` is kept for enumeration and for finding the port to
    /// connect, because repeatedly creating a fresh client on macOS returns a
    /// stale port list until the process restarts. `connect` consumes that
    /// client to open the connection; the next enumeration or connect creates
    /// a fresh one.
    ///
    pub struct NativeMidiBackend {
        enumeration: Mutex<Option<MidiOutput>>,
    }

    impl Default for NativeMidiBackend {
        fn default() -> Self {
            Self {
                enumeration: Mutex::new(None),
            }
        }
    }

    impl NativeMidiBackend {
        pub fn new() -> Self {
            Self::default()
        }

        fn enumeration_client(&self) -> Result<MidiOutput, MidiError> {
            let mut guard = self
                .enumeration
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if guard.is_none() {
                *guard = Some(MidiOutput::new(MIDI_CLIENT_NAME).map_err(midi_error)?);
            }
            guard
                .take()
                .ok_or_else(|| MidiError::new("MIDI enumeration client is already in use"))
        }

        fn restore_enumeration_client(&self, output: MidiOutput) {
            let mut guard = self
                .enumeration
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            *guard = Some(output);
        }
    }

    impl MidiBackend for NativeMidiBackend {
        fn destinations(&mut self) -> Result<Vec<MidiDestination>, MidiError> {
            let output = self.enumeration_client()?;
            let mut destinations = Vec::new();
            for port in output.ports() {
                match output.port_name(&port) {
                    Ok(name) => destinations.push(MidiDestination::new(port.id(), name)),
                    Err(error) => {
                        self.restore_enumeration_client(output);
                        return Err(midi_error(error));
                    }
                }
            }
            self.restore_enumeration_client(output);
            Ok(destinations)
        }

        fn connect(
            &mut self,
            destination_id: &MidiDestinationId,
        ) -> Result<orcvs::midi::MidiConnectionRequest, MidiError> {
            let destination_id = destination_id.clone();
            let output = self.enumeration_client()?;
            let port = match output.find_port_by_id(destination_id.as_str()) {
                Some(port) => port,
                None => {
                    self.restore_enumeration_client(output);
                    return Err(MidiError::new(
                        "the selected MIDI destination is no longer available",
                    ));
                }
            };
            let port_name = format!("{MIDI_CLIENT_NAME} output");
            match output.connect(&port, &port_name) {
                Ok(connection) => Ok(orcvs::midi::MidiConnectionRequest::Ready(Box::new(
                    MidirConnection(connection),
                )
                    as Box<dyn MidiConnection>)),
                Err(error) => {
                    let message = error.to_string();
                    self.restore_enumeration_client(error.into_inner());
                    Err(MidiError::new(message))
                }
            }
        }
    }

    struct MidirConnection(MidiOutputConnection);

    impl MidiConnection for MidirConnection {
        fn send(&mut self, message: &[u8]) -> Result<(), MidiError> {
            self.0.send(message).map_err(midi_error)
        }
    }

    fn midi_error(error: impl std::fmt::Display) -> MidiError {
        MidiError::new(error.to_string())
    }
}

#[cfg(target_arch = "wasm32")]
mod backend {
    use orcvs::midi::{MidiBackend, MidiDestination, MidiDestinationId, MidiError};

    use crate::web_midi::{BrowserMidi, WebMidiBackend};

    pub const AVAILABLE: bool = true;

    ///
    /// The browser's Web MIDI, reached on the console thread.
    ///
    /// Building one asks the browser for MIDI access if this page has not
    /// asked yet; see `crate::web_midi` for how discovery and connect answer
    /// while that request is outstanding and after it is refused.
    ///
    pub struct NativeMidiBackend(WebMidiBackend<BrowserMidi>);

    impl Default for NativeMidiBackend {
        fn default() -> Self {
            Self::new()
        }
    }

    impl NativeMidiBackend {
        pub fn new() -> Self {
            Self(WebMidiBackend::browser())
        }
    }

    ///
    /// Asks the browser for MIDI access and waits for its answer, but for no
    /// longer than `timeout`.
    ///
    /// A page whose permission is already granted or blocked answers well
    /// within it, so the console built afterwards discovers with the answer on
    /// its first frame. A first visit shows a permission prompt the performer
    /// may take any time to answer, or never answer; the wait ends at
    /// `timeout` and the console starts with access pending. A browser without
    /// Web MIDI answers at once and is not waited for.
    ///
    pub async fn request_access_within(timeout: std::time::Duration) {
        let _ = BrowserMidi::request();
        let Some(answered) = BrowserMidi::answered() else {
            return;
        };
        let Some(window) = web_sys::window() else {
            return;
        };
        let millis = i32::try_from(timeout.as_millis()).unwrap_or(i32::MAX);
        let elapsed = js_sys::Promise::new(&mut |resolve, _reject| {
            if let Err(reason) =
                window.set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, millis)
            {
                log::warn!("the MIDI access wait has no timer: {reason:?}");
                let _ = resolve.call0(&wasm_bindgen::JsValue::UNDEFINED);
            }
        });
        let first = js_sys::Promise::race(&js_sys::Array::of2(&answered, &elapsed));
        let _ = wasm_bindgen_futures::JsFuture::from(first).await;
    }

    impl MidiBackend for NativeMidiBackend {
        fn destinations(&mut self) -> Result<Vec<MidiDestination>, MidiError> {
            self.0.destinations()
        }

        fn connect(
            &mut self,
            destination_id: &MidiDestinationId,
        ) -> Result<orcvs::midi::MidiConnectionRequest, MidiError> {
            self.0.connect(destination_id)
        }
    }
}

#[cfg(not(any(
    target_arch = "wasm32",
    target_os = "macos",
    target_os = "windows",
    target_os = "linux"
)))]
mod backend {
    pub use super::silent::SilentMidiBackend as NativeMidiBackend;

    pub const AVAILABLE: bool = false;
}

///
/// The backend a target with no MIDI service builds: it finds no destination
/// and refuses every connect.
///
/// Compiled for tests on every target as well, so the silent fallback is
/// proved on the hosts the suite runs on rather than only on targets it does
/// not.
///
#[cfg(any(
    test,
    not(any(
        target_arch = "wasm32",
        target_os = "macos",
        target_os = "windows",
        target_os = "linux"
    ))
))]
mod silent {
    use orcvs::midi::{MidiBackend, MidiDestination, MidiDestinationId, MidiError};

    pub struct SilentMidiBackend;

    impl Default for SilentMidiBackend {
        fn default() -> Self {
            Self::new()
        }
    }

    impl SilentMidiBackend {
        pub fn new() -> Self {
            Self
        }
    }

    impl MidiBackend for SilentMidiBackend {
        fn destinations(&mut self) -> Result<Vec<MidiDestination>, MidiError> {
            Ok(Vec::new())
        }

        fn connect(
            &mut self,
            _destination_id: &MidiDestinationId,
        ) -> Result<orcvs::midi::MidiConnectionRequest, MidiError> {
            Err(MidiError::new("this build has no MIDI backend"))
        }
    }

    #[cfg(test)]
    mod tests {
        use orcvs::midi::{MidiBackend, MidiDestinationId};

        use super::SilentMidiBackend;

        #[test]
        fn a_target_without_a_midi_service_offers_an_empty_list_rather_than_an_error() {
            let mut backend = SilentMidiBackend::new();

            assert_eq!(backend.destinations(), Ok(Vec::new()));
            assert!(backend.connect(&MidiDestinationId::new("any")).is_err());
        }
    }
}
