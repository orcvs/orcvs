//!
//! Whether this target has a native MIDI backend, and what a running Orcvs
//! therefore uses for output.
//!
//! This module is the one place in Orcvs that asks the question. No other
//! module and no consuming crate names the operating systems that carry a MIDI
//! service, so nothing outside this file has a condition to keep in sync: a
//! consumer names [`NativeMidiBackend`], [`NativeMidiOutputAdapter`], or
//! [`AVAILABLE`] and gets an answer that is valid on every supported target,
//! the browser included. Inside this file the condition is spelled on each
//! `cfg` that selects between the two answers.
//!
//! One spelling survives outside Rust: `orcvs/Cargo.toml` names the same
//! operating systems over the `midir` dependency, because a target-specific
//! dependency table is the only way to tell Cargo which builds need the crate.
//! That table is the subject of the issue that replaces it with a
//! `native-midi` feature; until then it is the one restatement this module
//! cannot absorb.
//!
//! Both answers are the same shape, which is what makes the seam a seam. A
//! target with no native MIDI service still has a [`MidiBackend`] — one that
//! offers no destination and refuses to connect — so a running Orcvs there is
//! still a running Orcvs over a MIDI output adapter, and an adapter holding no
//! connection accepts every submission and delivers nothing. Playing on such a
//! target is silent rather than special-cased.
//!
//! [`MidiBackend`]: crate::midi::MidiBackend

use crate::midi::MidiOutputAdapter;

pub use backend::NativeMidiBackend;

///
/// Whether [`NativeMidiBackend`] reaches a platform MIDI service on this
/// target.
///
/// It is a fact about the build, not about the machine: a target that has a
/// native backend may still have no device plugged in, which is an empty
/// destination list rather than an unavailable backend. Callers that present
/// device selection use this to decide whether the selection exists at all.
///
pub const AVAILABLE: bool = backend::AVAILABLE;

///
/// The output adapter a running Orcvs uses unless it is handed another one.
///
pub type NativeMidiOutputAdapter = MidiOutputAdapter<NativeMidiBackend>;

///
/// One output adapter over this target's own MIDI backend.
///
/// [`NativeMidiBackend`] is nameable and constructible by anyone, so this is
/// not a privacy boundary; it is where the default pairing of adapter and
/// backend is chosen. A caller asks for the adapter it should use rather than
/// restating that choice, which is why [`crate::app::Orcvs::new`] does not
/// mention a backend at all.
///
pub fn output_adapter() -> NativeMidiOutputAdapter {
    NativeMidiOutputAdapter::new(NativeMidiBackend)
}

#[cfg(any(target_os = "macos", target_os = "windows", target_os = "linux"))]
mod backend {
    use midir::{MidiOutput, MidiOutputConnection};

    use crate::midi::{MidiBackend, MidiConnection, MidiDestination, MidiDestinationId, MidiError};

    pub const AVAILABLE: bool = true;

    const MIDI_CLIENT_NAME: &str = "Orcvs";

    ///
    /// The platform MIDI service, reached through `midir`.
    ///
    #[derive(Default)]
    pub struct NativeMidiBackend;

    impl MidiBackend for NativeMidiBackend {
        fn destinations(&mut self) -> Result<Vec<MidiDestination>, MidiError> {
            let output = MidiOutput::new(MIDI_CLIENT_NAME).map_err(midi_error)?;
            output
                .ports()
                .into_iter()
                .map(|port| {
                    let name = output.port_name(&port).map_err(midi_error)?;
                    Ok(MidiDestination::new(port.id(), name))
                })
                .collect()
        }

        fn connect(
            &mut self,
            destination_id: &MidiDestinationId,
        ) -> Result<Box<dyn MidiConnection>, MidiError> {
            let output = MidiOutput::new(MIDI_CLIENT_NAME).map_err(midi_error)?;
            let port = output
                .find_port_by_id(destination_id.as_str())
                .ok_or_else(|| {
                    MidiError::new("the selected MIDI destination is no longer available")
                })?;
            let port_name = format!("{MIDI_CLIENT_NAME} output");
            let connection = output.connect(&port, &port_name).map_err(midi_error)?;
            Ok(Box::new(MidirConnection(connection)))
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

#[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
mod backend {
    pub use super::silent::SilentMidiBackend as NativeMidiBackend;

    pub const AVAILABLE: bool = false;
}

///
/// The fallback backend, and the tests that hold it to what the seam promises.
///
/// It is `backend` on a target with no platform MIDI service. It is also
/// compiled under `cfg(test)` everywhere else, which is what makes its
/// behaviour verifiable: gated on the target alone it would be absent from
/// every test run CI executes — the native runs compile it out, and the browser
/// suite runs only the shell's own integration test target — so a fallback that
/// regressed to a panic or a wrong answer would type-check and ship. Compiled
/// here it is exercised by the ordinary `cargo nextest run` pass on every
/// target, and its role as `backend` is a re-export rather than a second
/// implementation, so the code under test is the code that ships.
///
#[cfg(any(
    test,
    not(any(target_os = "macos", target_os = "windows", target_os = "linux"))
))]
mod silent {
    use crate::midi::{MidiBackend, MidiConnection, MidiDestination, MidiDestinationId, MidiError};

    ///
    /// The backend a target with no platform MIDI service has.
    ///
    /// It answers the empty destination list rather than an error, because
    /// having nowhere to send MIDI is not a failure to discover destinations.
    /// Connecting is refused: no destination it offered can be named, so any
    /// identity handed to it came from somewhere else.
    ///
    #[derive(Default)]
    pub struct SilentMidiBackend;

    impl MidiBackend for SilentMidiBackend {
        fn destinations(&mut self) -> Result<Vec<MidiDestination>, MidiError> {
            Ok(Vec::new())
        }

        fn connect(
            &mut self,
            _destination_id: &MidiDestinationId,
        ) -> Result<Box<dyn MidiConnection>, MidiError> {
            Err(MidiError::new("this target has no native MIDI backend"))
        }
    }

    #[cfg(test)]
    mod tests {
        use super::SilentMidiBackend;
        use crate::midi::{MidiBackend, MidiDestinationId, MidiOutputAdapter};
        use crate::playback::{OutputAdapter, OutputCommand};
        use crate::source::{MidiChannel, Note, Velocity};

        #[test]
        fn offers_no_destination_without_reporting_a_discovery_failure() {
            let mut backend = SilentMidiBackend;

            assert_eq!(backend.destinations(), Ok(Vec::new()));
        }

        #[test]
        fn refuses_to_connect_to_a_destination_it_never_offered() {
            let mut backend = SilentMidiBackend;

            assert_eq!(
                backend
                    .connect(&MidiDestinationId::new("invented"))
                    .err()
                    .map(|error| error.message),
                Some("this target has no native MIDI backend".to_owned())
            );
        }

        #[test]
        fn a_default_output_adapter_accepts_playback_and_delivers_nothing() {
            let mut adapter = MidiOutputAdapter::new(SilentMidiBackend);

            assert!(
                adapter
                    .submit(&[OutputCommand::NoteOn {
                        channel: MidiChannel::try_from(0).unwrap(),
                        velocity: Velocity::try_from(0x7f).unwrap(),
                        note: Note::try_from(60).unwrap(),
                    }])
                    .is_ok()
            );
            assert!(adapter.safety_reset().is_ok());
        }
    }
}
