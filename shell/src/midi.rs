use orcvs::midi::{MidiBackend, MidiDestination, MidiDestinationId, MidiOutputAdapter};
use orcvs::playback::{PlaybackDiagnostic, PlaybackEngine};

use crate::diagnostics::failure_message;

pub(crate) struct MidiDeviceSelection<B: MidiBackend> {
    playback: PlaybackEngine<MidiOutputAdapter<B>>,
    destinations: Vec<MidiDestination>,
    status: Option<String>,
}

impl<B: MidiBackend> MidiDeviceSelection<B> {
    pub(crate) fn new(playback: PlaybackEngine<MidiOutputAdapter<B>>) -> Self {
        Self {
            playback,
            destinations: Vec::new(),
            status: None,
        }
    }

    pub(crate) fn refresh_destinations(&mut self) {
        match self.playback.midi_destinations() {
            Ok(destinations) => {
                self.destinations = destinations;
                self.status = None;
            }
            Err(error) => self.status = Some(error.message),
        }
    }

    pub(crate) fn destinations(&self) -> &[MidiDestination] {
        &self.destinations
    }

    pub(crate) fn select_destination(&mut self, destination_id: &MidiDestinationId) {
        match self.playback.select_midi_destination(destination_id) {
            Ok(()) => self.status = None,
            Err(error) => self.status = Some(error.message),
        }
    }

    pub(crate) fn selected_destination_id(&self) -> Option<MidiDestinationId> {
        self.playback.selected_midi_destination_id()
    }

    pub(crate) fn status(&self) -> Option<&str> {
        self.status.as_deref()
    }

    pub(crate) fn observe_diagnostics(&mut self, diagnostics: Vec<PlaybackDiagnostic>) {
        for diagnostic in diagnostics {
            if let Some(message) = failure_message(&diagnostic) {
                self.status = Some(message);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use orcvs::app::{InputEvent, InputKey, Orcvs};
    use orcvs::grid::Grid;
    use orcvs::midi::{
        MidiBackend, MidiConnection, MidiDestination, MidiDestinationId, MidiError,
        MidiOutputAdapter,
    };
    use orcvs::playback::PlaybackEngine;
    use orcvs::source::SourceCommander;

    use super::MidiDeviceSelection;

    fn selection_for<B: MidiBackend>(backend: B) -> MidiDeviceSelection<B> {
        let playback = PlaybackEngine::new(
            SourceCommander::new(Grid::new(1, 1)),
            MidiOutputAdapter::new(backend),
        );
        MidiDeviceSelection::new(playback)
    }

    struct FakeBackend;

    impl MidiBackend for FakeBackend {
        fn destinations(&mut self) -> Result<Vec<MidiDestination>, MidiError> {
            Ok(vec![MidiDestination::new("one", "Studio Synth")])
        }

        fn connect(
            &mut self,
            _destination_id: &MidiDestinationId,
        ) -> Result<Box<dyn MidiConnection>, MidiError> {
            Ok(Box::new(FakeConnection))
        }
    }

    struct FakeConnection;

    impl MidiConnection for FakeConnection {
        fn send(&mut self, _message: &[u8]) -> Result<(), MidiError> {
            Ok(())
        }
    }

    #[test]
    fn refresh_discovers_destinations() {
        let mut midi = selection_for(FakeBackend);

        midi.refresh_destinations();

        assert_eq!(
            midi.destinations(),
            &[MidiDestination::new("one", "Studio Synth")]
        );
    }

    #[test]
    fn selecting_a_destination_reports_the_selected_identity() {
        let mut midi = selection_for(FakeBackend);

        midi.select_destination(&MidiDestinationId::new("one"));

        assert_eq!(
            midi.selected_destination_id(),
            Some(MidiDestinationId::new("one"))
        );
    }

    struct FailingBackend;

    impl MidiBackend for FailingBackend {
        fn destinations(&mut self) -> Result<Vec<MidiDestination>, MidiError> {
            Err(MidiError::new("device discovery failed"))
        }

        fn connect(
            &mut self,
            _destination_id: &MidiDestinationId,
        ) -> Result<Box<dyn MidiConnection>, MidiError> {
            Err(MidiError::new("device connection failed"))
        }
    }

    #[test]
    fn backend_errors_are_exposed_as_status() {
        let mut midi = selection_for(FailingBackend);

        midi.refresh_destinations();
        assert_eq!(midi.status(), Some("device discovery failed"));

        midi.select_destination(&MidiDestinationId::new("missing"));
        assert_eq!(midi.status(), Some("device connection failed"));
    }

    #[test]
    fn playback_start_errors_are_exposed_as_status() {
        let mut orcvs = Orcvs::with_output_adapter(1, 1, MidiOutputAdapter::new(FakeBackend));
        let mut midi = MidiDeviceSelection::new(orcvs.playback_engine());

        orcvs.event_handler(vec![InputEvent::KeyPressed(InputKey::Space)]);
        midi.observe_diagnostics(orcvs.observe_playback());

        assert_eq!(midi.status(), Some("Playback requires a Tokio runtime"));
    }
}
