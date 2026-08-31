#![cfg(any(target_os = "macos", target_os = "windows", target_os = "linux"))]

use std::sync::{Arc, Mutex};

use orcvs::app::{InputEvent, InputKey, Orcvs};
use orcvs::midi::{
    MidiBackend, MidiConnection, MidiDestination, MidiDestinationId, MidiError, MidiOutputAdapter,
};

#[derive(Default)]
struct FakeState {
    messages: Vec<Vec<u8>>,
}

struct FakeBackend {
    state: Arc<Mutex<FakeState>>,
}

impl MidiBackend for FakeBackend {
    fn destinations(&mut self) -> Result<Vec<MidiDestination>, MidiError> {
        Ok(vec![MidiDestination::new("studio", "Studio Synth")])
    }

    fn connect(
        &mut self,
        destination_id: &MidiDestinationId,
    ) -> Result<Box<dyn MidiConnection>, MidiError> {
        assert_eq!(destination_id, &MidiDestinationId::new("studio"));
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
        self.state.lock().unwrap().messages.push(message.to_vec());
        Ok(())
    }
}

#[tokio::test(start_paused = true)]
async fn selected_destination_receives_playback_from_the_running_orcvs() {
    let state = Arc::new(Mutex::new(FakeState::default()));
    let mut orcvs = Orcvs::with_output_adapter(
        10,
        2,
        MidiOutputAdapter::new(FakeBackend {
            state: state.clone(),
        }),
    );
    let midi = orcvs.midi_selection_handle();

    assert_eq!(
        midi.destinations().unwrap(),
        vec![MidiDestination::new("studio", "Studio Synth")]
    );
    midi.select(&MidiDestinationId::new("studio")).unwrap();
    for content in "!>07FC4".chars() {
        orcvs.write(&content.to_string());
    }

    orcvs.event_handler(vec![InputEvent::KeyPressed(InputKey::Space)]);
    tokio::task::yield_now().await;

    assert_eq!(
        state.lock().unwrap().messages.last(),
        Some(&vec![0x90, 60, 0x7f])
    );
}

#[tokio::test(start_paused = true)]
async fn selection_handle_cannot_outlive_the_running_orcvs() {
    let state = Arc::new(Mutex::new(FakeState::default()));
    let mut orcvs = Orcvs::with_output_adapter(
        10,
        2,
        MidiOutputAdapter::new(FakeBackend {
            state: state.clone(),
        }),
    );
    let midi = orcvs.midi_selection_handle();
    midi.select(&MidiDestinationId::new("studio")).unwrap();

    orcvs.event_handler(vec![InputEvent::KeyPressed(InputKey::Space)]);
    tokio::task::yield_now().await;
    drop(orcvs);

    assert_eq!(
        state
            .lock()
            .unwrap()
            .messages
            .iter()
            .filter(|message| message.get(1..3) == Some(&[123, 0]))
            .count(),
        16
    );

    assert_eq!(
        midi.destinations().unwrap_err().message,
        "running Orcvs is no longer available"
    );
    assert_eq!(
        midi.select(&MidiDestinationId::new("studio"))
            .unwrap_err()
            .message,
        "running Orcvs is no longer available"
    );
    assert_eq!(
        midi.selected_destination_id().unwrap_err().message,
        "running Orcvs is no longer available"
    );
}
