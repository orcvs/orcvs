use orcvs::midi::{MidiBackend, MidiDestination, MidiDestinationId};
use orcvs::playback::{MidiSelectionHandle, PlaybackDiagnostic};

use crate::diagnostics::failure_message;

pub(crate) struct MidiDeviceSelection<B: MidiBackend> {
    selection: MidiSelectionHandle<B>,
    destinations: Vec<MidiDestination>,
    status: Option<String>,
}

impl<B: MidiBackend> MidiDeviceSelection<B> {
    pub(crate) fn new(selection: MidiSelectionHandle<B>) -> Self {
        Self {
            selection,
            destinations: Vec::new(),
            status: None,
        }
    }

    pub(crate) fn refresh_destinations(&mut self) {
        match self.selection.destinations() {
            Ok(destinations) => {
                self.destinations = destinations;
            }
            Err(error) => self.status = Some(error.message),
        }
    }

    pub(crate) fn destinations(&self) -> &[MidiDestination] {
        &self.destinations
    }

    pub(crate) fn select_destination(&mut self, destination_id: &MidiDestinationId) {
        match self.selection.select(destination_id) {
            Ok(()) => self.status = None,
            Err(error) => self.status = Some(error.message),
        }
    }

    pub(crate) fn selected_destination_id(&mut self) -> Option<MidiDestinationId> {
        match self.selection.selected_destination_id() {
            Ok(destination_id) => destination_id,
            Err(error) => {
                self.status = Some(error.message);
                None
            }
        }
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
    use orcvs::midi::{
        MidiBackend, MidiConnection, MidiDestination, MidiDestinationId, MidiError,
        MidiOutputAdapter,
    };

    use super::MidiDeviceSelection;

    fn selection_for<B: MidiBackend + 'static>(
        backend: B,
    ) -> (Orcvs<MidiOutputAdapter<B>>, MidiDeviceSelection<B>) {
        let orcvs = Orcvs::with_output_adapter(1, 1, MidiOutputAdapter::new(backend));
        let midi = MidiDeviceSelection::new(orcvs.midi_selection_handle());
        (orcvs, midi)
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
        let (_orcvs, mut midi) = selection_for(FakeBackend);

        midi.refresh_destinations();

        assert_eq!(
            midi.destinations(),
            &[MidiDestination::new("one", "Studio Synth")]
        );
    }

    #[test]
    fn selecting_a_destination_reports_the_selected_identity() {
        let (_orcvs, mut midi) = selection_for(FakeBackend);

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
        let (_orcvs, mut midi) = selection_for(FailingBackend);

        midi.refresh_destinations();
        assert_eq!(midi.status(), Some("device discovery failed"));

        midi.select_destination(&MidiDestinationId::new("missing"));
        assert_eq!(midi.status(), Some("device connection failed"));
    }

    #[test]
    fn playback_start_errors_are_exposed_as_status() {
        let mut orcvs = Orcvs::with_output_adapter(1, 1, MidiOutputAdapter::new(FakeBackend));
        let mut midi = MidiDeviceSelection::new(orcvs.midi_selection_handle());

        orcvs.event_handler(vec![InputEvent::KeyPressed(InputKey::Space)]);
        midi.observe_diagnostics(orcvs.observe_playback());

        assert_eq!(midi.status(), Some("Playback requires a Tokio runtime"));
    }

    #[test]
    fn refreshing_destinations_does_not_hide_an_active_output_failure() {
        let (_orcvs, mut midi) = selection_for(FakeBackend);
        midi.observe_diagnostics(vec![orcvs::playback::PlaybackDiagnostic::OutputFailure(
            orcvs::playback::OutputAdapterError::new("device lost"),
        )]);

        midi.refresh_destinations();

        assert_eq!(midi.status(), Some("device lost"));
    }
}
