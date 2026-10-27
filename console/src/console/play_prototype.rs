//! PROTOTYPE (throwaway, branch `prototype/play-control`): variants of an
//! explicit Play control, chosen by `?variant=` on the web and
//! `ORCVS_VARIANT` on native. See `.scratch/play-control/`.

use orcvs::app::{InputEvent, InputKey, Orcvs};
use orcvs::playback::PlaybackState;

pub(super) fn variant() -> char {
    #[cfg(target_arch = "wasm32")]
    let search = web_sys::window()
        .and_then(|window| window.location().search().ok())
        .unwrap_or_default();
    #[cfg(not(target_arch = "wasm32"))]
    let search = std::env::var("ORCVS_VARIANT")
        .map(|v| format!("variant={v}"))
        .unwrap_or_default();
    search
        .split(['?', '&'])
        .find_map(|pair| pair.strip_prefix("variant="))
        .and_then(|value| value.chars().next())
        .map(|c| c.to_ascii_uppercase())
        .unwrap_or('A')
}

pub(super) fn playing(orcvs: &Orcvs) -> bool {
    orcvs.playback_observation().state == PlaybackState::Playing
}

pub(super) fn toggle(orcvs: &mut Orcvs) {
    orcvs.event_handler(vec![InputEvent::KeyPressed(InputKey::Space)]);
}

pub(super) fn rewind(orcvs: &mut Orcvs) {
    orcvs.event_handler(vec![InputEvent::Rewind]);
}

/// An icon button in the console's one font, named for accessibility.
pub(super) fn glyph(ui: &mut egui::Ui, glyph: &str, name: &str, selected: bool) -> bool {
    let response = ui.selectable_label(selected, glyph);
    response
        .widget_info(|| egui::WidgetInfo::selected(egui::WidgetType::Button, true, selected, name));
    response.on_hover_text(name).clicked()
}
