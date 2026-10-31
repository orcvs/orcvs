//! Keyboard routing: which keys reach the Source and which a control holds,
//! and the translation of the toolkit's events into the Source's input and
//! the File commands' chords.

use egui::{Event, EventFilter, Key};
use orcvs::app::{Arrow, InputEvent, InputKey};

use super::Console;

fn keep_digits_in_text_events(events: &mut Vec<egui::Event>) {
    for event in events.iter_mut() {
        match event {
            egui::Event::Text(text) | egui::Event::Paste(text) => {
                text.retain(|c| c.is_ascii_digit());
            }
            _ => {}
        }
    }
    events.retain(|event| match event {
        egui::Event::Text(text) | egui::Event::Paste(text) => !text.is_empty(),
        _ => true,
    });
}

pub(super) fn translate_event(event: Event) -> Option<InputEvent> {
    match event {
        // Shift with an arrow keeps the anchor and extends the Region; a bare
        // arrow falls through to the arm below and collapses it.
        Event::Key {
            key,
            pressed: true,
            modifiers,
            ..
        } if modifiers.shift && arrow(key).is_some() => arrow(key).map(InputEvent::Extend),
        Event::Key {
            key: Key::A,
            pressed: true,
            modifiers,
            ..
        } if modifiers.command => Some(InputEvent::SelectAll),
        Event::Key {
            key: Key::Enter,
            pressed: true,
            modifiers,
            ..
        } if modifiers.command => Some(InputEvent::Fill),
        // Command Shift R rewinds Playback to Tick 0, Orca's chord for the
        // same. Native only: the browser reserves the chord for a reload that
        // bypasses its cache, and a page reload begins at Tick 0 regardless.
        // A held chord rewinds once.
        #[cfg(not(target_arch = "wasm32"))]
        Event::Key {
            key: Key::R,
            pressed: true,
            repeat,
            modifiers,
            ..
        } if modifiers.command && modifiers.shift && !modifiers.alt => {
            (!repeat).then_some(InputEvent::Rewind)
        }
        // Tab steps the Cursor a Sector at a time; Shift Tab steps it back.
        // `Console::keep_tab_for_source` cancels egui's own Tab focus
        // navigation for the same press whenever the Source holds the keys,
        // so this and that cancellation are two views of the one rule: Tab belongs to the
        // Source, not to focus. Only a bare Tab and a bare Shift Tab are
        // either of those — `modifiers.is_none()` and `modifiers.shift_only()`
        // are the same tests `Memory::begin_pass` uses to move focus,
        // so Ctrl, Command, or Alt held with Tab reaches neither egui's focus
        // navigation nor the Source here.
        Event::Key {
            key: Key::Tab,
            pressed: true,
            modifiers,
            ..
        } if modifiers.is_none() || modifiers.shift_only() => Some(if modifiers.shift_only() {
            InputEvent::PreviousSector
        } else {
            InputEvent::KeyPressed(InputKey::Tab)
        }),
        Event::Key {
            key, pressed: true, ..
        } => match key {
            Key::ArrowDown => Some(InputEvent::KeyPressed(InputKey::ArrowDown)),
            Key::ArrowLeft => Some(InputEvent::KeyPressed(InputKey::ArrowLeft)),
            Key::ArrowRight => Some(InputEvent::KeyPressed(InputKey::ArrowRight)),
            Key::ArrowUp => Some(InputEvent::KeyPressed(InputKey::ArrowUp)),
            Key::Backspace => Some(InputEvent::KeyPressed(InputKey::Backspace)),
            Key::Delete => Some(InputEvent::KeyPressed(InputKey::Delete)),
            Key::Space => Some(InputEvent::KeyPressed(InputKey::Space)),
            Key::Escape => Some(InputEvent::Collapse),
            _ => None,
        },
        Event::Text(text) => Some(InputEvent::Text(text)),
        Event::Copy => Some(InputEvent::Copy),
        Event::Cut => Some(InputEvent::Cut),
        Event::Paste(text) => Some(InputEvent::Paste(text)),
        // The running Orcvs models only input it acts on; all other toolkit
        // events remain presentation concerns and are dropped here.
        _ => None,
    }
}

///
/// The direction an arrow key moves the Cursor, or `None` for any other key.
///
fn arrow(key: Key) -> Option<Arrow> {
    match key {
        Key::ArrowDown => Some(Arrow::Down),
        Key::ArrowLeft => Some(Arrow::Left),
        Key::ArrowRight => Some(Arrow::Right),
        Key::ArrowUp => Some(Arrow::Up),
        _ => None,
    }
}

///
/// A File menu command, chosen from the menu or, on native, by its chord.
///
/// The web binds no chord, since the browser reserves ⌘N and ⌘Q, and offers
/// New alone.
///
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum FileCommand {
    New,
    #[cfg(not(target_arch = "wasm32"))]
    Open,
    #[cfg(not(target_arch = "wasm32"))]
    Save,
    #[cfg(not(target_arch = "wasm32"))]
    SaveAs,
}

impl FileCommand {
    /// Every command a chord reaches.
    #[cfg(not(target_arch = "wasm32"))]
    const CHORDED: [Self; 4] = [Self::New, Self::Open, Self::Save, Self::SaveAs];

    /// The File menu item's label.
    pub(super) fn label(self) -> &'static str {
        match self {
            Self::New => "New",
            #[cfg(not(target_arch = "wasm32"))]
            Self::Open => "Open…",
            #[cfg(not(target_arch = "wasm32"))]
            Self::Save => "Save",
            #[cfg(not(target_arch = "wasm32"))]
            Self::SaveAs => "Save As…",
        }
    }

    ///
    /// The chord that runs this command and that its File menu item shows:
    /// what [`file_command`] answers.
    ///
    #[cfg(not(target_arch = "wasm32"))]
    pub(super) fn shortcut(self) -> egui::KeyboardShortcut {
        let (modifiers, key) = match self {
            Self::New => (egui::Modifiers::COMMAND, Key::N),
            Self::Open => (egui::Modifiers::COMMAND, Key::O),
            Self::Save => (egui::Modifiers::COMMAND, Key::S),
            Self::SaveAs => (egui::Modifiers::COMMAND | egui::Modifiers::SHIFT, Key::S),
        };
        egui::KeyboardShortcut::new(modifiers, key)
    }
}

///
/// The chord that shows or hides Diagnostics, on native, where `View →
/// Diagnostics` shows it. The web binds none: the browser reserves ⌘D.
///
pub(super) const DIAGNOSTICS_SHORTCUT: egui::KeyboardShortcut =
    egui::KeyboardShortcut::new(egui::Modifiers::COMMAND, Key::D);

///
/// Whether a toolkit event is a fresh press of `shortcut`: its key with exactly
/// its Shift and Alt and the command modifier, and no key repeat.
///
/// Shift and Alt match exactly: do not use `InputState::consume_shortcut`,
/// whose logical match lets a held Shift through, so ⌘⇧S would also be Save.
///
#[cfg(not(target_arch = "wasm32"))]
fn is_fresh_press_of(shortcut: egui::KeyboardShortcut, event: &Event) -> bool {
    matches!(
        event,
        Event::Key {
            key,
            pressed: true,
            repeat: false,
            modifiers,
            ..
        } if *key == shortcut.logical_key && modifiers.matches_exact(shortcut.modifiers)
    )
}

///
/// Whether a toolkit event is [`DIAGNOSTICS_SHORTCUT`], by
/// [`is_fresh_press_of`].
///
#[cfg(not(target_arch = "wasm32"))]
fn toggles_diagnostics(event: &Event) -> bool {
    is_fresh_press_of(DIAGNOSTICS_SHORTCUT, event)
}

///
/// The File command whose [`FileCommand::shortcut`] a toolkit event is, by
/// [`is_fresh_press_of`], or none: ⌘⇧S is Save As and ⌘⇧N is nothing.
///
#[cfg(not(target_arch = "wasm32"))]
pub(super) fn file_command(event: &Event) -> Option<FileCommand> {
    FileCommand::CHORDED
        .into_iter()
        .find(|command| is_fresh_press_of(command.shortcut(), event))
}

impl Console {
    ///
    /// Keeps Tab for the Source while it holds the keys, before any widget is
    /// shown.
    ///
    /// `Memory::begin_pass` already turned an unmodified Tab into
    /// `FocusDirection::Next` and a Shift Tab into `FocusDirection::Previous`
    /// before the frame runs, and the first focusable widget shown — a
    /// menu-bar button — would otherwise claim it the moment it is shown.
    /// Cancelling before anything is shown is what
    /// keeps Tab off every widget rather than only the ones drawn after the
    /// call. `!self.keyboard_elsewhere` is last frame's answer, the same one
    /// [`Console::route_keys`] reads, so a focused control or an open menu
    /// keeps egui's own Tab navigation and the Source gets none of it — a
    /// viewer in a menu reaches its controls by Tab alone, because a pointer
    /// click there closes the menu first.
    ///
    pub(super) fn keep_tab_for_source(&self, ctx: &egui::Context) {
        if !self.keyboard_elsewhere {
            ctx.memory_mut(|memory| memory.move_focus(egui::FocusDirection::None));
        }
    }

    ///
    /// Routes this frame's keys: to the Source while it holds them, or away
    /// from it while a control does. Answers the File command a chord asked
    /// for, which only native binds.
    ///
    /// Keys belong to whichever control held focus when last frame's widgets
    /// were done. Last frame's, because this frame's events are collected
    /// before the widgets are shown and would otherwise write Source or toggle
    /// Playback in the same pass a control is already editing. egui offers no
    /// per-event answer to whether a widget used an event — `TextEdit` reads
    /// its events through `InputState::filtered_events`, which leaves them in
    /// place — and `RawInput::events` names `egui_wants_keyboard_input` as the
    /// question to ask instead.
    ///
    pub(super) fn route_keys(&mut self, ctx: &egui::Context) -> Option<FileCommand> {
        let event_filter = EventFilter {
            tab: true,
            horizontal_arrows: true,
            vertical_arrows: true,
            escape: true,
        };

        if ctx.memory(|memory| {
            memory.had_focus_last_frame(egui::Id::unique(super::panel::BPM_FIELD_ID))
        }) {
            ctx.input_mut(|i| keep_digits_in_text_events(&mut i.events));
        }
        if self.keyboard_elsewhere {
            // Keys a control took are still the event that follows a command
            // Enter, so they disarm its fill as one reaching the Source would.
            self.orcvs.disarm_fill();
            return None;
        }
        // A command `+`, `=`, `-` or `0` chord is egui's whole-UI zoom, which
        // `Context::end_pass` reads, and never the Source: `translate_event`
        // maps none of those keys to Source input, and `egui-winit`'s
        // `State::on_keyboard_input` and eframe's web keydown handler both
        // withhold `Event::Text` while a command modifier is held, so a
        // shipped build never raises the bare character alongside the chord.
        let events: Vec<_> = ctx.input(|i| {
            i.filtered_events(&event_filter)
                .into_iter()
                .filter_map(translate_event)
                .collect()
        });
        #[cfg(target_arch = "wasm32")]
        let events = self.hold_start_for_audio(ctx, events);
        // A Copy or Cut answers the text the platform clipboard is to
        // hold; `copy_text` is how egui hands it to the native and the
        // browser backend alike.
        if let Some(copied) = self.orcvs.event_handler(events).copied {
            ctx.copy_text(copied);
        }
        // A File chord runs `run_file_command`; `translate_event` already
        // keeps command-modified letters from the Source.
        #[cfg(not(target_arch = "wasm32"))]
        {
            if ctx.input(|i| i.events.iter().any(toggles_diagnostics)) {
                self.diagnostics_open = !self.diagnostics_open;
            }
            ctx.input(|i| i.events.iter().find_map(file_command))
        }
        #[cfg(target_arch = "wasm32")]
        {
            None
        }
    }

    ///
    /// `events` without a Space that starts Playback on the Built-in Synth
    /// while the page's audio is asleep, and with it again once the audio
    /// runs. The page announces its audio with a repaint; a timed repaint ends
    /// the wait without it.
    ///
    #[cfg(target_arch = "wasm32")]
    fn hold_start_for_audio(
        &mut self,
        ctx: &egui::Context,
        mut events: Vec<InputEvent>,
    ) -> Vec<InputEvent> {
        let stopped =
            self.orcvs.playback_observation().state == orcvs::playback::PlaybackState::Stopped;
        let now = ctx.input(|i| i.time);
        if self
            .synth
            .hold_start(&mut events, &mut self.midi, stopped, now)
        {
            ctx.request_repaint_after(std::time::Duration::from_millis(20));
        }
        events
    }

    ///
    /// Latches whether keyboard input belongs to a control rather than the
    /// Source, for the next frame's [`Console::keep_tab_for_source`] and
    /// [`Console::route_keys`].
    ///
    /// Sampled once every widget has taken or surrendered focus and every
    /// popup has opened or closed. A showing confirmation holds the keys.
    ///
    pub(super) fn latch_keyboard_owner(&mut self, ctx: &egui::Context) {
        self.keyboard_elsewhere = ctx.egui_wants_keyboard_input()
            || egui::Popup::is_any_open(ctx)
            || self.discard_confirmation.is_some();
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use egui::{Event, Key, Modifiers};

    use super::toggles_diagnostics;

    fn key(key: Key, modifiers: Modifiers, repeat: bool) -> Event {
        Event::Key {
            key,
            physical_key: None,
            pressed: true,
            repeat,
            modifiers,
        }
    }

    #[test]
    fn only_a_fresh_command_d_toggles_diagnostics() {
        let cases = [
            ("⌘D", key(Key::D, Modifiers::COMMAND, false), true),
            (
                "⌘D on macOS",
                key(Key::D, Modifiers::MAC_CMD | Modifiers::COMMAND, false),
                true,
            ),
            (
                "⌘⇧D",
                key(Key::D, Modifiers::COMMAND | Modifiers::SHIFT, false),
                false,
            ),
            (
                "⌘⌥D",
                key(Key::D, Modifiers::COMMAND | Modifiers::ALT, false),
                false,
            ),
            (
                "a repeated ⌘D",
                key(Key::D, Modifiers::COMMAND, true),
                false,
            ),
            ("a bare D", key(Key::D, Modifiers::NONE, false), false),
        ];
        for (chord, event, toggles) in cases {
            assert_eq!(toggles_diagnostics(&event), toggles, "{chord}");
        }
    }
}
