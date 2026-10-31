//! The top bar: the File, View and Help menus, the Notices menu and the
//! mode control.

use super::Console;
use super::files::EXAMPLE_CONFIRMATIONS;
#[cfg(not(target_arch = "wasm32"))]
use super::files::FUNCTION_REFERENCE_CONFIRMATION;
use super::input::FileCommand;
use super::panel::{BarEdge, show_bar};
use crate::examples::EXAMPLES;
use crate::theme::Appearance;
use crate::theme_selection::SelectedThemes;

/// The site's pages the Help menu links to, by item label.
const SITE_LINKS: [(&str, &str); 2] = [
    ("Learn", "https://orcvs.com/learn/"),
    ("Reference", "https://orcvs.com/reference/"),
];

/// The height the top panel takes from the window, leaving the rest to the
/// console: the bottom Panel's compact height, so the two bars match.
pub(super) const TOP_PANEL_HEIGHT: f32 = super::panel::BOTTOM_PANEL_HEIGHT;

///
/// The top bar's item spacing: what leaves the bottom Panel's Readout group
/// gap between two menu labels once each button has padded its own. No item
/// adds space of its own.
///
pub(super) fn menu_bar_gap(ui: &egui::Ui) -> f32 {
    (super::panel::readout_group_gap(ui) - 2.0 * ui.spacing().button_padding.x).max(0.0)
}

///
/// Opens `url` without leaving the console: in a new tab on the web, so the
/// open Source stays, and in the system browser on native.
///
fn open_link(ctx: &egui::Context, url: &str) {
    #[cfg(target_arch = "wasm32")]
    ctx.open_url(egui::OpenUrl::new_tab(url));
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = ctx;
        match browser(url).spawn() {
            // Waited for apart, so the opener leaves no zombie once it exits.
            Ok(mut opener) => {
                std::thread::spawn(move || opener.wait());
            }
            Err(error) => crate::report::error!("{url} could not be opened: {error}"),
        }
    }
}

///
/// The command that opens `url` in the system's browser. eframe opens a URL
/// natively only through its `links` feature, whose `webbrowser` brings `url`
/// and the ICU tables beneath it to parse two fixed links.
///
#[cfg(not(target_arch = "wasm32"))]
fn browser(url: &str) -> std::process::Command {
    let opener = if cfg!(target_os = "macos") {
        "open"
    } else if cfg!(target_os = "windows") {
        "explorer"
    } else {
        "xdg-open"
    };
    let mut command = std::process::Command::new(opener);
    command.arg(url);
    command
}

///
/// A File menu item for `command`, carrying its chord as shortcut text on
/// native, where the chord is bound. Answers whether it was chosen.
///
fn file_menu_item(ui: &mut egui::Ui, command: FileCommand) -> bool {
    let item = egui::Button::new(command.label());
    #[cfg(not(target_arch = "wasm32"))]
    let item = match command.shortcut() {
        Some(shortcut) => item.shortcut_text(ui.ctx().format_shortcut(&shortcut)),
        None => item,
    };
    ui.add(item).clicked()
}

///
/// A menu item that toggles `checked`, laid out as a button so its label
/// aligns with the menu's other items, with its check box after the label
/// and, on native, any `shortcut` before the box. Named and announced as a
/// check box. Answers the item's response.
///
fn trailing_checkbox(
    ui: &mut egui::Ui,
    checked: &mut bool,
    label: &str,
    #[cfg_attr(target_arch = "wasm32", expect(unused_variables))] shortcut: Option<
        egui::KeyboardShortcut,
    >,
) -> egui::Response {
    let box_id = egui::Id::new(("orcvs-trailing-checkbox", label));
    let box_size = egui::Vec2::splat(ui.spacing().icon_width);
    let item = egui::Button::new(label);
    #[cfg(not(target_arch = "wasm32"))]
    let item = match shortcut {
        Some(shortcut) => item.shortcut_text(ui.ctx().format_shortcut(&shortcut)),
        None => item,
    };
    let shown = item
        .right_text(egui::Atom::custom(box_id, box_size))
        .atom_ui(ui);
    let box_rect = shown.rect(box_id);
    let mut response = shown.response;
    if response.clicked() {
        *checked = !*checked;
        response.mark_changed();
    }
    let enabled = ui.is_enabled();
    response.widget_info(|| {
        egui::WidgetInfo::selected(egui::WidgetType::Checkbox, enabled, *checked, label)
    });
    if let Some(rect) = box_rect {
        // Painted as `egui::Checkbox` paints its box and mark, which lays its
        // box out before its label and so cannot trail one
        // (`tests::the_trailing_box_paints_as_egui_paints_a_checkbox`).
        let style = ui
            .style()
            .checkbox_style(&Default::default(), response.widget_state());
        let big = egui::Rect::from_center_size(rect.center(), box_size);
        let small = egui::Rect::from_center_size(big.center(), egui::Vec2::splat(style.check_size));
        ui.painter().add(egui::epaint::RectShape::new(
            big.expand(style.checkbox_frame.inner_margin.left.into()),
            style.checkbox_frame.corner_radius,
            style.checkbox_frame.fill,
            style.checkbox_frame.stroke,
            egui::StrokeKind::Inside,
        ));
        if *checked {
            ui.painter().add(egui::Shape::line(
                vec![
                    egui::pos2(small.left(), small.center().y),
                    egui::pos2(small.center().x, small.bottom()),
                    egui::pos2(small.right(), small.top()),
                ],
                style.check_stroke,
            ));
        }
    }
    response
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
    // Icon-sized and spaced as egui spaces them, not the bar's full height
    // and menu gap: the mode opens no menu. The spacing is set in a scope of
    // its own, so what the bar shows after the mode keeps the bar's spacing.
    ui.scope(|ui| {
        let global = ui.ctx().global_style();
        ui.spacing_mut().interact_size.y = global.spacing.interact_size.y;
        ui.spacing_mut().item_spacing.x = global.spacing.item_spacing.x;
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
    })
    .inner
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
#[derive(Default)]
pub(super) struct MenuBarChoice {
    /// A File command from the File menu, run once the chords are read.
    pub(super) file_command: Option<FileCommand>,
    /// A mode from the mode control, applied once the frame is done so no
    /// frame mixes two Themes.
    pub(super) appearance_change: Option<AppearanceChange>,
}

impl Console {
    ///
    /// Shows the top bar: File, View and Help with its Examples, then at its
    /// right edge the mode control and the Notices menu.
    ///
    /// View's zoom items on native are egui's own, which step the whole UI's
    /// zoom as its chords do; the web's View menu has none. Help's Function
    /// Reference asks before discarding on native at once; the File command
    /// and the mode are answered for the frame to run.
    ///
    pub(super) fn show_menu_bar(
        &mut self,
        root: &mut egui::Ui,
        appearance: Appearance,
    ) -> MenuBarChoice {
        let mut choice = MenuBarChoice {
            file_command: None,
            appearance_change: None,
        };
        // No vertical margin, so the menu buttons can reach the bar's bottom
        // edge.
        let mut frame = egui::Frame::side_top_panel(root.style().as_ref());
        frame.inner_margin.top = 0;
        frame.inner_margin.bottom = 0;
        let top_panel = egui::Panel::top("top_panel")
            .resizable(false)
            .exact_size(TOP_PANEL_HEIGHT);
        let separator = crate::style::bar_separator(self.themes.presented(appearance));

        show_bar(root, top_panel, frame, BarEdge::Bottom, separator, |ui| {
            // A menu opens from its button's bottom edge, so each menu button
            // is the bar's full height, its label centred within it, and its
            // menu opens below the bar.
            ui.spacing_mut().interact_size.y = ui.available_height();
            egui::MenuBar::new().ui(ui, |ui| {
                ui.spacing_mut().item_spacing.x = menu_bar_gap(ui);
                // New, Open…, Save, Save As…, Copy Link and Quit on native;
                // New and Copy Link on the web.
                ui.menu_button("File", |ui| {
                    if file_menu_item(ui, FileCommand::New) {
                        choice.file_command = Some(FileCommand::New);
                    }
                    #[cfg(not(target_arch = "wasm32"))]
                    for command in [FileCommand::Open, FileCommand::Save, FileCommand::SaveAs] {
                        if file_menu_item(ui, command) {
                            choice.file_command = Some(command);
                        }
                    }
                    ui.separator();
                    if file_menu_item(ui, FileCommand::CopyLink) {
                        choice.file_command = Some(FileCommand::CopyLink);
                    }
                    #[cfg(not(target_arch = "wasm32"))]
                    {
                        ui.separator();
                        // Quit is a close request, asked about in `guard_close`.
                        // It shows no chord: on macOS the app menu's ⌘Q ends the
                        // process without a close request.
                        if ui.button("Quit").clicked() {
                            ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    }
                });
                ui.menu_button("View", |ui| {
                    // The web has no zoom items: the browser's page zoom is
                    // its one zoom, and egui's factor would multiply it.
                    #[cfg(not(target_arch = "wasm32"))]
                    {
                        egui::gui_zoom::zoom_menu_buttons(ui);
                        ui.separator();
                    }
                    trailing_checkbox(
                        ui,
                        &mut self.diagnostics_open,
                        "Diagnostics",
                        Some(super::input::DIAGNOSTICS_SHORTCUT),
                    );
                    trailing_checkbox(ui, &mut self.overview_open, "Overview", None);
                });
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
                    // Each example asks before discarding, as New does.
                    ui.menu_button("Examples", |ui| {
                        for (example, confirmation) in EXAMPLES.iter().zip(EXAMPLE_CONFIRMATIONS) {
                            if ui.button(example.name).clicked() {
                                self.discard_asking_first(confirmation);
                            }
                        }
                    });
                    ui.separator();
                    for (label, url) in SITE_LINKS {
                        // The URL on hover tells a site page from the
                        // Function Reference above, which stays in the console.
                        if ui.button(label).on_hover_text(url).clicked() {
                            open_link(ui.ctx(), url);
                        }
                    }
                });
                // Notices are status, not menus: they sit at the bar's right
                // edge. Right to left, so what is shown first is rightmost.
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // Rightmost: the mode, which is a control rather than a
                    // menu, and applied once the frame is done.
                    choice.appearance_change = mode_control(ui);
                    // Theme, settings and Source File notices sit
                    // in the bar until dismissed; `report` has already sent
                    // each to the developer console.
                    #[cfg(not(target_arch = "wasm32"))]
                    show_notices(ui, &mut self.themes, &mut self.file_notices);
                    // The web opens and saves no file.
                    #[cfg(target_arch = "wasm32")]
                    show_notices(ui, &mut self.themes, &mut Vec::new());
                });
            });
        });
        choice
    }

    ///
    /// The small console's top bar (ADR 0086), shown only while there are
    /// notices: the Notices menu alone, at the right edge where the full bar
    /// shows it, so a refused Source says why the open one stayed.
    ///
    #[cfg(any(target_arch = "wasm32", test))]
    pub(super) fn show_notice_bar(&mut self, root: &mut egui::Ui, appearance: Appearance) {
        if self.themes.notice_count() == 0 {
            return;
        }
        let mut frame = egui::Frame::side_top_panel(root.style().as_ref());
        frame.inner_margin.top = 0;
        frame.inner_margin.bottom = 0;
        let top_panel = egui::Panel::top("top_panel")
            .resizable(false)
            .exact_size(TOP_PANEL_HEIGHT);
        let separator = crate::style::bar_separator(self.themes.presented(appearance));
        show_bar(root, top_panel, frame, BarEdge::Bottom, separator, |ui| {
            ui.spacing_mut().interact_size.y = ui.available_height();
            egui::MenuBar::new().ui(ui, |ui| {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // A small console opens no Source File, so it has no
                    // Source File notices.
                    show_notices(ui, &mut self.themes, &mut Vec::new());
                });
            });
        });
    }
}

#[cfg(test)]
mod tests {
    use super::trailing_checkbox;

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn the_browser_command_opens_the_url_with_the_systems_opener() {
        let command = super::browser("https://orcvs.com/learn/");
        let opener = if cfg!(target_os = "macos") {
            "open"
        } else if cfg!(target_os = "windows") {
            "explorer"
        } else {
            "xdg-open"
        };
        assert_eq!(command.get_program(), opener);
        assert_eq!(
            command.get_args().collect::<Vec<_>>(),
            ["https://orcvs.com/learn/"]
        );
    }

    ///
    /// The box and mark `add` paints, the mark placed relative to the box's
    /// centre, so two boxes at different places compare equal.
    ///
    fn box_shapes(ctx: &egui::Context, add: impl FnMut(&mut egui::Ui)) -> Vec<String> {
        let mut output = ctx.run_ui(egui::RawInput::default(), add);
        // The font atlas upload has no renderer here.
        output.textures_delta.clear();
        let style = ctx
            .global_style()
            .checkbox_style(&Default::default(), Default::default());
        let frame = style.checkbox_frame;
        let mut shapes = Vec::new();
        let mut centre = None;
        for clipped in output.shapes {
            match clipped.shape {
                egui::Shape::Rect(rect)
                    if rect.fill == frame.fill && rect.stroke == frame.stroke =>
                {
                    // A button's frame shares the box's colours and paints
                    // first, so the last such rect is the box.
                    shapes.clear();
                    centre = Some(rect.rect.center());
                    shapes.push(format!(
                        "box {:?} {:?}",
                        rect.rect.size(),
                        rect.corner_radius
                    ));
                }
                egui::Shape::Path(path) if path.stroke.width == style.check_stroke.width => {
                    let centre = centre.expect("the box paints before its mark");
                    let points: Vec<_> = path.points.iter().map(|point| *point - centre).collect();
                    shapes.push(format!("mark {points:?} {:?}", path.stroke));
                }
                _ => {}
            }
        }
        shapes
    }

    ///
    /// The trailing box paints exactly the box and mark `egui::Checkbox`
    /// paints, so an egui upgrade that changes a checkbox's look fails here
    /// rather than leaving the View menu's toggles unlike every other checkbox.
    ///
    #[test]
    fn the_trailing_box_paints_as_egui_paints_a_checkbox() {
        let ctx = egui::Context::default();
        for checked in [false, true] {
            let egui_box = box_shapes(&ctx, |ui| {
                ui.add(egui::Checkbox::without_text(&mut checked.clone()));
            });
            let trailing = box_shapes(&ctx, |ui| {
                trailing_checkbox(
                    ui,
                    &mut checked.clone(),
                    "Diagnostics",
                    Some(super::super::input::DIAGNOSTICS_SHORTCUT),
                );
            });
            assert!(!egui_box.is_empty(), "egui painted no box");
            assert_eq!(trailing, egui_box, "checked: {checked}");
        }
    }
}
