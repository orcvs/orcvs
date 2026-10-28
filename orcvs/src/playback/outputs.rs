//!
//! Every destination a Tick is delivered to, and the policy each keeps
//! (ADR 0090).
//!
//! MIDI waits for a connection, owns the notes it started and takes the safety
//! action. The network does none of that: its sends go on while no MIDI
//! destination is connected, and stopping, rewinding and disconnecting send it
//! nothing. Within a Tick, MIDI is delivered before the Network Sends.
//!

use super::diagnostics::{DiagnosticLog, Report};
use super::schedule::OwnedNotes;
use super::{ClockInstant, OutputAdapter, OutputAdapterError};
use crate::network::NetworkOutput;
use crate::source::TickPlan;
use lang::Tick;

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
    network: NetworkOutput,
    diagnostics: DiagnosticLog,
}

impl<A: OutputAdapter> Outputs<A> {
    pub(super) fn new(midi: A, network: NetworkOutput, diagnostics: DiagnosticLog) -> Self {
        Self {
            midi,
            connected: true,
            owned: OwnedNotes::default(),
            midi_failure: None,
            network,
            diagnostics,
        }
    }

    ///
    /// Delivers `plan`, the Tick Plan of `tick`, which was due at `deadline`:
    /// its MIDI first, then its Network Sends.
    ///
    /// Answers whether the Tick's Network Sends failed, or `None` when it made
    /// none.
    ///
    pub(super) fn deliver(
        &mut self,
        tick: Tick,
        plan: &TickPlan,
        deadline: ClockInstant,
    ) -> Option<bool> {
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
                }
                Err(error) => self.record_midi_failure(error),
            }
        }
        if plan.network_sends.is_empty() {
            return None;
        }
        // Each failed send is its own report, unlatched, so every activation
        // that could not send says so, and the performance goes on.
        let failures = self.network.deliver(&plan.network_sends);
        let failed = !failures.is_empty();
        for failure in failures {
            self.diagnostics.report(Report::NetworkFailure(failure));
        }
        Some(failed)
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
    }

    fn send_safety_reset(&mut self) {
        if let Err(error) = self.midi.safety_reset() {
            self.record_midi_failure(error);
        }
    }

    fn record_midi_failure(&mut self, error: OutputAdapterError) {
        if self.midi_failure.as_ref() != Some(&error) {
            self.diagnostics
                .report(Report::OutputFailure(error.clone()));
            self.midi_failure = Some(error);
        }
    }

    /// Sends the Tick's network output through `network` instead.
    #[cfg(test)]
    pub(super) fn set_network(&mut self, network: NetworkOutput) {
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
    use crate::network::{NetworkDestination, NetworkOutput, RecordingAdapter, RefusingAdapter};
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
    ) -> (Outputs<A>, DiagnosticLog) {
        // The sender is dropped: the receiver keeps the last destination.
        let (_, destination) = watch::channel(NetworkDestination::default());
        let diagnostics = DiagnosticLog::new();
        let outputs = Outputs::new(
            midi,
            NetworkOutput::new(destination, network),
            diagnostics.clone(),
        );
        (outputs, diagnostics)
    }

    fn deliver<A: OutputAdapter>(outputs: &mut Outputs<A>, tick: u64) -> Option<bool> {
        outputs.deliver(Tick::new(tick), &plan(), ClockInstant::now())
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
        let (mut outputs, _) = outputs(midi, Box::new(network.clone()));
        assert_eq!(deliver(&mut outputs, 0), Some(false));
        assert_eq!(outputs.midi.seen, [0]);
        assert_eq!(network.sends().len(), 1);
    }

    #[test]
    fn network_sends_go_on_while_midi_is_disconnected() {
        let midi = InMemoryOutputAdapter::default();
        let network = RecordingAdapter::default();
        let (mut outputs, _) = outputs(midi.clone(), Box::new(network.clone()));
        outputs.disconnect();
        deliver(&mut outputs, 0);
        assert!(midi.command_lists().is_empty());
        assert_eq!(network.sends().len(), 1);
    }

    #[test]
    fn stopping_and_disconnecting_send_the_network_nothing() {
        let midi = InMemoryOutputAdapter::default();
        let network = RecordingAdapter::default();
        let (mut outputs, _) = outputs(midi.clone(), Box::new(network.clone()));
        outputs.silence();
        outputs.release();
        outputs.disconnect();
        outputs.begin_run();
        assert_eq!(midi.safety_reset_count(), 2);
        assert!(network.sends().is_empty());
    }

    #[test]
    fn a_tick_with_no_network_sends_answers_none() {
        let (mut outputs, _) = outputs(
            InMemoryOutputAdapter::default(),
            Box::new(RecordingAdapter::default()),
        );
        let mut plan = plan();
        plan.network_sends.clear();
        assert_eq!(
            outputs.deliver(Tick::ZERO, &plan, ClockInstant::now()),
            None
        );
    }

    #[test]
    fn the_web_build_reports_every_refused_send() {
        let (mut outputs, diagnostics) =
            outputs(InMemoryOutputAdapter::default(), Box::new(RefusingAdapter));
        assert_eq!(deliver(&mut outputs, 0), Some(true));
        assert_eq!(deliver(&mut outputs, 1), Some(true));
        let refused = PlaybackDiagnostic::NetworkFailure(OutputAdapterError::new(
            "this build cannot send UDP or OSC",
        ));
        assert_eq!(diagnostics.drain(), [refused.clone(), refused]);
    }
}
