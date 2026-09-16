#[cfg(not(target_arch = "wasm32"))]
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use orcvs::app::{InputEvent, InputKey, Orcvs};
use orcvs::midi::{
    MidiBackend, MidiConnection, MidiDestination, MidiDestinationId, MidiError, MidiOutputAdapter,
};
#[cfg(not(target_arch = "wasm32"))]
use orcvs::playback::PlaybackDiagnostic;

#[derive(Default)]
struct FakeState {
    messages: Vec<Vec<u8>>,
}

struct FakeBackend {
    state: Arc<Mutex<FakeState>>,
}

impl FakeBackend {
    fn install(
        &mut self,
        handle: &orcvs::midi::MidiSelectionHandle,
        destination_id: &MidiDestinationId,
    ) -> Result<(), MidiError> {
        let connection = self.connect(destination_id)?;
        handle.install(destination_id.clone(), connection)
    }
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

#[cfg(not(target_arch = "wasm32"))]
struct PanickingConnection {
    delivery_started: Arc<AtomicBool>,
}

#[cfg(not(target_arch = "wasm32"))]
impl MidiConnection for PanickingConnection {
    fn send(&mut self, _message: &[u8]) -> Result<(), MidiError> {
        self.delivery_started.store(true, Ordering::SeqCst);
        panic!("adapter death");
    }
}

#[cfg(not(target_arch = "wasm32"))]
struct PanickingBackend {
    delivery_started: Arc<AtomicBool>,
}

#[cfg(not(target_arch = "wasm32"))]
impl PanickingBackend {
    fn install(
        &mut self,
        handle: &orcvs::midi::MidiSelectionHandle,
        destination_id: &MidiDestinationId,
    ) -> Result<(), MidiError> {
        let connection = self.connect(destination_id)?;
        handle.install(destination_id.clone(), connection)
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl MidiBackend for PanickingBackend {
    fn destinations(&mut self) -> Result<Vec<MidiDestination>, MidiError> {
        Ok(vec![MidiDestination::new("studio", "Studio Synth")])
    }

    fn connect(
        &mut self,
        destination_id: &MidiDestinationId,
    ) -> Result<Box<dyn MidiConnection>, MidiError> {
        assert_eq!(destination_id, &MidiDestinationId::new("studio"));
        Ok(Box::new(PanickingConnection {
            delivery_started: self.delivery_started.clone(),
        }))
    }
}

#[tokio::test(start_paused = true)]
async fn selected_destination_receives_playback_from_the_running_orcvs() {
    let state = Arc::new(Mutex::new(FakeState::default()));
    let mut backend = FakeBackend {
        state: state.clone(),
    };
    let mut orcvs =
        Orcvs::with_midi_output_adapter(10, 3, MidiOutputAdapter::new()).expect("the test runtime");
    let midi = orcvs.midi_selection_handle();

    assert_eq!(
        backend.destinations().unwrap(),
        vec![MidiDestination::new("studio", "Studio Synth")]
    );
    backend
        .install(&midi, &MidiDestinationId::new("studio"))
        .unwrap();
    assert_eq!(midi.selected_destination_id().unwrap(), None);
    for content in ".=0101".chars() {
        orcvs.write(&content.to_string());
    }
    orcvs.event_handler(vec![InputEvent::KeyPressed(InputKey::ArrowDown); 2]);
    orcvs.event_handler(vec![InputEvent::KeyPressed(InputKey::ArrowLeft); 6]);
    for content in "!>007FC4".chars() {
        orcvs.write(&content.to_string());
    }

    orcvs.event_handler(vec![InputEvent::KeyPressed(InputKey::Space)]);
    tokio::task::yield_now().await;

    assert_eq!(
        midi.selected_destination_id().unwrap(),
        Some(MidiDestinationId::new("studio"))
    );
    assert_eq!(
        state.lock().unwrap().messages.last(),
        Some(&vec![0x90, 60, 0x7f])
    );
}

#[tokio::test(start_paused = true)]
async fn selection_observations_become_unavailable_as_soon_as_the_owner_is_dropped() {
    let mut backend = FakeBackend {
        state: Arc::new(Mutex::new(FakeState::default())),
    };
    let orcvs =
        Orcvs::with_midi_output_adapter(10, 2, MidiOutputAdapter::new()).expect("the test runtime");
    let midi = orcvs.midi_selection_handle();
    backend
        .install(&midi, &MidiDestinationId::new("studio"))
        .unwrap();
    tokio::task::yield_now().await;
    assert_eq!(
        midi.selected_destination_id().unwrap(),
        Some(MidiDestinationId::new("studio"))
    );

    let surviving_selection = midi.clone();
    drop(midi);
    drop(orcvs);
    let unavailable = MidiError::new("running Orcvs is no longer available");
    assert_eq!(
        surviving_selection.selected_destination_id(),
        Err(unavailable.clone())
    );
}

#[cfg(not(target_arch = "wasm32"))]
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn selection_requests_fail_when_the_engine_task_has_ended() {
    let delivery_started = Arc::new(AtomicBool::new(false));
    let mut backend = PanickingBackend {
        delivery_started: delivery_started.clone(),
    };
    let mut orcvs =
        Orcvs::with_midi_output_adapter(10, 6, MidiOutputAdapter::new()).expect("the test runtime");
    let midi = orcvs.midi_selection_handle();
    backend
        .install(&midi, &MidiDestinationId::new("studio"))
        .unwrap();
    for content in ".=0101".chars() {
        orcvs.write(&content.to_string());
    }
    orcvs.event_handler(vec![InputEvent::KeyPressed(InputKey::ArrowDown); 2]);
    orcvs.event_handler(vec![InputEvent::KeyPressed(InputKey::ArrowLeft); 6]);
    for content in "!>007FC4".chars() {
        orcvs.write(&content.to_string());
    }

    orcvs.event_handler(vec![InputEvent::KeyPressed(InputKey::Space)]);

    let mut ended = false;
    for _ in 0..1_000 {
        if delivery_started.load(Ordering::SeqCst)
            && orcvs
                .drain_playback_diagnostics()
                .iter()
                .any(|diagnostic| matches!(diagnostic, PlaybackDiagnostic::ClockFailure { .. }))
        {
            ended = true;
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(1)).await;
    }
    assert!(ended, "the engine never reported that its run ended");

    let unavailable = MidiError::new("running Orcvs is no longer available");
    assert_eq!(
        backend.install(&midi, &MidiDestinationId::new("studio")),
        Err(unavailable)
    );
}

#[tokio::test(start_paused = true)]
async fn selection_handle_cannot_outlive_the_running_orcvs() {
    let state = Arc::new(Mutex::new(FakeState::default()));
    let mut backend = FakeBackend {
        state: state.clone(),
    };
    let mut orcvs =
        Orcvs::with_midi_output_adapter(10, 2, MidiOutputAdapter::new()).expect("the test runtime");
    let midi = orcvs.midi_selection_handle();
    backend
        .install(&midi, &MidiDestinationId::new("studio"))
        .unwrap();
    tokio::task::yield_now().await;

    orcvs.event_handler(vec![InputEvent::KeyPressed(InputKey::Space)]);
    tokio::task::yield_now().await;
    drop(orcvs);
    tokio::task::yield_now().await;

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
        midi.install(
            MidiDestinationId::new("studio"),
            backend.connect(&MidiDestinationId::new("studio")).unwrap()
        )
        .unwrap_err()
        .message,
        "running Orcvs is no longer available"
    );
    assert_eq!(
        midi.selected_destination_id().unwrap_err().message,
        "running Orcvs is no longer available"
    );
}
