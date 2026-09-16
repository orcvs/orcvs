use orcvs::midi::{MidiBackend, MidiDestination, MidiDestinationId, MidiSelectionHandle};
use orcvs::playback::PlaybackDiagnostic;

use crate::diagnostics::failure_message;

const UNAVAILABLE: &str = "running Orcvs is no longer available";

///
/// Output readout copy when nothing is selected or the last discovery returned
/// no destinations.
///
pub(crate) const OUTPUT_NONE: &str = "None";

///
/// The text the destination ComboBox shows for the current selection.
///
/// An empty discovery is the empty copy, not a device name and not "not
/// found". A kept selection missing from a later non-empty list is still a
/// selection: the ComboBox shows that id rather than the empty copy.
///
pub(crate) fn destination_selected_text<'a>(
    destinations: &'a [MidiDestination],
    selected_id: Option<&'a MidiDestinationId>,
) -> &'a str {
    if destinations.is_empty() {
        return OUTPUT_NONE;
    }
    match selected_id {
        Some(id) => destinations
            .iter()
            .find(|destination| &destination.id == id)
            .map(|destination| destination.name.as_str())
            .unwrap_or_else(|| id.as_str()),
        None => OUTPUT_NONE,
    }
}

///
/// How the Panel presents destination selection for this build.
///
/// When the backend is missing, the ComboBox is still drawn — disabled, with
/// the empty copy — and Refresh is not. The build flag is read here rather
/// than passed in: a parameter only tests would flip is a seam
/// `AGENTS.md` forbids.
///
pub(crate) struct DestinationPresentation<'a> {
    pub enabled: bool,
    pub show_refresh: bool,
    pub selected_text: &'a str,
}

pub(crate) fn destination_presentation<'a>(
    destinations: &'a [MidiDestination],
    selected_id: Option<&'a MidiDestinationId>,
) -> DestinationPresentation<'a> {
    if !crate::native_midi::AVAILABLE {
        return DestinationPresentation {
            enabled: false,
            show_refresh: false,
            selected_text: OUTPUT_NONE,
        };
    }
    DestinationPresentation {
        enabled: true,
        show_refresh: true,
        selected_text: destination_selected_text(destinations, selected_id),
    }
}

///
/// The presentation a test builds for a backend availability production
/// cannot construct on this target. Production goes through
/// [`destination_presentation`], which reads `native_midi::AVAILABLE`.
///
#[cfg(test)]
fn destination_presentation_for<'a>(
    available: bool,
    destinations: &'a [MidiDestination],
    selected_id: Option<&'a MidiDestinationId>,
) -> DestinationPresentation<'a> {
    if !available {
        return DestinationPresentation {
            enabled: false,
            show_refresh: false,
            selected_text: OUTPUT_NONE,
        };
    }
    DestinationPresentation {
        enabled: true,
        show_refresh: true,
        selected_text: destination_selected_text(destinations, selected_id),
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
enum SelectionOrigin {
    Auto,
    User,
}

pub(crate) struct MidiDeviceSelection {
    selection: MidiSelectionHandle,
    backend: Box<dyn MidiBackend>,
    destinations: Vec<MidiDestination>,
    status: Option<String>,
    engine_status: Option<String>,
    ///
    /// Whether automatic destination selection has already been tried without
    /// an explicit user action. A refused connect leaves the selection empty,
    /// so without this guard the Panel would ask again on every frame and
    /// erase the refusal just received.
    ///
    auto_select_attempted: bool,
}

impl MidiDeviceSelection {
    pub(crate) fn new(selection: MidiSelectionHandle, backend: Box<dyn MidiBackend>) -> Self {
        Self {
            selection,
            backend,
            destinations: Vec::new(),
            status: None,
            engine_status: None,
            auto_select_attempted: false,
        }
    }

    ///
    /// Looks for destinations on the console thread and updates the menu list.
    ///
    pub(crate) fn refresh_destinations(&mut self) {
        self.auto_select_attempted = false;
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
        self.select_destination_with_origin(destination_id, SelectionOrigin::User);
    }

    fn select_destination_with_origin(
        &mut self,
        destination_id: &MidiDestinationId,
        origin: SelectionOrigin,
    ) {
        if origin == SelectionOrigin::User {
            self.auto_select_attempted = true;
        }
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

    ///
    /// Selects the first discovered destination when nothing is selected yet.
    ///
    /// A later read that already has a selection leaves it, even if that id is
    /// gone from the new list — Refresh must not steal a choice the user has
    /// made.
    ///
    pub(crate) fn auto_select_first_if_unselected(&mut self) {
        if self.auto_select_attempted || self.selected_destination_id().is_some() {
            return;
        }
        let first_id = self
            .destinations()
            .first()
            .map(|destination| destination.id.clone());
        if let Some(id) = first_id {
            self.auto_select_attempted = true;
            self.select_destination_with_origin(&id, SelectionOrigin::Auto);
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

    use super::{MidiDeviceSelection, destination_presentation_for, destination_selected_text};

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

    /// Yield until `cond` is true, or panic after a bounded number of turns.
    macro_rules! settle_until {
        ($cond:expr) => {{
            let mut spins = 0;
            while !($cond) {
                assert!(
                    spins < 32,
                    "condition did not settle: {}",
                    stringify!($cond)
                );
                tokio::task::yield_now().await;
                spins += 1;
            }
        }};
    }

    #[tokio::test]
    async fn the_console_selection_comes_from_a_default_running_orcvs() {
        let orcvs = Orcvs::new(1, 1).expect("the test runtime");

        let mut midi =
            MidiDeviceSelection::new(orcvs.midi_selection_handle(), Box::new(FakeBackend));

        assert_eq!(midi.selected_destination_id(), None);
        assert_eq!(midi.status(), None);
    }

    ///
    /// A backend whose destination list a test can replace between refreshes,
    /// so auto-select can be shown not to steal a choice the user already made.
    ///
    #[derive(Clone)]
    struct ConfigurableBackend {
        destinations: std::sync::Arc<std::sync::Mutex<Vec<MidiDestination>>>,
    }

    impl ConfigurableBackend {
        fn new(destinations: Vec<MidiDestination>) -> Self {
            Self {
                destinations: std::sync::Arc::new(std::sync::Mutex::new(destinations)),
            }
        }

        fn set(&self, destinations: Vec<MidiDestination>) {
            *self
                .destinations
                .lock()
                .expect("the test still holds the destination list") = destinations;
        }
    }

    impl MidiBackend for ConfigurableBackend {
        fn destinations(&mut self) -> Result<Vec<MidiDestination>, MidiError> {
            Ok(self
                .destinations
                .lock()
                .expect("the test still holds the destination list")
                .clone())
        }

        fn connect(
            &mut self,
            _destination_id: &MidiDestinationId,
        ) -> Result<Box<dyn MidiConnection>, MidiError> {
            Ok(Box::new(FakeConnection))
        }
    }

    ///
    /// Discovery that returns a non-empty list while nothing is selected
    /// selects the first destination.
    ///
    ///
    /// A refused automatic connect is reported once and not retried on every
    /// frame; Refresh is the explicit action that may ask again.
    ///
    #[tokio::test]
    async fn a_failed_automatic_selection_is_not_retried_until_refresh() {
        let (mut orcvs, mut midi) = selection_for(RefusingConnectBackend);
        midi.refresh_destinations();
        settle_until!(!midi.destinations().is_empty());

        midi.auto_select_first_if_unselected();
        settle_until!({
            midi.observe_diagnostics(orcvs.drain_playback_diagnostics());
            midi.status() == Some("device connection failed")
        });

        for _ in 0..5 {
            midi.auto_select_first_if_unselected();
            midi.observe_diagnostics(orcvs.drain_playback_diagnostics());
        }

        assert_eq!(midi.selected_destination_id(), None);
        assert_eq!(midi.status(), Some("device connection failed"));

        midi.refresh_destinations();
        settle_until!(!midi.destinations().is_empty());
        midi.auto_select_first_if_unselected();
        settle_until!({
            midi.observe_diagnostics(orcvs.drain_playback_diagnostics());
            midi.status() == Some("device connection failed")
        });
    }

    #[tokio::test]
    async fn an_empty_selection_takes_the_first_discovered_destination() {
        let (_orcvs, mut midi) = selection_for(FakeBackend);
        midi.refresh_destinations();
        settle_until!(!midi.destinations().is_empty());

        midi.auto_select_first_if_unselected();
        settle_until!(midi.selected_destination_id().is_some());

        assert_eq!(
            midi.selected_destination_id(),
            Some(MidiDestinationId::new("one"))
        );
    }

    ///
    /// A later Refresh does not steal a selection the user already made, even
    /// when the new list's first destination is a different one, and even when
    /// the chosen id is missing from that list.
    ///
    #[tokio::test]
    async fn a_later_refresh_does_not_steal_a_selection() {
        let backend = ConfigurableBackend::new(vec![
            MidiDestination::new("one", "First"),
            MidiDestination::new("two", "Second"),
        ]);
        let control = backend.clone();
        let (_orcvs, mut midi) = selection_for(backend);

        midi.refresh_destinations();
        settle_until!(midi.destinations().len() == 2);
        midi.auto_select_first_if_unselected();
        settle_until!(midi.selected_destination_id() == Some(MidiDestinationId::new("one")));

        midi.select_destination(&MidiDestinationId::new("two"));
        settle_until!(midi.selected_destination_id() == Some(MidiDestinationId::new("two")));

        control.set(vec![
            MidiDestination::new("three", "Third"),
            MidiDestination::new("two", "Second"),
        ]);
        midi.refresh_destinations();
        settle_until!(
            midi.destinations()
                .first()
                .map(|destination| destination.id.clone())
                == Some(MidiDestinationId::new("three"))
        );
        midi.auto_select_first_if_unselected();
        assert_eq!(
            midi.selected_destination_id(),
            Some(MidiDestinationId::new("two")),
            "a different first destination stole the user's choice"
        );

        control.set(vec![
            MidiDestination::new("three", "Third"),
            MidiDestination::new("four", "Fourth"),
        ]);
        midi.refresh_destinations();
        settle_until!(
            midi.destinations()
                == [
                    MidiDestination::new("three", "Third"),
                    MidiDestination::new("four", "Fourth")
                ]
        );
        midi.auto_select_first_if_unselected();
        assert_eq!(
            midi.selected_destination_id(),
            Some(MidiDestinationId::new("two")),
            "a missing id was replaced by the new first destination"
        );
    }

    ///
    /// Empty discovery leaves the selection empty: there is no phantom first
    /// destination to choose.
    ///
    #[tokio::test]
    async fn empty_discovery_does_not_select_a_phantom_first() {
        let (_orcvs, mut midi) = selection_for(ConfigurableBackend::new(Vec::new()));
        midi.refresh_destinations();
        midi.auto_select_first_if_unselected();

        assert_eq!(midi.selected_destination_id(), None);
        assert_eq!(midi.destinations(), &[]);
        assert_eq!(
            destination_selected_text(&[], midi.selected_destination_id().as_ref()),
            super::OUTPUT_NONE
        );
    }

    ///
    /// When the last discovery returned no destinations, the Output readout is
    /// exactly "None" — not "not found" and not "No MIDI destinations found."
    ///
    #[test]
    fn an_empty_discovery_presents_none() {
        assert_eq!(destination_selected_text(&[], None), super::OUTPUT_NONE);
        assert_eq!(
            destination_selected_text(&[], Some(&MidiDestinationId::new("one"))),
            super::OUTPUT_NONE
        );
    }

    ///
    /// A kept selection whose id is missing from a later non-empty list is
    /// still a selection: the empty copy is only for an empty discovery, so
    /// the ComboBox shows the id rather than pretending nothing is chosen.
    ///
    #[test]
    fn a_kept_selection_missing_from_the_list_is_not_the_empty_copy() {
        let destinations = [MidiDestination::new("three", "Third")];
        assert_eq!(
            destination_selected_text(&destinations, Some(&MidiDestinationId::new("two"))),
            "two"
        );
    }

    ///
    /// A build with no native MIDI backend still draws the destination
    /// Readout, disabled, with the empty copy, and does not offer Refresh.
    ///
    #[test]
    fn an_unavailable_backend_disables_the_destination_and_hides_refresh() {
        let destinations = [MidiDestination::new("one", "Studio Synth")];
        let selected = MidiDestinationId::new("one");
        let presentation = destination_presentation_for(false, &destinations, Some(&selected));
        assert!(!presentation.enabled);
        assert!(!presentation.show_refresh);
        assert_eq!(presentation.selected_text, super::OUTPUT_NONE);
    }

    ///
    /// A build that has a backend keeps the Output readout enabled and offers
    /// Refresh in its menu, even when the last discovery was empty.
    ///
    #[test]
    fn an_available_backend_keeps_refresh_and_the_destination_enabled() {
        let presentation = destination_presentation_for(true, &[], None);
        assert!(presentation.enabled);
        assert!(presentation.show_refresh);
        assert_eq!(presentation.selected_text, super::OUTPUT_NONE);
    }

    ///
    /// Drawing the menu does not overwrite the failure the engine just
    /// reported.
    ///
    /// The published discovery outcome persists until a later discovery
    /// replaces it, and the menu reads it on every frame it is open — twice
    /// per frame, in fact. A read that writes the status line therefore
    /// restates a discovery error the user has already seen, in the same frame
    /// that an output failure was put there, and in every frame after it. The
    /// status line belongs to whatever happened last, not to whatever was read
    /// last.
    ///
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

    #[derive(Clone)]
    struct OrderedConnectBackend {
        connect_order: std::sync::Arc<std::sync::Mutex<Vec<MidiDestinationId>>>,
    }

    impl OrderedConnectBackend {
        fn new() -> Self {
            Self {
                connect_order: std::sync::Arc::new(std::sync::Mutex::new(Vec::new())),
            }
        }

        fn connect_order(&self) -> Vec<MidiDestinationId> {
            self.connect_order
                .lock()
                .expect("the test still holds the connect log")
                .clone()
        }
    }

    impl MidiBackend for OrderedConnectBackend {
        fn destinations(&mut self) -> Result<Vec<MidiDestination>, MidiError> {
            Ok(vec![
                MidiDestination::new("one", "First"),
                MidiDestination::new("two", "Second"),
            ])
        }

        fn connect(
            &mut self,
            destination_id: &MidiDestinationId,
        ) -> Result<Box<dyn MidiConnection>, MidiError> {
            self.connect_order
                .lock()
                .expect("the test still holds the connect log")
                .push(destination_id.clone());
            Ok(Box::new(FakeConnection))
        }
    }

    ///
    /// An explicit user choice is not followed by an automatic attempt on the
    /// first destination while that choice is still pending.
    ///
    #[tokio::test]
    async fn an_explicit_selection_is_not_followed_by_automatic_selection() {
        let backend = OrderedConnectBackend::new();
        let (_orcvs, mut midi) = selection_for(backend.clone());

        midi.refresh_destinations();
        settle_until!(midi.destinations().len() == 2);

        midi.select_destination(&MidiDestinationId::new("two"));
        for _ in 0..5 {
            midi.auto_select_first_if_unselected();
            tokio::task::yield_now().await;
        }
        settle_until!(midi.selected_destination_id() == Some(MidiDestinationId::new("two")));

        assert_eq!(
            backend.connect_order(),
            vec![MidiDestinationId::new("two")],
            "automatic selection queued a connect to the first destination"
        );
    }

    struct RefusingConnectBackend;

    impl MidiBackend for RefusingConnectBackend {
        fn destinations(&mut self) -> Result<Vec<MidiDestination>, MidiError> {
            Ok(vec![MidiDestination::new("one", "Studio Synth")])
        }

        fn connect(
            &mut self,
            _destination_id: &MidiDestinationId,
        ) -> Result<Box<dyn MidiConnection>, MidiError> {
            Err(MidiError::new("device connection failed"))
        }
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
