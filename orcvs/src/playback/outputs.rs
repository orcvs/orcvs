//!
//! Every destination a Tick is delivered to, and the policy each keeps
//! (ADR 0090).
//!
//! MIDI waits for a connection, owns the notes it started and takes the safety
//! action. The network does none of that: its sends go on while no MIDI
//! destination is connected, and stopping, rewinding and disconnecting send it
//! nothing. Within a Tick, MIDI is delivered before the Network Sends.
//!
//! Each destination latches its failure: one is reported once until it
//! changes, and a delivery that succeeds clears it. The web build, which can
//! never send, reports every refused Network Send instead.
//!

use super::diagnostics::{DiagnosticLog, Report};
use super::schedule::OwnedNotes;
use super::{ClockInstant, OutputAdapter, OutputAdapterError};
use crate::network::{NetworkAdapter, NetworkDestination};
use crate::source::TickPlan;
use lang::Tick;
use tokio::sync::watch;

pub(super) struct Outputs<A: OutputAdapter> {
    pub(super) midi: A,
    /// Whether MIDI has a destination. The network does not read it.
    connected: bool,
    ///
    /// The notes Timed and Monophonic Play own, and the Tick each is stopped
    /// at.
    ///
    /// Here rather than beside the Source because a scheduled stop belongs to
    /// one Playback run.
    ///
    pub(super) owned: OwnedNotes,
    midi_failure: Option<OutputAdapterError>,
    ///
    /// Whether the latest MIDI delivery failed. Apart from the latch because
    /// a new run clears the latch so the next failure is reported again,
    /// while only a delivery answers whether the device recovered.
    ///
    midi_failed: bool,
    /// Where the Network Sends go, read at each Tick that makes one.
    destination: watch::Receiver<NetworkDestination>,
    network: Box<dyn NetworkAdapter>,
    network_failure: Option<OutputAdapterError>,
    diagnostics: DiagnosticLog,
}

impl<A: OutputAdapter> Outputs<A> {
    pub(super) fn new(
        midi: A,
        destination: watch::Receiver<NetworkDestination>,
        network: Box<dyn NetworkAdapter>,
        diagnostics: DiagnosticLog,
    ) -> Self {
        Self {
            midi,
            connected: true,
            owned: OwnedNotes::default(),
            midi_failure: None,
            midi_failed: false,
            destination,
            network,
            network_failure: None,
            diagnostics,
        }
    }

    ///
    /// Delivers `plan`, the Tick Plan of `tick`, which was due at `deadline`:
    /// its MIDI first, then its Network Sends, in their order.
    ///
    pub(super) fn deliver(&mut self, tick: Tick, plan: &TickPlan, deadline: ClockInstant) {
        if self.connected {
            // Nothing is delivered while disconnected, so nothing is owned
            // while disconnected either: resolving the Tick Plan here keeps a
            // Note Off from being scheduled for a note that never sounded.
            //
            // The schedule describes notes that are sounding, so it may record
            // only what the adapter accepted. The Tick is resolved against a
            // copy and adopted once the submission is: a refused submission
            // leaves every claim and every expiry standing, and the stop this
            // Tick drained is drained again by the next executed Tick rather
            // than discarded with no retry and no diagnostic. Do not adopt the
            // resolved schedule before the submission answers: `OutputAdapter`
            // is a trait, and that would rest on every implementation giving
            // up its connection the way `MidiOutputAdapter` does. The copy is
            // two maps of the notes currently sounding, taken once per
            // executed Tick.
            let mut owned = self.owned.clone();
            let delivery = owned.deliver(tick, &plan.play_commands);
            // Read at the submission rather than at the clock's wake, so the
            // time this Tick took to interpret counts as lateness too.
            let since_due = ClockInstant::now().saturating_duration_since(deadline);
            match self.midi.submit(&delivery, since_due) {
                Ok(()) => {
                    self.owned = owned;
                    self.midi_failure = None;
                    self.midi_failed = false;
                }
                Err(error) => self.record_midi_failure(error),
            }
        }
        if plan.network_sends.is_empty() {
            return;
        }
        // A new destination is owed its own answer, as a new MIDI connection
        // is. A closed channel keeps the last destination it published.
        if self.destination.has_changed().unwrap_or(false) {
            self.network_failure = None;
        }
        let destination = *self.destination.borrow_and_update();
        // A failure stops nothing: the sends after it are still made.
        for send in &plan.network_sends {
            match self.network.send(send, &destination) {
                Ok(()) => self.network_failure = None,
                Err(error) => self.record_network_failure(error),
            }
        }
    }

    /// Whether the latest MIDI delivery failed.
    pub(super) fn midi_failed(&self) -> bool {
        self.midi_failed
    }

    /// Whether the latest Network Send failed.
    pub(super) fn network_failed(&self) -> bool {
        self.network_failure.is_some()
    }

    /// Sends MIDI the safety action, when it has a destination to send it to.
    pub(super) fn silence(&mut self) {
        if self.connected {
            self.send_safety_reset();
        }
    }

    /// Ends every claim on a note, and with them every scheduled stop.
    pub(super) fn release(&mut self) {
        self.owned.clear();
    }

    ///
    /// Gives up MIDI's destination, sending the safety action to the device it
    /// is leaving.
    ///
    pub(super) fn disconnect(&mut self) {
        if self.connected {
            self.send_safety_reset();
            self.connected = false;
        }
        // Nothing this engine owns is sounding on a disconnected output, and
        // nothing it delivers while disconnected can start a note, so the
        // schedule goes with the connection.
        self.owned.clear();
    }

    ///
    /// Readies every destination for a new run.
    ///
    pub(super) fn begin_run(&mut self) {
        // The stop that ended the last run sent the safety action and
        // discarded the schedule, so a run never begins owning a note. This
        // holds it for an engine that reaches here by any other path: an
        // inherited claim would schedule a stop for a note this run never
        // started.
        self.owned.clear();
        // The latch suppresses a report of the failure it already reported, so
        // a device refusing every Tick is reported once rather than once per
        // Tick. That is one report per run, not one per adapter lifetime: a
        // drain takes the diagnostics away, so a run inheriting the latch would
        // say nothing at all about a device still failing in front of the user.
        //
        // Here alone, and deliberately not in `stop`. The safety action `stop`
        // sends is the last submission of the run that recorded the latch, so a
        // refusal repeating that run's failure is the same fault, which is what
        // the latch is for; clearing before sending it would report one device
        // fault twice. A stopped engine submits nothing, so a latch left
        // standing after `stop` suppresses nothing until this line clears it.
        self.midi_failure = None;
        self.network_failure = None;
    }

    fn send_safety_reset(&mut self) {
        if let Err(error) = self.midi.safety_reset() {
            self.record_midi_failure(error);
        }
    }

    fn record_midi_failure(&mut self, error: OutputAdapterError) {
        self.midi_failed = true;
        if self.midi_failure.as_ref() != Some(&error) {
            self.diagnostics
                .report(Report::OutputFailure(error.clone()));
            self.midi_failure = Some(error);
        }
    }

    fn record_network_failure(&mut self, error: OutputAdapterError) {
        if self.network.reports_every_failure() || self.network_failure.as_ref() != Some(&error) {
            self.diagnostics
                .report(Report::NetworkFailure(error.clone()));
        }
        self.network_failure = Some(error);
    }

    /// Sends the Network Sends through `network` instead.
    #[cfg(test)]
    pub(super) fn set_network(&mut self, network: Box<dyn NetworkAdapter>) {
        self.network = network;
    }
}

impl Outputs<crate::midi::MidiOutputAdapter> {
    ///
    /// Installs an already-open MIDI connection the console opened on its own
    /// thread.
    ///
    pub(super) fn install_connection(
        &mut self,
        destination_id: crate::midi::MidiDestinationId,
        connection: Box<dyn crate::midi::MidiConnection>,
    ) {
        // The notes this engine owned are sounding on the destination it is
        // leaving, which is sent the safety action before the new connection is
        // reached. Their scheduled stops would arrive at a device that never
        // started them, and a claim kept across the change would stop a note
        // the Source starts on that voice afterwards.
        self.owned.clear();
        // A selection is not a Tick: it is a thing the user just asked for,
        // and it is owed its own answer even when the answer is the
        // one the last selection got. Clearing before the install is what
        // makes a safety-action refusal that repeats the latched failure
        // visible.
        self.midi_failure = None;
        self.midi_failed = false;
        let selection = self.midi.install_connection(destination_id, connection);
        if let Some(error) = selection.safety_failure() {
            self.record_midi_failure(OutputAdapterError::new(error.message));
        }
        self.connected = true;
    }
}

#[cfg(test)]
mod tests {
    use super::Outputs;
    use crate::network::{NetworkDestination, RecordingAdapter, RefusingAdapter};
    use crate::playback::diagnostics::DiagnosticLog;
    use crate::playback::{
        ClockInstant, InMemoryOutputAdapter, OutputAdapter, OutputAdapterError, OutputCommand,
        PlaybackDiagnostic,
    };
    use crate::source::{MidiChannel, NetworkSend, Note, PlayCommand, TickPlan, Velocity};
    use lang::Tick;
    use std::time::Duration;
    use tokio::sync::watch;

    /// A Tick Plan with one Raw Play and one UDP send.
    fn plan() -> TickPlan {
        TickPlan {
            writes: vec![],
            play_commands: vec![PlayCommand::Raw {
                channel: MidiChannel::try_from(0).unwrap(),
                velocity: Velocity::try_from(0x7F).unwrap(),
                note: Note::try_from(60).unwrap(),
            }],
            network_sends: vec![NetworkSend::Udp(b"hi".as_slice().into())],
            diagnostics: vec![],
            locks: vec![],
        }
    }

    fn outputs<A: OutputAdapter>(
        midi: A,
        network: Box<dyn crate::network::NetworkAdapter>,
    ) -> (Outputs<A>, DiagnosticLog, watch::Sender<NetworkDestination>) {
        let (sender, destination) = watch::channel(NetworkDestination::default());
        let diagnostics = DiagnosticLog::new();
        let outputs = Outputs::new(midi, destination, network, diagnostics.clone());
        (outputs, diagnostics, sender)
    }

    fn deliver<A: OutputAdapter>(outputs: &mut Outputs<A>, tick: u64) {
        outputs.deliver(Tick::new(tick), &plan(), ClockInstant::now());
    }

    fn refused(message: &str) -> PlaybackDiagnostic {
        PlaybackDiagnostic::NetworkFailure(OutputAdapterError::new(message))
    }

    /// A MIDI adapter that notes, at each submission, how many Network Sends
    /// had been made.
    struct CountsNetworkSends {
        network: RecordingAdapter,
        seen: Vec<usize>,
    }

    impl OutputAdapter for CountsNetworkSends {
        fn submit(
            &mut self,
            _commands: &[OutputCommand],
            _since_due: Duration,
        ) -> Result<(), OutputAdapterError> {
            self.seen.push(self.network.sends().len());
            Ok(())
        }

        fn safety_reset(&mut self) -> Result<(), OutputAdapterError> {
            Ok(())
        }
    }

    #[test]
    fn a_ticks_midi_is_delivered_before_its_network_sends() {
        let network = RecordingAdapter::default();
        let midi = CountsNetworkSends {
            network: network.clone(),
            seen: vec![],
        };
        let (mut outputs, _, _) = outputs(midi, Box::new(network.clone()));
        deliver(&mut outputs, 0);
        assert_eq!(outputs.midi.seen, [0]);
        assert_eq!(network.sends().len(), 1);
    }

    #[test]
    fn network_sends_go_on_while_midi_is_disconnected() {
        let midi = InMemoryOutputAdapter::default();
        let network = RecordingAdapter::default();
        let (mut outputs, _, _) = outputs(midi.clone(), Box::new(network.clone()));
        outputs.disconnect();
        deliver(&mut outputs, 0);
        assert!(midi.command_lists().is_empty());
        assert_eq!(network.sends().len(), 1);
    }

    #[test]
    fn stopping_and_disconnecting_send_the_network_nothing() {
        let midi = InMemoryOutputAdapter::default();
        let network = RecordingAdapter::default();
        let (mut outputs, _, _) = outputs(midi.clone(), Box::new(network.clone()));
        outputs.silence();
        outputs.release();
        outputs.disconnect();
        outputs.begin_run();
        assert_eq!(midi.safety_reset_count(), 2);
        assert!(network.sends().is_empty());
    }

    #[test]
    fn a_tick_with_no_network_sends_leaves_the_network_failure_standing() {
        let network = RecordingAdapter::default();
        network.refuse(Some("refused"));
        let (mut outputs, _, _) =
            outputs(InMemoryOutputAdapter::default(), Box::new(network.clone()));
        deliver(&mut outputs, 0);
        let mut plan = plan();
        plan.network_sends.clear();
        outputs.deliver(Tick::new(1), &plan, ClockInstant::now());
        assert!(outputs.network_failed());
    }

    #[test]
    fn a_native_network_failure_is_reported_once_until_it_changes() {
        let network = RecordingAdapter::default();
        let (mut outputs, diagnostics, _) =
            outputs(InMemoryOutputAdapter::default(), Box::new(network.clone()));
        network.refuse(Some("refused"));
        deliver(&mut outputs, 0);
        deliver(&mut outputs, 1);
        network.refuse(Some("refused again"));
        deliver(&mut outputs, 2);
        assert_eq!(
            diagnostics.drain(),
            [refused("refused"), refused("refused again")]
        );
        assert!(outputs.network_failed());
    }

    #[test]
    fn a_send_that_leaves_clears_the_network_failure() {
        let network = RecordingAdapter::default();
        let (mut outputs, diagnostics, _) =
            outputs(InMemoryOutputAdapter::default(), Box::new(network.clone()));
        network.refuse(Some("refused"));
        deliver(&mut outputs, 0);
        network.refuse(None);
        deliver(&mut outputs, 1);
        assert!(!outputs.network_failed());
        network.refuse(Some("refused"));
        deliver(&mut outputs, 2);
        assert_eq!(
            diagnostics.drain(),
            [refused("refused"), refused("refused")]
        );
    }

    #[test]
    fn a_new_destination_and_a_new_run_each_report_the_failure_again() {
        let network = RecordingAdapter::default();
        let (mut outputs, diagnostics, destination) =
            outputs(InMemoryOutputAdapter::default(), Box::new(network.clone()));
        network.refuse(Some("refused"));
        deliver(&mut outputs, 0);
        destination.send_modify(|destination| destination.udp_port = 9000);
        deliver(&mut outputs, 1);
        outputs.begin_run();
        deliver(&mut outputs, 2);
        assert_eq!(diagnostics.drain(), vec![refused("refused"); 3]);
    }

    #[test]
    fn the_web_build_reports_every_refused_send() {
        let (mut outputs, diagnostics, _) =
            outputs(InMemoryOutputAdapter::default(), Box::new(RefusingAdapter));
        deliver(&mut outputs, 0);
        deliver(&mut outputs, 1);
        assert!(outputs.network_failed());
        let refused = refused("this build cannot send UDP or OSC");
        assert_eq!(diagnostics.drain(), [refused.clone(), refused]);
    }

    #[test]
    fn sends_keep_their_order_and_go_to_the_latest_destination() {
        let network = RecordingAdapter::default();
        let (mut outputs, _, destination) =
            outputs(InMemoryOutputAdapter::default(), Box::new(network.clone()));
        let mut plan = plan();
        let osc = NetworkSend::Osc(lang::OscMessage {
            address: "/amp".to_owned(),
            arguments: vec![],
        });
        plan.network_sends.push(osc.clone());
        outputs.deliver(Tick::ZERO, &plan, ClockInstant::now());
        destination.send_modify(|destination| destination.udp_port = 9000);
        deliver(&mut outputs, 1);
        let sends = network.sends();
        let sent: Vec<&NetworkSend> = sends.iter().map(|(send, _)| send).collect();
        assert_eq!(sent, [&plan.network_sends[0], &osc, &plan.network_sends[0]]);
        let ports: Vec<u16> = sends.iter().map(|(_, to)| to.udp_port).collect();
        assert_eq!(ports, [49161, 49161, 9000]);
    }
}
