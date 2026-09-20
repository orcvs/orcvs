use eframe::egui;

use egui::{Color32, CornerRadius, Shadow, Stroke, Style, Visuals, style::Selection};

use orcvs::source::{OperandState, SourcePaint, Token};

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
/// Composes the Source Paint decision with Cursor precedence, applied here
/// rather than inside it:
/// `syntax-highlighting/01`'s rule that the Cursor's own fill beats the tint
/// outright on its own Cell, and the border a Cursor or a Selection draws
/// regardless of what stands on the Cell. `paint` is ADR 0050's finished
/// per-Cell language fact, including Pending, Valid, or Invalid for an
/// Operand. `output_portal` is the independent fact added by
/// `syntax-highlighting/06`: whether this Cell lies in a root Function's
/// Output Portal Reservation (`RenderCell::output_portal`), known from the
/// current Source revision alone.
///
pub(crate) fn cell_visuals_with_cursor_colour(
    paint: SourcePaint,
    output_portal: bool,
    selected: bool,
    cursor_visible: bool,
    cursor_colour: Option<Color32>,
    source_paint: SourcePaintSettings,
) -> CellVisuals {
    let (foreground, tint) = source_paint_visuals(paint, output_portal, source_paint);
    // The Cursor's own fill wins outright on its Cell (`syntax-highlighting/
    // 01`): `cursor_colour` answers there whether it is `Some` or `None`,
    // the same as before this ticket. Every other Cell answers
    // `syntax-highlighting/02`'s Fill tint instead of the bare `None` a
    // Cell with nothing to tint used to fall back to.
    let background = if selected { cursor_colour } else { tint };
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
/// Foreground and tint together, from a Cell's finished Source Paint fact and
/// whether it lies in a root Function's Output Portal Reservation.
///
/// Every Source Paint role a Token reads is `source_paint`'s, not this
/// module's fixed [`PALETTE`]: `syntax-highlighting/01` made the Source Grid
/// paint from a console-owned settings value rather than from a constant, so
/// a viewer's `Theme → Source colours` edits reach here on the very next
/// frame.
///
/// # Output Portal precedence
///
/// `output_portal` (`RenderCell::output_portal()`, `.scratch/syntax-
/// highlighting/issues/05`'s and `10`'s Answers) is read first, because the
/// Reservation it names "covers every Cell of the Reservation whatever else
/// claims it" (`05`'s Overlap rule) — a written scalar or Sequence answer, an
/// empty Cell still waiting for one, or another Expression's operand slot the
/// Reservation happens to land on. The one exception is a bound Function
/// claim: a Cell that is itself a Function's own two-Cell spelling —
/// `syntax-highlighting/06`'s precedence decision — keeps its Function paint
/// outright, root or nested, the same as when `output_portal` is false. That
/// is the smallest rule that both lets a producer's Output Portal show
/// through a consumer's operand Cells (the written value is the producer's
/// output) and still lets a reader find the Function that stands on a Cell:
/// distinguishing a root's own spelling from a nested one would need the
/// Expression itself, which this decision does not have and does not need.
/// A Bang answer is the one further exception among non-Function claims: it
/// keeps its own Bang glyph colour rather than the Output Portal colour,
/// because a Bang is what a Producer emits rather than a value it writes, but
/// it still takes the Output Portal's own Fill tint in place of Bang's usual
/// bare `None` — so a Bang answer still reads as an Output Portal Cell.
///
/// # Otherwise, from the Source Paint fact
///
/// An Unclaimed Cell answers Ordinary with no tint. Bang and Comment always
/// answer their own colour with no tint. Function — root or nested — draws
/// its colour on its own tint. Text that spells no
/// Function where an Expression could start (`hi`, both Cells of a written
/// `07`, a lone `|`) answers Ordinary with no tint, as an unclaimed Cell
/// does: the Language Map answers it as Unclaimed because the Parser's attempted
/// Function classification is not a Paint distinction. Diagnostic belongs to
/// an Invalid operand slot. Number, Note, Atom and Sequence Operand facts read
/// [`operand_paint`].
///
fn source_paint_visuals(
    paint: SourcePaint,
    output_portal: bool,
    source_paint: SourcePaintSettings,
) -> (Color32, Option<Color32>) {
    let bound_function = paint == SourcePaint::Function;
    if output_portal && !bound_function {
        let bang = paint == SourcePaint::Bang;
        let foreground = if bang {
            source_paint.bang()
        } else {
            source_paint.output_portal()
        };
        return (
            foreground,
            fill_tint_colour(source_paint.output_portal(), source_paint),
        );
    }

    match paint {
        SourcePaint::Unclaimed => (source_paint.ordinary(), None),
        SourcePaint::Bang => (source_paint.bang(), None),
        SourcePaint::Comment => (source_paint.comment(), None),
        SourcePaint::Function => (
            source_paint.function(),
            fill_tint_colour(source_paint.function(), source_paint),
        ),
        SourcePaint::Operand { token, state } => match token {
            Token::Number => operand_paint(source_paint.number(), state, source_paint),
            Token::Note => operand_paint(source_paint.note(), state, source_paint),
            Token::Atom => operand_paint(source_paint.ordinary(), state, source_paint),
            Token::Sequence => operand_paint(source_paint.sequence(), state, source_paint),
            Token::Bang | Token::Comment | Token::Function | Token::Char => {
                unreachable!("SourcePaint::Operand carries only a declared operand Token")
            }
        },
    }
}

///
/// An Operand Cell's foreground and tint, from its declared `colour`
/// (`Number`, `Note`, `Atom` or `Sequence`) and its finished state.
///
/// The Parser labels an operand slot with its signature's declared Token
/// whether or not what stands there binds, so Valid, Pending, and Invalid all
/// keep `colour` as their tint —
/// `syntax-highlighting/02`'s Fill tint on a Pending, Valid or Invalid
/// operand alike. The foreground differs: a Valid or Pending slot draws
/// `colour`, and an Invalid one — Cells whose written content failed
/// to bind — draws Diagnostic instead. `.+0`'s second operand is this
/// Invalid case: one Cell holds `0`, the other is blank, and `written` is
/// true for the whole slot, so both Cells alike answer Diagnostic —
/// `paint.rs`'s blank-glyph fallback is what keeps the blank one's colour
/// from ever being drawn, not a different verdict for it. An entirely blank
/// slot is Pending instead, and keeps `colour` rather than turning Diagnostic — a distinction with no
/// visible effect today, since a Pending Cell's content is always the blank
/// glyph regardless of foreground, but the Language Map answers it once per
/// Source revision rather than leaving the console to infer it (ADR 0050).
///
fn operand_paint(
    colour: Color32,
    state: OperandState,
    source_paint: SourcePaintSettings,
) -> (Color32, Option<Color32>) {
    let foreground = match state {
        OperandState::Pending | OperandState::Valid => colour,
        OperandState::Invalid => source_paint.diagnostic(),
    };
    (foreground, fill_tint_colour(colour, source_paint))
}

///
/// `colour` mixed toward `source_paint.source_background()` by
/// `source_paint.fill_tint()` — `100` paints `colour` outright and `0` paints
/// nothing, which is why the strength is checked before mixing rather than
/// left to a mix that would round back to the background anyway. `None` at
/// `0` rather than `Some(source_background)`, so a Cell with nothing to tint
/// and a Cell tinted at `0%` answer identically and neither costs
/// `background_runs` a Cell it has to walk.
///
fn fill_tint_colour(colour: Color32, source_paint: SourcePaintSettings) -> Option<Color32> {
    let strength = f32::from(source_paint.fill_tint()) / 100.0;

    (strength > 0.0).then(|| {
        source_paint
            .source_background()
            .lerp_to_gamma(colour, strength)
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
    use super::{
        CellVisuals, ConsolePalette, PALETTE, cell_visuals_with_cursor_colour, sector_line,
    };
    use crate::source_paint::SourcePaintSettings;
    use egui::{Color32, Stroke};
    use orcvs::source::{Atom, OperandState, SourcePaint, Token};

    #[derive(Debug)]
    struct Claim {
        token: Token,
        atom: Option<Atom>,
    }

    ///
    /// A claim built directly, with no egui `Context` and no running `Orcvs`
    /// (ADR 0040): `cells` does not matter to this layer, which never reads
    /// it — only `Paint::derive_with_colours` does, to compute `written`.
    /// `atom` carries the one fact `claim_paint` reads from it: `Some` for a
    /// bound claim, `None` for an unbound one.
    ///
    fn claim(token: Token, atom: Option<Atom>) -> Claim {
        Claim { token, atom }
    }

    ///
    /// A bound claim of `token`. The actual `Atom` variant it carries is
    /// never read by anything under test — `claim_paint` only asks
    /// `atom.is_some()` — so every bound claim here carries the same filler
    /// value regardless of its Token, the way `lang`'s own
    /// `declaration_agreement::lowest` stands `Atom::Number(0)` in for
    /// `Token::Atom` and `Token::Sequence`, neither of which has an `Atom`
    /// variant of its own.
    ///
    fn bound(token: Token) -> Claim {
        claim(token, Some(Atom::Number(0)))
    }

    ///
    /// An unbound claim of `token`: `atom: None`.
    ///
    /// Which of the two a fixture below may reach for is not free, because a
    /// claim built by hand can state a pair the Parser never produces.
    /// `Token::Atom` and `Token::Sequence` are "declarations no Cells spell"
    /// (`lang/src/expression.rs`'s own words on `Token`): `Token::decode`
    /// refuses both outright, so the only thing that can satisfy either slot
    /// is a nested Function — and when one stands there,
    /// `take_language_unit`'s `is_function_next()` branch records the entry
    /// under `Token::Function` (`lang/src/parser.rs`), so the slot's declared
    /// Token never reaches this layer at all. Every `Atom` or `Sequence`
    /// claim the console can be handed is therefore `atom: None`: Pending
    /// where its Cells are blank, Invalid where they are written. So the
    /// tint and glyph-colour fixtures below claim those two here and use
    /// [`bound`] only where a Source can bind — `:#C4D4` binds both Note
    /// slots, `.+0102` both Number ones.
    ///
    /// Which claims a Source actually produces is a fact about the Parser
    /// rather than about this decision function, so it is proven where a
    /// Source can be written rather than asserted here:
    /// `paint::tests::an_operand_cell_of_every_token_a_source_can_claim_is_
    /// tinted_with_its_own_colour` drives `.+`, `:#C4D4`, `:&` and `:<XY`
    /// through the Render Frame and asserts the claim on each operand Cell,
    /// binding state included.
    ///
    fn unbound(token: Token) -> Claim {
        claim(token, None)
    }

    ///
    /// [`cell_visuals_with_cursor_colour`] with no Cursor on the Cell:
    /// unselected, with the Cursor's own colour set the way `Paint::derive`
    /// sets it in production, so a test that does not exercise Cursor
    /// precedence need not repeat it.
    ///
    fn painted(
        claim: Option<&Claim>,
        written: bool,
        output_portal: bool,
        source_paint: SourcePaintSettings,
    ) -> CellVisuals {
        let paint = match claim {
            None => SourcePaint::Unclaimed,
            Some(Claim {
                token: Token::Bang, ..
            }) => SourcePaint::Bang,
            Some(Claim {
                token: Token::Comment,
                ..
            }) => SourcePaint::Comment,
            Some(Claim {
                token: Token::Function,
                atom: Some(_),
            }) => SourcePaint::Function,
            Some(Claim {
                token: Token::Function | Token::Char,
                ..
            }) => SourcePaint::Unclaimed,
            Some(Claim { token, atom }) => SourcePaint::Operand {
                token: *token,
                state: if atom.is_some() {
                    OperandState::Valid
                } else if written {
                    OperandState::Invalid
                } else {
                    OperandState::Pending
                },
            },
        };
        cell_visuals_with_cursor_colour(
            paint,
            output_portal,
            false,
            false,
            Some(PALETTE.selection_fill),
            source_paint,
        )
    }

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

    ///
    /// Covers a bound claim of every Token that draws its own colour, plus
    /// an unclaimed Cell (`None`) for Ordinary — the role a Leftover Char
    /// shares with it, since a Leftover Char has no claim either
    /// (`SourceRevision::token_at`'s fallback stands in for a Cell no
    /// Expression claimed).
    ///
    #[test]
    fn semantic_glyph_colours_are_distinct_and_read_from_the_settings_value() {
        let source_paint = SourcePaintSettings::default();
        let function = painted(Some(&bound(Token::Function)), false, false, source_paint);
        let number = painted(Some(&bound(Token::Number)), false, false, source_paint);
        let note = painted(Some(&bound(Token::Note)), false, false, source_paint);
        let ordinary = painted(None, false, false, source_paint);
        let bang = painted(Some(&bound(Token::Bang)), false, false, source_paint);
        let comment = painted(Some(&unbound(Token::Comment)), false, false, source_paint);
        // Atom and Sequence are claimed unbound because that is the only
        // shape a Source produces for them — see [`unbound`]. Neither
        // assertion turns on it: an unbound operand slot with no written
        // content is Pending, and a Pending slot draws its declared colour.
        let sequence = painted(Some(&unbound(Token::Sequence)), false, false, source_paint);

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
        // Atom follows Ordinary: a Cell painting no glyph of its own has
        // nothing to colour differently.
        assert_eq!(
            painted(Some(&unbound(Token::Atom)), false, false, source_paint).foreground,
            ordinary.foreground
        );
        // Sequence does not share Ordinary's colour: it has its own field
        // and its own default, distinct from every other role including
        // Ordinary.
        assert_ne!(sequence.foreground, ordinary.foreground);
        assert_ne!(sequence.foreground, comment.foreground);

        // A changed settings value reaches `cell_visuals_with_cursor_colour`
        // on the very next call — the Source Grid paints from this value, not
        // from a fixed palette, so a Theme edit previews immediately.
        let mut retuned = source_paint;
        *retuned.function_mut() = Color32::from_rgb(1, 2, 3);
        assert_eq!(
            painted(Some(&bound(Token::Function)), false, false, retuned).foreground,
            Color32::from_rgb(1, 2, 3)
        );
    }

    ///
    /// A Comment is the row that says nothing, so it reads dimmer than the
    /// Glyphs that carry meaning rather than as another semantic colour
    /// beside them — and it is still prose a person reads, so dimmer stops at
    /// legible.
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
    /// Restated means named on both sides, not relaxed to "dimmer than
    /// Ordinary". That weaker reading leaves Note at 15.88:1 and Number at
    /// 9.10:1 — the operand colours a Comment exists to stay behind —
    /// unguarded, so a Comment retuned past either of them would keep the
    /// suite green. So each colour is pinned to the side of Comment it sits
    /// on: Ordinary, Note, Output Portal and Number brighter, the three
    /// exceptions dimmer. A retune that crosses in either direction fails
    /// here — one because the rule broke, the other because the exception
    /// list went stale.
    ///
    #[test]
    fn comment_reads_dimmer_than_every_colour_but_its_named_exceptions_which_all_clear_the_floor() {
        let source_paint = SourcePaintSettings::default();
        let background = source_paint.source_background();
        let comment = contrast(source_paint.comment(), background);

        assert!(
            comment >= 4.5,
            "a Comment is read, not merely seen: {comment:.2}:1 against the Source background",
        );

        // Every Source colour but Sequence: the floor it clears, and the side
        // of a Comment it reads on. `true` is brighter than a Comment, `false`
        // is one of the three named exceptions that are dimmer and still
        // legible. Ordinary's own ratio is not restated here — it follows the
        // Ordinary default, which this test reads rather than pins.
        for (name, colour, brighter_than_comment) in [
            ("ordinary", source_paint.ordinary(), true),
            ("note", source_paint.note(), true),
            ("output_portal", source_paint.output_portal(), true),
            ("number", source_paint.number(), true),
            ("bang", source_paint.bang(), false),
            ("function", source_paint.function(), false),
            ("diagnostic", source_paint.diagnostic(), false),
        ] {
            let ratio = contrast(colour, background);
            assert!(
                ratio >= 4.5,
                "{name} is {ratio:.2}:1, below the 4.5:1 floor"
            );
            assert_eq!(
                ratio > comment,
                brighter_than_comment,
                "{name} is {ratio:.2}:1 against a Comment's {comment:.2}:1, the wrong side \
                 of the dimmest-Comment rule as the exceptions are named above",
            );
        }
    }

    ///
    /// The Okabe–Ito assignment's own choice for Sequence, `#0072B2`, measures
    /// 4.05:1 against the Source background — below the 4.5:1 floor every
    /// other Source colour clears. Restating the rule with this exception
    /// named, rather than silently lowering the floor or silently excluding
    /// Sequence from `comment_reads_dimmer_than_every_colour_but_its_named_
    /// exceptions_which_all_clear_the_floor`'s loop, is
    /// `syntax-highlighting/01`'s own acceptance criterion.
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
        let cursor_colour = Some(PALETTE.selection_fill);
        let ordinary = cell_visuals_with_cursor_colour(
            SourcePaint::Unclaimed,
            false,
            false,
            false,
            cursor_colour,
            source_paint,
        );
        let selected = cell_visuals_with_cursor_colour(
            SourcePaint::Unclaimed,
            false,
            true,
            false,
            cursor_colour,
            source_paint,
        );
        let cursor = cell_visuals_with_cursor_colour(
            SourcePaint::Unclaimed,
            false,
            true,
            true,
            cursor_colour,
            source_paint,
        );

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
                SourcePaint::Unclaimed,
                false,
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
                SourcePaint::Unclaimed,
                false,
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
    /// real nested Expression in `paint::tests::nested_function_and_operand_
    /// cells_tint_and_adjacent_same_colour_cells_merge_into_one_run`, and
    /// again on four different roots in `paint::tests::an_operand_cell_of_
    /// every_token_a_source_can_claim_is_tinted_with_its_own_colour`, which
    /// asserts each root's own two Cells beside its operands.
    ///
    #[test]
    fn a_function_cell_is_tinted_with_the_function_colour() {
        let source_paint = SourcePaintSettings::default();
        let function = painted(Some(&bound(Token::Function)), false, false, source_paint);

        assert_eq!(
            function.background,
            Some(tinted(source_paint, source_paint.function()))
        );
    }

    ///
    /// An Operand Cell is tinted with its declared Token's colour: Number,
    /// Note, Atom and Sequence. `Token::Char` is not one of them —
    /// `claim_paint`'s own doc explains why a claim's Token is never `Char`
    /// at all.
    ///
    /// Each Token is claimed in a shape a Source can produce — bound for
    /// Number and Note, unbound for Atom and Sequence, per [`unbound`] — so
    /// this reads as four arms of `claim_paint` rather than as a rule for a
    /// pair the Parser never mints. The tint itself is indifferent to the
    /// binding state by design (`operand_paint`: a Pending, Valid or Invalid
    /// operand tints alike), which is why the distinction costs the
    /// assertion nothing and is worth making anyway.
    /// `paint::tests::an_operand_cell_of_every_token_a_source_can_claim_is_
    /// tinted_with_its_own_colour` is the same rule from written Source.
    ///
    #[test]
    fn an_operand_cell_of_each_declared_token_is_tinted_with_its_own_colour() {
        let source_paint = SourcePaintSettings::default();

        for (claim, colour) in [
            (bound(Token::Number), source_paint.number()),
            (bound(Token::Note), source_paint.note()),
            (unbound(Token::Atom), source_paint.ordinary()),
            (unbound(Token::Sequence), source_paint.sequence()),
        ] {
            let visuals = painted(Some(&claim), false, false, source_paint);
            assert_eq!(
                visuals.background,
                Some(tinted(source_paint, colour)),
                "{:?} was not tinted with its own colour",
                claim.token
            );
        }
    }

    ///
    /// A Fill tint of `0` paints no background at all on a Function or
    /// Operand Cell, the same `None` an untinted role answers — not
    /// `Some` of the Source background, which `background_runs` would still
    /// have to walk as a Cell wanting a fill of its own.
    ///
    /// Function is claimed bound here and stays bound: an *unbound* Function
    /// claim is text that spells no Function and answers `None` whatever the
    /// strength, so it would pass this test for a reason that has nothing to
    /// do with `0%`. Atom and Sequence are claimed unbound because that is
    /// the only shape a Source produces for them (see [`unbound`]), and an
    /// unbound operand slot still tints at any non-zero strength, so they
    /// carry the same weight here that a bound Number does.
    /// `paint::tests::zero_percent_fill_tint_paints_no_tint_on_any_cell_a_
    /// source_claims` asks the same question of a whole Grid derived from
    /// written Source, where a Cell that does tint at the default strength is
    /// the control.
    ///
    #[test]
    fn zero_percent_fill_tint_paints_no_tint() {
        let mut source_paint = SourcePaintSettings::default();
        *source_paint.fill_tint_mut() = 0;

        for claim in [
            bound(Token::Function),
            bound(Token::Number),
            bound(Token::Note),
            unbound(Token::Atom),
            unbound(Token::Sequence),
        ] {
            let visuals = painted(Some(&claim), false, false, source_paint);
            assert_eq!(
                visuals.background, None,
                "{:?} was tinted at 0%",
                claim.token
            );
        }
    }

    ///
    /// Comment, Bang and an empty unclaimed Cell are not tinted, at the
    /// ticket's default (non-zero) Fill tint strength — so a missing
    /// exclusion cannot hide behind `zero_percent_fill_tint_paints_no_tint`'s
    /// strength being zero. A Leftover Char is not a fourth case here: it has
    /// no claim, so it is exactly the unclaimed-Cell case (`None`) below, not
    /// a Token this layer ever sees.
    ///
    /// `paint::tests::a_comment_a_bang_and_an_unclaimed_cell_take_no_tint_
    /// from_source` is the same three roles reached from written Source —
    /// `||hello`, a standalone `**`, and the blank Cells east of it.
    ///
    #[test]
    fn comment_bang_and_empty_unclaimed_cells_are_not_tinted() {
        let source_paint = SourcePaintSettings::default();
        assert_ne!(source_paint.fill_tint(), 0);

        let comment = unbound(Token::Comment);
        let bang = bound(Token::Bang);
        for claim in [Some(&comment), Some(&bang), None] {
            let visuals = painted(claim, false, false, source_paint);
            assert_eq!(visuals.background, None, "{claim:?} was tinted");
        }
    }

    ///
    /// The Cursor's own fill wins over the tint on its Cell: a Function or
    /// Operand Cell that is also selected answers the Cursor's colour, not
    /// the tint, the same priority `cursor_and_selection_override_the_
    /// ambient_field` already pins for a Cell with nothing to tint.
    ///
    /// `paint::tests::the_cursors_own_fill_wins_over_a_function_cells_tint`
    /// parks a real Cursor on `.+0102`'s `.` for the Function Cell, and
    /// `paint::tests::the_cursor_fills_over_an_invalid_operands_tint_and_
    /// leaves_its_diagnostic_glyph` does the same for an Operand Cell.
    ///
    #[test]
    fn the_cursors_own_fill_wins_over_the_tint_on_its_cell() {
        let source_paint = SourcePaintSettings::default();
        let cursor_colour = Color32::from_rgb(9, 8, 7);
        let tinted_function = tinted(source_paint, source_paint.function());
        let unselected = super::cell_visuals_with_cursor_colour(
            SourcePaint::Function,
            false,
            false,
            false,
            Some(cursor_colour),
            source_paint,
        );
        let selected = super::cell_visuals_with_cursor_colour(
            SourcePaint::Function,
            false,
            true,
            false,
            Some(cursor_colour),
            source_paint,
        );
        let cursor = super::cell_visuals_with_cursor_colour(
            SourcePaint::Function,
            false,
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

    ///
    /// `.+c40G`: an unbound Number entry — `c4` or `0G`, neither hexadecimal
    /// — draws its glyph in Diagnostic rather than Number, but keeps the
    /// Number tint on its background: the declared Token stays Number, which
    /// is why the tint stays (`syntax-highlighting/04`'s own Comment). This is
    /// the Invalid case: `written` is `true` because at least one Cell of the
    /// slot holds content that failed to bind. `.+0`'s second operand is the
    /// same shape with only one of its two Cells written, and `written`
    /// answers `true` for that whole slot too, so its blank Cell answers
    /// Diagnostic exactly as its written one does — `paint.rs`'s blank-glyph
    /// fallback is what keeps a Diagnostic foreground from ever being drawn
    /// on a Cell with no content, not a different verdict for it (ADR 0044).
    /// A bound Number is unaffected by `written` either way.
    ///
    #[test]
    fn an_invalid_operand_draws_diagnostic_but_keeps_its_declared_tint() {
        let source_paint = SourcePaintSettings::default();

        let invalid = painted(Some(&unbound(Token::Number)), true, false, source_paint);
        let valid = painted(Some(&bound(Token::Number)), true, false, source_paint);

        assert_eq!(invalid.foreground, source_paint.diagnostic());
        assert_eq!(
            invalid.background,
            Some(tinted(source_paint, source_paint.number())),
            "an unbound Number Cell still tints as Number"
        );
        assert_eq!(valid.foreground, source_paint.number());
        assert_eq!(valid.background, invalid.background);
    }

    ///
    /// An entirely blank Pending slot — an operand a Function has claimed but
    /// nothing yet fills — draws its declared Token colour rather than
    /// Diagnostic: `written` is `false` because no Cell of the slot holds
    /// content, so nothing there has failed to bind yet. The tint is
    /// unaffected either way: `syntax-highlighting/02` tints a Pending,
    /// Valid, or Invalid operand alike. This has no visible effect today —
    /// `paint.rs`'s blank-glyph fallback draws no character on a Pending
    /// Cell regardless of foreground — but it is the distinction `written`
    /// exists to answer correctly rather than by coincidence (ADR 0044).
    ///
    #[test]
    fn a_pending_operand_keeps_its_declared_colour_rather_than_diagnostic() {
        let source_paint = SourcePaintSettings::default();

        let pending = painted(Some(&unbound(Token::Number)), false, false, source_paint);

        assert_eq!(pending.foreground, source_paint.number());
        assert_eq!(
            pending.background,
            Some(tinted(source_paint, source_paint.number()))
        );
    }

    ///
    /// Text that spells no Function where an Expression could start — a lone
    /// `|`, both Cells of a written `07` — is `(Token::Function, atom: None)`
    /// with no spelling-specific case. No signature declared anything there,
    /// so it draws Ordinary with no tint, as an unclaimed Cell does. `written`
    /// plays no part in this rule.
    ///
    #[test]
    fn an_unbound_function_entry_draws_ordinary_with_no_tint() {
        let source_paint = SourcePaintSettings::default();

        let refused = painted(Some(&unbound(Token::Function)), false, false, source_paint);
        let recognized = painted(Some(&bound(Token::Function)), false, false, source_paint);

        assert_eq!(refused, painted(None, false, false, source_paint));
        assert_eq!(refused.foreground, source_paint.ordinary());
        assert_eq!(refused.background, None);
        assert_eq!(recognized.foreground, source_paint.function());
        assert_eq!(
            recognized.background,
            Some(tinted(source_paint, source_paint.function()))
        );
    }

    ///
    /// `output_portal` overrides an unclaimed Cell, an unbound refused
    /// Function claim (what a written scalar or Sequence answer parses as,
    /// per `.scratch/syntax-highlighting/issues/05`'s Answer), and a bound
    /// Operand claim alike: every one of them draws the Output Portal colour
    /// on the Output Portal's own Fill tint instead of whatever its claim
    /// alone would answer.
    ///
    #[test]
    fn output_portal_paints_over_an_unclaimed_an_unbound_and_a_bound_operand_cell() {
        let source_paint = SourcePaintSettings::default();
        let output_tinted = tinted(source_paint, source_paint.output_portal());

        for claim in [
            None,
            Some(&unbound(Token::Function)),
            Some(&bound(Token::Number)),
        ] {
            let visuals = painted(claim, false, true, source_paint);
            assert_eq!(
                visuals.foreground,
                source_paint.output_portal(),
                "{claim:?} did not draw the Output Portal colour"
            );
            assert_eq!(
                visuals.background,
                Some(output_tinted),
                "{claim:?} did not carry the Output Portal tint"
            );
        }
    }

    ///
    /// A Bang answer keeps its own Bang glyph colour rather than the Output
    /// Portal colour — a Bang is what a Producer emits, not a value it
    /// writes — but it still takes the Output Portal's Fill tint in place of
    /// its usual bare `None`, so it still reads as an Output Portal Cell.
    ///
    #[test]
    fn output_portal_keeps_the_bang_glyph_colour_but_takes_its_own_tint() {
        let source_paint = SourcePaintSettings::default();
        let bang = bound(Token::Bang);

        let ordinary_bang = painted(Some(&bang), false, false, source_paint);
        let portal_bang = painted(Some(&bang), false, true, source_paint);

        assert_eq!(
            ordinary_bang.background, None,
            "Bang is untinted ordinarily"
        );
        assert_eq!(portal_bang.foreground, source_paint.bang());
        assert_eq!(
            portal_bang.background,
            Some(tinted(source_paint, source_paint.output_portal()))
        );
    }

    ///
    /// A Cell that is a bound Function's own two-Cell spelling keeps its
    /// Function paint outright when `output_portal` is also true —
    /// `.scratch/syntax-highlighting/issues/06`'s precedence decision. This
    /// is the smallest rule that lets a producer's Output Portal show
    /// through a consumer's operand Cells while still letting a reader find
    /// the Function that stands on a Cell; `claim_paint` cannot and need not
    /// tell a root's own spelling from a nested one to apply it, since every
    /// Function's own two-Cell spelling carries `Token::Function` regardless
    /// of nesting.
    ///
    #[test]
    fn a_bound_function_spelling_wins_over_output_portal() {
        let source_paint = SourcePaintSettings::default();
        let function = bound(Token::Function);

        let ordinary = painted(Some(&function), false, false, source_paint);
        let overlapped = painted(Some(&function), false, true, source_paint);

        assert_eq!(
            overlapped, ordinary,
            "Output Portal changed a Function's own paint"
        );
        assert_eq!(overlapped.foreground, source_paint.function());
        assert_eq!(
            overlapped.background,
            Some(tinted(source_paint, source_paint.function()))
        );
    }

    ///
    /// The Cursor's own fill still wins outright over Output Portal, the
    /// same precedence it already holds over the Fill tint
    /// (`the_cursors_own_fill_wins_over_the_tint_on_its_cell`).
    ///
    #[test]
    fn the_cursors_own_fill_wins_over_output_portal_too() {
        let source_paint = SourcePaintSettings::default();
        let cursor_colour = Color32::from_rgb(9, 8, 7);

        let cursor = cell_visuals_with_cursor_colour(
            SourcePaint::Unclaimed,
            true,
            true,
            true,
            Some(cursor_colour),
            source_paint,
        );

        assert_eq!(cursor.background, Some(cursor_colour));
    }
}
