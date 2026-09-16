use orcvs::midi::{MidiBackend, MidiDestination, MidiDestinationId, MidiSelectionHandle};
use orcvs::playback::PlaybackDiagnostic;

use crate::diagnostics::failure_message;

const UNAVAILABLE: &str = "running Orcvs is no longer available";

pub(crate) struct MidiDeviceSelection {
    selection: MidiSelectionHandle,
    backend: Box<dyn MidiBackend>,
    destinations: Vec<MidiDestination>,
    status: Option<String>,
    engine_status: Option<String>,
}

impl MidiDeviceSelection {
    pub(crate) fn new(selection: MidiSelectionHandle, backend: Box<dyn MidiBackend>) -> Self {
        Self {
            selection,
            backend,
            destinations: Vec::new(),
            status: None,
            engine_status: None,
        }
    }

    ///
    /// Looks for destinations on the console thread and updates the menu list.
    ///
    pub(crate) fn refresh_destinations(&mut self) {
        match self.backend.destinations() {
            Ok(destinations) => {
                self.destinations = destinations;
                self.status = None;
            }
            Err(error) => {
                self.destinations = Vec::new();
                self.status = Some(error.message);
            }
        }
    }

    pub(crate) fn destinations(&self) -> &[MidiDestination] {
        &self.destinations
    }

    ///
    /// Opens the port on the console thread and queues the connection for Playback.
    ///
    pub(crate) fn select_destination(&mut self, destination_id: &MidiDestinationId) {
        let connection = match self.backend.connect(destination_id) {
            Ok(connection) => connection,
            Err(error) => {
                self.status = Some(error.message);
                return;
            }
        };
        match self.selection.install(destination_id.clone(), connection) {
            Ok(()) => {
                self.status = None;
                self.engine_status = None;
            }
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
        if self.status.as_deref() == Some(UNAVAILABLE) {
            return Some(UNAVAILABLE);
        }
        self.status.as_deref().or(self.engine_status.as_deref())
    }

    pub(crate) fn observe_diagnostics(&mut self, diagnostics: Vec<PlaybackDiagnostic>) {
        for diagnostic in diagnostics {
            if let Some(message) = failure_message(&diagnostic) {
                self.engine_status = Some(message);
                self.status = None;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use orcvs::app::Orcvs;
    use orcvs::midi::{
        MidiBackend, MidiConnection, MidiDestination, MidiDestinationId, MidiError,
        MidiOutputAdapter,
    };
    use orcvs::playback::{OutputAdapterError, PlaybackDiagnostic};

    use super::MidiDeviceSelection;

    fn selection_for(backend: impl MidiBackend + 'static) -> (Orcvs, MidiDeviceSelection) {
        let orcvs = Orcvs::with_midi_output_adapter(1, 1, MidiOutputAdapter::new())
            .expect("the test runtime");
        let midi = MidiDeviceSelection::new(
            orcvs.midi_selection_handle(),
            Box::new(backend) as Box<dyn MidiBackend>,
        );
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

    struct FailingDiscoveryBackend;

    impl MidiBackend for FailingDiscoveryBackend {
        fn destinations(&mut self) -> Result<Vec<MidiDestination>, MidiError> {
            Err(MidiError::new("device discovery failed"))
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

    #[tokio::test]
    async fn the_console_selection_comes_from_a_default_running_orcvs() {
        let orcvs = Orcvs::new(1, 1).expect("the test runtime");

        let mut midi =
            MidiDeviceSelection::new(orcvs.midi_selection_handle(), Box::new(FakeBackend));

        assert_eq!(midi.selected_destination_id(), None);
        assert_eq!(midi.status(), None);
    }

    #[tokio::test]
    async fn drawing_the_menu_does_not_hide_an_output_failure_behind_a_stale_discovery_error() {
        let (_orcvs, mut midi) = selection_for(FailingDiscoveryBackend);
        midi.refresh_destinations();
        assert_eq!(midi.destinations(), &[] as &[MidiDestination]);
        assert_eq!(midi.status(), Some("device discovery failed"));

        midi.observe_diagnostics(vec![PlaybackDiagnostic::OutputFailure(
            OutputAdapterError::new("device lost"),
        )]);

        let _ = midi.destinations();
        let _ = midi.destinations();

        assert_eq!(midi.status(), Some("device lost"));
    }

    #[tokio::test]
    async fn refresh_discovers_destinations() {
        let (_orcvs, mut midi) = selection_for(FakeBackend);

        midi.refresh_destinations();

        assert_eq!(
            midi.destinations(),
            &[MidiDestination::new("one", "Studio Synth")]
        );
    }

    struct ChangingBackend {
        alternate: std::sync::Arc<std::sync::atomic::AtomicBool>,
    }

    impl MidiBackend for ChangingBackend {
        fn destinations(&mut self) -> Result<Vec<MidiDestination>, MidiError> {
            if self.alternate.load(std::sync::atomic::Ordering::SeqCst) {
                Ok(vec![MidiDestination::new("two", "Second Synth")])
            } else {
                Ok(vec![MidiDestination::new("one", "Studio Synth")])
            }
        }

        fn connect(
            &mut self,
            _destination_id: &MidiDestinationId,
        ) -> Result<Box<dyn MidiConnection>, MidiError> {
            Ok(Box::new(FakeConnection))
        }
    }

    #[tokio::test]
    async fn refresh_picks_up_changed_destinations() {
        let alternate = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let backend = ChangingBackend {
            alternate: alternate.clone(),
        };
        let (_orcvs, mut midi) = selection_for(backend);

        midi.refresh_destinations();
        assert_eq!(
            midi.destinations(),
            &[MidiDestination::new("one", "Studio Synth")]
        );

        alternate.store(true, std::sync::atomic::Ordering::SeqCst);
        midi.refresh_destinations();

        assert_eq!(
            midi.destinations(),
            &[MidiDestination::new("two", "Second Synth")]
        );
    }

    #[tokio::test]
    async fn selecting_a_destination_reports_the_selected_identity() {
        let (_orcvs, mut midi) = selection_for(FakeBackend);

        midi.select_destination(&MidiDestinationId::new("one"));
        tokio::task::yield_now().await;

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

    #[tokio::test]
    async fn backend_errors_are_exposed_as_status() {
        let (mut orcvs, mut midi) = selection_for(FailingBackend);

        midi.refresh_destinations();
        assert_eq!(midi.status(), Some("device discovery failed"));

        midi.select_destination(&MidiDestinationId::new("missing"));
        assert_eq!(midi.status(), Some("device connection failed"));

        tokio::task::yield_now().await;
        midi.observe_diagnostics(orcvs.drain_playback_diagnostics());
        assert_eq!(midi.status(), Some("device connection failed"));
    }

    #[tokio::test]
    async fn a_refused_start_reaches_the_status_line() {
        let (_orcvs, mut midi) = selection_for(FakeBackend);

        midi.observe_diagnostics(vec![PlaybackDiagnostic::StartFailure {
            message: "Tick period is too long to schedule a deadline for".to_owned(),
        }]);

        assert_eq!(
            midi.status(),
            Some("Tick period is too long to schedule a deadline for")
        );
    }

    #[tokio::test]
    async fn unavailability_replaces_a_stale_refresh_error() {
        let calls = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let backend = RecoveringDiscoveryBackend {
            calls: calls.clone(),
        };
        let (orcvs, mut midi) = selection_for(backend);

        midi.refresh_destinations();
        assert_eq!(midi.status(), Some("device discovery failed"));

        midi.refresh_destinations();
        assert_eq!(midi.destinations().len(), 1);

        drop(orcvs);

        let _ = midi.selected_destination_id();
        assert_eq!(midi.status(), Some("running Orcvs is no longer available"));
    }

    struct RecoveringDiscoveryBackend {
        calls: std::sync::Arc<std::sync::atomic::AtomicUsize>,
    }

    impl MidiBackend for RecoveringDiscoveryBackend {
        fn destinations(&mut self) -> Result<Vec<MidiDestination>, MidiError> {
            if self.calls.fetch_add(1, std::sync::atomic::Ordering::SeqCst) == 0 {
                Err(MidiError::new("device discovery failed"))
            } else {
                Ok(vec![MidiDestination::new("one", "Studio Synth")])
            }
        }

        fn connect(
            &mut self,
            _destination_id: &MidiDestinationId,
        ) -> Result<Box<dyn MidiConnection>, MidiError> {
            Ok(Box::new(FakeConnection))
        }
    }

    #[tokio::test]
    async fn unavailability_replaces_a_stale_engine_diagnostic() {
        let (orcvs, mut midi) = selection_for(FakeBackend);
        midi.observe_diagnostics(vec![PlaybackDiagnostic::OutputFailure(
            OutputAdapterError::new("device lost"),
        )]);

        drop(orcvs);

        let _ = midi.selected_destination_id();

        assert_eq!(midi.status(), Some("running Orcvs is no longer available"));
    }

    #[tokio::test]
    async fn a_successful_selection_clears_a_stale_engine_diagnostic() {
        let (_orcvs, mut midi) = selection_for(FakeBackend);
        midi.observe_diagnostics(vec![PlaybackDiagnostic::OutputFailure(
            OutputAdapterError::new("device lost"),
        )]);
        assert_eq!(midi.status(), Some("device lost"));

        midi.select_destination(&MidiDestinationId::new("one"));
        tokio::task::yield_now().await;

        assert_eq!(midi.status(), None);
    }

    #[tokio::test]
    async fn a_connect_error_reaches_the_status_line_after_an_engine_failure() {
        let (_orcvs, mut midi) = selection_for(FailingBackend);
        midi.observe_diagnostics(vec![PlaybackDiagnostic::OutputFailure(
            OutputAdapterError::new("device lost"),
        )]);

        midi.select_destination(&MidiDestinationId::new("missing"));

        assert_eq!(midi.status(), Some("device connection failed"));
    }

    #[tokio::test]
    async fn refreshing_destinations_does_not_hide_an_active_output_failure() {
        let (_orcvs, mut midi) = selection_for(FakeBackend);
        midi.observe_diagnostics(vec![PlaybackDiagnostic::OutputFailure(
            OutputAdapterError::new("device lost"),
        )]);

        midi.refresh_destinations();

        assert_eq!(midi.status(), Some("device lost"));
    }

    #[tokio::test]
    async fn a_refused_port_opening_leaves_a_visible_message() {
        let (_orcvs, mut midi) = selection_for(FailingBackend);

        midi.select_destination(&MidiDestinationId::new("missing"));

        assert_eq!(midi.status(), Some("device connection failed"));
    }
}
