//!
//! Whether this build has a native MIDI backend reachable from the Orcvs crate.
//!
//! Platform discovery and port opening live in the console; the toolkit-free crate
//! exposes only the silent fallback so a running Orcvs still composes with a MIDI
//! output adapter that accepts submissions and delivers nothing when no port was
//! installed.
//!
//! [`MidiBackend`]: crate::midi::MidiBackend

use crate::midi::{MidiBackend, MidiConnection, MidiDestination, MidiDestinationId, MidiError};

///
/// Whether a platform MIDI service is reachable through this crate.
///
/// Always `false` here: callers that present device selection use the console's
/// own `native_midi` module instead.
///
pub const AVAILABLE: bool = false;

///
/// The fallback backend a build with no native MIDI backend in this crate has.
///
#[derive(Default)]
pub struct SilentMidiBackend;

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
        let mut adapter = MidiOutputAdapter::new();

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
