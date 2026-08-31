use eframe::egui;

use egui::{Color32, CornerRadius, Shadow, Stroke, Style, Visuals, style::Selection};

use orcvs::{glyph::Glyph, render_frame::CursorBloom};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ConsolePalette {
    pub page: Color32,
    pub source: Color32,
    pub grid_line: Color32,
    pub sector_line: Color32,
    pub ordinary: Color32,
    pub function: Color32,
    pub bang: Color32,
    pub number: Color32,
    pub note: Color32,
    pub marker: Color32,
    pub highlight: Color32,
    pub bloom_core_fill: Color32,
    pub bloom_core_line: Color32,
    pub bloom_inner_fill: Color32,
    pub bloom_inner_line: Color32,
    pub bloom_mid_fill: Color32,
    pub bloom_mid_line: Color32,
    pub bloom_outer_fill: Color32,
    pub bloom_outer_line: Color32,
    pub selection_fill: Color32,
    pub selection_stroke_rest: Color32,
    pub selection_stroke: Color32,
}

pub const PALETTE: ConsolePalette = ConsolePalette {
    page: Color32::from_rgb(11, 17, 18),  // #0B1112
    source: Color32::from_rgb(7, 13, 13), // #070D0D
    grid_line: Color32::from_rgba_unmultiplied_const(29, 55, 49, 72),
    sector_line: Color32::from_rgba_unmultiplied_const(55, 101, 86, 110),
    ordinary: Color32::from_rgb(165, 183, 178), // #A5B7B2
    function: Color32::from_rgb(104, 224, 184), // #68E0B8
    bang: Color32::from_rgb(255, 127, 135),     // #FF7F87
    number: Color32::from_rgb(131, 166, 216),   // #83A6D8
    note: Color32::from_rgb(170, 145, 214),     // #AA91D6
    marker: Color32::from_rgba_unmultiplied_const(46, 82, 72, 112),
    highlight: Color32::from_rgb(42, 90, 78), // #2A5A4E
    bloom_core_fill: Color32::from_rgb(10, 30, 26), // #0A1E1A
    bloom_core_line: Color32::from_rgba_unmultiplied_const(76, 190, 156, 150),
    bloom_inner_fill: Color32::from_rgb(9, 26, 23), // #091A17
    bloom_inner_line: Color32::from_rgba_unmultiplied_const(58, 148, 122, 125),
    bloom_mid_fill: Color32::from_rgb(8, 22, 20), // #081614
    bloom_mid_line: Color32::from_rgba_unmultiplied_const(43, 110, 92, 100),
    bloom_outer_fill: Color32::from_rgb(8, 18, 17), // #081211
    bloom_outer_line: Color32::from_rgba_unmultiplied_const(34, 78, 67, 82),
    selection_fill: Color32::from_rgb(10, 42, 34), // #0A2A22
    selection_stroke_rest: Color32::from_rgb(82, 195, 163), // #52C3A3
    selection_stroke: Color32::from_rgb(101, 230, 190), // #65E6BE
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct CellVisuals {
    pub background: Color32,
    pub border: Color32,
    pub foreground: Color32,
}

pub(crate) fn cell_visuals(
    glyph: Glyph,
    cursor_bloom: Option<CursorBloom>,
    selected: bool,
    cursor_visible: bool,
) -> CellVisuals {
    let foreground = match glyph {
        Glyph::Bang => PALETTE.bang,
        Glyph::Function => PALETTE.function,
        Glyph::Number => PALETTE.number,
        Glyph::Note => PALETTE.note,
        Glyph::Marker => PALETTE.marker,
        Glyph::Highlight => PALETTE.highlight,
        Glyph::Char | Glyph::Space => PALETTE.ordinary,
    };
    CellVisuals {
        background: if selected && !cursor_visible {
            PALETTE.selection_fill
        } else if cursor_visible {
            PALETTE.source
        } else if let Some(bloom) = cursor_bloom {
            bloom_colours(bloom).0
        } else {
            PALETTE.source
        },
        border: if cursor_visible {
            PALETTE.selection_stroke
        } else if selected {
            PALETTE.selection_stroke_rest
        } else if let Some(bloom) = cursor_bloom {
            bloom_colours(bloom).1
        } else {
            PALETTE.grid_line
        },
        foreground,
    }
}

fn bloom_colours(bloom: CursorBloom) -> (Color32, Color32) {
    match bloom {
        CursorBloom::Core => (PALETTE.bloom_core_fill, PALETTE.bloom_core_line),
        CursorBloom::Inner => (PALETTE.bloom_inner_fill, PALETTE.bloom_inner_line),
        CursorBloom::Mid => (PALETTE.bloom_mid_fill, PALETTE.bloom_mid_line),
        CursorBloom::Outer => (PALETTE.bloom_outer_fill, PALETTE.bloom_outer_line),
    }
}

pub(crate) fn sector_line(strength_percent: u8) -> Color32 {
    let [red, green, blue, base_alpha] = PALETTE.sector_line.to_srgba_unmultiplied();
    let alpha = u16::from(base_alpha) * u16::from(strength_percent.min(100)) / 100;
    Color32::from_rgba_unmultiplied(red, green, blue, alpha as u8)
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
    use super::{PALETTE, cell_visuals, sector_line};
    use orcvs::{glyph::Glyph, render_frame::CursorBloom};

    #[test]
    fn semantic_glyph_colours_are_distinct_and_bang_is_soft_red() {
        let function = cell_visuals(Glyph::Function, None, false, false);
        let number = cell_visuals(Glyph::Number, None, false, false);
        let note = cell_visuals(Glyph::Note, None, false, false);
        let ordinary = cell_visuals(Glyph::Char, None, false, false);
        let bang = cell_visuals(Glyph::Bang, None, false, false);

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
    fn local_highlights_are_distinct_from_global_markers() {
        let marker = cell_visuals(Glyph::Marker, None, false, false);
        let highlight = cell_visuals(Glyph::Highlight, None, false, false);

        assert_eq!(marker.foreground, PALETTE.marker);
        assert_eq!(highlight.foreground, PALETTE.highlight);
        assert_ne!(marker.foreground, highlight.foreground);
    }

    #[test]
    fn sector_line_strength_only_attenuates_palette_alpha() {
        assert_eq!(sector_line(100), PALETTE.sector_line);
        let [red, green, blue, alpha] = sector_line(50).to_srgba_unmultiplied();
        assert!(red.abs_diff(55) <= 2);
        assert!(green.abs_diff(101) <= 2);
        assert!(blue.abs_diff(86) <= 2);
        assert_eq!(alpha, 55);
        assert_eq!(sector_line(8).a(), 8);
        assert_eq!(sector_line(255), PALETTE.sector_line);
    }

    #[test]
    fn cursor_and_selection_override_the_ambient_field() {
        let ordinary = cell_visuals(Glyph::Char, None, false, false);
        let selected = cell_visuals(Glyph::Char, Some(CursorBloom::Core), true, false);
        let cursor = cell_visuals(Glyph::Char, Some(CursorBloom::Core), true, true);

        assert_eq!(ordinary.background, PALETTE.source);
        assert_eq!(ordinary.border, PALETTE.grid_line);
        assert_eq!(selected.background, PALETTE.selection_fill);
        assert_eq!(selected.border, PALETTE.selection_stroke_rest);
        assert_eq!(cursor.background, PALETTE.source);
        assert_eq!(cursor.border, PALETTE.selection_stroke);
        assert_ne!(cursor, selected);
    }

    #[test]
    fn caret_border_change_is_visible_but_restrained() {
        let resting = PALETTE.selection_stroke_rest;
        let visible = PALETTE.selection_stroke;
        let channel_delta = resting.r().abs_diff(visible.r()) as u16
            + resting.g().abs_diff(visible.g()) as u16
            + resting.b().abs_diff(visible.b()) as u16;

        assert!(channel_delta >= 80, "border delta was only {channel_delta}");
        assert!(channel_delta <= 120, "border delta was {channel_delta}");
    }

    #[test]
    fn cursor_bloom_grades_both_fill_and_grid_line() {
        let core = cell_visuals(Glyph::Space, Some(CursorBloom::Core), false, false);
        let inner = cell_visuals(Glyph::Space, Some(CursorBloom::Inner), false, false);
        let mid = cell_visuals(Glyph::Space, Some(CursorBloom::Mid), false, false);
        let outer = cell_visuals(Glyph::Space, Some(CursorBloom::Outer), false, false);
        let distant = cell_visuals(Glyph::Space, None, false, false);

        assert_eq!(core.background, PALETTE.bloom_core_fill);
        assert_eq!(inner.background, PALETTE.bloom_inner_fill);
        assert_eq!(mid.background, PALETTE.bloom_mid_fill);
        assert_eq!(outer.background, PALETTE.bloom_outer_fill);
        assert_eq!(distant.background, PALETTE.source);
        assert_eq!(core.border, PALETTE.bloom_core_line);
        assert_eq!(inner.border, PALETTE.bloom_inner_line);
        assert_eq!(mid.border, PALETTE.bloom_mid_line);
        assert_eq!(outer.border, PALETTE.bloom_outer_line);
        assert_eq!(distant.border, PALETTE.grid_line);
        assert_eq!(
            [
                core.background,
                inner.background,
                mid.background,
                outer.background,
                distant.background,
            ]
            .windows(2)
            .filter(|pair| pair[0] != pair[1])
            .count(),
            4
        );
    }
}
