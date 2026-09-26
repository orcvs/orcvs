//! Keyboard routing: which keys reach the Source and which a control holds,
//! and the translation of the toolkit's events into the Source's input, the
//! Source View's Zoom commands and the File commands' chords.

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
        // Tab steps the Cursor a Sector at a time; Shift Tab steps it back.
        // `Console::keep_tab_for_source` cancels egui's own Tab focus
        // navigation for the same press whenever the Source holds the keys,
        // so this and that cancellation are two views of the one rule: Tab belongs to the
        // Source, not to focus. Only a bare Tab and a bare Shift Tab are
        // either of those — `modifiers.is_none()` and `modifiers.shift_only()`
        // are the same tests `Memory::begin_pass` itself uses to turn a Tab
        // into `FocusDirection::Next`/`Previous` (`egui-0.36.2/src/memory/mod.rs:596-597`),
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
/// A keyboard Zoom command: a command chord for `=`/`+`, `-`, or `0`.
///
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ZoomCommand {
    In,
    Out,
    Reset,
}

impl ZoomCommand {
    pub(super) const ALL: [Self; 3] = [Self::In, Self::Out, Self::Reset];

    ///
    /// The key that, with a command modifier, is this command's chord: what
    /// [`zoom_command`] answers and the View menu shows. `+` is also Zoom In,
    /// since it shares `=`'s key on most layouts.
    ///
    pub(super) fn key(self) -> Key {
        match self {
            Self::In => Key::Equals,
            Self::Out => Key::Minus,
            Self::Reset => Key::Num0,
        }
    }

    /// The chord as the View menu shows it.
    pub(super) fn shortcut(self) -> egui::KeyboardShortcut {
        egui::KeyboardShortcut::new(egui::Modifiers::COMMAND, self.key())
    }

    /// The View menu item's label.
    pub(super) fn label(self) -> &'static str {
        match self {
            Self::In => "Zoom In",
            Self::Out => "Zoom Out",
            Self::Reset => "Reset Zoom",
        }
    }
}

///
/// The Zoom command a toolkit event asks for, or none.
///
/// Only a held [`egui::Modifiers::command`] turns `=`, `+`, `-` or `0` into a
/// Zoom step. Bare, they are Source characters — [`translate_event`] reaches
/// them as [`Event::Text`], never through this — so this answers `None` for
/// an unmodified key and
/// [`show_source_scene`](super::source_view::show_source_scene) leaves the
/// Zoom exactly where it was.
///
pub(super) fn zoom_command(event: &Event) -> Option<ZoomCommand> {
    match event {
        Event::Key {
            key,
            pressed: true,
            modifiers,
            ..
        } if modifiers.command => match key {
            Key::Plus => Some(ZoomCommand::In),
            key => ZoomCommand::ALL
                .into_iter()
                .find(|command| command.key() == *key),
        },
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
/// The File command a toolkit event's chord asks for, or none.
///
/// Matches the command modifier with exactly the shortcut's Shift and no Alt,
/// so ⌘⇧S is Save As and ⌘⇧N is nothing. Key repeats run nothing.
///
#[cfg(not(target_arch = "wasm32"))]
pub(super) fn file_command(event: &Event) -> Option<FileCommand> {
    match event {
        Event::Key {
            key,
            pressed: true,
            repeat: false,
            modifiers,
            ..
        } if modifiers.command && !modifiers.alt => {
            FileCommand::CHORDED.into_iter().find(|command| {
                let shortcut = command.shortcut();
                shortcut.logical_key == *key && shortcut.modifiers.shift == modifiers.shift
            })
        }
        _ => None,
    }
}

impl Console {
    ///
    /// Keeps Tab for the Source while it holds the keys, before any widget is
    /// shown.
    ///
    /// `Memory::begin_pass` already turned an unmodified Tab into
    /// `FocusDirection::Next` and a Shift Tab into `FocusDirection::Previous`
    /// before the frame runs (`egui-0.36.2/src/memory/mod.rs:596-597`), and the
    /// first focusable widget shown — a menu-bar button — would otherwise claim
    /// it the moment it is shown. Cancelling before anything is shown is what
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
    /// its events without consuming them
    /// (`egui-0.36.2/src/widgets/text_edit/builder.rs:1098`) — and names
    /// `egui_wants_keyboard_input` as the question to ask instead
    /// (`egui-0.36.2/src/data/input/raw_input.rs:56-60`).
    ///
    pub(super) fn route_keys(&mut self, ctx: &egui::Context) -> Option<FileCommand> {
        let event_filter = EventFilter {
            tab: true,
            horizontal_arrows: true,
            vertical_arrows: true,
            escape: true,
        };

        if ctx
            .memory(|memory| memory.had_focus_last_frame(egui::Id::new(super::panel::BPM_FIELD_ID)))
        {
            ctx.input_mut(|i| keep_digits_in_text_events(&mut i.events));
        }
        if self.keyboard_elsewhere {
            // Keys a control took are still the event that follows a command
            // Enter, so they disarm its fill as one reaching the Source would.
            self.orcvs.disarm_fill();
            // `show_source_scene` reads Zoom chords, so drop them here while
            // the keys are elsewhere, such as a discard confirmation.
            ctx.input_mut(|i| i.events.retain(|event| zoom_command(event).is_none()));
            return None;
        }
        // A command Zoom chord answers `show_source_scene`, not the
        // Source. `egui-winit` and eframe's web backend both withhold
        // `Event::Text` while a command modifier is held
        // (`egui-winit-0.36.2/src/lib.rs:1059-1065`,
        // `eframe-0.36.2/src/web/events.rs:155-162`), so a shipped build
        // never raises the matching bare character alongside the chord
        // that already answered it.
        let events = ctx.input(|i| {
            i.filtered_events(&event_filter)
                .into_iter()
                .filter_map(translate_event)
                .collect()
        });
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
            ctx.input(|i| i.events.iter().find_map(file_command))
        }
        #[cfg(target_arch = "wasm32")]
        {
            None
        }
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
