//!
//! Whether this build has a native MIDI backend, and what a running Orcvs
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
//! The question has two halves, and both are spelled on the same `cfg`. The
//! `native-midi` feature — declared in `orcvs/Cargo.toml`, on by default —
//! says whether this build wants a native backend at all; the target condition
//! says whether the target carries a MIDI service one could reach. `midir` is
//! optional and lives in a target-specific dependency table, so a build that
//! answers no to either half does not have the crate to call: turning the
//! feature off removes it from the tree, and a WASM build never sees it
//! whichever way the feature is set. The manifest and this file therefore state
//! one condition between them rather than two conditions to keep in sync.
//!
//! Both answers are the same shape, which is what makes the seam a seam. A
//! build with no native MIDI backend still has a [`MidiBackend`] — one that
//! offers no destination and refuses to connect — so a running Orcvs there is
//! still a running Orcvs over a MIDI output adapter, and an adapter holding no
//! connection accepts every submission and delivers nothing. Playing there is
//! silent rather than special-cased.
//!
//! [`MidiBackend`]: crate::midi::MidiBackend

use crate::midi::MidiOutputAdapter;

pub use backend::NativeMidiBackend;

///
/// Whether [`NativeMidiBackend`] reaches a platform MIDI service in this
/// build.
///
/// It is a fact about the build, not about the machine: a build that has a
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

#[cfg(all(
    feature = "native-midi",
    any(target_os = "macos", target_os = "windows", target_os = "linux")
))]
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

#[cfg(not(all(
    feature = "native-midi",
    any(target_os = "macos", target_os = "windows", target_os = "linux")
)))]
mod backend {
    pub use super::silent::SilentMidiBackend as NativeMidiBackend;

    pub const AVAILABLE: bool = false;
}

///
/// The fallback backend, and the tests that hold it to what the seam promises.
///
/// It is `backend` wherever there is no native MIDI backend: a target with no
/// platform MIDI service, or `native-midi` turned off. It is also compiled
/// under `cfg(test)` everywhere else, which is what makes its behaviour
/// verifiable: gated on the build alone it would run in one pass of the
/// pull-request tier and in none of the default-featured runs, and before that
/// pass existed it ran nowhere at all — the native runs compiled it out and the
/// browser suite runs only the console's own integration test target — so a
/// fallback that regressed to a panic or a wrong answer would type-check and
/// ship. Compiled here it is exercised by the ordinary `cargo nextest run` pass
/// on every target, and its role as `backend` is a re-export rather than a
/// second implementation, so the code under test is the code that ships.
///
#[cfg(any(
    test,
    not(all(
        feature = "native-midi",
        any(target_os = "macos", target_os = "windows", target_os = "linux")
    ))
))]
mod silent {
    use crate::midi::{MidiBackend, MidiConnection, MidiDestination, MidiDestinationId, MidiError};

    ///
    /// The backend a build with no native MIDI backend has.
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
            Err(MidiError::new("this build has no native MIDI backend"))
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
                Some("this build has no native MIDI backend".to_owned())
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

///
/// Half of what turning `native-midi` off gives up, and the half a `const` can
/// state: no target reaches a platform MIDI service.
///
/// It is not a fourth spelling of the condition. The target half can only ever
/// add a no, so with the feature off there is nothing left for [`AVAILABLE`] to
/// be true about, wherever the build is going. A `const` claim about the build
/// belongs in the build rather than in a test run, so this holds in every
/// feature-off compilation — the pull-request tier's `--no-default-features`
/// pass, and the browser build, whose `orcvs` dependency asks for no native
/// backend either.
///
#[cfg(not(feature = "native-midi"))]
const _: () = assert!(
    !AVAILABLE,
    "with native-midi disabled no target has a native MIDI backend"
);

///
/// The other half: a running Orcvs still has an output adapter, and it accepts
/// everything and delivers nothing.
///
/// This one is behaviour rather than a constant, so it runs. The pass that runs
/// it is the tier's `cargo nextest run --package orcvs --no-default-features` —
/// a test compiled by no tier is what `.scratch/native-midi/issues/01` already
/// had to repair once.
///
#[cfg(all(test, not(feature = "native-midi")))]
mod feature_disabled_tests {
    use super::output_adapter;
    use crate::midi::MidiDestinationId;
    use crate::playback::{OutputAdapter, OutputCommand};
    use crate::source::{MidiChannel, Note, Velocity};

    ///
    /// What the sibling tests in `silent` cannot say: the adapter a running
    /// Orcvs is handed is that backend, not merely one that could be built from
    /// it. `silent::tests` constructs `MidiOutputAdapter::new(SilentMidiBackend)`
    /// itself, so it holds the backend to its contract and says nothing about
    /// which backend `output_adapter()` reaches for. Asserting only that submit
    /// and safety reset return `Ok` here restated the sibling and left the
    /// wiring untested: a feature-off `backend` re-exporting something that
    /// forwarded commands and returned `Ok` passed both.
    ///
    /// So this asserts the pairing by its observable consequences — nowhere to
    /// send, and no way to obtain a connection — before asserting that playing
    /// anyway is accepted and silent.
    ///
    #[test]
    fn a_running_orcvs_has_an_adapter_over_a_backend_with_nowhere_to_deliver() {
        let mut adapter = output_adapter();

        assert_eq!(adapter.destinations(), Ok(Vec::new()));
        assert_eq!(
            adapter
                .select(&MidiDestinationId::new("invented"))
                .err()
                .map(|error| error.message),
            Some("this build has no native MIDI backend".to_owned())
        );
        assert_eq!(adapter.selected_destination_id(), None);

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
