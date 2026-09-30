//! Browser MIDI discovery and opening, with page-scoped port ownership.
//!
//! Access is requested once and discovery reads the stored answer without
//! blocking a frame. Each connect returns an owned opening request. Polling
//! observes completion; dropping the request abandons only its own claim.
//!
//! The shared port lifecycle survives Source replacement and counts pending
//! requests together with live connections. A port closes after its final claim
//! is released, including when an abandoned browser open completes later.
//! Playback releases connections after its outgoing safety action.
//!
//! Browser operations and test completions drive the same lifecycle in `ports`.
//! JavaScript values stay on the browser thread; connections reach them by id.

use orcvs::midi::{
    MidiBackend, MidiConnectionRequest, MidiDestination, MidiDestinationId, MidiError,
};

mod ports;
use ports::Ports;

use crate::console_midi::DESTINATION_GONE;

type Completion = Box<dyn FnOnce(Result<(), MidiError>) + Send>;

///
/// The message discovery and connect answer while the browser has not yet
/// granted or refused access.
///
pub(crate) const ACCESS_PENDING: &str = "waiting for the browser to grant MIDI access";

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

    /// Starts the browser operation and reports its eventual outcome exactly once.
    /// Completion may run before this call returns; no lifecycle lock is held.
    fn open(&self, id: &str, complete: Completion);

    /// Reports completion after previously submitted messages have been delivered.
    fn close(&self, id: &str, complete: Completion);
}

pub(crate) struct WebMidiBackend<A> {
    ports: Ports<A>,
}

impl<A: WebMidiAccess> WebMidiBackend<A> {
    fn new(ports: Ports<A>) -> Self {
        Self { ports }
    }
}

impl<A: WebMidiAccess> MidiBackend for WebMidiBackend<A> {
    fn destinations(&mut self) -> Result<Vec<MidiDestination>, MidiError> {
        match self.ports.access().status() {
            AccessStatus::Pending => Err(MidiError::pending(ACCESS_PENDING)),
            AccessStatus::Unavailable => Ok(Vec::new()),
            AccessStatus::Granted => Ok(self
                .ports
                .access()
                .outputs()
                .into_iter()
                .filter(|output| output.connected)
                .map(|output| {
                    let name = output.name.unwrap_or_else(|| output.id.clone());
                    MidiDestination::new(output.id, name)
                })
                .collect()),
        }
    }

    fn connect(
        &mut self,
        destination_id: &MidiDestinationId,
    ) -> Result<MidiConnectionRequest, MidiError> {
        match self.ports.access().status() {
            AccessStatus::Pending => Err(MidiError::pending(ACCESS_PENDING)),
            AccessStatus::Unavailable => Err(MidiError::new(NO_ACCESS)),
            AccessStatus::Granted => {
                if !self
                    .ports
                    .access()
                    .outputs()
                    .iter()
                    .any(|output| output.connected && output.id == destination_id.as_str())
                {
                    return Err(MidiError::new(DESTINATION_GONE));
                }
                Ok(self.ports.acquire(destination_id.as_str().to_owned()))
            }
        }
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

    use super::{
        AccessStatus, Completion, DESTINATION_GONE, Ports, WebMidiAccess, WebMidiBackend,
        WebMidiOutput,
    };

    ///
    /// The prefix of the status a port the browser could not open reports.
    ///
    const PORT_REFUSED: &str = "the browser could not open the MIDI destination";

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

        static PORTS: Ports<BrowserMidi> = Ports::new(BrowserMidi);

        ///
        /// What runs once the browser answers an open or close, so the frame
        /// that polls the owned request runs without waiting for the
        /// performer's next input.
        ///
        static PORT_ANSWERED: RefCell<Option<Box<dyn Fn()>>> = const { RefCell::new(None) };
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
        pub(crate) fn request() {
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

        ///
        /// Runs `wake` each time the browser answers an open or close,
        /// in place of whatever ran before.
        ///
        pub(crate) fn on_port_answer(wake: impl Fn() + 'static) {
            PORT_ANSWERED.with(|answered| *answered.borrow_mut() = Some(Box::new(wake)));
        }
    }

    impl WebMidiBackend<BrowserMidi> {
        pub(crate) fn browser() -> Self {
            BrowserMidi::request();
            PORTS.with(|ports| Self::new(ports.clone()))
        }
    }

    fn finish(promise: js_sys::Promise, complete: Completion) {
        wasm_bindgen_futures::spawn_local(async move {
            let answer = wasm_bindgen_futures::JsFuture::from(promise)
                .await
                .map(|_| ())
                .map_err(|reason| {
                    let reason = reason
                        .dyn_ref::<js_sys::Error>()
                        .map(|error| String::from(error.message()))
                        .unwrap_or_else(|| format!("{reason:?}"));
                    MidiError::new(reason)
                });
            complete(answer);
            PORT_ANSWERED.with(|answered| {
                if let Some(wake) = &*answered.borrow() {
                    wake();
                }
            });
        });
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

        fn open(&self, id: &str, complete: Completion) {
            let Some(output) = with_granted(|granted| granted.outputs().get(id)).flatten() else {
                complete(Err(MidiError::new(DESTINATION_GONE)));
                return;
            };
            finish(
                output.open(),
                Box::new(move |answer| {
                    complete(answer.map_err(|error| {
                        MidiError::new(format!("{PORT_REFUSED}: {}", error.message))
                    }));
                }),
            );
        }

        fn close(&self, id: &str, complete: Completion) {
            let Some(output) = with_granted(|granted| granted.outputs().get(id)).flatten() else {
                complete(Ok(()));
                return;
            };
            finish(output.close(), complete);
        }
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use std::collections::{HashMap, HashSet};
    use std::sync::{Arc, Mutex, MutexGuard};

    use orcvs::app::{InputEvent, InputKey, Orcvs};
    use orcvs::midi::{
        MidiBackend, MidiDestination, MidiDestinationId, MidiError, MidiOutputAdapter,
    };

    use super::{
        ACCESS_PENDING, AccessStatus, Completion, DESTINATION_GONE, NO_ACCESS, Ports,
        WebMidiAccess, WebMidiBackend, WebMidiOutput,
    };
    use crate::midi::MidiDeviceSelection;

    #[derive(Default)]
    struct FakeState {
        status: Option<AccessStatus>,
        outputs: Vec<WebMidiOutput>,
        sent: Vec<(String, Vec<u8>)>,
        /// Each close, with how many messages its output had accepted when it closed.
        closed: Vec<(String, usize)>,
        auto_open: HashSet<String>,
        physical_open: HashSet<String>,
        opening: HashMap<String, Completion>,
        closing: HashMap<String, Completion>,
        delay_close: bool,
        opened: Vec<String>,
    }

    ///
    /// Web MIDI as a test drives it: the access status, the output map, how
    /// each operation completes, and every accepted message, in order.
    /// Outputs complete opening immediately unless a test delays them.
    ///
    #[derive(Clone, Default)]
    pub(crate) struct BrowserEffects(Arc<Mutex<FakeState>>);

    impl BrowserEffects {
        pub(crate) fn with(status: AccessStatus, outputs: &[(&str, Option<&str>)]) -> Self {
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
            fake.state().auto_open = outputs.iter().map(|(id, _)| (*id).to_owned()).collect();
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

        pub(crate) fn sent_to(&self, id: &str) -> Vec<Vec<u8>> {
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

        pub(crate) fn closed(&self) -> Vec<(String, usize)> {
            self.state().closed.clone()
        }

        /// Leaves the output with `id` closed, so the next open has to wait.
        pub(crate) fn close_port(&self, id: &str) {
            self.state().auto_open.remove(id);
        }

        /// Answers the open the browser is working on for `id`.
        pub(crate) fn answer_open(&self, id: &str, answer: Result<(), &str>) {
            let complete = self
                .state()
                .opening
                .remove(id)
                .expect("one pending browser open");
            if answer.is_ok() {
                self.state().physical_open.insert(id.to_owned());
            }
            complete(answer.map_err(MidiError::new));
        }

        fn answer_close(&self, id: &str) {
            let complete = self
                .state()
                .closing
                .remove(id)
                .expect("one pending browser close");
            self.state().physical_open.remove(id);
            complete(Ok(()));
        }
    }

    impl WebMidiAccess for BrowserEffects {
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
            assert!(
                state.physical_open.contains(id),
                "send requires an opened port"
            );
            state.sent.push((id.to_owned(), message.to_vec()));
            Ok(())
        }

        fn open(&self, id: &str, complete: Completion) {
            let mut state = self.state();
            state.opened.push(id.to_owned());
            if state.auto_open.contains(id) {
                state.physical_open.insert(id.to_owned());
                drop(state);
                complete(Ok(()));
            } else {
                assert!(state.opening.insert(id.to_owned(), complete).is_none());
            }
        }

        fn close(&self, id: &str, complete: Completion) {
            let mut state = self.state();
            let accepted = state.sent.iter().filter(|(to, _)| to == id).count();
            state.closed.push((id.to_owned(), accepted));
            if state.delay_close {
                assert!(state.closing.insert(id.to_owned(), complete).is_none());
            } else {
                state.physical_open.remove(id);
                drop(state);
                complete(Ok(()));
            }
        }
    }

    pub(crate) struct FakeWebMidi {
        effects: BrowserEffects,
        ports: Ports<BrowserEffects>,
    }

    impl FakeWebMidi {
        pub(crate) fn with(status: AccessStatus, outputs: &[(&str, Option<&str>)]) -> Self {
            let effects = BrowserEffects::with(status, outputs);
            Self {
                ports: Ports::new(effects.clone()),
                effects,
            }
        }
    }

    impl std::ops::Deref for FakeWebMidi {
        type Target = BrowserEffects;
        fn deref(&self) -> &Self::Target {
            &self.effects
        }
    }

    pub(crate) fn backend(fake: &FakeWebMidi) -> WebMidiBackend<BrowserEffects> {
        WebMidiBackend::new(fake.ports.clone())
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
    /// A console built after the browser has answered, as the web entry point
    /// arranges for a page whose permission is already settled, shows that
    /// answer from its startup discovery on: no frame reads access as pending,
    /// a granted page selects its first output, and a refused one reads `None`.
    ///
    #[tokio::test]
    async fn a_console_built_after_the_answer_never_shows_access_pending() {
        let cases = [
            (AccessStatus::Granted, Some(MidiDestinationId::new("a"))),
            (AccessStatus::Unavailable, None),
        ];
        for (answer, selected) in cases {
            let fake = FakeWebMidi::with(answer, &[("a", Some("Synth"))]);
            let (_orcvs, mut midi) = playing(&fake);
            midi.refresh_destinations();
            assert_eq!(midi.status(), None, "{answer:?}");

            for _ in 0..3 {
                midi.observe_frame();
                tokio::task::yield_now().await;
                assert_eq!(midi.status(), None, "{answer:?}");
            }
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

    ///
    /// An output Playback stops using is closed once its safety action is
    /// delivered, and an output still held by the selection is not: choosing
    /// the destination that is already selected keeps its port open, and a
    /// refusing output is closed after the teardown behind the refusal.
    ///
    #[tokio::test(start_paused = true)]
    async fn an_output_is_closed_once_no_connection_holds_it() {
        let fake = FakeWebMidi::with(
            AccessStatus::Granted,
            &[("a", Some("First")), ("b", Some("Second"))],
        );
        let (mut orcvs, mut midi) = playing(&fake);

        midi.refresh_destinations();
        midi.select_destination(&MidiDestinationId::new("a"));
        toggle_playback(&mut orcvs).await;
        toggle_playback(&mut orcvs).await;
        midi.select_destination(&MidiDestinationId::new("a"));
        tokio::task::yield_now().await;

        assert_eq!(fake.closed(), Vec::new());

        midi.select_destination(&MidiDestinationId::new("b"));
        tokio::task::yield_now().await;

        let delivered_to_a = fake.sent_to("a").len();
        assert_eq!(fake.sent_to("a").last(), safety_action().last());
        assert_eq!(fake.closed(), vec![("a".to_owned(), delivered_to_a)]);

        fake.disconnect("b");
        toggle_playback(&mut orcvs).await;

        assert_eq!(midi.selected_destination_id(), None);
        assert_eq!(
            fake.closed(),
            vec![("a".to_owned(), delivered_to_a), ("b".to_owned(), 0)]
        );
    }

    ///
    /// A connected output the browser cannot open, for instance one another
    /// application holds, never replaces the destination that is playing:
    /// choosing it answers that the port is opening, and the browser's refusal
    /// reaches the status line while the old output keeps every note and
    /// never receives the safety action.
    ///
    #[tokio::test(start_paused = true)]
    async fn an_output_that_fails_to_open_leaves_the_playing_destination() {
        let fake = FakeWebMidi::with(
            AccessStatus::Granted,
            &[("a", Some("First")), ("b", Some("Held elsewhere"))],
        );
        fake.close_port("b");
        let (mut orcvs, mut midi) = playing(&fake);

        midi.refresh_destinations();
        midi.select_destination(&MidiDestinationId::new("a"));
        toggle_playback(&mut orcvs).await;
        toggle_playback(&mut orcvs).await;
        fake.clear_sent();

        midi.select_destination(&MidiDestinationId::new("b"));
        tokio::task::yield_now().await;

        assert_eq!(midi.status(), Some("Opening Held elsewhere…"));
        assert_eq!(
            midi.selected_destination_id(),
            Some(MidiDestinationId::new("a"))
        );
        assert_eq!(fake.sent_to("a"), Vec::<Vec<u8>>::new());

        fake.answer_open("b", Err("the port is in use"));
        midi.observe_frame();
        tokio::task::yield_now().await;

        assert_eq!(midi.status(), Some("the port is in use"));
        assert_eq!(
            midi.selected_destination_id(),
            Some(MidiDestinationId::new("a"))
        );
        toggle_playback(&mut orcvs).await;
        assert_eq!(fake.sent_to("a"), vec![NOTE_ON_C4.to_vec()]);
        assert_eq!(fake.sent_to("b"), Vec::<Vec<u8>>::new());
        assert_eq!(fake.closed(), Vec::new());
    }

    ///
    /// Selecting an output the browser has still to open answers that it is
    /// opening, and the first frame after the browser opens it installs it,
    /// without a Scan or a second choice.
    ///
    #[tokio::test(start_paused = true)]
    async fn the_frame_after_an_output_opens_installs_it() {
        let fake = FakeWebMidi::with(AccessStatus::Granted, &[("a", Some("Synth"))]);
        fake.close_port("a");
        let (mut orcvs, mut midi) = playing(&fake);

        midi.refresh_destinations();
        midi.observe_frame();
        tokio::task::yield_now().await;

        assert_eq!(midi.status(), Some("Opening Synth…"));
        assert_eq!(midi.selected_destination_id(), None);

        midi.observe_frame();
        assert_eq!(midi.status(), Some("Opening Synth…"));

        fake.answer_open("a", Ok(()));
        midi.observe_frame();
        tokio::task::yield_now().await;

        assert_eq!(midi.status(), None);
        assert_eq!(
            midi.selected_destination_id(),
            Some(MidiDestinationId::new("a"))
        );
        toggle_playback(&mut orcvs).await;
        assert_eq!(fake.sent_to("a"), vec![NOTE_ON_C4.to_vec()]);
    }

    ///
    /// An open the performer walks away from, by choosing another output or
    /// Scanning, is released rather than left holding the device, and its
    /// answer never installs it.
    ///
    #[tokio::test(start_paused = true)]
    async fn an_abandoned_open_releases_its_output() {
        let fake = FakeWebMidi::with(
            AccessStatus::Granted,
            &[
                ("a", Some("First")),
                ("b", Some("Second")),
                ("c", Some("Third")),
            ],
        );
        fake.close_port("a");
        fake.close_port("c");
        let (_orcvs, mut midi) = playing(&fake);

        midi.refresh_destinations();
        midi.select_destination(&MidiDestinationId::new("a"));
        midi.select_destination(&MidiDestinationId::new("b"));
        tokio::task::yield_now().await;

        assert_eq!(
            midi.selected_destination_id(),
            Some(MidiDestinationId::new("b"))
        );
        assert!(fake.closed().is_empty());
        fake.answer_open("a", Ok(()));
        assert_eq!(fake.closed(), vec![("a".to_owned(), 0)]);

        midi.select_destination(&MidiDestinationId::new("c"));
        midi.refresh_destinations();
        fake.answer_open("c", Ok(()));
        assert_eq!(
            fake.closed(),
            vec![("a".to_owned(), 0), ("c".to_owned(), 0)]
        );

        midi.observe_frame();
        tokio::task::yield_now().await;
        assert_eq!(
            midi.selected_destination_id(),
            Some(MidiDestinationId::new("b"))
        );
        assert_eq!(midi.status(), None);
    }
    #[tokio::test]
    async fn replacing_a_source_does_not_close_the_new_backends_connection() {
        let fake = FakeWebMidi::with(AccessStatus::Granted, &[("a", Some("Synth"))]);
        let (old_source, mut old_selection) = playing(&fake);
        old_selection.select_destination(&MidiDestinationId::new("a"));
        tokio::task::yield_now().await;
        let (_new_source, mut new_selection) = playing(&fake);
        new_selection.select_destination(&MidiDestinationId::new("a"));
        tokio::task::yield_now().await;
        drop(old_selection);
        drop(old_source);
        tokio::task::yield_now().await;
        assert!(fake.closed().is_empty(), "the new connection still owns A");
    }

    #[tokio::test]
    async fn dropping_selection_releases_its_pending_open() {
        let fake = FakeWebMidi::with(AccessStatus::Granted, &[("a", Some("Synth"))]);
        fake.close_port("a");
        let (_source, mut selection) = playing(&fake);
        selection.select_destination(&MidiDestinationId::new("a"));
        drop(selection);
        assert!(fake.closed().is_empty());
        fake.answer_open("a", Ok(()));
        assert_eq!(fake.closed(), vec![("a".to_owned(), 0)]);
    }

    #[tokio::test]
    async fn a_stale_replacement_releases_the_previous_request() {
        let fake = FakeWebMidi::with(AccessStatus::Granted, &[("a", None), ("b", None)]);
        fake.close_port("a");
        let (_source, mut selection) = playing(&fake);
        selection.select_destination(&MidiDestinationId::new("a"));
        fake.disconnect("b");
        selection.select_destination(&MidiDestinationId::new("b"));
        assert_eq!(selection.status(), Some(DESTINATION_GONE));
        assert!(fake.closed().is_empty());
        fake.answer_open("a", Ok(()));
        assert_eq!(fake.closed(), vec![("a".to_owned(), 0)]);
    }

    #[tokio::test]
    async fn a_new_selection_reuses_an_abandoned_open_before_it_completes() {
        let fake = FakeWebMidi::with(AccessStatus::Granted, &[("a", None)]);
        fake.close_port("a");
        let (_old_source, mut old_selection) = playing(&fake);
        old_selection.select_destination(&MidiDestinationId::new("a"));
        drop(old_selection);
        let (_new_source, mut new_selection) = playing(&fake);
        new_selection.select_destination(&MidiDestinationId::new("a"));
        assert_eq!(fake.state().opened, ["a"]);
        fake.answer_open("a", Ok(()));
        new_selection.observe_frame();
        tokio::task::yield_now().await;
        assert_eq!(
            new_selection.selected_destination_id(),
            Some(MidiDestinationId::new("a"))
        );
        assert!(fake.closed().is_empty());
    }

    #[tokio::test]
    async fn dropping_one_of_two_pending_owners_keeps_the_other_request() {
        let fake = FakeWebMidi::with(AccessStatus::Granted, &[("a", None)]);
        fake.close_port("a");
        let (_source_a, mut a) = playing(&fake);
        let (_source_b, mut b) = playing(&fake);
        a.select_destination(&MidiDestinationId::new("a"));
        b.select_destination(&MidiDestinationId::new("a"));
        drop(a);
        fake.answer_open("a", Ok(()));
        b.observe_frame();
        tokio::task::yield_now().await;
        assert_eq!(
            b.selected_destination_id(),
            Some(MidiDestinationId::new("a"))
        );
        assert_eq!(fake.state().opened, ["a"]);
        assert!(fake.closed().is_empty());
    }

    #[tokio::test]
    async fn a_pending_destination_that_disappears_releases_its_late_open() {
        let fake = FakeWebMidi::with(AccessStatus::Granted, &[("a", None)]);
        fake.close_port("a");
        let (_source, mut selection) = playing(&fake);
        selection.select_destination(&MidiDestinationId::new("a"));
        fake.disconnect("a");
        selection.observe_frame();
        assert_eq!(selection.status(), Some(DESTINATION_GONE));
        fake.answer_open("a", Ok(()));
        assert_eq!(fake.closed(), vec![("a".to_owned(), 0)]);
    }

    #[tokio::test]
    async fn a_new_claim_waits_for_an_in_flight_close_before_reopening() {
        let fake = FakeWebMidi::with(AccessStatus::Granted, &[("a", None), ("b", None)]);
        fake.state().delay_close = true;
        let (_source, mut selection) = playing(&fake);
        selection.select_destination(&MidiDestinationId::new("a"));
        tokio::task::yield_now().await;
        selection.select_destination(&MidiDestinationId::new("b"));
        tokio::task::yield_now().await;
        selection.select_destination(&MidiDestinationId::new("a"));
        assert_eq!(selection.status(), Some("Opening a…"));
        assert_eq!(fake.state().opened, ["a", "b"]);
        fake.answer_close("a");
        selection.observe_frame();
        tokio::task::yield_now().await;
        assert_eq!(
            selection.selected_destination_id(),
            Some(MidiDestinationId::new("a"))
        );
        assert_eq!(fake.state().opened, ["a", "b", "a"]);
        fake.answer_close("b");
        fake.state().delay_close = false;
    }

    #[tokio::test]
    async fn an_abandoned_refusal_does_not_poison_a_later_request() {
        let fake = FakeWebMidi::with(AccessStatus::Granted, &[("a", None)]);
        fake.close_port("a");
        let (_source, mut selection) = playing(&fake);
        selection.select_destination(&MidiDestinationId::new("a"));
        selection.refresh_destinations();
        fake.answer_open("a", Err("busy"));
        assert!(fake.closed().is_empty());
        selection.select_destination(&MidiDestinationId::new("a"));
        fake.answer_open("a", Ok(()));
        selection.observe_frame();
        tokio::task::yield_now().await;
        assert_eq!(
            selection.selected_destination_id(),
            Some(MidiDestinationId::new("a"))
        );
        assert_eq!(fake.state().opened, ["a", "a"]);
    }
    #[tokio::test]
    async fn choosing_the_installed_output_cancels_without_safety_messages() {
        let fake = FakeWebMidi::with(
            AccessStatus::Granted,
            &[("a", Some("First")), ("b", Some("Second"))],
        );
        fake.close_port("b");
        let (_source, mut selection) = playing(&fake);
        selection.refresh_destinations();
        selection.select_destination(&MidiDestinationId::new("a"));
        tokio::task::yield_now().await;
        fake.clear_sent();
        selection.select_destination(&MidiDestinationId::new("b"));
        selection.select_destination(&MidiDestinationId::new("a"));
        tokio::task::yield_now().await;
        assert!(
            fake.sent_to("a").is_empty(),
            "cancellation must not reinstall A"
        );
        assert_eq!(selection.status(), None);
        fake.answer_open("b", Ok(()));
        selection.observe_frame();
        tokio::task::yield_now().await;
        assert_eq!(
            selection.selected_destination_id(),
            Some(MidiDestinationId::new("a"))
        );
        assert_eq!(fake.closed(), [("b".to_owned(), 0)]);
    }

    #[tokio::test]
    async fn choosing_the_pending_output_keeps_its_completed_request() {
        let fake = FakeWebMidi::with(AccessStatus::Granted, &[("b", Some("Second"))]);
        fake.close_port("b");
        let (_source, mut selection) = playing(&fake);
        selection.refresh_destinations();
        selection.select_destination(&MidiDestinationId::new("b"));
        fake.answer_open("b", Ok(()));
        selection.select_destination(&MidiDestinationId::new("b"));
        selection.observe_frame();
        tokio::task::yield_now().await;
        assert!(
            fake.closed().is_empty(),
            "reselecting must retain the completed request"
        );
        assert_eq!(fake.state().opened, ["b"]);
        assert_eq!(
            selection.selected_destination_id(),
            Some(MidiDestinationId::new("b"))
        );
    }
}
