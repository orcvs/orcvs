use eframe::egui;

use egui::{Color32, CornerRadius, Shadow, Stroke, Style, Visuals, style::Selection};

use orcvs::source::Token;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ConsolePalette {
    pub page: Color32,
    pub source: Color32,
    pub grid_line: Color32,
    pub sector_line: Color32,
    pub ordinary: Color32,
    pub comment: Color32,
    pub function: Color32,
    pub bang: Color32,
    pub number: Color32,
    pub note: Color32,
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
    comment: Color32::from_rgb(122, 135, 132),  // #7A8784
    function: Color32::from_rgb(104, 224, 184), // #68E0B8
    bang: Color32::from_rgb(255, 127, 135),     // #FF7F87
    number: Color32::from_rgb(131, 166, 216),   // #83A6D8
    note: Color32::from_rgb(170, 145, 214),     // #AA91D6
    selection_fill: Color32::from_rgb(10, 42, 34), // #0A2A22
    selection_stroke_rest: Color32::from_rgb(82, 195, 163), // #52C3A3
    selection_stroke: Color32::from_rgb(101, 230, 190), // #65E6BE
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct CellVisuals {
    pub background: Option<Color32>,
    pub border: Color32,
    pub foreground: Color32,
}

///
/// How one Cell is coloured: fill, border and Token.
///
/// `background` is `None` when the Cell needs no fill of its own — the
/// `source_panel_frame` behind the Grid has already painted `PALETTE.source`
/// across the console, so the two arms that would answer that colour answer
/// `None` instead of asking every ordinary Cell to repaint it. The Cursor's
/// own Cell is one of those arms: painting the Source fill again would hide
/// the Cursor Effect's presentation.
///
#[cfg(test)]
pub(crate) fn cell_visuals(
    token: Option<Token>,
    selected: bool,
    cursor_visible: bool,
) -> CellVisuals {
    cell_visuals_with_cursor_colour(
        token,
        selected,
        cursor_visible,
        Some(PALETTE.selection_fill),
    )
}

pub(crate) fn cell_visuals_with_cursor_colour(
    token: Option<Token>,
    selected: bool,
    cursor_visible: bool,
    cursor_colour: Option<Color32>,
) -> CellVisuals {
    let foreground = match token {
        Some(Token::Bang) => PALETTE.bang,
        Some(Token::Comment) => PALETTE.comment,
        Some(Token::Function) => PALETTE.function,
        Some(Token::Number) => PALETTE.number,
        Some(Token::Note) => PALETTE.note,
        Some(Token::Char | Token::Atom | Token::Sequence) | None => PALETTE.ordinary,
    };
    CellVisuals {
        background: selected.then_some(cursor_colour).flatten(),
        border: if cursor_visible {
            PALETTE.selection_stroke
        } else if selected {
            PALETTE.selection_stroke_rest
        } else {
            PALETTE.grid_line
        },
        foreground,
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
    use super::{ConsolePalette, PALETTE, cell_visuals, sector_line};
    use egui::Color32;
    use orcvs::source::Token;

    ///
    /// `theme.md` is the decided record. These literals are that record in
    /// `Color32` form: a later palette change fails here, and the same commit
    /// must change the document.
    ///
    #[test]
    fn palette_tokens_match_the_decided_record() {
        assert_eq!(
            PALETTE,
            ConsolePalette {
                page: Color32::from_rgb(11, 17, 18),  // #0B1112
                source: Color32::from_rgb(7, 13, 13), // #070D0D
                grid_line: Color32::from_rgba_unmultiplied_const(29, 55, 49, 72), // rgba(29, 55, 49, 0.28)
                sector_line: Color32::from_rgba_unmultiplied_const(55, 101, 86, 110), // rgba(55, 101, 86, 0.43)
                ordinary: Color32::from_rgb(165, 183, 178),                           // #A5B7B2
                comment: Color32::from_rgb(122, 135, 132),                            // #7A8784
                function: Color32::from_rgb(104, 224, 184),                           // #68E0B8
                bang: Color32::from_rgb(255, 127, 135),                               // #FF7F87
                number: Color32::from_rgb(131, 166, 216),                             // #83A6D8
                note: Color32::from_rgb(170, 145, 214),                               // #AA91D6
                selection_fill: Color32::from_rgb(10, 42, 34),                        // #0A2A22
                selection_stroke_rest: Color32::from_rgb(82, 195, 163),               // #52C3A3
                selection_stroke: Color32::from_rgb(101, 230, 190),                   // #65E6BE
            }
        );
    }

    #[test]
    fn semantic_glyph_colours_are_distinct_and_bang_is_soft_red() {
        let function = cell_visuals(Some(Token::Function), false, false);
        let number = cell_visuals(Some(Token::Number), false, false);
        let note = cell_visuals(Some(Token::Note), false, false);
        let ordinary = cell_visuals(Some(Token::Char), false, false);
        let bang = cell_visuals(Some(Token::Bang), false, false);
        let comment = cell_visuals(Some(Token::Comment), false, false);

        assert_eq!(function.foreground, PALETTE.function);
        assert_eq!(number.foreground, PALETTE.number);
        assert_eq!(note.foreground, PALETTE.note);
        assert_eq!(ordinary.foreground, PALETTE.ordinary);
        assert_eq!(bang.foreground, PALETTE.bang);
        assert_eq!(comment.foreground, PALETTE.comment);
        assert_ne!(number.foreground, function.foreground);
        assert_ne!(number.foreground, note.foreground);
        assert_ne!(number.foreground, ordinary.foreground);
        assert_ne!(comment.foreground, ordinary.foreground);
        // Atom and Sequence keep Char's colour until typed-source-paint/03
        // gives them colours of their own.
        assert_eq!(
            cell_visuals(Some(Token::Atom), false, false).foreground,
            ordinary.foreground
        );
        assert_eq!(
            cell_visuals(Some(Token::Sequence), false, false).foreground,
            ordinary.foreground
        );
    }

    ///
    /// A Comment is the row that says nothing, so it reads dimmer than the
    /// ordinary Glyph rather than as another semantic colour beside it — and
    /// it is still prose a person reads, so dimmer stops at legible.
    ///
    /// Both halves are stated as measurements because neither survives being
    /// stated as a difference: `assert_ne!` against the ordinary Glyph passes
    /// for a Comment brighter than it, and passes again for one all but
    /// indistinguishable from the Cell it sits on. The floor is WCAG AA for
    /// normal text, which is the threshold every other Glyph in this palette
    /// already clears by some margin.
    ///
    #[test]
    fn a_comment_reads_dimmer_than_ordinary_source_and_stays_legible() {
        let comment = contrast(PALETTE.comment, PALETTE.source);
        let ordinary = contrast(PALETTE.ordinary, PALETTE.source);

        assert!(
            comment >= 4.5,
            "a Comment is read, not merely seen: {comment:.2}:1 against the Cell it sits on",
        );
        assert!(
            comment < ordinary,
            "a Comment reads dimmer than ordinary Source: {comment:.2}:1 against {ordinary:.2}:1",
        );
        // Every other Glyph a person reads clears the same floor, so the
        // Comment is dimmest without being the one exception to legibility.
        for (name, colour) in [
            ("ordinary", PALETTE.ordinary),
            ("function", PALETTE.function),
            ("bang", PALETTE.bang),
            ("number", PALETTE.number),
            ("note", PALETTE.note),
        ] {
            let ratio = contrast(colour, PALETTE.source);
            assert!(ratio >= 4.5, "{name} is {ratio:.2}:1");
            assert!(
                ratio > comment,
                "{name} is {ratio:.2}:1, dimmer than a Comment"
            );
        }
    }

    /// The WCAG 2.1 contrast ratio between two opaque colours, which is what
    /// "dimmer" and "legible" are measured with above rather than asserted as
    /// a difference of two constants.
    fn contrast(foreground: egui::Color32, background: egui::Color32) -> f32 {
        let luminance = |colour: egui::Color32| {
            let channel = |value: u8| {
                let value = f32::from(value) / 255.0;
                if value <= 0.03928 {
                    value / 12.92
                } else {
                    ((value + 0.055) / 1.055).powf(2.4)
                }
            };
            0.2126 * channel(colour.r())
                + 0.7152 * channel(colour.g())
                + 0.0722 * channel(colour.b())
        };
        let (first, second) = (luminance(foreground), luminance(background));
        (first.max(second) + 0.05) / (first.min(second) + 0.05)
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
        let ordinary = cell_visuals(Some(Token::Char), false, false);
        let selected = cell_visuals(Some(Token::Char), true, false);
        let cursor = cell_visuals(Some(Token::Char), true, true);

        // `None`: the panel behind the Grid has already painted the Source colour.
        assert_eq!(ordinary.background, None);
        assert_eq!(ordinary.border, PALETTE.grid_line);
        assert_eq!(selected.background, Some(PALETTE.selection_fill));
        assert_eq!(selected.border, PALETTE.selection_stroke_rest);
        assert_eq!(cursor.background, Some(PALETTE.selection_fill));
        assert_eq!(cursor.border, PALETTE.selection_stroke);
        assert_ne!(cursor, selected);
    }

    #[test]
    fn cursor_cell_colour_is_optional_and_defaults_to_the_theme_background() {
        let colour = Color32::from_rgb(1, 2, 3);
        assert_eq!(
            super::cell_visuals_with_cursor_colour(Some(Token::Char), true, true, None).background,
            None
        );
        assert_eq!(
            super::cell_visuals_with_cursor_colour(Some(Token::Char), true, true, Some(colour))
                .background,
            Some(colour)
        );
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
}
