//! Native and browser MIDI discovery for the console.
//!
//! Device enumeration and port opening happen on the console's thread. Playback
//! receives an already-open connection through [`MidiSelectionHandle::install`].
//!
//! [`MidiSelectionHandle::install`]: orcvs::midi::MidiSelectionHandle::install

pub use backend::NativeMidiBackend;

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
        ) -> Result<Box<dyn MidiConnection>, MidiError> {
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
                Ok(connection) => {
                    Ok(Box::new(MidirConnection(connection)) as Box<dyn MidiConnection>)
                }
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

#[cfg(not(all(
    not(target_arch = "wasm32"),
    any(target_os = "macos", target_os = "windows", target_os = "linux")
)))]
mod backend {
    use orcvs::midi::{MidiBackend, MidiConnection, MidiDestination, MidiDestinationId, MidiError};

    ///
    /// A build with no platform MIDI service, the browser among them: it finds
    /// no destination and refuses every connect.
    ///
    pub struct NativeMidiBackend;

    pub const AVAILABLE: bool = false;

    impl Default for NativeMidiBackend {
        fn default() -> Self {
            Self::new()
        }
    }

    impl NativeMidiBackend {
        pub fn new() -> Self {
            Self
        }
    }

    impl MidiBackend for NativeMidiBackend {
        fn destinations(&mut self) -> Result<Vec<MidiDestination>, MidiError> {
            Ok(Vec::new())
        }

        fn connect(
            &mut self,
            _destination_id: &MidiDestinationId,
        ) -> Result<Box<dyn MidiConnection>, MidiError> {
            Err(MidiError::new("this build has no native MIDI backend"))
        }
    }
}
