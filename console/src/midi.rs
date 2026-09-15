use orcvs::midi::{MidiBackend, MidiDestination, MidiDestinationId};
use orcvs::playback::{MidiSelectionHandle, PlaybackDiagnostic};

use crate::diagnostics::failure_message;

///
/// ComboBox copy when the last discovery returned no destinations.
///
pub(crate) const NO_OUTPUT_DESTINATION: &str = "No output destination";

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
        return NO_OUTPUT_DESTINATION;
    }
    match selected_id {
        Some(id) => destinations
            .iter()
            .find(|destination| &destination.id == id)
            .map(|destination| destination.name.as_str())
            .unwrap_or_else(|| id.as_str()),
        None => NO_OUTPUT_DESTINATION,
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
    if !orcvs::native_midi::AVAILABLE {
        return DestinationPresentation {
            enabled: false,
            show_refresh: false,
            selected_text: NO_OUTPUT_DESTINATION,
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
            selected_text: NO_OUTPUT_DESTINATION,
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

pub(crate) struct MidiDeviceSelection<B: MidiBackend> {
    selection: MidiSelectionHandle<B>,
    destinations: Vec<MidiDestination>,
    status: Option<String>,
    ///
    /// Whether automatic destination selection has already been tried without
    /// an explicit user action. A refused connect leaves the published
    /// selection empty, so without this guard the Panel would ask again on
    /// every frame and erase the refusal `observe_diagnostics` just received.
    ///
    auto_select_attempted: bool,
    ///
    /// The discovery failure this selection has already put on the status
    /// line.
    ///
    /// The engine publishes the outcome of the last discovery and it stays
    /// published until another one replaces it, so reading it is not news.
    /// This is what tells a repeated read of one failure from the arrival of
    /// another, which is the difference between reporting a device that cannot
    /// be found and overwriting whatever the user should be reading instead.
    ///
    reported_discovery_failure: Option<String>,
}

impl<B: MidiBackend + 'static> MidiDeviceSelection<B> {
    pub(crate) fn new(selection: MidiSelectionHandle<B>) -> Self {
        Self {
            selection,
            destinations: Vec::new(),
            status: None,
            reported_discovery_failure: None,
            auto_select_attempted: false,
        }
    }

    ///
    /// Asks the running Orcvs to look for destinations again.
    ///
    /// Discovery reaches a platform MIDI service through the Playback Engine's
    /// adapter, which per ADR 0041 that engine's task owns, so the answer is
    /// published rather than returned: what this reports here is only whether
    /// there was still a running Orcvs to ask. The list arrives through
    /// `destinations`, which every frame that draws the ComboBox reads.
    ///
    pub(crate) fn refresh_destinations(&mut self) {
        self.auto_select_attempted = false;
        if let Err(error) = self.selection.refresh_destinations() {
            self.status = Some(error.message);
        }
    }

    ///
    /// The destinations the running Orcvs last published.
    ///
    /// Read every frame the ComboBox is drawn, so a refresh asked for on one
    /// frame appears on the frame after the engine answered it, with nothing
    /// for the user to click twice.
    ///
    pub(crate) fn destinations(&mut self) -> &[MidiDestination] {
        match self.selection.destinations() {
            Ok(destinations) => {
                self.destinations = destinations;
                self.reported_discovery_failure = None;
            }
            Err(error) => {
                // Read every frame the ComboBox is drawn, and twice per frame
                // at that, against a value that persists until the next
                // discovery answers. Reporting on each read would restate one
                // failure over whatever the engine reported since.
                if self.reported_discovery_failure.as_deref() != Some(error.message.as_str()) {
                    self.status = Some(error.message.clone());
                    self.reported_discovery_failure = Some(error.message);
                }
            }
        }
        &self.destinations
    }

    ///
    /// Asks the running Orcvs to connect its output to `destination_id`.
    ///
    /// A device that refuses the connection is reported by the engine on its
    /// diagnostics stream, which `observe_diagnostics` puts in this status
    /// line; what is answered here is whether there was a running Orcvs to ask.
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
            self.auto_select_attempted = false;
        }
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

    ///
    /// Selects the first published destination when nothing is selected yet.
    ///
    /// Discovery answers through a `watch`, so the frame that first reads a
    /// non-empty list is the moment to choose. A later read that already has a
    /// selection leaves it, even if that id is gone from the new list — Refresh
    /// must not steal a choice the user has made.
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
    use orcvs::app::Orcvs;
    use orcvs::midi::{
        MidiBackend, MidiConnection, MidiDestination, MidiDestinationId, MidiError,
        MidiOutputAdapter,
    };
    use orcvs::native_midi::NativeMidiBackend;
    use orcvs::playback::{OutputAdapterError, PlaybackDiagnostic};

    use super::{MidiDeviceSelection, destination_presentation_for, destination_selected_text};

    ///
    /// Waits until the running Orcvs has answered, or gives up and says so.
    ///
    /// Discovery and selection are messages to the Playback Engine's task now,
    /// and what they found is published back, so a test reads the answer only
    /// once that task has had its turn. How many turns that takes is the
    /// runtime's business and not something a test should be asserting by
    /// proxy: a single `yield_now` happens to be enough today and says nothing
    /// about what the test is actually waiting for. This names the condition
    /// and waits for it, so a test that fails here fails because the answer
    /// never came rather than because it came late.
    ///
    macro_rules! settle_until {
        ($condition:expr) => {{
            let mut answered = false;
            for _ in 0..1_000 {
                if $condition {
                    answered = true;
                    break;
                }
                tokio::time::sleep(std::time::Duration::from_millis(1)).await;
            }
            assert!(answered, "the running Orcvs never answered");
        }};
    }

    ///
    /// What the console does, with nothing target-specific in it. The console
    /// takes a default running Orcvs and asks it for a selection over whatever
    /// backend `orcvs` decided this target has; it never names the operating
    /// systems that carry one. Naming `NativeMidiBackend` is the whole
    /// assertion — it compiles on a target with no native backend exactly as
    /// it does on one with a native backend, so the console has no flag to keep
    /// in sync. What follows states the selection a console opens with, which
    /// no device has been chosen for yet and so asks the backend nothing.
    ///
    #[tokio::test]
    async fn the_console_selection_comes_from_a_default_running_orcvs() {
        let orcvs = Orcvs::new(1, 1).expect("the test runtime");

        let mut midi: MidiDeviceSelection<NativeMidiBackend> =
            MidiDeviceSelection::new(orcvs.midi_selection_handle());

        assert_eq!(midi.selected_destination_id(), None);
        assert_eq!(midi.status(), None);
    }

    fn selection_for<B: MidiBackend + 'static>(
        backend: B,
    ) -> (Orcvs<MidiOutputAdapter<B>>, MidiDeviceSelection<B>) {
        let orcvs = Orcvs::with_output_adapter(1, 1, MidiOutputAdapter::new(backend))
            .expect("the test runtime");
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

    ///
    /// A backend that can find nothing, every time it is asked.
    ///
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
            "No output destination"
        );
    }

    ///
    /// When the last discovery returned no destinations, the ComboBox copy is
    /// exactly "No output destination" — not "not found" and not "No MIDI
    /// destinations found."
    ///
    #[test]
    fn an_empty_discovery_presents_no_output_destination() {
        assert_eq!(
            destination_selected_text(&[], None),
            "No output destination"
        );
        assert_eq!(
            destination_selected_text(&[], Some(&MidiDestinationId::new("one"))),
            "No output destination"
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
        assert_eq!(presentation.selected_text, "No output destination");
    }

    ///
    /// A build that has a backend keeps the ComboBox enabled and shows
    /// Refresh, even when the last discovery was empty.
    ///
    #[test]
    fn an_available_backend_keeps_refresh_and_the_destination_enabled() {
        let presentation = destination_presentation_for(true, &[], None);
        assert!(presentation.enabled);
        assert!(presentation.show_refresh);
        assert_eq!(presentation.selected_text, "No output destination");
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
        settle_until!({
            // Reading is what publishes the failure to the status line, so the
            // wait has to do the reading the menu would.
            let _destinations = midi.destinations();
            midi.status().is_some()
        });
        // The discovery error is shown once, on the frame it arrived.
        assert_eq!(midi.destinations(), &[]);
        assert_eq!(midi.status(), Some("device discovery failed"));

        midi.observe_diagnostics(vec![PlaybackDiagnostic::OutputFailure(
            OutputAdapterError::new("device lost"),
        )]);

        // What the menu body does on every frame it is drawn.
        let _ = midi.destinations();
        let _ = midi.destinations();

        assert_eq!(midi.status(), Some("device lost"));
    }

    #[tokio::test]
    async fn refresh_discovers_destinations() {
        let (_orcvs, mut midi) = selection_for(FakeBackend);

        midi.refresh_destinations();
        settle_until!(!midi.destinations().is_empty());

        assert_eq!(
            midi.destinations(),
            &[MidiDestination::new("one", "Studio Synth")]
        );
    }

    ///
    /// A refresh that has not been answered yet leaves the menu showing what it
    /// showed before, rather than emptying it.
    ///
    /// The console reads the published list every frame, so the answer appears
    /// on the frame after the engine found it and the user clicks nothing
    /// twice.
    ///
    #[tokio::test]
    async fn a_refresh_that_has_not_been_answered_keeps_the_menu_it_had() {
        let (_orcvs, mut midi) = selection_for(FakeBackend);
        midi.refresh_destinations();
        settle_until!(midi.destinations().len() == 1);
        assert_eq!(midi.destinations().len(), 1);

        midi.refresh_destinations();

        assert_eq!(
            midi.destinations(),
            &[MidiDestination::new("one", "Studio Synth")]
        );
    }

    #[tokio::test]
    async fn selecting_a_destination_reports_the_selected_identity() {
        let (_orcvs, mut midi) = selection_for(FakeBackend);

        midi.select_destination(&MidiDestinationId::new("one"));
        settle_until!(midi.selected_destination_id().is_some());

        assert_eq!(
            midi.selected_destination_id(),
            Some(MidiDestinationId::new("one"))
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

    ///
    /// Both ways a backend can refuse the console reach the status line, by the
    /// two routes ADR 0041 leaves open to an engine that cannot answer a
    /// caller: a discovery failure is published in place of the list, and a
    /// connection failure is reported on the diagnostics stream the console
    /// drains each frame.
    ///
    #[tokio::test]
    async fn backend_errors_are_exposed_as_status() {
        let (mut orcvs, mut midi) = selection_for(FailingBackend);

        midi.refresh_destinations();
        settle_until!({
            let _destinations = midi.destinations();
            midi.status() == Some("device discovery failed")
        });

        midi.select_destination(&MidiDestinationId::new("missing"));
        settle_until!({
            midi.observe_diagnostics(orcvs.drain_playback_diagnostics());
            midi.status() == Some("device connection failed")
        });

        assert_eq!(midi.status(), Some("device connection failed"));
    }

    ///
    /// A refused `start` reaches the status line.
    ///
    /// `console::midi::playback_start_errors_are_exposed_as_status` used to
    /// state this end to end, by building an `Orcvs` outside a runtime so that
    /// Space produced `RuntimeUnavailable`. ADR 0041 moved that failure to
    /// construction and made `Orcvs::new` fallible, so the path the old test
    /// walked no longer exists — and no console gesture left on this branch
    /// produces a `StartFailure` at all: `set_bpm` holds a `Bpm` that cannot be
    /// zero, which is ticket `08`'s subject.
    ///
    /// What remains worth pinning is this end of it. `StartFailure` is one of
    /// the three variants `failure_message` maps, the engine can put one on the
    /// stream from either of its two new refusals, and the panel is where a
    /// user would read it. Asserted over `observe_diagnostics` rather than over
    /// a staged refusal, because staging one would mean inventing a caller the
    /// console does not have.
    ///
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
    async fn refreshing_destinations_does_not_hide_an_active_output_failure() {
        let (_orcvs, mut midi) = selection_for(FakeBackend);
        midi.observe_diagnostics(vec![orcvs::playback::PlaybackDiagnostic::OutputFailure(
            orcvs::playback::OutputAdapterError::new("device lost"),
        )]);

        midi.refresh_destinations();
        settle_until!({
            let _destinations = midi.destinations();
            midi.status() == Some("device lost")
        });

        assert_eq!(midi.status(), Some("device lost"));
    }
}
