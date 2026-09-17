use eframe::egui;

use egui::{Color32, CornerRadius, Shadow, Stroke, Style, Visuals, style::Selection};

use orcvs::source::Token;

use crate::source_paint::{
    DEFAULT_BANG, DEFAULT_ORDINARY, DEFAULT_SOURCE_BACKGROUND, SourcePaintSettings,
};

///
/// The console's fixed palette: chrome, grid geometry, and Cursor/selection
/// colours a viewer does not retheme.
///
/// The Source background and every Token's glyph colour used to live here too,
/// but `syntax-highlighting/01` moved them into [`SourcePaintSettings`] — a
/// console-owned settings value a viewer edits under `Theme → Source colours`
/// and persistence restores independently — so the Source Grid paints from a
/// value rather than from this fixed constant. The Cell grid line stays here
/// deliberately: the ticket that moved the rest names it as the one Source
/// geometry colour that is not a Source colour setting.
///
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ConsolePalette {
    pub page: Color32,
    pub grid_line: Color32,
    pub sector_line: Color32,
    pub selection_fill: Color32,
    pub selection_stroke_rest: Color32,
    pub selection_stroke: Color32,
}

pub const PALETTE: ConsolePalette = ConsolePalette {
    page: Color32::from_rgb(11, 17, 18), // #0B1112
    grid_line: Color32::from_rgba_unmultiplied_const(29, 55, 49, 72),
    sector_line: Color32::from_rgba_unmultiplied_const(55, 101, 86, 110),
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
/// `source_panel_frame` behind the Grid has already painted the live
/// `SourcePaintSettings::source_background` across the console, so an
/// ordinary Cell with nothing to tint answers `None` instead of repainting
/// it. The Cursor's own Cell is one such arm: painting the Source fill again
/// would hide the Cursor Effect's presentation. `Some` answers one of two
/// fills: the Cursor's own colour on the selected Cell
/// (`syntax-highlighting/01`), which wins outright, or otherwise
/// `syntax-highlighting/02`'s Fill tint on a Function or Operand Cell.
///
#[cfg(test)]
pub(crate) fn cell_visuals(
    token: Option<Token>,
    selected: bool,
    cursor_visible: bool,
    source_paint: SourcePaintSettings,
) -> CellVisuals {
    cell_visuals_with_cursor_colour(
        token,
        selected,
        cursor_visible,
        Some(PALETTE.selection_fill),
        source_paint,
    )
}

///
/// Every Source Paint role a Token reads is `source_paint`'s, not this
/// module's fixed [`PALETTE`]: `syntax-highlighting/01` made the Source Grid
/// paint from a console-owned settings value rather than from a constant, so
/// a viewer's `Theme → Source colours` edits reach here on the very next
/// frame. Atom follows Ordinary, as Char already did; Sequence has had its own
/// field since that change and no longer falls into the same arm.
///
pub(crate) fn cell_visuals_with_cursor_colour(
    token: Option<Token>,
    selected: bool,
    cursor_visible: bool,
    cursor_colour: Option<Color32>,
    source_paint: SourcePaintSettings,
) -> CellVisuals {
    let foreground = match token {
        Some(Token::Bang) => source_paint.bang(),
        Some(Token::Comment) => source_paint.comment(),
        Some(Token::Function) => source_paint.function(),
        Some(Token::Number) => source_paint.number(),
        Some(Token::Note) => source_paint.note(),
        Some(Token::Sequence) => source_paint.sequence(),
        Some(Token::Char | Token::Atom) | None => source_paint.ordinary(),
    };
    // The Cursor's own fill wins outright on its Cell (`syntax-highlighting/
    // 01`): `cursor_colour` answers there whether it is `Some` or `None`,
    // the same as before this ticket. Every other Cell answers
    // `syntax-highlighting/02`'s Fill tint instead of the bare `None` a
    // Cell with nothing to tint used to fall back to — `fill_tint_colour`
    // answers that same `None` for a Function's or Operand's blank counterpart
    // to keep it.
    let background = if selected {
        cursor_colour
    } else {
        fill_tint_colour(token, source_paint)
    };
    CellVisuals {
        background,
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

///
/// The background a Function or Operand Cell is tinted with: `token`'s own
/// glyph colour mixed toward `source_paint.source_background()` by
/// `source_paint.fill_tint()` — `100` paints the Token colour outright and
/// `0` paints nothing, which is why the strength is checked before mixing
/// rather than left to a mix that would round back to the background anyway.
///
/// A Function Cell, nested Functions included, answers `function()`: every
/// Function's own two-Cell spelling carries `Token::Function` regardless of
/// nesting, so no separate nesting fact is read. An Operand Cell answers its
/// declared Token's colour — `Number`, `Note`, `Atom`, `Sequence` — whether
/// its Cells are still Pending, hold a Valid Operand, or hold an Invalid one:
/// the Parser labels a claimed operand slot with its signature's declared
/// `Token` whether or not what stands there binds, so this reads the same
/// fact `cell_visuals_with_cursor_colour`'s foreground match already reads
/// rather than a second classifier of Function versus Operand.
///
/// `Token::Bang`, `Token::Comment` and `None` answer `None`: a Bang and a
/// Comment are their own Language Unit, never a Function's operand, and an
/// empty unclaimed Cell has no Token to tint with.
///
/// `Token::Char` answers `None` too, even though `spec.md`'s Function
/// vocabulary names Char among the Tokens a Valid Operand can declare. No
/// operand ever does in this vocabulary: `operand_token!` in `lang/src/
/// atom.rs` never mints `Token::Char` for a signature, and `lang/src/
/// stack.rs`'s `check_token` marks that arm `unreachable!` outright — "no
/// operand type declares a Token the Parser mints only as a label". A Cell
/// this function sees `Some(Token::Char)` on can therefore only be
/// `SourceRevision::token_at`'s leftover-content fallback (`orcvs/src/
/// source/mod.rs`) standing in for a Cell no Expression claimed — the
/// Leftover Char role this ticket's acceptance leaves untinted alongside
/// Comment, Bang and an empty unclaimed Cell.
///
fn fill_tint_colour(token: Option<Token>, source_paint: SourcePaintSettings) -> Option<Color32> {
    let token_colour = match token {
        Some(Token::Function) => source_paint.function(),
        Some(Token::Number) => source_paint.number(),
        Some(Token::Note) => source_paint.note(),
        Some(Token::Atom) => source_paint.ordinary(),
        Some(Token::Sequence) => source_paint.sequence(),
        Some(Token::Bang | Token::Comment | Token::Char) | None => return None,
    };
    let strength = f32::from(source_paint.fill_tint()) / 100.0;

    (strength > 0.0).then(|| {
        source_paint
            .source_background()
            .lerp_to_gamma(token_colour, strength)
    })
}

pub(crate) fn sector_line(strength_percent: u8) -> Color32 {
    let [red, green, blue, base_alpha] = PALETTE.sector_line.to_srgba_unmultiplied();
    let alpha = u16::from(base_alpha) * u16::from(strength_percent.min(100)) / 100;
    Color32::from_rgba_unmultiplied(red, green, blue, alpha as u8)
}

///
/// The style the console opens with, before any restored `SourcePaintSettings`
/// is known.
///
/// Set once at `Console::new` rather than read every frame — unlike the Source
/// Grid, egui's own `extreme_bg_color`, `faint_bg_color`, `error_fg_color` and
/// `warn_fg_color` are not consulted per Cell, so there is no seam that would
/// make them track a live `Theme → Source colours` edit the way `show_source`
/// does. They start at the Source Paint defaults so the chrome and the Grid
/// agree on first paint; a viewer who then retunes Source background or Bang
/// sees the Grid move and this baseline stay, the same way a restored session
/// would.
///
pub fn style() -> Style {
    let mut visuals = Visuals::dark();
    visuals.panel_fill = PALETTE.page;
    visuals.window_fill = PALETTE.page;
    visuals.extreme_bg_color = DEFAULT_SOURCE_BACKGROUND;
    visuals.faint_bg_color = DEFAULT_SOURCE_BACKGROUND;
    visuals.error_fg_color = DEFAULT_BANG;
    visuals.warn_fg_color = DEFAULT_BANG;
    visuals.selection = Selection {
        bg_fill: PALETTE.selection_fill,
        stroke: Stroke::new(1.0, PALETTE.selection_stroke),
    };
    visuals.window_corner_radius = CornerRadius::ZERO;
    visuals.menu_corner_radius = CornerRadius::ZERO;
    visuals.window_shadow = Shadow::NONE;
    visuals.popup_shadow = Shadow::NONE;
    // `Frame::window` and `Panel`'s separator both read these.
    // Cell `grid_line` is alpha for Source compositing; chrome is the same hue.
    let chrome = Stroke::new(1.0, PALETTE.grid_line.to_opaque());
    visuals.window_stroke = chrome;
    visuals.widgets.noninteractive.bg_fill = PALETTE.page;
    visuals.widgets.noninteractive.weak_bg_fill = PALETTE.page;
    visuals.widgets.noninteractive.bg_stroke = chrome;
    visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, DEFAULT_ORDINARY);
    visuals.widgets.inactive.bg_fill = PALETTE.page;
    visuals.widgets.inactive.weak_bg_fill = PALETTE.page;
    visuals.widgets.inactive.bg_stroke = Stroke::NONE;
    visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, DEFAULT_ORDINARY);
    visuals.widgets.hovered.bg_fill = PALETTE.selection_fill;
    visuals.widgets.hovered.weak_bg_fill = PALETTE.selection_fill;
    visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, PALETTE.selection_stroke_rest);
    visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, PALETTE.selection_stroke);
    visuals.widgets.active.bg_fill = PALETTE.selection_fill;
    visuals.widgets.active.weak_bg_fill = PALETTE.selection_fill;
    visuals.widgets.active.bg_stroke = Stroke::new(1.0, PALETTE.selection_stroke);
    visuals.widgets.active.fg_stroke = Stroke::new(1.0, PALETTE.selection_stroke);
    visuals.widgets.open.bg_fill = PALETTE.selection_fill;
    visuals.widgets.open.weak_bg_fill = PALETTE.page;
    visuals.widgets.open.bg_stroke = Stroke::new(1.0, PALETTE.selection_stroke_rest);
    visuals.widgets.open.fg_stroke = Stroke::new(1.0, DEFAULT_ORDINARY);
    for widget in [
        &mut visuals.widgets.noninteractive,
        &mut visuals.widgets.inactive,
        &mut visuals.widgets.hovered,
        &mut visuals.widgets.active,
        &mut visuals.widgets.open,
    ] {
        widget.corner_radius = CornerRadius::ZERO;
        widget.expansion = 0.0;
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
    use crate::source_paint::SourcePaintSettings;
    use egui::{Color32, Stroke};
    use orcvs::source::Token;

    ///
    /// `theme.md` is the decided record for the fixed palette. These literals
    /// are that record in `Color32` form: a later change to the chrome, grid
    /// geometry, or Cursor/selection colours fails here, and the same commit
    /// must change the document. The Source background and every Token's
    /// glyph colour are pinned the same way in `source_paint::tests`, against
    /// `SourcePaintSettings::default` rather than this constant.
    ///
    #[test]
    fn palette_tokens_match_the_decided_record() {
        assert_eq!(
            PALETTE,
            ConsolePalette {
                page: Color32::from_rgb(11, 17, 18), // #0B1112
                grid_line: Color32::from_rgba_unmultiplied_const(29, 55, 49, 72), // rgba(29, 55, 49, 0.28)
                sector_line: Color32::from_rgba_unmultiplied_const(55, 101, 86, 110), // rgba(55, 101, 86, 0.43)
                selection_fill: Color32::from_rgb(10, 42, 34),                        // #0A2A22
                selection_stroke_rest: Color32::from_rgb(82, 195, 163),               // #52C3A3
                selection_stroke: Color32::from_rgb(101, 230, 190),                   // #65E6BE
            }
        );
    }

    #[test]
    fn semantic_glyph_colours_are_distinct_and_read_from_the_settings_value() {
        let source_paint = SourcePaintSettings::default();
        let function = cell_visuals(Some(Token::Function), false, false, source_paint);
        let number = cell_visuals(Some(Token::Number), false, false, source_paint);
        let note = cell_visuals(Some(Token::Note), false, false, source_paint);
        let ordinary = cell_visuals(Some(Token::Char), false, false, source_paint);
        let bang = cell_visuals(Some(Token::Bang), false, false, source_paint);
        let comment = cell_visuals(Some(Token::Comment), false, false, source_paint);
        let sequence = cell_visuals(Some(Token::Sequence), false, false, source_paint);

        assert_eq!(function.foreground, source_paint.function());
        assert_eq!(number.foreground, source_paint.number());
        assert_eq!(note.foreground, source_paint.note());
        assert_eq!(ordinary.foreground, source_paint.ordinary());
        assert_eq!(bang.foreground, source_paint.bang());
        assert_eq!(comment.foreground, source_paint.comment());
        assert_eq!(sequence.foreground, source_paint.sequence());
        assert_ne!(number.foreground, function.foreground);
        assert_ne!(number.foreground, note.foreground);
        assert_ne!(number.foreground, ordinary.foreground);
        assert_ne!(comment.foreground, ordinary.foreground);
        // Atom follows Ordinary, the way Char already did: a Cell painting no
        // glyph of its own has nothing to colour differently.
        assert_eq!(
            cell_visuals(Some(Token::Atom), false, false, source_paint).foreground,
            ordinary.foreground
        );
        // Sequence stopped sharing Ordinary's colour in this same change: it
        // has its own field and its own default, distinct from every other
        // role including Ordinary.
        assert_ne!(sequence.foreground, ordinary.foreground);
        assert_ne!(sequence.foreground, comment.foreground);

        // A changed settings value reaches `cell_visuals_with_cursor_colour`
        // on the very next call — the Source Grid paints from this value, not
        // from a fixed palette, so a Theme edit previews immediately.
        let mut retuned = source_paint;
        *retuned.function_mut() = Color32::from_rgb(1, 2, 3);
        assert_eq!(
            cell_visuals(Some(Token::Function), false, false, retuned).foreground,
            Color32::from_rgb(1, 2, 3)
        );
    }

    ///
    /// A Comment is the row that says nothing, so it reads dimmer than the
    /// ordinary Glyph rather than as another semantic colour beside it — and
    /// it is still prose a person reads, so dimmer stops at legible.
    ///
    /// Under the previous, hand-picked palette Comment was also the dimmest
    /// Glyph that cleared the floor. The Okabe–Ito assignment does not carry
    /// that second property over: measured against `#000000`, Diagnostic is
    /// 5.43:1, Function is 6.14:1 and Bang is 6.86:1, each dimmer than
    /// Comment's own 7.37:1, while every one of them still clears 4.5:1. This
    /// restates the rule with that measured fact rather than silently keeping
    /// an ordering the new defaults do not hold — `syntax-highlighting/01`'s
    /// own instruction for the floor's Sequence exception applies here too,
    /// to a property rather than a single colour. Sequence itself is measured
    /// in `sequence_is_the_named_exception_to_the_contrast_floor` below.
    ///
    #[test]
    fn comment_reads_dimmer_than_ordinary_and_every_non_sequence_colour_clears_the_floor() {
        let source_paint = SourcePaintSettings::default();
        let background = source_paint.source_background();
        let comment = contrast(source_paint.comment(), background);
        let ordinary = contrast(source_paint.ordinary(), background);

        assert!(
            comment >= 4.5,
            "a Comment is read, not merely seen: {comment:.2}:1 against the Source background",
        );
        assert!(
            comment < ordinary,
            "a Comment reads dimmer than ordinary Source: {comment:.2}:1 against {ordinary:.2}:1",
        );
        // Every Source colour but Sequence clears the floor. None of them is
        // required to read brighter than Comment — Diagnostic, Function and
        // Bang do not, and that is the fact this test pins rather than hides.
        for (name, colour) in [
            ("ordinary", source_paint.ordinary()),
            ("function", source_paint.function()),
            ("bang", source_paint.bang()),
            ("number", source_paint.number()),
            ("note", source_paint.note()),
            ("diagnostic", source_paint.diagnostic()),
            ("result", source_paint.result()),
        ] {
            let ratio = contrast(colour, background);
            assert!(
                ratio >= 4.5,
                "{name} is {ratio:.2}:1, below the 4.5:1 floor"
            );
        }
    }

    ///
    /// The Okabe–Ito assignment's own choice for Sequence, `#0072B2`, measures
    /// 4.05:1 against the Source background — below the 4.5:1 floor every
    /// other Source colour clears. Restating the rule with this exception
    /// named, rather than silently lowering the floor or silently excluding
    /// Sequence from `comment_is_the_dimmest_glyph_that_still_clears_the_
    /// contrast_floor`'s loop, is `syntax-highlighting/01`'s own acceptance
    /// criterion.
    ///
    #[test]
    fn sequence_is_the_named_exception_to_the_contrast_floor() {
        let source_paint = SourcePaintSettings::default();
        let sequence = contrast(source_paint.sequence(), source_paint.source_background());

        assert!(
            sequence < 4.5,
            "Sequence no longer needs the named exception: {sequence:.2}:1"
        );
        assert!(
            (sequence - 4.05).abs() < 0.01,
            "Sequence drifted off the measured exception: {sequence:.2}:1, expected 4.05:1"
        );
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
        let source_paint = SourcePaintSettings::default();
        let ordinary = cell_visuals(Some(Token::Char), false, false, source_paint);
        let selected = cell_visuals(Some(Token::Char), true, false, source_paint);
        let cursor = cell_visuals(Some(Token::Char), true, true, source_paint);

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
        let source_paint = SourcePaintSettings::default();
        assert_eq!(
            super::cell_visuals_with_cursor_colour(
                Some(Token::Char),
                true,
                true,
                None,
                source_paint
            )
            .background,
            None
        );
        assert_eq!(
            super::cell_visuals_with_cursor_colour(
                Some(Token::Char),
                true,
                true,
                Some(colour),
                source_paint
            )
            .background,
            Some(colour)
        );
    }

    #[test]
    fn panel_separator_uses_the_grid_line() {
        let style = super::style();
        let chrome = Stroke::new(1.0, PALETTE.grid_line.to_opaque());
        assert_eq!(style.visuals.window_stroke, chrome);
        assert_eq!(
            style.visuals.widgets.noninteractive.bg_stroke, chrome,
            "Panel::show_separator_line reads noninteractive.bg_stroke"
        );
    }

    #[test]
    fn idle_widgets_have_no_rest_outline() {
        let style = super::style();
        assert_eq!(style.visuals.widgets.inactive.bg_stroke, Stroke::NONE);
        assert_eq!(
            style.visuals.widgets.active.bg_stroke.color,
            PALETTE.selection_stroke
        );
        assert_eq!(
            style.visuals.selection.stroke.color,
            PALETTE.selection_stroke
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

    ///
    /// The reading every tint test below shares: `source_paint`'s
    /// `lerp_to_gamma` at its own Fill tint strength, restated independently
    /// of `super::fill_tint_colour` so a broken mix is caught rather than
    /// mirrored.
    ///
    fn tinted(source_paint: SourcePaintSettings, colour: Color32) -> Color32 {
        let strength = f32::from(source_paint.fill_tint()) / 100.0;
        source_paint
            .source_background()
            .lerp_to_gamma(colour, strength)
    }

    ///
    /// A Function Cell is tinted with the Function colour. Nested Functions
    /// are not a separate case here: every Function's own two-Cell spelling
    /// carries `Token::Function` regardless of nesting (`render_frame.rs`'s
    /// `a_self_banging_function_is_painted_as_a_function` and
    /// `orcvs`'s own nested-Function tests pin that at the Token layer), so
    /// this one arm already covers a nested Function's Cells — proven with a
    /// real nested Expression in `paint::tests::
    /// a_nested_functions_own_cells_are_tinted_like_its_parents`.
    ///
    #[test]
    fn a_function_cell_is_tinted_with_the_function_colour() {
        let source_paint = SourcePaintSettings::default();
        let function = cell_visuals(Some(Token::Function), false, false, source_paint);

        assert_eq!(
            function.background,
            Some(tinted(source_paint, source_paint.function()))
        );
    }

    ///
    /// An Operand Cell is tinted with its declared Token's colour: Number,
    /// Note, Atom and Sequence. `Token::Char` is not one of them —
    /// `fill_tint_colour`'s own doc explains why a Cell can carry that Token
    /// at all without ever being a declared operand.
    ///
    #[test]
    fn an_operand_cell_of_each_declared_token_is_tinted_with_its_own_colour() {
        let source_paint = SourcePaintSettings::default();

        for (token, colour) in [
            (Token::Number, source_paint.number()),
            (Token::Note, source_paint.note()),
            (Token::Atom, source_paint.ordinary()),
            (Token::Sequence, source_paint.sequence()),
        ] {
            let visuals = cell_visuals(Some(token), false, false, source_paint);
            assert_eq!(
                visuals.background,
                Some(tinted(source_paint, colour)),
                "{token:?} was not tinted with its own colour"
            );
        }
    }

    ///
    /// A Fill tint of `0` paints no background at all on a Function or
    /// Operand Cell, the same `None` an untinted role answers — not
    /// `Some` of the Source background, which `background_runs` would still
    /// have to walk as a Cell wanting a fill of its own.
    ///
    #[test]
    fn zero_percent_fill_tint_paints_no_tint() {
        let mut source_paint = SourcePaintSettings::default();
        *source_paint.fill_tint_mut() = 0;

        for token in [
            Token::Function,
            Token::Number,
            Token::Note,
            Token::Atom,
            Token::Sequence,
        ] {
            let visuals = cell_visuals(Some(token), false, false, source_paint);
            assert_eq!(visuals.background, None, "{token:?} was tinted at 0%");
        }
    }

    ///
    /// Comment, Bang, Leftover Char and an empty unclaimed Cell are not
    /// tinted, at the ticket's default (non-zero) Fill tint strength — so a
    /// missing exclusion cannot hide behind `zero_percent_fill_tint_paints_
    /// no_tint`'s strength being zero.
    ///
    #[test]
    fn comment_bang_leftover_char_and_empty_unclaimed_cells_are_not_tinted() {
        let source_paint = SourcePaintSettings::default();
        assert_ne!(source_paint.fill_tint(), 0);

        for token in [
            Some(Token::Comment),
            Some(Token::Bang),
            Some(Token::Char),
            None,
        ] {
            let visuals = cell_visuals(token, false, false, source_paint);
            assert_eq!(visuals.background, None, "{token:?} was tinted");
        }
    }

    ///
    /// The Cursor's own fill wins over the tint on its Cell: a Function or
    /// Operand Cell that is also selected answers the Cursor's colour, not
    /// the tint, the same priority `cursor_and_selection_override_the_
    /// ambient_field` already pins for a Cell with nothing to tint.
    ///
    #[test]
    fn the_cursors_own_fill_wins_over_the_tint_on_its_cell() {
        let source_paint = SourcePaintSettings::default();
        let cursor_colour = Color32::from_rgb(9, 8, 7);
        let tinted_function = tinted(source_paint, source_paint.function());

        let unselected = super::cell_visuals_with_cursor_colour(
            Some(Token::Function),
            false,
            false,
            Some(cursor_colour),
            source_paint,
        );
        let selected = super::cell_visuals_with_cursor_colour(
            Some(Token::Function),
            true,
            false,
            Some(cursor_colour),
            source_paint,
        );
        let cursor = super::cell_visuals_with_cursor_colour(
            Some(Token::Function),
            true,
            true,
            Some(cursor_colour),
            source_paint,
        );

        assert_eq!(unselected.background, Some(tinted_function));
        assert_eq!(selected.background, Some(cursor_colour));
        assert_eq!(cursor.background, Some(cursor_colour));
        assert_ne!(cursor_colour, tinted_function);
    }
}
