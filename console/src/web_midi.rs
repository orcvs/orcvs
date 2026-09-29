//!
//! MIDI discovery and port opening in the browser, over the Web MIDI API.
//!
//! The browser grants MIDI access through a Promise, and neither discovery nor
//! connect may wait for it: the console calls both from a frame, and the
//! browser main thread has no blocking receive. Access is therefore requested
//! once, when the backend is built, and the answer is kept. Every call after
//! that reads where the request stands and answers synchronously:
//!
//! - while the browser has not answered, discovery and connect both answer a
//!   pending error saying access is still being waited for, so the menu says
//!   why its list is empty, and the answer wakes the Panel so its next frame
//!   discovers again;
//! - once access is granted, the output map is enumerated and a port is found
//!   on the thread that asked, exactly as the native backend does;
//! - a browser that offers no Web MIDI, or refuses access, answers an empty
//!   destination list, the silent fallback every target without a MIDI service
//!   gives.
//!
//! What crosses into Playback is a connection that names its output by id and
//! reaches the port through the same kept access when it sends.
//!
//! [`WebMidiBackend`] holds that logic over a [`WebMidiAccess`], so it is the
//! same code whether the access is the browser's or a test's.
//!

use orcvs::midi::{MidiBackend, MidiConnection, MidiDestination, MidiDestinationId, MidiError};

///
/// The message discovery and connect answer while the browser has not yet
/// granted or refused access.
///
pub(crate) const ACCESS_PENDING: &str = "waiting for the browser to grant MIDI access";

///
/// The message a connect answers when the output it names is gone from the
/// output map or is disconnected. It is the native backend's copy, so a
/// vanished device reads the same on either target.
///
pub(crate) const DESTINATION_GONE: &str = "the selected MIDI destination is no longer available";

///
/// The message a connect answers when there is no MIDI access to open a port
/// with. Discovery on such a browser lists nothing, so only a connect to an id
/// from somewhere other than this browser's list reaches it.
///
pub(crate) const NO_ACCESS: &str = "this browser offers no MIDI access";

///
/// Where the one request for MIDI access stands.
///
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum AccessStatus {
    /// The browser has not answered yet.
    Pending,
    /// The output map can be read and its ports sent to.
    Granted,
    /// The browser has no Web MIDI, or refused access. Final.
    Unavailable,
}

///
/// One entry of the output map, read at discovery.
///
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct WebMidiOutput {
    pub id: String,
    pub name: Option<String>,
    pub connected: bool,
}

///
/// What the browser backend reads from and writes to Web MIDI.
///
/// `Send` because a connection holding one crosses into the Playback Engine's
/// task, and `MidiConnection` is `Send`. The browser's own implementation
/// carries no JavaScript value for that reason; see `browser::BrowserMidi`.
///
pub(crate) trait WebMidiAccess: Clone + Send + 'static {
    fn status(&self) -> AccessStatus;

    ///
    /// Every output in the order the output map iterates them. Read only once
    /// `status` answered `Granted`.
    ///
    fn outputs(&self) -> Vec<WebMidiOutput>;

    ///
    /// Sends one complete MIDI message to the output with `id`, refusing when
    /// that output is gone or disconnected.
    ///
    fn send(&self, id: &str, message: &[u8]) -> Result<(), MidiError>;
}

///
/// The browser's [`MidiBackend`], over whichever [`WebMidiAccess`] it was built
/// with.
///
pub(crate) struct WebMidiBackend<A> {
    access: A,
}

impl<A: WebMidiAccess> WebMidiBackend<A> {
    pub(crate) fn new(access: A) -> Self {
        Self { access }
    }
}

impl<A: WebMidiAccess> MidiBackend for WebMidiBackend<A> {
    fn destinations(&mut self) -> Result<Vec<MidiDestination>, MidiError> {
        match self.access.status() {
            AccessStatus::Pending => Err(MidiError::pending(ACCESS_PENDING)),
            AccessStatus::Unavailable => Ok(Vec::new()),
            AccessStatus::Granted => Ok(self
                .access
                .outputs()
                .into_iter()
                .filter(|output| output.connected)
                .map(|output| {
                    // Web MIDI lets a port have no name. The id is what the
                    // menu can still tell apart.
                    let name = output.name.unwrap_or_else(|| output.id.clone());
                    MidiDestination::new(output.id, name)
                })
                .collect()),
        }
    }

    fn connect(
        &mut self,
        destination_id: &MidiDestinationId,
    ) -> Result<Box<dyn MidiConnection>, MidiError> {
        match self.access.status() {
            AccessStatus::Pending => Err(MidiError::pending(ACCESS_PENDING)),
            AccessStatus::Unavailable => Err(MidiError::new(NO_ACCESS)),
            AccessStatus::Granted => {
                let open = self
                    .access
                    .outputs()
                    .iter()
                    .any(|output| output.connected && output.id == destination_id.as_str());
                if !open {
                    return Err(MidiError::new(DESTINATION_GONE));
                }
                Ok(Box::new(WebMidiConnection {
                    access: self.access.clone(),
                    id: destination_id.as_str().to_owned(),
                }))
            }
        }
    }
}

///
/// An output the console found connected, delivered to by id.
///
/// Web MIDI opens a port implicitly on its first `send`, so the connection
/// needs no open step of its own, and a port that disconnects afterwards
/// refuses the next message, which the adapter turns into a Playback
/// diagnostic.
///
struct WebMidiConnection<A> {
    access: A,
    id: String,
}

impl<A: WebMidiAccess> MidiConnection for WebMidiConnection<A> {
    fn send(&mut self, message: &[u8]) -> Result<(), MidiError> {
        self.access.send(&self.id, message)
    }
}

#[cfg(target_arch = "wasm32")]
pub(crate) use browser::BrowserMidi;

#[cfg(target_arch = "wasm32")]
mod browser {
    use std::cell::RefCell;

    use orcvs::midi::MidiError;
    use wasm_bindgen::JsCast;
    use web_sys::{MidiAccess, MidiOutput, MidiPortDeviceState};

    use super::{AccessStatus, DESTINATION_GONE, WebMidiAccess, WebMidiOutput};

    ///
    /// The page's one request for MIDI access and its answer.
    ///
    enum Access {
        NotRequested,
        ///
        /// The browser has not answered. The Promise resolves once it has and
        /// the answer is stored here, so whoever awaits it reads the answer.
        ///
        Pending(js_sys::Promise),
        Granted(MidiAccess),
        Unavailable,
    }

    thread_local! {
        ///
        /// Kept here rather than in `BrowserMidi` because a `MidiAccess` is a
        /// JavaScript value and is not `Send`, while a connection that reaches
        /// it has to be. The browser runs the console and the Playback Engine's
        /// task on its one main thread (`spawn_local`), so every reader finds
        /// the value the request stored.
        ///
        static ACCESS: RefCell<Access> = const { RefCell::new(Access::NotRequested) };
    }

    ///
    /// The browser's Web MIDI, reached through the page's kept access.
    ///
    #[derive(Clone, Copy)]
    pub(crate) struct BrowserMidi;

    impl BrowserMidi {
        ///
        /// Asks the browser for MIDI access unless this page already has, and
        /// answers without waiting for the reply.
        ///
        /// System exclusive is not requested: Orcvs sends channel messages
        /// only, and asking for it makes the browser's permission prompt ask
        /// for more than the console uses.
        ///
        pub(crate) fn request() -> Self {
            let requested = ACCESS.with(|access| {
                let mut access = access.borrow_mut();
                if !matches!(*access, Access::NotRequested) {
                    return None;
                }
                let promise = web_sys::window()
                    .ok_or_else(|| wasm_bindgen::JsValue::from_str("no window"))
                    .and_then(|window| window.navigator().request_midi_access());
                match promise {
                    Ok(promise) => {
                        let mut settle = None;
                        let answered = js_sys::Promise::new(&mut |resolve, _reject| {
                            settle = Some(resolve);
                        });
                        *access = Access::Pending(answered);
                        Some((promise, settle))
                    }
                    Err(reason) => {
                        log::warn!("Web MIDI is unavailable: {reason:?}");
                        *access = Access::Unavailable;
                        None
                    }
                }
            });
            if let Some((promise, settle)) = requested {
                wasm_bindgen_futures::spawn_local(async move {
                    let answer = wasm_bindgen_futures::JsFuture::from(promise).await;
                    let next = match answer.map(JsCast::dyn_into::<MidiAccess>) {
                        Ok(Ok(granted)) => Access::Granted(granted),
                        Ok(Err(other)) => {
                            log::warn!("Web MIDI answered something other than access: {other:?}");
                            Access::Unavailable
                        }
                        Err(reason) => {
                            log::warn!("Web MIDI access was refused: {reason:?}");
                            Access::Unavailable
                        }
                    };
                    ACCESS.with(|access| *access.borrow_mut() = next);
                    if let Some(settle) = settle {
                        let _ = settle.call0(&wasm_bindgen::JsValue::UNDEFINED);
                    }
                });
            }
            Self
        }

        ///
        /// A Promise that resolves once the browser has answered the page's
        /// request and the answer is stored, or `None` when there is nothing
        /// to wait for: access was never requested, or it is already settled.
        ///
        pub(crate) fn answered() -> Option<js_sys::Promise> {
            ACCESS.with(|access| match &*access.borrow() {
                Access::Pending(answered) => Some(answered.clone()),
                Access::NotRequested | Access::Granted(_) | Access::Unavailable => None,
            })
        }
    }

    fn with_granted<T>(read: impl FnOnce(&MidiAccess) -> T) -> Option<T> {
        ACCESS.with(|access| match &*access.borrow() {
            Access::Granted(granted) => Some(read(granted)),
            _ => None,
        })
    }

    impl WebMidiAccess for BrowserMidi {
        fn status(&self) -> AccessStatus {
            ACCESS.with(|access| match &*access.borrow() {
                Access::Pending(_) => AccessStatus::Pending,
                Access::Granted(_) => AccessStatus::Granted,
                // A thread that never requested has no access to read, which
                // is the same answer as a browser that has none.
                Access::NotRequested | Access::Unavailable => AccessStatus::Unavailable,
            })
        }

        fn outputs(&self) -> Vec<WebMidiOutput> {
            with_granted(|granted| {
                granted
                    .outputs()
                    .values()
                    .into_iter()
                    .filter_map(Result::ok)
                    .filter_map(|value| value.dyn_into::<MidiOutput>().ok())
                    .map(|output| WebMidiOutput {
                        id: output.id(),
                        name: output.name(),
                        connected: output.state() == MidiPortDeviceState::Connected,
                    })
                    .collect()
            })
            .unwrap_or_default()
        }

        fn send(&self, id: &str, message: &[u8]) -> Result<(), MidiError> {
            let output = with_granted(|granted| granted.outputs().get(id))
                .flatten()
                .filter(|output| output.state() == MidiPortDeviceState::Connected)
                .ok_or_else(|| MidiError::new(DESTINATION_GONE))?;
            let bytes = js_sys::Uint8Array::from(message);
            output
                .send(&bytes)
                .map_err(|reason| MidiError::new(format!("MIDI send refused: {reason:?}")))
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex, MutexGuard};

    use orcvs::app::{InputEvent, InputKey, Orcvs};
    use orcvs::midi::{
        MidiBackend, MidiDestination, MidiDestinationId, MidiError, MidiOutputAdapter,
    };

    use super::{
        ACCESS_PENDING, AccessStatus, DESTINATION_GONE, NO_ACCESS, WebMidiAccess, WebMidiBackend,
        WebMidiOutput,
    };
    use crate::midi::MidiDeviceSelection;

    #[derive(Default)]
    struct FakeState {
        status: Option<AccessStatus>,
        outputs: Vec<WebMidiOutput>,
        sent: Vec<(String, Vec<u8>)>,
    }

    ///
    /// Web MIDI as a test drives it: the access status, the output map, and
    /// every message a port accepted, in order.
    ///
    #[derive(Clone, Default)]
    struct FakeWebMidi(Arc<Mutex<FakeState>>);

    impl FakeWebMidi {
        fn with(status: AccessStatus, outputs: &[(&str, Option<&str>)]) -> Self {
            let fake = Self::default();
            fake.set_status(status);
            fake.state().outputs = outputs
                .iter()
                .map(|(id, name)| WebMidiOutput {
                    id: (*id).to_owned(),
                    name: name.map(str::to_owned),
                    connected: true,
                })
                .collect();
            fake
        }

        fn state(&self) -> MutexGuard<'_, FakeState> {
            self.0.lock().expect("the test still holds the fake")
        }

        fn set_status(&self, status: AccessStatus) {
            self.state().status = Some(status);
        }

        fn disconnect(&self, id: &str) {
            for output in &mut self.state().outputs {
                if output.id == id {
                    output.connected = false;
                }
            }
        }

        fn sent_to(&self, id: &str) -> Vec<Vec<u8>> {
            self.state()
                .sent
                .iter()
                .filter(|(to, _)| to == id)
                .map(|(_, message)| message.clone())
                .collect()
        }

        fn clear_sent(&self) {
            self.state().sent.clear();
        }
    }

    impl WebMidiAccess for FakeWebMidi {
        fn status(&self) -> AccessStatus {
            self.state().status.unwrap_or(AccessStatus::Pending)
        }

        fn outputs(&self) -> Vec<WebMidiOutput> {
            assert_eq!(
                self.status(),
                AccessStatus::Granted,
                "the output map is read only once access is granted"
            );
            self.state().outputs.clone()
        }

        fn send(&self, id: &str, message: &[u8]) -> Result<(), MidiError> {
            let mut state = self.state();
            if !state
                .outputs
                .iter()
                .any(|output| output.id == id && output.connected)
            {
                return Err(MidiError::new(DESTINATION_GONE));
            }
            state.sent.push((id.to_owned(), message.to_vec()));
            Ok(())
        }
    }

    fn backend(fake: &FakeWebMidi) -> WebMidiBackend<FakeWebMidi> {
        WebMidiBackend::new(fake.clone())
    }

    ///
    /// The safety action's bytes for every channel, in the order they go out.
    ///
    fn safety_action() -> Vec<Vec<u8>> {
        (0..16u8)
            .flat_map(|channel| {
                [
                    vec![0xB0 | channel, 123, 0x00],
                    vec![0xB0 | channel, 121, 0x00],
                    vec![0xE0 | channel, 0x00, 0x40],
                ]
            })
            .collect()
    }

    ///
    /// A running Orcvs whose Source sounds `!>007FC4` — channel 0, velocity
    /// 0x7F, C4 — on the first Tick of every run, with MIDI selection over
    /// `fake`.
    ///
    fn playing(fake: &FakeWebMidi) -> (Orcvs, MidiDeviceSelection) {
        let mut orcvs = Orcvs::with_source_and_midi_output_adapter(
            orcvs::source::Source::new(orcvs::grid::Grid::with_shape(10, 3)),
            MidiOutputAdapter::new(),
        )
        .expect("the test runtime");
        for content in ".=0101".chars() {
            orcvs.write(&content.to_string());
        }
        orcvs.event_handler(vec![InputEvent::KeyPressed(InputKey::ArrowDown); 2]);
        orcvs.event_handler(vec![InputEvent::KeyPressed(InputKey::ArrowLeft); 6]);
        for content in "!>007FC4".chars() {
            orcvs.write(&content.to_string());
        }
        let midi = MidiDeviceSelection::new(orcvs.midi_selection_handle(), Box::new(backend(fake)));
        (orcvs, midi)
    }

    /// Starts the run, or stops it, and lets the engine's task take the request.
    async fn toggle_playback(orcvs: &mut Orcvs) {
        orcvs.event_handler(vec![InputEvent::KeyPressed(InputKey::Space)]);
        tokio::task::yield_now().await;
    }

    const NOTE_ON_C4: [u8; 3] = [0x90, 60, 0x7F];

    #[test]
    fn a_pending_request_answers_that_access_is_awaited() {
        let fake = FakeWebMidi::with(AccessStatus::Pending, &[("a", Some("Synth"))]);
        let mut backend = backend(&fake);

        assert_eq!(
            backend.destinations(),
            Err(MidiError::pending(ACCESS_PENDING))
        );
        assert_eq!(
            backend
                .connect(&MidiDestinationId::new("a"))
                .err()
                .map(|error| error.message),
            Some(ACCESS_PENDING.to_owned())
        );
    }

    ///
    /// A browser without Web MIDI, or one that refused access, offers the
    /// silent fallback: an empty list, not an error, so the menu shows the
    /// empty copy and no failure.
    ///
    #[tokio::test]
    async fn a_browser_without_midi_access_offers_an_empty_list_rather_than_an_error() {
        let fake = FakeWebMidi::with(AccessStatus::Unavailable, &[]);
        assert_eq!(backend(&fake).destinations(), Ok(Vec::new()));
        assert_eq!(
            backend(&fake)
                .connect(&MidiDestinationId::new("a"))
                .err()
                .map(|error| error.message),
            Some(NO_ACCESS.to_owned())
        );

        let (_orcvs, mut midi) = playing(&fake);
        midi.refresh_destinations();
        midi.auto_select_first_if_unselected();

        assert_eq!(midi.destinations(), &[] as &[MidiDestination]);
        assert_eq!(midi.status(), None);
        assert_eq!(midi.selected_destination_id(), None);
    }

    ///
    /// Discovery lists the connected outputs in the output map's order, and
    /// a port without a name is listed by its id.
    ///
    #[test]
    fn granted_access_lists_the_connected_outputs() {
        let fake = FakeWebMidi::with(
            AccessStatus::Granted,
            &[("a", Some("Synth")), ("b", None), ("c", Some("Unplugged"))],
        );
        fake.disconnect("c");

        assert_eq!(
            backend(&fake).destinations(),
            Ok(vec![
                MidiDestination::new("a", "Synth"),
                MidiDestination::new("b", "b"),
            ])
        );
    }

    #[test]
    fn a_connect_to_a_gone_or_disconnected_output_is_refused() {
        let fake = FakeWebMidi::with(AccessStatus::Granted, &[("a", Some("Synth"))]);
        fake.disconnect("a");

        for id in ["a", "missing"] {
            assert_eq!(
                backend(&fake)
                    .connect(&MidiDestinationId::new(id))
                    .err()
                    .map(|error| error.message),
                Some(DESTINATION_GONE.to_owned()),
                "{id}"
            );
        }
    }

    ///
    /// A discovery made before the browser answered says so; the performer's
    /// next Scan, once access is granted, lists the outputs and selects the
    /// first.
    ///
    #[tokio::test]
    async fn a_scan_after_access_is_granted_lists_and_selects() {
        let fake = FakeWebMidi::with(AccessStatus::Pending, &[("a", Some("Synth"))]);
        let (_orcvs, mut midi) = playing(&fake);

        midi.refresh_destinations();
        midi.auto_select_first_if_unselected();
        assert_eq!(midi.status(), Some(ACCESS_PENDING));
        assert_eq!(midi.destinations(), &[] as &[MidiDestination]);

        fake.set_status(AccessStatus::Granted);
        midi.refresh_destinations();
        midi.auto_select_first_if_unselected();
        tokio::task::yield_now().await;

        assert_eq!(midi.status(), None);
        assert_eq!(midi.destinations(), &[MidiDestination::new("a", "Synth")]);
        assert_eq!(
            midi.selected_destination_id(),
            Some(MidiDestinationId::new("a"))
        );
    }

    ///
    /// The console's startup discovery runs before the browser can answer, so
    /// it finds access pending. The first frame after the browser grants
    /// access lists the outputs and selects the first, and the first frame
    /// after it refuses clears the pending status, both without a Scan.
    ///
    #[tokio::test]
    async fn the_frame_after_the_browser_answers_catches_up_without_a_scan() {
        let cases = [
            (
                AccessStatus::Granted,
                vec![MidiDestination::new("a", "Synth")],
                Some(MidiDestinationId::new("a")),
            ),
            (AccessStatus::Unavailable, Vec::new(), None),
        ];
        for (answer, listed, selected) in cases {
            let fake = FakeWebMidi::with(AccessStatus::Pending, &[("a", Some("Synth"))]);
            let (_orcvs, mut midi) = playing(&fake);
            midi.refresh_destinations();
            midi.observe_frame();
            assert_eq!(midi.status(), Some(ACCESS_PENDING), "{answer:?}");

            fake.set_status(answer);
            midi.observe_frame();
            tokio::task::yield_now().await;

            assert_eq!(midi.status(), None, "{answer:?}");
            assert_eq!(midi.destinations(), listed.as_slice(), "{answer:?}");
            assert_eq!(midi.selected_destination_id(), selected, "{answer:?}");
        }
    }

    ///
    /// The whole browser path over a running Orcvs: the output the console
    /// opened is the one Playback delivers to, byte for byte; changing
    /// destination sends the outgoing output the safety action and the
    /// incoming one the next run's notes; and an output that disconnects
    /// refuses delivery, which reaches the status line.
    ///
    #[tokio::test(start_paused = true)]
    async fn playback_delivers_exact_bytes_to_the_output_the_console_opened() {
        let fake = FakeWebMidi::with(
            AccessStatus::Granted,
            &[("a", Some("First")), ("b", Some("Second"))],
        );
        let (mut orcvs, mut midi) = playing(&fake);

        midi.refresh_destinations();
        midi.select_destination(&MidiDestinationId::new("a"));
        toggle_playback(&mut orcvs).await;

        assert_eq!(
            midi.selected_destination_id(),
            Some(MidiDestinationId::new("a"))
        );
        assert_eq!(fake.sent_to("a"), vec![NOTE_ON_C4.to_vec()]);
        assert_eq!(fake.sent_to("b"), Vec::<Vec<u8>>::new());

        fake.clear_sent();
        midi.select_destination(&MidiDestinationId::new("b"));
        tokio::task::yield_now().await;

        assert_eq!(
            midi.selected_destination_id(),
            Some(MidiDestinationId::new("b"))
        );
        assert_eq!(fake.sent_to("a"), safety_action());

        toggle_playback(&mut orcvs).await;
        fake.clear_sent();
        toggle_playback(&mut orcvs).await;

        assert_eq!(fake.sent_to("a"), Vec::<Vec<u8>>::new());
        assert_eq!(fake.sent_to("b"), vec![NOTE_ON_C4.to_vec()]);

        toggle_playback(&mut orcvs).await;
        fake.disconnect("b");
        toggle_playback(&mut orcvs).await;
        midi.observe_diagnostics(orcvs.drain_playback_diagnostics());

        assert_eq!(midi.status(), Some(DESTINATION_GONE));
        assert_eq!(midi.selected_destination_id(), None);
    }
}
