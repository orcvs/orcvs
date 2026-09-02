use crate::playback::{OutputAdapter, OutputAdapterError};
use crate::source::PlayCommand;

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
            .then(|| self.send_all_notes_off().err())
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

    fn send_all_notes_off(&mut self) -> Result<(), MidiError> {
        let Some(connection) = self.connection.as_mut() else {
            return Ok(());
        };
        let mut first_error = None;
        for channel in 0..16 {
            if let Err(error) = connection.send(&[0xb0 | channel, 123, 0]) {
                first_error.get_or_insert(error);
            }
        }
        if let Some(error) = first_error {
            Err(error)
        } else {
            Ok(())
        }
    }
}

impl<B: MidiBackend> OutputAdapter for MidiOutputAdapter<B> {
    fn submit(&mut self, commands: &[PlayCommand]) -> Result<(), OutputAdapterError> {
        let Some(connection) = self.connection.as_mut() else {
            return self.delivery_failure.clone().map_or(Ok(()), Err);
        };
        for command in commands {
            // The Play Command carries validated MIDI values; turning them
            // into a status byte and its data bytes is this adapter's whole
            // job, and the only place in Orcvs that knows the wire format.
            let message = match *command {
                PlayCommand::Raw {
                    channel,
                    velocity,
                    note,
                } => {
                    // `0x90 | channel` is a channel nibble only while the
                    // channel is in range; a wider value would rewrite the
                    // status byte into a different MIDI message entirely.
                    debug_assert!(
                        channel <= 0x0F,
                        "Play channel {channel:02X} is out of range"
                    );
                    debug_assert!(
                        velocity <= 0x7F,
                        "Play velocity {velocity:02X} is out of range"
                    );
                    [0x90 | channel, note, velocity]
                }
            };
            if let Err(error) = connection.send(&message) {
                let delivery_error = OutputAdapterError::new(error.message);
                let _ = self.send_all_notes_off();
                self.connection = None;
                self.selected_destination_id = None;
                self.delivery_failure = Some(delivery_error.clone());
                return Err(delivery_error);
            }
        }
        Ok(())
    }

    fn all_notes_off(&mut self) -> Result<(), OutputAdapterError> {
        self.send_all_notes_off()
            .map_err(|error| OutputAdapterError::new(error.message))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grid::Grid;
    use crate::playback::{OutputAdapter, PlaybackEngine};
    use crate::source::{PlayCommand, SourceCommander};
    use std::sync::{Arc, Mutex};
    use std::time::Duration;

    #[derive(Default)]
    struct FakeState {
        messages: Vec<Vec<u8>>,
        fail_next_send: bool,
        connection_count: usize,
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
            self.state.lock().unwrap().connection_count += 1;
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
            if state.fail_next_send {
                state.fail_next_send = false;
                return Err(MidiError::new("device lost"));
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
                PlayCommand::Raw {
                    channel: 0x0f,
                    velocity: 0,
                    note: 0x15,
                },
                PlayCommand::Raw {
                    channel: 2,
                    velocity: 0x7f,
                    note: 0x45,
                },
            ])
            .unwrap();

        assert_eq!(
            state.lock().unwrap().messages,
            vec![vec![0x9f, 0x15, 0], vec![0x92, 0x45, 0x7f]]
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
    fn delivery_failure_attempts_all_notes_off_and_reselection_reconnects() {
        let state = Arc::new(Mutex::new(FakeState::default()));
        let mut adapter = MidiOutputAdapter::new(FakeBackend {
            state: state.clone(),
        });
        adapter.select(&MidiDestinationId::new("one")).unwrap();
        state.lock().unwrap().fail_next_send = true;

        let error = adapter
            .submit(&[PlayCommand::Raw {
                channel: 0,
                velocity: 0x7f,
                note: 60,
            }])
            .unwrap_err();

        assert_eq!(error, OutputAdapterError::new("device lost"));
        assert_eq!(state.lock().unwrap().messages.len(), 16);
        assert_eq!(adapter.selected_destination_id(), None);
        assert_eq!(
            adapter
                .submit(&[PlayCommand::Raw {
                    channel: 0,
                    velocity: 1,
                    note: 60,
                }])
                .unwrap_err(),
            OutputAdapterError::new("device lost")
        );

        adapter.select(&MidiDestinationId::new("one")).unwrap();
        adapter
            .submit(&[PlayCommand::Raw {
                channel: 1,
                velocity: 1,
                note: 61,
            }])
            .unwrap();

        let state = state.lock().unwrap();
        assert_eq!(state.connection_count, 2);
        assert_eq!(state.messages.last(), Some(&vec![0x91, 61, 1]));
    }

    #[test]
    fn all_notes_off_addresses_every_channel() {
        let state = Arc::new(Mutex::new(FakeState::default()));
        let mut adapter = MidiOutputAdapter::new(FakeBackend {
            state: state.clone(),
        });
        adapter.select(&MidiDestinationId::new("one")).unwrap();

        adapter.all_notes_off().unwrap();

        let messages = &state.lock().unwrap().messages;
        assert_eq!(messages.len(), 16);
        assert_eq!(messages.first(), Some(&vec![0xb0, 123, 0]));
        assert_eq!(messages.last(), Some(&vec![0xbf, 123, 0]));
    }

    #[tokio::test(start_paused = true)]
    async fn selecting_a_destination_after_disconnect_restores_output() {
        let state = Arc::new(Mutex::new(FakeState::default()));
        let source = SourceCommander::new(Grid::new(10, 2));
        // The Bang one row below the root anchor keeps the Raw Play active on
        // every Tick; without it a terminal root emits nothing at all.
        for (index, content) in "!>007FC4  **".chars().enumerate() {
            source.set(index, &content.to_string()).unwrap();
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
        let source = SourceCommander::new(Grid::new(10, 2));
        // The Bang one row below the root anchor keeps the Raw Play active on
        // every Tick; without it a terminal root emits nothing at all.
        for (index, content) in "!>007FC4  **".chars().enumerate() {
            source.set(index, &content.to_string()).unwrap();
        }
        let adapter = MidiOutputAdapter::new(FakeBackend {
            state: state.clone(),
        });
        let playback = PlaybackEngine::new(source, adapter);
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
