use crate::playback::{OutputAdapter, OutputAdapterError, OutputCommand};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MidiDestinationId(String);

impl MidiDestinationId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<String> for MidiDestinationId {
    fn from(id: String) -> Self {
        Self::new(id)
    }
}

impl From<&str> for MidiDestinationId {
    fn from(id: &str) -> Self {
        Self::new(id)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MidiDestination {
    pub id: MidiDestinationId,
    pub name: String,
}

impl MidiDestination {
    pub fn new(id: impl Into<MidiDestinationId>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MidiError {
    pub message: String,
}

impl MidiError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

pub trait MidiConnection: Send {
    fn send(&mut self, message: &[u8]) -> Result<(), MidiError>;
}

pub trait MidiBackend: Send {
    fn destinations(&mut self) -> Result<Vec<MidiDestination>, MidiError>;
    fn connect(
        &mut self,
        destination_id: &MidiDestinationId,
    ) -> Result<Box<dyn MidiConnection>, MidiError>;
}

pub struct MidiSelection {
    safety_failure: Option<MidiError>,
}

impl MidiSelection {
    pub fn safety_failure(self) -> Option<MidiError> {
        self.safety_failure
    }
}

pub struct MidiOutputAdapter<B> {
    backend: B,
    connection: Option<Box<dyn MidiConnection>>,
    delivery_failure: Option<OutputAdapterError>,
    selected_destination_id: Option<MidiDestinationId>,
}

impl<B: MidiBackend> MidiOutputAdapter<B> {
    pub fn new(backend: B) -> Self {
        Self {
            backend,
            connection: None,
            delivery_failure: None,
            selected_destination_id: None,
        }
    }

    pub fn destinations(&mut self) -> Result<Vec<MidiDestination>, MidiError> {
        self.backend.destinations()
    }

    pub fn select(
        &mut self,
        destination_id: &MidiDestinationId,
    ) -> Result<MidiSelection, MidiError> {
        let safety_failure = self
            .connection
            .is_some()
            .then(|| self.send_safety_reset().err())
            .flatten();
        let connection = self.backend.connect(destination_id)?;
        self.connection = Some(connection);
        self.delivery_failure = None;
        self.selected_destination_id = Some(destination_id.clone());
        Ok(MidiSelection { safety_failure })
    }

    pub fn selected_destination_id(&self) -> Option<&MidiDestinationId> {
        self.selected_destination_id.as_ref()
    }

    ///
    /// Returns every channel of the connected destination to silence and to
    /// its defaults, and answers the first refusal if any message was refused.
    ///
    /// A refusal does not end the run. The action exists to leave no device
    /// holding state Orcvs can no longer reach, and a channel skipped because
    /// an earlier one failed is exactly such a device; the first error is the
    /// one reported because it is the one that describes what went wrong.
    ///
    fn send_safety_reset(&mut self) -> Result<(), MidiError> {
        let Some(connection) = self.connection.as_mut() else {
            return Ok(());
        };
        let mut first_error = None;
        for channel in 0..16 {
            for message in safety_reset_messages(channel) {
                if let Err(error) = connection.send(&message) {
                    first_error.get_or_insert(error);
                }
            }
        }
        if let Some(error) = first_error {
            Err(error)
        } else {
            Ok(())
        }
    }
}

/// MIDI's All Notes Off, which silences the channel and says nothing about
/// anything else it is holding.
const ALL_NOTES_OFF: u8 = 123;

/// MIDI's Reset All Controllers: the message the protocol provides for a
/// sequencer stopping mid-gesture, which releases the sustain pedal CC 123
/// leaves down and returns the mod wheel to zero.
const RESET_ALL_CONTROLLERS: u8 = 121;

///
/// The safety action's messages for one channel, in the order they go out.
///
/// The order is part of the action rather than a detail of this loop. All
/// Notes Off comes first so the channel is silent before anything else is
/// changed on it. Reset All Controllers follows, because it is the message
/// that clears what a Control Change latched. The centred bend comes last and
/// is sent at all because device support for CC 121 varies: `0x2000` is
/// unambiguous where CC 121's coverage is not, and a device that does honour
/// CC 121 may move the wheel itself, so the explicit centre has to be the last
/// word on it rather than the first.
///
fn safety_reset_messages(channel: u8) -> [[u8; 3]; 3] {
    [
        [0xB0 | channel, ALL_NOTES_OFF, 0x00],
        [0xB0 | channel, RESET_ALL_CONTROLLERS, 0x00],
        [0xE0 | channel, 0x00, 0x40],
    ]
}

impl<B: MidiBackend> OutputAdapter for MidiOutputAdapter<B> {
    fn submit(&mut self, commands: &[OutputCommand]) -> Result<(), OutputAdapterError> {
        let Some(connection) = self.connection.as_mut() else {
            return self.delivery_failure.clone().map_or(Ok(()), Err);
        };
        for command in commands {
            // The Output Command carries validated MIDI values; turning them
            // into a status byte and its data bytes is this adapter's whole
            // job, and the only place in Orcvs that knows the wire format.
            let message = match *command {
                OutputCommand::NoteOn {
                    channel,
                    velocity,
                    note,
                } => {
                    // `0x90 | channel` is a channel nibble only while the
                    // channel is in range, and a wider value would rewrite the
                    // status byte into a different MIDI message entirely. The
                    // range is not re-derived here: `MidiChannel` and
                    // `Velocity` cannot hold one, so the interpreter's check is
                    // the only check there is.
                    [0x90 | channel.value(), note.value(), velocity.value()]
                }
                OutputCommand::ControlChange {
                    channel,
                    controller,
                    value,
                } => [0xB0 | channel.value(), controller.value(), value.value()],
                // The LSB precedes the MSB, which is the protocol's order and
                // not a choice: a bend is one fourteen-bit value sent low half
                // first, and the two halves are separate types precisely so
                // that this line is the only place their order can be got
                // wrong.
                OutputCommand::PitchBend { channel, lsb, msb } => {
                    [0xE0 | channel.value(), lsb.value(), msb.value()]
                }
            };
            if let Err(error) = connection.send(&message) {
                let delivery_error = OutputAdapterError::new(error.message);
                let _ = self.send_safety_reset();
                self.connection = None;
                self.selected_destination_id = None;
                self.delivery_failure = Some(delivery_error.clone());
                return Err(delivery_error);
            }
        }
        Ok(())
    }

    fn safety_reset(&mut self) -> Result<(), OutputAdapterError> {
        self.send_safety_reset()
            .map_err(|error| OutputAdapterError::new(error.message))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grid::{CellIndex, Grid};
    use crate::playback::{OutputAdapter, OutputCommand, PlaybackEngine};
    use crate::source::{
        BendLsb, BendMsb, ControlValue, Controller, MidiChannel, Note, SourceCommander, Velocity,
    };
    use std::sync::{Arc, Mutex};
    use std::time::Duration;

    ///
    /// The index `grid` mints for `idx`. A Cell is named by an index its Grid
    /// minted, so a test states the number and the Grid answers with the Cell.
    ///
    fn cell(grid: Grid, idx: usize) -> CellIndex {
        grid.cell_index(idx).expect("inside the Grid")
    }

    #[derive(Default)]
    struct FakeState {
        messages: Vec<Vec<u8>>,
        fail_next_send: bool,
        fail_next_connect: bool,
        connection_count: usize,
        /// The zero-based send attempts this connection refuses, each with an
        /// error naming its own index. A safety action refused part-way owes
        /// the first error and the remaining attempts, and neither claim is
        /// visible through a fake that can only fail once or can only fail
        /// with one message.
        failing_sends: Vec<usize>,
        send_count: usize,
    }

    struct FakeBackend {
        state: Arc<Mutex<FakeState>>,
    }

    impl MidiBackend for FakeBackend {
        fn destinations(&mut self) -> Result<Vec<MidiDestination>, MidiError> {
            Ok(vec![MidiDestination::new("one", "Synth")])
        }

        fn connect(
            &mut self,
            _destination_id: &MidiDestinationId,
        ) -> Result<Box<dyn MidiConnection>, MidiError> {
            let mut state = self.state.lock().unwrap();
            if state.fail_next_connect {
                state.fail_next_connect = false;
                return Err(MidiError::new("device unplugged"));
            }
            state.connection_count += 1;
            drop(state);
            Ok(Box::new(FakeConnection {
                state: self.state.clone(),
            }))
        }
    }

    struct FakeConnection {
        state: Arc<Mutex<FakeState>>,
    }

    impl MidiConnection for FakeConnection {
        fn send(&mut self, message: &[u8]) -> Result<(), MidiError> {
            let mut state = self.state.lock().unwrap();
            let attempt = state.send_count;
            state.send_count += 1;
            if state.fail_next_send {
                state.fail_next_send = false;
                return Err(MidiError::new("device lost"));
            }
            if state.failing_sends.contains(&attempt) {
                return Err(MidiError::new(format!("device lost on send {attempt}")));
            }
            state.messages.push(message.to_vec());
            Ok(())
        }
    }

    #[test]
    fn submits_commands_as_ordered_note_on_messages() {
        let state = Arc::new(Mutex::new(FakeState::default()));
        let mut adapter = MidiOutputAdapter::new(FakeBackend {
            state: state.clone(),
        });
        adapter.select(&MidiDestinationId::new("one")).unwrap();

        adapter
            .submit(&[
                OutputCommand::NoteOn {
                    channel: MidiChannel::try_from(0x0f).unwrap(),
                    velocity: Velocity::try_from(0).unwrap(),
                    note: Note::try_from(0x15).unwrap(),
                },
                OutputCommand::NoteOn {
                    channel: MidiChannel::try_from(2).unwrap(),
                    velocity: Velocity::try_from(0x7f).unwrap(),
                    note: Note::try_from(0x45).unwrap(),
                },
            ])
            .unwrap();

        assert_eq!(
            state.lock().unwrap().messages,
            vec![vec![0x9f, 0x15, 0], vec![0x92, 0x45, 0x7f]]
        );
    }

    #[test]
    fn submits_control_change_and_pitch_bend_as_their_wire_bytes() {
        let state = Arc::new(Mutex::new(FakeState::default()));
        let mut adapter = MidiOutputAdapter::new(FakeBackend {
            state: state.clone(),
        });
        adapter.select(&MidiDestinationId::new("one")).unwrap();

        adapter
            .submit(&[
                OutputCommand::ControlChange {
                    channel: MidiChannel::try_from(0x0F).unwrap(),
                    controller: Controller::try_from(0x07).unwrap(),
                    value: ControlValue::try_from(0x40).unwrap(),
                },
                OutputCommand::PitchBend {
                    channel: MidiChannel::try_from(0x0A).unwrap(),
                    lsb: BendLsb::try_from(0x2A).unwrap(),
                    msb: BendMsb::try_from(0x33).unwrap(),
                },
            ])
            .unwrap();

        // `0xB0 | channel` and `0xE0 | channel` are the two statuses, and a
        // Pitch Bend puts its LSB on the wire before its MSB. Every byte of
        // each message differs from every other byte of it, so an assembly
        // that transposed two of them answers different vectors rather than
        // agreeing with an expectation transposed the same way.
        //
        // Both channels are upper-nibble, as the Note On wire test's `0x0f`
        // already is: a status byte that dropped or masked the channel's high
        // bit agrees with every low-nibble channel a test might otherwise
        // reach for. The Source-path test below carries `01` and `03`, so the
        // low nibble is covered for both spellings without weakening this one.
        assert_eq!(
            state.lock().unwrap().messages,
            vec![vec![0xBF, 0x07, 0x40], vec![0xEA, 0x2A, 0x33]]
        );
    }

    #[tokio::test(start_paused = true)]
    async fn a_source_control_change_and_pitch_bend_reach_the_wire_as_their_bytes() {
        let state = Arc::new(Mutex::new(FakeState::default()));
        let grid = Grid::new(10, 6);
        let source = SourceCommander::new(grid);
        // The whole path in one run: two terminal roots, each activated by the
        // producer one row above it, delivered through the Playback Engine to
        // the bytes a device would receive. Per ADR 0032 a produced Bang's
        // north anchor is its own producer, so each terminal needs its own.
        for (index, content) in ".=0101              !c010207  .=0101              !b032A33  "
            .chars()
            .enumerate()
        {
            source.set(cell(grid, index), &content.to_string()).unwrap();
        }
        let adapter = MidiOutputAdapter::new(FakeBackend {
            state: state.clone(),
        });
        let playback = PlaybackEngine::new(source, adapter);
        playback
            .select_midi_destination(&MidiDestinationId::new("one"))
            .unwrap();

        playback.start(Duration::from_secs(1)).unwrap();
        tokio::task::yield_now().await;

        assert_eq!(
            state.lock().unwrap().messages,
            vec![vec![0xB1, 0x02, 0x07], vec![0xE3, 0x2A, 0x33]]
        );
    }

    #[test]
    fn enumerates_and_selects_a_destination() {
        let state = Arc::new(Mutex::new(FakeState::default()));
        let mut adapter = MidiOutputAdapter::new(FakeBackend {
            state: state.clone(),
        });

        assert_eq!(
            adapter.destinations().unwrap(),
            vec![MidiDestination::new("one", "Synth")]
        );
        adapter.select(&MidiDestinationId::new("one")).unwrap();

        assert_eq!(
            adapter.selected_destination_id(),
            Some(&MidiDestinationId::new("one"))
        );
        assert_eq!(state.lock().unwrap().connection_count, 1);
    }

    #[test]
    fn destination_identity_is_distinct_from_its_display_name() {
        let destination = MidiDestination::new("core-midi:17", "Studio Synth");

        assert_eq!(destination.id, MidiDestinationId::new("core-midi:17"));
        assert_eq!(destination.name, "Studio Synth");
    }

    #[test]
    fn delivery_failure_attempts_the_safety_action_and_reselection_reconnects() {
        let state = Arc::new(Mutex::new(FakeState::default()));
        let mut adapter = MidiOutputAdapter::new(FakeBackend {
            state: state.clone(),
        });
        adapter.select(&MidiDestinationId::new("one")).unwrap();
        state.lock().unwrap().fail_next_send = true;

        let error = adapter
            .submit(&[OutputCommand::NoteOn {
                channel: MidiChannel::try_from(0).unwrap(),
                velocity: Velocity::try_from(0x7f).unwrap(),
                note: Note::try_from(60).unwrap(),
            }])
            .unwrap_err();

        assert_eq!(error, OutputAdapterError::new("device lost"));
        // The refused Note On is not recorded, so every message here belongs
        // to the teardown: the same widened action a stop sends, on all
        // sixteen channels.
        assert_eq!(state.lock().unwrap().messages.len(), 48);
        assert_eq!(adapter.selected_destination_id(), None);
        assert_eq!(
            adapter
                .submit(&[OutputCommand::NoteOn {
                    channel: MidiChannel::try_from(0).unwrap(),
                    velocity: Velocity::try_from(1).unwrap(),
                    note: Note::try_from(60).unwrap()
                }])
                .unwrap_err(),
            OutputAdapterError::new("device lost")
        );

        adapter.select(&MidiDestinationId::new("one")).unwrap();
        adapter
            .submit(&[OutputCommand::NoteOn {
                channel: MidiChannel::try_from(1).unwrap(),
                velocity: Velocity::try_from(1).unwrap(),
                note: Note::try_from(61).unwrap(),
            }])
            .unwrap();

        let state = state.lock().unwrap();
        assert_eq!(state.connection_count, 2);
        assert_eq!(state.messages.last(), Some(&vec![0x91, 61, 1]));
    }

    ///
    /// The three messages the safety action owes `channel`, in the order it
    /// owes them. A test states this rather than deriving it from the adapter,
    /// so an implementation that reordered the triple or dropped one of its
    /// members disagrees with the expectation instead of moving it.
    ///
    fn safety_triple(channel: u8) -> Vec<Vec<u8>> {
        vec![
            vec![0xB0 | channel, 123, 0],
            vec![0xB0 | channel, 121, 0],
            vec![0xE0 | channel, 0x00, 0x40],
        ]
    }

    #[test]
    fn the_safety_action_clears_notes_controllers_and_bend_on_every_channel() {
        let state = Arc::new(Mutex::new(FakeState::default()));
        let mut adapter = MidiOutputAdapter::new(FakeBackend {
            state: state.clone(),
        });
        adapter.select(&MidiDestinationId::new("one")).unwrap();

        adapter.safety_reset().unwrap();

        let messages = state.lock().unwrap().messages.clone();
        // Sixteen channels, three messages each, channel-major: the triple is
        // what one channel is owed, so a channel is finished before the next
        // is begun.
        assert_eq!(messages.len(), 48);
        // The ends of the run spelled out, so a loop expectation sharing the
        // adapter's `0xB0 | channel` arithmetic cannot be the only thing that
        // pins the status bytes.
        assert_eq!(messages.first(), Some(&vec![0xB0, 123, 0]));
        assert_eq!(messages[47], vec![0xEF, 0x00, 0x40]);
        for channel in 0..16u8 {
            let at = usize::from(channel) * 3;
            assert_eq!(
                messages[at..at + 3],
                safety_triple(channel)[..],
                "channel {channel:#04X}"
            );
        }
    }

    #[test]
    fn a_refused_safety_message_reports_the_first_error_and_attempts_the_rest() {
        let state = Arc::new(Mutex::new(FakeState::default()));
        let mut adapter = MidiOutputAdapter::new(FakeBackend {
            state: state.clone(),
        });
        adapter.select(&MidiDestinationId::new("one")).unwrap();
        // Two refusals, neither of them the last attempt, so a run that
        // stopped at the first would deliver fewer messages and a run that
        // reported the last would name the wrong one.
        state.lock().unwrap().failing_sends = vec![1, 20];

        let error = adapter.safety_reset().unwrap_err();

        assert_eq!(error, OutputAdapterError::new("device lost on send 1"));
        let state = state.lock().unwrap();
        assert_eq!(state.send_count, 48);
        assert_eq!(state.messages.len(), 46);
    }

    #[tokio::test(start_paused = true)]
    async fn a_stop_clears_the_bend_and_the_latched_controller_a_source_left_standing() {
        let state = Arc::new(Mutex::new(FakeState::default()));
        let grid = Grid::new(10, 6);
        let source = SourceCommander::new(grid);
        // Controller `40` is sustain and value `7F` latches it down; the bend
        // on channel `03` is deflected to its top. Neither is a note, so
        // neither ends when the Source stops writing it: what the device is
        // left holding is decided by the safety action alone.
        for (index, content) in ".=0101              !c01407F  .=0101              !b03007F  "
            .chars()
            .enumerate()
        {
            source.set(cell(grid, index), &content.to_string()).unwrap();
        }
        let adapter = MidiOutputAdapter::new(FakeBackend {
            state: state.clone(),
        });
        let playback = PlaybackEngine::new(source, adapter);
        playback
            .select_midi_destination(&MidiDestinationId::new("one"))
            .unwrap();

        playback.start(Duration::from_secs(1)).unwrap();
        tokio::task::yield_now().await;
        assert_eq!(
            state.lock().unwrap().messages,
            vec![vec![0xB1, 0x40, 0x7F], vec![0xE3, 0x00, 0x7F]]
        );

        playback.stop();

        let messages = state.lock().unwrap().messages.clone();
        assert_eq!(messages.len(), 2 + 48);
        // The two channels the Source touched, read out of the run that
        // follows the stop: the sustain pedal on `01` is released by CC 121
        // and the wheel on `03` is returned to `0x2000` explicitly.
        assert_eq!(messages[2 + 3..2 + 6], safety_triple(0x01)[..]);
        assert_eq!(messages[2 + 9..2 + 12], safety_triple(0x03)[..]);
    }

    #[tokio::test(start_paused = true)]
    async fn selecting_a_destination_after_disconnect_restores_output() {
        let state = Arc::new(Mutex::new(FakeState::default()));
        let grid = Grid::new(10, 4);
        let source = SourceCommander::new(grid);
        // The comparison produces a fresh Bang one row above the Raw Play
        // each Tick; displayed or manually entered Bangs do not activate it.
        for (index, content) in ".=0101              !>007FC4".chars().enumerate() {
            source.set(cell(grid, index), &content.to_string()).unwrap();
        }
        let adapter = MidiOutputAdapter::new(FakeBackend {
            state: state.clone(),
        });
        let playback = PlaybackEngine::new(source, adapter);
        playback
            .select_midi_destination(&MidiDestinationId::new("one"))
            .unwrap();
        playback.start(Duration::from_secs(1)).unwrap();
        playback.disconnect();

        playback
            .select_midi_destination(&MidiDestinationId::new("one"))
            .unwrap();
        tokio::task::yield_now().await;

        assert_eq!(
            state.lock().unwrap().messages.last(),
            Some(&vec![0x90, 60, 0x7f])
        );
    }

    #[tokio::test(start_paused = true)]
    async fn disconnected_output_reports_delivery_failure_once() {
        let state = Arc::new(Mutex::new(FakeState::default()));
        let grid = Grid::new(10, 4);
        let source = SourceCommander::new(grid);
        // The comparison generates a fresh Bang for every attempted delivery, including
        // repeated attempts after the adapter has disconnected.
        for (index, content) in ".=0101              !>007FC4".chars().enumerate() {
            source.set(cell(grid, index), &content.to_string()).unwrap();
        }
        let adapter = MidiOutputAdapter::new(FakeBackend {
            state: state.clone(),
        });
        let playback = PlaybackEngine::new(source.clone(), adapter);
        playback
            .select_midi_destination(&MidiDestinationId::new("one"))
            .unwrap();
        state.lock().unwrap().fail_next_send = true;

        playback.start(Duration::from_secs(1)).unwrap();
        tokio::task::yield_now().await;
        for _ in 0..2 {
            tokio::time::advance(Duration::from_secs(1)).await;
            tokio::task::yield_now().await;
        }

        assert_eq!(
            playback.observe().diagnostics,
            vec![crate::playback::PlaybackDiagnostic::OutputFailure(
                OutputAdapterError::new("device lost")
            )]
        );
    }

    #[tokio::test(start_paused = true)]
    async fn changing_destination_clears_every_scheduled_stop() {
        // Both owning spellings, because a destination change clears the one
        // schedule they share and a test of `!~` alone would pass on a clear
        // that reached the Timed claims and left the Mono ones standing.
        for expression in ["!~007FC402", "!%007FC402"] {
            let state = Arc::new(Mutex::new(FakeState::default()));
            let grid = Grid::new(10, 4);
            let source = SourceCommander::new(grid);
            // A note stopped two Ticks after it starts, and the comparison
            // whose output one row above the root activates it.
            for (index, content) in ".=0101".chars().enumerate() {
                source.set(cell(grid, index), &content.to_string()).unwrap();
            }
            for (index, content) in expression.chars().enumerate() {
                source
                    .set(cell(grid, 20 + index), &content.to_string())
                    .unwrap();
            }
            let adapter = MidiOutputAdapter::new(FakeBackend {
                state: state.clone(),
            });
            let playback = PlaybackEngine::new(source.clone(), adapter);
            playback
                .select_midi_destination(&MidiDestinationId::new("one"))
                .unwrap();

            playback.start(Duration::from_secs(1)).unwrap();
            tokio::task::yield_now().await;
            assert_eq!(
                state.lock().unwrap().messages.last(),
                Some(&vec![0x90, 60, 0x7f]),
                "{expression} did not start its note"
            );

            // Make the comparison false so nothing new plays, then change
            // destination. The note is sounding on the destination being
            // left, which is sent the safety action as it goes, so its scheduled
            // stop belongs to a device this engine no longer holds.
            source.set(cell(grid, 5), "2").unwrap();
            playback
                .select_midi_destination(&MidiDestinationId::new("one"))
                .unwrap();
            let delivered = state.lock().unwrap().messages.len();

            for _ in 0..3 {
                tokio::time::advance(Duration::from_secs(1)).await;
                tokio::task::yield_now().await;
            }

            let (sent, connections) = {
                let state = state.lock().unwrap();
                (state.messages.len(), state.connection_count)
            };
            assert_eq!(sent, delivered, "{expression} delivered a cleared stop");
            assert_eq!(connections, 2, "{expression} did not reconnect");
        }
    }

    #[tokio::test(start_paused = true)]
    async fn a_destination_change_that_fails_to_connect_clears_the_scheduled_stop() {
        // Both owning spellings, for the reason the successful change tests
        // both: the clear is the one the two schedules share.
        for expression in ["!~007FC402", "!%007FC402"] {
            let state = Arc::new(Mutex::new(FakeState::default()));
            let grid = Grid::new(10, 4);
            let source = SourceCommander::new(grid);
            // The same Play the successful change uses: a note stopped two
            // Ticks after it starts, and the comparison that generates its
            // activation.
            for (index, content) in ".=0101".chars().enumerate() {
                source.set(cell(grid, index), &content.to_string()).unwrap();
            }
            for (index, content) in expression.chars().enumerate() {
                source
                    .set(cell(grid, 20 + index), &content.to_string())
                    .unwrap();
            }
            let adapter = MidiOutputAdapter::new(FakeBackend {
                state: state.clone(),
            });
            let playback = PlaybackEngine::new(source.clone(), adapter);
            playback
                .select_midi_destination(&MidiDestinationId::new("one"))
                .unwrap();

            playback.start(Duration::from_secs(1)).unwrap();
            tokio::task::yield_now().await;
            assert_eq!(
                state.lock().unwrap().messages.last(),
                Some(&vec![0x90, 60, 0x7f]),
                "{expression} did not start its note"
            );

            // Make the comparison false, then attempt a change the device
            // refuses. The safety action that precedes the connection is sent
            // regardless, so the note is silenced whether or not the new
            // destination is reached: a change that silences the old device
            // owes the same cleared schedule whether it completes or fails.
            source.set(cell(grid, 5), "2").unwrap();
            state.lock().unwrap().fail_next_connect = true;
            playback
                .select_midi_destination(&MidiDestinationId::new("one"))
                .unwrap_err();
            let delivered = state.lock().unwrap().messages.len();

            for _ in 0..3 {
                tokio::time::advance(Duration::from_secs(1)).await;
                tokio::task::yield_now().await;
            }

            // Read both counters out before asserting: a guard held across a
            // failing assertion poisons the fake, and the engine's own drop
            // then panics inside a destructor and hides which one failed.
            let (sent, connections) = {
                let state = state.lock().unwrap();
                (state.messages.len(), state.connection_count)
            };
            assert_eq!(sent, delivered, "{expression} delivered a cleared stop");
            assert_eq!(connections, 1, "{expression} reconnected after a refusal");
        }
    }

    #[test]
    fn reselection_reports_safety_failure_and_connects_new_destination() {
        let state = Arc::new(Mutex::new(FakeState::default()));
        let adapter = MidiOutputAdapter::new(FakeBackend {
            state: state.clone(),
        });
        let source = SourceCommander::new(Grid::new(1, 1));
        let playback = PlaybackEngine::new(source, adapter);
        playback
            .select_midi_destination(&MidiDestinationId::new("one"))
            .unwrap();
        state.lock().unwrap().fail_next_send = true;

        playback
            .select_midi_destination(&MidiDestinationId::new("one"))
            .unwrap();

        assert_eq!(state.lock().unwrap().connection_count, 2);
        let observation = playback.observe();
        assert_eq!(
            observation.diagnostics,
            vec![crate::playback::PlaybackDiagnostic::OutputFailure(
                OutputAdapterError::new("device lost")
            )]
        );
    }
}
