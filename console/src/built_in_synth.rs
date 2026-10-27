//! The Built-in Synth as a MIDI destination (ADR 0087).
//!
//! A page that frames the console announces the synth its sound module holds
//! with `{ type: "orcvs.synth", available }`. While one is announced the
//! console lists "Built-in Synth" after every MIDI port, and its connection
//! posts each message to the page as `{ type: "orcvs.midi", bytes, time }`,
//! where `time` is the message's Tick's planned instant on the page's clock.
//! The logic is compiled for tests on every target, driven by a fake of the
//! page it reads and posts to.

use std::time::Duration;

use orcvs::midi::{
    MidiBackend, MidiConnection, MidiConnectionRequest, MidiDestination, MidiDestinationId,
    MidiError,
};

use crate::console_midi::DESTINATION_GONE;
use crate::midi::MidiDeviceSelection;

///
/// The synth's destination id and the name the MIDI menu shows.
///
pub(crate) const BUILT_IN_SYNTH: &str = "Built-in Synth";

///
/// What the framing page last said about its synth.
///
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum Announcement {
    /// The page can make sound.
    Available,
    /// The page cannot make sound, for the reason it gives.
    Unavailable(String),
}

///
/// The page that frames the console, as the synth reaches it.
///
/// `Send` because a connection holding one crosses into the Playback Engine's
/// task, as `crate::web_midi::WebMidiAccess` is for the same reason.
///
pub(crate) trait SynthPage: Clone + Send + 'static {
    /// The page's latest announcement, or `None` while it has made none.
    fn announced(&self) -> Option<Announcement>;

    ///
    /// Posts one MIDI message to the page, stamped with the instant its Tick
    /// was planned for: `since_due` before now.
    ///
    fn post(&self, message: &[u8], since_due: Duration) -> Result<(), MidiError>;
}

///
/// A MIDI backend with the Built-in Synth listed after its ports while the
/// page announces one.
///
pub(crate) struct WithBuiltInSynth<B, P> {
    midi: B,
    page: P,
}

impl<B: MidiBackend, P: SynthPage> WithBuiltInSynth<B, P> {
    pub(crate) fn new(midi: B, page: P) -> Self {
        Self { midi, page }
    }
}

impl<B: MidiBackend, P: SynthPage> MidiBackend for WithBuiltInSynth<B, P> {
    ///
    /// The ports, then the synth while it is announced. A discovery still
    /// waiting on the browser's MIDI access lists the synth alone, so a reader
    /// who never answers the permission prompt can still play it; a Scan
    /// lists the ports once access is answered.
    ///
    fn destinations(&mut self) -> Result<Vec<MidiDestination>, MidiError> {
        let ports = self.midi.destinations();
        if self.page.announced().is_none() {
            return ports;
        }
        let mut destinations = match ports {
            Ok(ports) => ports,
            Err(error) if error.is_pending() => Vec::new(),
            Err(error) => return Err(error),
        };
        destinations.push(MidiDestination::new(BUILT_IN_SYNTH, BUILT_IN_SYNTH));
        Ok(destinations)
    }

    fn connect(
        &mut self,
        destination_id: &MidiDestinationId,
    ) -> Result<MidiConnectionRequest, MidiError> {
        if destination_id.as_str() != BUILT_IN_SYNTH {
            return self.midi.connect(destination_id);
        }
        match self.page.announced() {
            Some(Announcement::Available) => Ok(MidiConnectionRequest::Ready(Box::new(
                SynthConnection(self.page.clone()),
            ))),
            Some(Announcement::Unavailable(reason)) => Err(MidiError::new(reason)),
            None => Err(MidiError::new(DESTINATION_GONE)),
        }
    }
}

struct SynthConnection<P>(P);

impl<P: SynthPage> MidiConnection for SynthConnection<P> {
    fn send(&mut self, message: &[u8]) -> Result<(), MidiError> {
        self.0.post(message, Duration::ZERO)
    }

    fn send_due(&mut self, message: &[u8], since_due: Duration) -> Result<(), MidiError> {
        self.0.post(message, since_due)
    }
}

///
/// Follows the page's announcements from frame to frame.
///
pub(crate) struct SynthAnnouncements<P> {
    page: P,
    seen: Option<Announcement>,
    /// Whether the console is Linked, which always picks the synth.
    linked: bool,
}

impl<P: SynthPage> SynthAnnouncements<P> {
    pub(crate) fn new(page: P, linked: bool) -> Self {
        Self {
            page,
            seen: None,
            linked,
        }
    }

    ///
    /// Answers an announcement the last frame had not seen: discovers again,
    /// so the menu lists the synth, and a Linked console picks it. An
    /// unannounced console that finds no port picks it as its first
    /// destination, through `observe_frame`'s automatic selection.
    ///
    pub(crate) fn observe(&mut self, midi: &mut MidiDeviceSelection) {
        let announced = self.page.announced();
        if announced == self.seen {
            return;
        }
        self.seen = announced;
        midi.refresh_destinations();
        if self.linked && self.seen.is_some() {
            midi.select_destination(&MidiDestinationId::new(BUILT_IN_SYNTH));
        }
    }
}

#[cfg(target_arch = "wasm32")]
pub(crate) use browser::BrowserPage;

#[cfg(target_arch = "wasm32")]
mod browser {
    use std::cell::RefCell;
    use std::time::Duration;

    use orcvs::midi::MidiError;
    use wasm_bindgen::JsCast as _;
    use wasm_bindgen::JsValue;
    use wasm_bindgen::closure::Closure;

    use super::{Announcement, SynthPage};

    thread_local! {
        ///
        /// The page's latest announcement. Kept here rather than in
        /// `BrowserPage` because the console and the Playback Engine's task
        /// share the browser's one main thread, and a connection that reads it
        /// has to be `Send`.
        ///
        static ANNOUNCED: RefCell<Option<Announcement>> = const { RefCell::new(None) };

        /// Whether this page is listening for announcements yet.
        static LISTENING: RefCell<bool> = const { RefCell::new(false) };
    }

    ///
    /// The page that frames this console, reached through `window.parent`.
    ///
    #[derive(Clone, Copy)]
    pub(crate) struct BrowserPage;

    impl BrowserPage {
        ///
        /// Listens for the framing page's `orcvs.synth` announcements, and
        /// posts it `{ type: "orcvs.ready" }` so it knows to send one (ADR
        /// 0086). An announcement repaints `ctx`, so the next frame answers
        /// it. A console no page frames listens for nothing. Only the first
        /// call listens; the listener lasts as long as the page.
        ///
        pub(crate) fn listen(ctx: egui::Context) {
            if LISTENING.with(|listening| listening.replace(true)) {
                return;
            }
            let Some((window, parent, origin)) = framing() else {
                return;
            };
            let listened_parent = parent.clone();
            let listened_origin = origin.clone();
            let listener = Closure::<dyn FnMut(web_sys::MessageEvent)>::new(
                move |event: web_sys::MessageEvent| {
                    let from_parent = event
                        .source()
                        .is_some_and(|sender| js_sys::Object::is(&sender, &listened_parent));
                    if !from_parent || event.origin() != listened_origin {
                        return;
                    }
                    if let Some(announcement) = announcement(&event.data()) {
                        ANNOUNCED.with(|announced| *announced.borrow_mut() = Some(announcement));
                        ctx.request_repaint();
                    }
                },
            );
            if let Err(reason) = window
                .add_event_listener_with_callback("message", listener.as_ref().unchecked_ref())
            {
                log::warn!("the console cannot listen for the Built-in Synth: {reason:?}");
                return;
            }
            let ready = js_sys::Object::new();
            // Setting a field on a fresh plain object cannot fail.
            let _ = js_sys::Reflect::set(&ready, &"type".into(), &"orcvs.ready".into());
            if let Err(reason) = parent.post_message(&ready, &origin) {
                log::warn!("the console cannot tell its page it is ready: {reason:?}");
            }
            // The page holds the listener for its whole life.
            listener.forget();
        }
    }

    ///
    /// This console's window, the page that frames it, and the console's own
    /// origin, or `None` when no page frames it.
    ///
    fn framing() -> Option<(web_sys::Window, web_sys::Window, String)> {
        let window = web_sys::window()?;
        let origin = window.location().origin().ok()?;
        let parent = window.parent().ok().flatten()?;
        if js_sys::Object::is(&parent, &window) {
            return None;
        }
        Some((window, parent, origin))
    }

    ///
    /// The announcement `data` makes, or `None` when it is not one: a message
    /// of another type, or one whose `available` is not a boolean.
    ///
    fn announcement(data: &JsValue) -> Option<Announcement> {
        let field = |name: &str| js_sys::Reflect::get(data, &name.into()).ok();
        if field("type")?.as_string().as_deref() != Some("orcvs.synth") {
            return None;
        }
        if field("available")?.as_bool()? {
            return Some(Announcement::Available);
        }
        let reason = field("reason")
            .and_then(|reason| reason.as_string())
            .unwrap_or_else(|| "this page cannot make sound".to_owned());
        Some(Announcement::Unavailable(reason))
    }

    impl SynthPage for BrowserPage {
        fn announced(&self) -> Option<Announcement> {
            ANNOUNCED.with(|announced| announced.borrow().clone())
        }

        fn post(&self, message: &[u8], since_due: Duration) -> Result<(), MidiError> {
            let (window, parent, origin) =
                framing().ok_or_else(|| MidiError::new(super::DESTINATION_GONE))?;
            let performance = window
                .performance()
                .ok_or_else(|| MidiError::new("this page has no clock to time a message by"))?;
            // `performance.timeOrigin` plus `performance.now()` is the one
            // clock this console's frame and its page share.
            let time =
                performance.time_origin() + performance.now() - since_due.as_secs_f64() * 1_000.0;
            let bytes = js_sys::Array::new();
            for byte in message {
                bytes.push(&JsValue::from(*byte));
            }
            let posted = js_sys::Object::new();
            // Setting fields on a fresh plain object cannot fail.
            let _ = js_sys::Reflect::set(&posted, &"type".into(), &"orcvs.midi".into());
            let _ = js_sys::Reflect::set(&posted, &"bytes".into(), &bytes);
            let _ = js_sys::Reflect::set(&posted, &"time".into(), &JsValue::from(time));
            parent
                .post_message(&posted, &origin)
                .map_err(|reason| MidiError::new(format!("{reason:?}")))
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};
    use std::time::Duration;

    use orcvs::app::{InputEvent, InputKey, Orcvs};
    use orcvs::midi::{
        MidiBackend, MidiConnection, MidiConnectionRequest, MidiDestination, MidiDestinationId,
        MidiError, MidiOutputAdapter,
    };

    use super::{Announcement, BUILT_IN_SYNTH, SynthAnnouncements, SynthPage, WithBuiltInSynth};
    use crate::console_midi::DESTINATION_GONE;
    use crate::midi::MidiDeviceSelection;

    #[derive(Default)]
    struct PageState {
        announced: Option<Announcement>,
        posted: Vec<(Vec<u8>, Duration)>,
    }

    #[derive(Clone, Default)]
    struct FakePage(Arc<Mutex<PageState>>);

    impl FakePage {
        fn announce(&self, announcement: Announcement) {
            self.0.lock().unwrap().announced = Some(announcement);
        }

        fn posted(&self) -> Vec<(Vec<u8>, Duration)> {
            self.0.lock().unwrap().posted.clone()
        }
    }

    impl SynthPage for FakePage {
        fn announced(&self) -> Option<Announcement> {
            self.0.lock().unwrap().announced.clone()
        }

        fn post(&self, message: &[u8], since_due: Duration) -> Result<(), MidiError> {
            self.0
                .lock()
                .unwrap()
                .posted
                .push((message.to_vec(), since_due));
            Ok(())
        }
    }

    ///
    /// MIDI ports as discovery answers them: a list, or an error.
    ///
    struct Ports(Result<Vec<MidiDestination>, MidiError>);

    struct Port;

    impl MidiConnection for Port {
        fn send(&mut self, _message: &[u8]) -> Result<(), MidiError> {
            Ok(())
        }
    }

    impl MidiBackend for Ports {
        fn destinations(&mut self) -> Result<Vec<MidiDestination>, MidiError> {
            self.0.clone()
        }

        fn connect(
            &mut self,
            destination_id: &MidiDestinationId,
        ) -> Result<MidiConnectionRequest, MidiError> {
            match &self.0 {
                Ok(ports) if ports.iter().any(|port| &port.id == destination_id) => {
                    Ok(MidiConnectionRequest::Ready(Box::new(Port)))
                }
                _ => Err(MidiError::new(DESTINATION_GONE)),
            }
        }
    }

    fn port() -> MidiDestination {
        MidiDestination::new("port", "A port")
    }

    fn synth() -> MidiDestination {
        MidiDestination::new(BUILT_IN_SYNTH, BUILT_IN_SYNTH)
    }

    fn with_synth(
        ports: Result<Vec<MidiDestination>, MidiError>,
        page: &FakePage,
    ) -> WithBuiltInSynth<Ports, FakePage> {
        WithBuiltInSynth::new(Ports(ports), page.clone())
    }

    fn ready(request: MidiConnectionRequest) -> Box<dyn MidiConnection> {
        match request {
            MidiConnectionRequest::Ready(connection) => connection,
            MidiConnectionRequest::Pending(_) => panic!("the synth opens at once"),
        }
    }

    #[test]
    fn the_synth_is_listed_after_the_ports_only_while_a_page_announces_it() {
        let page = FakePage::default();
        let mut backend = with_synth(Ok(vec![port()]), &page);
        assert_eq!(backend.destinations(), Ok(vec![port()]));

        page.announce(Announcement::Available);
        assert_eq!(backend.destinations(), Ok(vec![port(), synth()]));

        page.announce(Announcement::Unavailable("no audio".to_owned()));
        assert_eq!(backend.destinations(), Ok(vec![port(), synth()]));
    }

    #[test]
    fn the_synth_is_listed_while_midi_access_is_pending() {
        let page = FakePage::default();
        page.announce(Announcement::Available);
        let mut backend = with_synth(Err(MidiError::pending("waiting")), &page);

        assert_eq!(backend.destinations(), Ok(vec![synth()]));
    }

    #[test]
    fn each_message_is_posted_with_how_late_its_tick_was() {
        let page = FakePage::default();
        page.announce(Announcement::Available);
        let mut backend = with_synth(Ok(Vec::new()), &page);
        let mut connection = ready(
            backend
                .connect(&MidiDestinationId::new(BUILT_IN_SYNTH))
                .expect("an announced synth connects"),
        );

        connection
            .send_due(&[0x90, 60, 0x7F], Duration::from_millis(12))
            .unwrap();
        connection.send(&[0xB0, 123, 0]).unwrap();

        assert_eq!(
            page.posted(),
            vec![
                (vec![0x90, 60, 0x7F], Duration::from_millis(12)),
                (vec![0xB0, 123, 0], Duration::ZERO),
            ]
        );
    }

    #[test]
    fn an_unavailable_synth_answers_its_reason_on_connect() {
        let page = FakePage::default();
        let mut backend = with_synth(Ok(Vec::new()), &page);
        let synth_id = MidiDestinationId::new(BUILT_IN_SYNTH);
        assert_eq!(
            backend.connect(&synth_id).err().map(|error| error.message),
            Some(DESTINATION_GONE.to_owned())
        );

        page.announce(Announcement::Unavailable("audio is blocked".to_owned()));
        assert_eq!(
            backend.connect(&synth_id).err().map(|error| error.message),
            Some("audio is blocked".to_owned())
        );
    }

    fn selection(ports: Vec<MidiDestination>, page: &FakePage) -> (Orcvs, MidiDeviceSelection) {
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
        let mut midi = MidiDeviceSelection::new(
            orcvs.midi_selection_handle(),
            Box::new(with_synth(Ok(ports), page)),
        );
        midi.refresh_destinations();
        (orcvs, midi)
    }

    async fn frame(
        announcements: &mut SynthAnnouncements<FakePage>,
        midi: &mut MidiDeviceSelection,
    ) {
        announcements.observe(midi);
        midi.observe_frame();
        tokio::task::yield_now().await;
    }

    #[tokio::test]
    async fn a_linked_console_picks_the_synth_over_a_port() {
        let page = FakePage::default();
        let (_orcvs, mut midi) = selection(vec![port()], &page);
        let mut announcements = SynthAnnouncements::new(page.clone(), true);
        frame(&mut announcements, &mut midi).await;
        assert_eq!(midi.selected_destination_id(), Some(port().id));

        page.announce(Announcement::Available);
        frame(&mut announcements, &mut midi).await;

        assert_eq!(midi.destinations(), &[port(), synth()]);
        assert_eq!(midi.selected_destination_id(), Some(synth().id));
    }

    #[tokio::test]
    async fn an_unlinked_console_picks_the_synth_only_without_a_port() {
        for (ports, picked) in [(vec![port()], port().id), (Vec::new(), synth().id)] {
            let page = FakePage::default();
            page.announce(Announcement::Available);
            let (_orcvs, mut midi) = selection(ports, &page);
            let mut announcements = SynthAnnouncements::new(page.clone(), false);

            frame(&mut announcements, &mut midi).await;

            assert_eq!(midi.selected_destination_id(), Some(picked));
        }
    }

    #[tokio::test]
    async fn a_linked_console_shows_why_an_unavailable_synth_cannot_play() {
        let page = FakePage::default();
        page.announce(Announcement::Unavailable("audio is blocked".to_owned()));
        let (_orcvs, mut midi) = selection(Vec::new(), &page);
        let mut announcements = SynthAnnouncements::new(page.clone(), true);

        frame(&mut announcements, &mut midi).await;

        assert_eq!(midi.selected_destination_id(), None);
        assert_eq!(midi.status(), Some("audio is blocked"));
    }

    #[tokio::test]
    async fn playback_posts_its_notes_to_the_page() {
        let page = FakePage::default();
        page.announce(Announcement::Available);
        let (mut orcvs, mut midi) = selection(Vec::new(), &page);
        let mut announcements = SynthAnnouncements::new(page.clone(), true);
        frame(&mut announcements, &mut midi).await;

        orcvs.event_handler(vec![InputEvent::KeyPressed(InputKey::Space)]);
        for _ in 0..10 {
            tokio::task::yield_now().await;
        }

        let posted = page.posted();
        assert_eq!(
            posted.first().map(|(bytes, _)| bytes.clone()),
            Some(vec![0x90, 60, 0x7F])
        );
    }
}
