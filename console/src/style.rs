use eframe::egui;

use egui::{Color32, CornerRadius, Shadow, Stroke, Style, Visuals, style::Selection};

use crate::glyph::Glyph;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ConsolePalette {
    pub page: Color32,
    pub source: Color32,
    pub grid_line: Color32,
    pub ordinary: Color32,
    pub function: Color32,
    pub bang: Color32,
    pub number: Color32,
    pub note: Color32,
    pub marker: Color32,
    pub selection_fill: Color32,
    pub selection_stroke: Color32,
}

pub const PALETTE: ConsolePalette = ConsolePalette {
    page: Color32::from_rgb(16, 20, 23),               // #101417
    source: Color32::from_rgb(11, 16, 19),             // #0B1013
    grid_line: Color32::from_rgb(30, 41, 47),          // #1E292F
    ordinary: Color32::from_rgb(185, 197, 202),        // #B9C5CA
    function: Color32::from_rgb(99, 213, 179),         // #63D5B3
    bang: Color32::from_rgb(255, 133, 133),            // #FF8585
    number: Color32::from_rgb(143, 167, 216),          // #8FA7D8
    note: Color32::from_rgb(174, 159, 205),            // #AE9FCD
    marker: Color32::from_rgb(52, 65, 73),             // #344149
    selection_fill: Color32::from_rgb(17, 48, 42),     // #11302A
    selection_stroke: Color32::from_rgb(99, 213, 179), // #63D5B3
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct CellVisuals {
    pub background: Color32,
    pub border: Color32,
    pub foreground: Color32,
}

pub(crate) fn cell_visuals(
    glyph: Glyph,
    content: Option<char>,
    selected: bool,
    cursor_visible: bool,
) -> CellVisuals {
    let foreground = if content == Some('*') {
        PALETTE.bang
    } else {
        match glyph {
            Glyph::Function => PALETTE.function,
            Glyph::Number => PALETTE.number,
            Glyph::Note => PALETTE.note,
            Glyph::Marker | Glyph::Highlight => PALETTE.marker,
            Glyph::Char | Glyph::Space => PALETTE.ordinary,
        }
    };
    let meaningful_selection = selected || cursor_visible;
    CellVisuals {
        background: if selected && !cursor_visible {
            PALETTE.selection_fill
        } else {
            PALETTE.source
        },
        border: if meaningful_selection {
            PALETTE.selection_stroke
        } else {
            PALETTE.grid_line
        },
        foreground,
    }
}

pub fn style() -> Style {
    let mut visuals = Visuals::dark();
    visuals.panel_fill = PALETTE.page;
    visuals.window_fill = PALETTE.page;
    visuals.extreme_bg_color = PALETTE.source;
    visuals.faint_bg_color = PALETTE.source;
    visuals.error_fg_color = PALETTE.bang;
    visuals.warn_fg_color = PALETTE.bang;
    visuals.selection = Selection {
        bg_fill: PALETTE.selection_fill,
        stroke: Stroke::new(1.0, PALETTE.selection_stroke),
    };
    visuals.window_corner_radius = CornerRadius::ZERO;
    visuals.menu_corner_radius = CornerRadius::ZERO;
    visuals.window_shadow = Shadow::NONE;
    visuals.popup_shadow = Shadow::NONE;
    for widget in [
        &mut visuals.widgets.noninteractive,
        &mut visuals.widgets.inactive,
        &mut visuals.widgets.hovered,
        &mut visuals.widgets.active,
        &mut visuals.widgets.open,
    ] {
        widget.corner_radius = CornerRadius::ZERO;
    }

    Style {
        visuals,
        animation_time: 0.0,
        ..Style::default()
    }
}

#[cfg(test)]
mod tests {
    use super::{PALETTE, cell_visuals};
    use crate::glyph::Glyph;

    #[test]
    fn semantic_glyph_colours_are_distinct_and_bang_is_soft_red() {
        let function = cell_visuals(Glyph::Function, Some('+'), false, false);
        let number = cell_visuals(Glyph::Number, Some('A'), false, false);
        let note = cell_visuals(Glyph::Note, Some('C'), false, false);
        let ordinary = cell_visuals(Glyph::Char, Some('x'), false, false);
        let bang = cell_visuals(Glyph::Char, Some('*'), false, false);

        assert_eq!(function.foreground, PALETTE.function);
        assert_eq!(number.foreground, PALETTE.number);
        assert_eq!(note.foreground, PALETTE.note);
        assert_eq!(ordinary.foreground, PALETTE.ordinary);
        assert_eq!(bang.foreground, PALETTE.bang);
        assert_ne!(number.foreground, function.foreground);
        assert_ne!(number.foreground, note.foreground);
        assert_ne!(number.foreground, ordinary.foreground);
    }

    #[test]
    fn only_cursor_and_selection_add_meaningful_cell_fill() {
        let ordinary = cell_visuals(Glyph::Char, Some('x'), false, false);
        let selected = cell_visuals(Glyph::Char, Some('x'), true, false);
        let cursor = cell_visuals(Glyph::Char, Some('x'), true, true);

        assert_eq!(ordinary.background, PALETTE.source);
        assert_eq!(ordinary.border, PALETTE.grid_line);
        assert_eq!(selected.background, PALETTE.selection_fill);
        assert_eq!(selected.border, PALETTE.selection_stroke);
        assert_eq!(cursor.background, PALETTE.source);
        assert_eq!(cursor.border, PALETTE.selection_stroke);
        assert_ne!(cursor, selected);
    }
}
