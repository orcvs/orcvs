use orcvs::midi::{MidiBackend, MidiDestination, MidiDestinationId};
use orcvs::playback::{MidiSelectionHandle, PlaybackDiagnostic};

use crate::diagnostics::failure_message;

pub(crate) struct MidiDeviceSelection<B: MidiBackend> {
    selection: MidiSelectionHandle<B>,
    destinations: Vec<MidiDestination>,
    status: Option<String>,
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
        }
    }

    ///
    /// Asks the running Orcvs to look for destinations again.
    ///
    /// Discovery reaches a platform MIDI service through the Playback Engine's
    /// adapter, which per ADR 0040 that engine's task owns, so the answer is
    /// published rather than returned: what this reports here is only whether
    /// there was still a running Orcvs to ask. The list arrives through
    /// `destinations`, which every frame that draws the menu reads.
    ///
    pub(crate) fn refresh_destinations(&mut self) {
        if let Err(error) = self.selection.refresh_destinations() {
            self.status = Some(error.message);
        }
    }

    ///
    /// The destinations the running Orcvs last published.
    ///
    /// Read every frame the menu is drawn, so a refresh asked for on one frame
    /// appears on the frame after the engine answered it, with nothing for the
    /// user to click twice.
    ///
    pub(crate) fn destinations(&mut self) -> &[MidiDestination] {
        match self.selection.destinations() {
            Ok(destinations) => {
                self.destinations = destinations;
                self.reported_discovery_failure = None;
            }
            Err(error) => {
                // Read every frame the menu is open, and twice per frame at
                // that, against a value that persists until the next discovery
                // answers. Reporting on each read would restate one failure
                // over whatever the engine reported since.
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
    use orcvs::app::Orcvs;
    use orcvs::midi::{
        MidiBackend, MidiConnection, MidiDestination, MidiDestinationId, MidiError,
        MidiOutputAdapter,
    };
    use orcvs::native_midi::NativeMidiBackend;
    use orcvs::playback::{OutputAdapterError, PlaybackDiagnostic};

    use super::MidiDeviceSelection;

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
    /// two routes ADR 0040 leaves open to an engine that cannot answer a
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
    /// Space produced `RuntimeUnavailable`. ADR 0040 moved that failure to
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
