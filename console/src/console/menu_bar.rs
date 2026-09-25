//! The top bar: the File, View and Help menus, the Notices menu, the
//! persistence notice and the mode control.

use super::Console;
#[cfg(not(target_arch = "wasm32"))]
use super::files::FUNCTION_REFERENCE_CONFIRMATION;
use super::input::{FileCommand, ZoomCommand};
use crate::theme_selection::SelectedThemes;

/// The height the top panel takes from the window, leaving the rest to the
/// console. It is the panel's own minimum, which the menu bar does not exceed.
pub(super) const TOP_PANEL_HEIGHT: f32 = 32.0;

/// The horizontal gap between the top menu bar's own items — its menu
/// buttons and the persistence notice.
pub(super) const MENU_BAR_GAP: f32 = 16.0;

///
/// A File menu item for `command`, carrying its chord as shortcut text on
/// native, where the chord is bound. Answers whether it was chosen.
///
fn file_menu_item(ui: &mut egui::Ui, command: FileCommand) -> bool {
    let item = egui::Button::new(command.label());
    #[cfg(not(target_arch = "wasm32"))]
    let item = item.shortcut_text(ui.ctx().format_shortcut(&command.shortcut()));
    ui.add(item).clicked()
}

///
/// The top bar's Notices menu, when there are any: the Theme and settings
/// notices, then the Source File notices in `files`, with a Dismiss button.
///
fn show_notices(ui: &mut egui::Ui, themes: &mut SelectedThemes, files: &mut Vec<String>) {
    let count = themes.notice_count() + files.len();
    if count == 0 {
        return;
    }
    ui.add_space(MENU_BAR_GAP);
    let title =
        egui::RichText::new(format!("Notices ({count})")).color(ui.visuals().error_fg_color);
    ui.menu_button(title, |ui| {
        ui.set_max_width(THEME_NOTICE_WIDTH);
        egui::ScrollArea::vertical()
            .max_height(THEME_NOTICE_HEIGHT)
            .show(ui, |ui| {
                for notice in themes.notices().chain(files.iter()) {
                    ui.label(notice);
                }
            });
        if ui.button("Dismiss").clicked() {
            themes.dismiss_notices();
            files.clear();
            ui.close();
        }
    });
}

/// How wide the notices menu grows before its messages wrap.
const THEME_NOTICE_WIDTH: f32 = 480.0;
/// How tall the notices list grows before it scrolls.
const THEME_NOTICE_HEIGHT: f32 = 320.0;

///
/// A viewer's change to the console's appearance, made with the top bar's
/// mode control. The dark and light Themes themselves are settings.
///
#[derive(Debug)]
pub(super) enum AppearanceChange {
    /// Follow the operating system's appearance, or hold dark or light.
    Mode(egui::ThemePreference),
}

///
/// The mode control's glyphs: Follow the OS, Dark, Light. Each must be in the
/// console's one font (`kittest_tests::the_mode_glyphs_are_in_the_console_font`).
///
pub(super) const MODE_GLYPHS: [&str; 3] = ["◐", "☾", "☼"];

///
/// The top bar's mode control: Follow the OS, Dark and Light as icon-only
/// selectable buttons, each named for accessibility and explained on hover.
/// Answers the choice a viewer made this frame, if any.
///
/// Shown into a right-to-left layout, so the buttons are added in reverse.
///
fn mode_control(ui: &mut egui::Ui) -> Option<AppearanceChange> {
    let mode = ui.ctx().options(|options| options.theme_preference);
    let system = ui.input(|input| input.raw.system_theme);
    let choices = [
        (
            egui::ThemePreference::System,
            "Follow the OS",
            "Follow the operating system's appearance.",
        ),
        (
            egui::ThemePreference::Dark,
            "Dark",
            "Always use the dark appearance.",
        ),
        (
            egui::ThemePreference::Light,
            "Light",
            "Always use the light appearance.",
        ),
    ];
    let mut change = None;
    for ((preference, name, explanation), glyph) in choices.into_iter().zip(MODE_GLYPHS).rev() {
        let selected = mode == preference;
        let response = ui.selectable_label(selected, glyph);
        response.widget_info(|| {
            egui::WidgetInfo::selected(egui::WidgetType::Button, true, selected, name)
        });
        let response = response.on_hover_ui(|ui| {
            ui.label(explanation);
            if preference == egui::ThemePreference::System {
                ui.label(match system {
                    Some(egui::Theme::Dark) => "The operating system's appearance is dark.",
                    Some(egui::Theme::Light) => "The operating system's appearance is light.",
                    None => "The operating system's appearance is unknown.",
                });
            }
        });
        if response.clicked() {
            change = Some(AppearanceChange::Mode(preference));
        }
    }
    change
}

impl Console {
    ///
    /// Applies an appearance change. A mode is egui's own `ThemePreference`,
    /// which egui memory holds and eframe persists.
    ///
    pub(super) fn change_appearance(ctx: &egui::Context, change: AppearanceChange) {
        match change {
            AppearanceChange::Mode(preference) => ctx.set_theme(preference),
        }
        ctx.request_repaint();
    }
}

///
/// What the viewer chose in the top bar this frame, for the frame to act on
/// once the bar is done.
///
pub(super) struct MenuBarChoice {
    /// A File command from the File menu, run once the chords are read.
    pub(super) file_command: Option<FileCommand>,
    /// A mode from the mode control, applied once the frame is done so no
    /// frame mixes two Themes.
    pub(super) appearance_change: Option<AppearanceChange>,
}

impl Console {
    ///
    /// Shows the top bar: File, View and Help, then at its right edge the
    /// mode control, the Notices menu and the persistence notice.
    ///
    /// A View menu Zoom is handed to the Source View for this frame's
    /// `show_source_scene`, which runs after the bar, as the chord is. Help's
    /// Function Reference asks before discarding on native at once; the File
    /// command and the mode are answered for the frame to run.
    ///
    pub(super) fn show_menu_bar(&mut self, root: &mut egui::Ui) -> MenuBarChoice {
        let mut choice = MenuBarChoice {
            file_command: None,
            appearance_change: None,
        };
        let top_panel = egui::Panel::top("top_panel")
            .resizable(true)
            .min_size(TOP_PANEL_HEIGHT);

        top_panel.show(root, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                // New, Open…, Save, Save As… and Quit on native; New alone on
                // the web.
                ui.menu_button("File", |ui| {
                    if file_menu_item(ui, FileCommand::New) {
                        choice.file_command = Some(FileCommand::New);
                    }
                    #[cfg(not(target_arch = "wasm32"))]
                    {
                        for command in [FileCommand::Open, FileCommand::Save, FileCommand::SaveAs] {
                            if file_menu_item(ui, command) {
                                choice.file_command = Some(command);
                            }
                        }
                        ui.separator();
                        // Quit is a close request, asked about in `guard_close`.
                        // It shows no chord: on macOS the app menu's ⌘Q ends the
                        // process without a close request.
                        if ui.button("Quit").clicked() {
                            ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    }
                });
                ui.add_space(MENU_BAR_GAP);
                ui.menu_button("View", |ui| {
                    for command in ZoomCommand::ALL {
                        let item = egui::Button::new(command.label())
                            .shortcut_text(ui.ctx().format_shortcut(&command.shortcut()));
                        if ui.add(item).clicked() {
                            // Applied by this frame's `show_source_scene`,
                            // which runs after the menu bar, as the chord is.
                            self.source_view.requested_zoom = Some(command);
                        }
                    }
                    ui.separator();
                    ui.checkbox(&mut self.diagnostics_open, "Diagnostics");
                });
                ui.add_space(MENU_BAR_GAP);
                ui.menu_button("Help", |ui| {
                    if ui.button("Function Reference").clicked() {
                        // The web keeps its unasked Function reference; on
                        // native it discards unsaved changes as New does.
                        #[cfg(not(target_arch = "wasm32"))]
                        {
                            self.discard_asking_first(FUNCTION_REFERENCE_CONFIRMATION);
                        }
                        #[cfg(target_arch = "wasm32")]
                        self.load_function_reference();
                    }
                });
                // Notices are status, not menus: they sit at the bar's right
                // edge. Right to left, so what is shown first is rightmost.
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // Rightmost: the mode, which is a control rather than a
                    // menu, and applied once the frame is done.
                    choice.appearance_change = mode_control(ui);
                    // Theme, settings, Source File and persistence notices sit
                    // in the bar until dismissed; `report` has already sent
                    // each to the developer console.
                    #[cfg(not(target_arch = "wasm32"))]
                    show_notices(ui, &mut self.themes, &mut self.file_notices);
                    // The web opens and saves no file.
                    #[cfg(target_arch = "wasm32")]
                    show_notices(ui, &mut self.themes, &mut Vec::new());
                    #[cfg(feature = "persistence")]
                    if self.persistence.notice_visible() {
                        ui.add_space(MENU_BAR_GAP);
                        if ui.button("Dismiss").clicked() {
                            self.persistence.dismiss_notice();
                        }
                        // Truncated to the space the menus leave, with its
                        // whole text on hover, so a narrow window never
                        // lays it over them.
                        ui.add(
                            egui::Label::new(
                                egui::RichText::new(format!(
                                    "Stored Source could not be read back; it was kept under \
                                     \"{}\"",
                                    crate::persistence::REFUSED_KEY
                                ))
                                .color(ui.visuals().error_fg_color),
                            )
                            .truncate(),
                        );
                    }
                });
            });
        });
        choice
    }
}
