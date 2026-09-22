use std::sync::Arc;

use eframe::egui;

use egui::{Color32, CornerRadius, Shadow, Stroke, Style, Visuals, style::Selection};

use orcvs::source::{OperandState, SourcePaint, Token};

use crate::theme::Theme;

///
/// The chrome baseline `style()` opens with, before ADR 0053's chrome
/// derivation (`.scratch/theming/issues/03`) reads the resolved Theme for
/// these too. Spelled out here rather than read from [`crate::theme::okabe_ito`]
/// so this module states its own baseline independently of the Theme model —
/// the two are cross-checked in `theme::tests::okabe_ito_matches_todays_style_and_palette_constants`.
///
pub(crate) const DEFAULT_SOURCE_BACKGROUND: Color32 = Color32::from_rgb(0, 0, 0); // #000000 black
pub(crate) const DEFAULT_ORDINARY: Color32 = Color32::from_rgb(234, 235, 229); // #EAEBE5
pub(crate) const DEFAULT_BANG: Color32 = Color32::from_rgb(204, 121, 167); // #CC79A7 reddish purple

///
/// The console's fixed palette: chrome, grid geometry, and Cursor/selection
/// colours a viewer does not retheme.
///
/// The Source background and every Token's glyph colour used to live here too,
/// but `syntax-highlighting/01` moved them into `SourcePaintSettings` — a
/// console-owned settings value a viewer edited under `Theme → Source colours`
/// and persistence restored independently — so the Source Grid painted from a
/// value rather than from this fixed constant. `.scratch/theming/issues/06`
/// then removed `SourcePaintSettings` outright: every Source colour, border
/// and width, including the Cell grid line and Sector Seam this struct used
/// to hold on chrome's behalf, now comes from the resolved
/// [`crate::theme::Theme`]. What remains here is chrome's own baseline,
/// consumed by [`style`] and this module's `install_style` until
/// `.scratch/theming/issues/03` derives it from that same Theme.
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

///
/// `border_width` is in display points, already the fact-priority-selected
/// value `ordinary_border` (or the single-Cell Cursor's own
/// `cell.selection.border.width`) answers — never `Eq` since it carries an
/// `f32`, unlike this struct's other fields.
///
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct CellVisuals {
    pub background: Option<Color32>,
    pub border: Color32,
    pub border_width: f32,
    pub foreground: Color32,
}

///
/// How one Cell is coloured: fill, border and Token.
///
/// `background` is `None` when the Cell needs no fill of its own beyond the
/// live `theme.grid_background` — the `source_panel_frame` behind the Grid
/// has already painted that colour across the console, so a Cell with
/// nothing more to show answers `None` instead of repainting it. The
/// Cursor's own Cell is one such arm when its own fill is unset: painting a
/// fill there would hide the Cursor Effect's presentation. `Some` answers a
/// composited colour: the Cursor's own fill on the selected Cell, which wins
/// outright, or otherwise the resolved Theme's role/Diagnostic/Output Portal
/// channel for the Cell's fact, each composited over `theme.cell_background`
/// (see [`source_paint_visuals`]).
///
/// Composes the Source Paint decision with Cursor precedence, applied here
/// rather than inside it: the Cursor's own fill beats a role's channel
/// outright on its own Cell, and the single-Cell Cursor's own border
/// (`selection.border`/`.rest`) draws regardless of what stands on the Cell.
/// Every other Cell's border instead depends on what stands there:
/// [`ordinary_border`] composites Diagnostic/Output Portal over the ordinary
/// Grid border by the same fact priority `role_and_portal` already applies
/// to foreground and background. `paint` is the per-Cell language fact
/// `RenderCell::source_paint` answers from the shared Claim (ADR 0052),
/// including Pending, Valid, or Invalid for an Operand. `output_portal` is
/// the independent fact added by `syntax-highlighting/06`: whether this Cell
/// lies in a root Function's Output Portal Reservation
/// (`RenderCell::output_portal`), known from the current Source revision
/// alone.
///
pub(crate) fn cell_visuals_with_cursor_colour(
    paint: SourcePaint,
    output_portal: bool,
    selected: bool,
    cursor_visible: bool,
    cursor_fill: Option<Color32>,
    theme: &Theme,
) -> CellVisuals {
    let (foreground, role_background) = source_paint_visuals(paint, output_portal, theme);
    // The Cursor's own fill wins outright on its Cell, discarding whatever
    // the role/Diagnostic/Output Portal channels above answered: `cursor_fill`
    // decides alone, composited over the uniform Cell base the same way a
    // role channel is. Every other Cell keeps the role answer.
    let background = if selected {
        compose_cell_fill(theme.cell_background, cursor_fill)
    } else {
        role_background
    };
    // The single-Cell Cursor's own border is `selection.border`/`.rest` at
    // `cell.selection.border.width`, unaffected by Diagnostic/Output Portal —
    // `.scratch/theming/schema.md`'s Source composition step 5. Every other
    // Cell, including a multi-Cell Region's Cursor Cell (`selected` is false
    // there — see [`crate::paint::Paint::derive_with_theme`]), keeps the
    // ordinary Grid border, which [`ordinary_border`] composites with
    // Diagnostic/Output Portal by the same fact priority `role_and_portal`
    // already applies to foreground and background.
    let (border, border_width) = if cursor_visible {
        (
            theme.selection_border,
            theme.cell_selection_border_width.points(),
        )
    } else if selected {
        (
            theme.selection_border_rest,
            theme.cell_selection_border_width.points(),
        )
    } else {
        ordinary_border(paint, output_portal, theme)
    };
    CellVisuals {
        background,
        border,
        border_width,
        foreground,
    }
}

///
/// The ordinary Grid border's colour and width, composited with the
/// Diagnostic and Output Portal border channels by the same fact priority
/// [`role_and_portal`] applies to foreground and background —
/// `.scratch/theming/schema.md`'s composition step 4, "Border channels use
/// the same fact priority and compose over the ordinary Grid border": Output
/// Portal over Diagnostic over the plain Grid border, with a bound Function
/// bypassed exactly as it is there.
///
/// Colour composites with [`blend_channel`], equivalent to the pinned [`Color32::blend`] operation
/// every other channel in this module uses, so `diagnostic.border`'s and
/// `output_portal.border`'s transparent Okabe–Ito defaults leave
/// `theme.grid_border` showing unchanged — this is defect 3's fix: neither
/// channel was read at all before, so a non-transparent custom value never
/// painted.
///
/// Width is *picked* rather than blended — a stroke has one width, and
/// composing two would not mean alpha compositing — so whichever channel
/// wins the same priority decides the stroke's own display-point width.
/// `diagnostic.border.width`/`output_portal.border.width` at zero therefore
/// hides that Cell's whole border exactly as `grid.border.width` zero
/// already hides an unaffected Cell's, per schema's "Width 0 hides the
/// stroke" — extended here to whichever channel is actually drawn once
/// Diagnostic or Output Portal wins.
///
/// Called only for a Cell keeping the ordinary Grid border: the single-Cell
/// Cursor's own frame is decided in [`cell_visuals_with_cursor_colour`]
/// before this function is ever reached.
///
fn ordinary_border(paint: SourcePaint, output_portal: bool, theme: &Theme) -> (Color32, f32) {
    let invalid_operand = matches!(
        paint,
        SourcePaint::Operand {
            state: OperandState::Invalid,
            ..
        }
    );
    // "A bound Function retains all its own paint inside a Portal" — the
    // same bypass `role_and_portal` applies to foreground and background.
    let portal_applies = output_portal && paint != SourcePaint::Function;

    match (invalid_operand, portal_applies) {
        (false, false) => (theme.grid_border, theme.grid_border_width.points()),
        (true, false) => (
            blend_channel(theme.grid_border, theme.diagnostic_border),
            theme.diagnostic_border_width.points(),
        ),
        (false, true) => (
            blend_channel(theme.grid_border, theme.output_portal_border),
            theme.output_portal_border_width.points(),
        ),
        (true, true) => (
            blend_channel(
                blend_channel(theme.grid_border, theme.diagnostic_border),
                theme.output_portal_border,
            ),
            theme.output_portal_border_width.points(),
        ),
    }
}

///
/// Composites `fill` — a Cursor or Region fill, or `None` when neither
/// applies — over `cell_background`, then answers `None` when the composited
/// result is fully transparent so a Cell with nothing to show still costs
/// [`crate::paint::Paint::background_runs`] no Cell to walk.
///
/// [`blend_channel`] preserves `Color32::blend`, `ecolor`'s premultiplied-alpha "self behind
/// on_top" compositing (`ecolor-0.36.2/src/color32.rs`), the "pinned colour
/// operations" `.scratch/theming/schema.md`'s composition step 6 asks every
/// resolved-Theme composite to share with painting. `fill.unwrap_or
/// (TRANSPARENT)` composited onto `cell_background` when `fill` is `None`
/// leaves `cell_background` itself unchanged — blending a transparent colour
/// on top of anything is the identity — so this one expression also answers
/// the plain "nothing above the uniform Cell base" case every caller below
/// asks for with `fill: None`.
///
pub(crate) fn compose_cell_fill(
    cell_background: Color32,
    fill: Option<Color32>,
) -> Option<Color32> {
    let composed = blend_channel(cell_background, fill.unwrap_or(Color32::TRANSPARENT));

    (composed.a() != 0).then_some(composed)
}

/// Preserve egui's blend for partial alpha while avoiding its channel arithmetic
/// for the transparent and opaque channels used by the built-in Theme.
#[inline]
fn blend_channel(base: Color32, on_top: Color32) -> Color32 {
    if on_top == Color32::TRANSPARENT {
        base
    } else if on_top.a() == 255 {
        on_top
    } else {
        base.blend(on_top)
    }
}

///
/// Foreground and background together, from a Cell's finished Source Paint
/// fact and whether it lies in a root Function's Output Portal Reservation,
/// resolved from `theme` — the flat, once-per-frame lookup every field below
/// is a direct read of, never a per-Cell walk, match over a table, or hash
/// (`.scratch/theming/issues/06`'s `paint-cell-cost` constraint).
///
/// # Channel composition
///
/// Every composited channel below uses [`compose_cell_fill`] (background) or
/// [`blend_channel`] (foreground): a transparent Theme channel
/// therefore reveals whatever sits beneath it, and an opaque one replaces it
/// outright, which is what lets the Okabe–Ito built-in — every role channel
/// opaque except Ordinary, Comment and untouched Bang, which are transparent
/// — reproduce today's exact appearance while an arbitrary custom Theme can
/// still compose partial alpha correctly. [`role_and_portal`] answers the
/// foreground and the *raw*, not-yet-composited background (a role's own
/// channel, blended with Diagnostic and Output Portal as those apply); this
/// function composites that raw answer over `theme.cell_background` and
/// collapses a fully transparent result to `None` for [`CellVisuals`]'s
/// existing "nothing to paint" fast path.
///
/// | Fact | Foreground | Background (before Cursor/Region) |
/// |---|---|---|
/// | Unclaimed | `source.ordinary` | `source.ordinary.background` |
/// | Comment | `source.comment` | `source.comment.background` |
/// | Bang (no Portal) | `source.bang` | `source.bang.background` |
/// | Function | `source.function` | `source.function.background` |
/// | Operand, Pending/Valid | declared Token colour | declared Token background |
/// | Operand, Invalid | Token colour ▸ `diagnostic.foreground` | Token background ▸ `diagnostic.background` |
/// | any non-Function, Portal | above ▸ `output_portal.foreground` | above ▸ `output_portal.background` |
/// | Bang, Portal | `source.bang` (no blend) | `source.bang.background` ▸ `output_portal.background` |
/// | Function, Portal | `source.function` (bypassed) | `source.function.background` (bypassed) |
///
/// "▸" is [`Color32::blend`]: the right side composited over the left, so an
/// opaque right side replaces the left outright and a transparent one leaves
/// it showing — this is `.scratch/theming/schema.md`'s "A transparent fact
/// channel reveals the underlying Token channel with alpha compositing."
/// Every "before Cursor/Region" background above is finally composited over
/// `theme.cell_background` by this function, which is why the table states
/// it separately: `cell.background` is a uniform base beneath every role,
/// never itself one of the facts above, so it never suppresses the Region
/// fallback [`crate::paint::Paint::derive_with_theme`] applies when a Cell's
/// answer here is `None`.
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
/// keeps its own Bang glyph colour rather than blending with the Output
/// Portal foreground, because a Bang is what a Producer emits rather than a
/// value it writes, but its background still takes the Output Portal's own
/// channel in place of Bang's usual transparent one — so a Bang answer still
/// reads as an Output Portal Cell.
///
/// # Otherwise, from the Source Paint fact
///
/// An Unclaimed Cell answers Ordinary. Bang and Comment always answer their
/// own channel. Function — root or nested — draws its own channel. Text that
/// spells no Function where an Expression could start (`hi`, both Cells of a
/// written `07`, a lone `|`) answers Ordinary, as an unclaimed Cell does:
/// `RenderCell::source_paint` answers it as Unclaimed because the Parser's
/// attempted Function classification is not a Paint distinction. Diagnostic
/// belongs to an Invalid operand slot. Number, Note, Atom and Sequence
/// Operand facts read [`operand_paint`].
///
fn source_paint_visuals(
    paint: SourcePaint,
    output_portal: bool,
    theme: &Theme,
) -> (Color32, Option<Color32>) {
    let (foreground, background) = role_and_portal(paint, output_portal, theme);

    // The `None`/`Some` decision reads `background`'s *own* alpha — before
    // it is composited with `theme.cell_background` — not the composited
    // result's: `.scratch/theming/schema.md`'s "[`cell.background`] does not
    // count as a fact fill when evaluating Region fallback," which
    // `Paint::derive_with_theme` reads this `None` to decide, so a role
    // that contributes nothing must answer `None` here even when
    // `cell.background` is itself visible — [`compose_cell_fill`]'s
    // composed-alpha collapse is the wrong check for that signal, though it
    // is the right one for the Cursor's own fill and the Region fill below,
    // which have no further fallback of their own to protect.
    let background =
        (background.a() != 0).then(|| blend_channel(theme.cell_background, background));

    (foreground, background)
}

///
/// [`source_paint_visuals`]'s foreground and *raw* (not yet composited over
/// `theme.cell_background`) background, before the final composite that
/// function performs — factored out so [`source_paint_visuals`] is the one
/// place that composite happens.
///
fn role_and_portal(paint: SourcePaint, output_portal: bool, theme: &Theme) -> (Color32, Color32) {
    let (foreground, background) = role(paint, theme);

    if paint == SourcePaint::Function {
        // "A bound Function retains all its own paint inside a Portal."
        return (foreground, background);
    }
    if output_portal {
        let bang = paint == SourcePaint::Bang;
        // "Bang retains its glyph foreground" — no blend at all, not even
        // with an opaque Output Portal foreground.
        let foreground = if bang {
            foreground
        } else {
            blend_channel(foreground, theme.output_portal_foreground)
        };
        return (
            foreground,
            blend_channel(background, theme.output_portal_background),
        );
    }
    (foreground, background)
}

///
/// The declared role's own foreground and raw background for `paint`, before
/// Output Portal composition — Diagnostic already applied for an Invalid
/// Operand by [`operand_paint`].
///
fn role(paint: SourcePaint, theme: &Theme) -> (Color32, Color32) {
    match paint {
        SourcePaint::Unclaimed => (theme.source_ordinary, theme.source_ordinary_background),
        SourcePaint::Bang => (theme.source_bang, theme.source_bang_background),
        SourcePaint::Comment => (theme.source_comment, theme.source_comment_background),
        SourcePaint::Function => (theme.source_function, theme.source_function_background),
        SourcePaint::Operand { token, state } => {
            let (colour, background) = match token {
                Token::Number => (theme.source_number, theme.source_number_background),
                Token::Note => (theme.source_note, theme.source_note_background),
                Token::Atom => (theme.source_ordinary, theme.source_atom_background),
                Token::Sequence => (theme.source_sequence, theme.source_sequence_background),
                Token::Bang | Token::Comment | Token::Function | Token::Char => {
                    unreachable!("SourcePaint::Operand carries only a declared operand Token")
                }
            };
            operand_paint(colour, background, state, theme)
        }
    }
}

///
/// An Operand Cell's foreground and raw background, from its declared
/// `colour`/`background` (`Number`, `Note`, `Atom` or `Sequence`) and its
/// finished state.
///
/// The Parser labels an operand slot with its signature's declared Token
/// whether or not what stands there binds, so Valid, Pending, and Invalid all
/// keep the declared role's own background — `.scratch/theming/schema.md`:
/// "Declared Number/Note/Atom/Sequence operands retain their role background
/// in Pending, Valid and Invalid states." The foreground differs: a Valid or
/// Pending slot draws `colour` outright, and an Invalid one — Cells whose
/// written content failed to bind — blends the Diagnostic foreground channel
/// over it, so a transparent `diagnostic.foreground` still reveals the
/// declared role while an opaque one (Okabe–Ito's) replaces it exactly as
/// before. `.+0`'s second operand is this Invalid case: one Cell holds `0`,
/// the other is blank, and `written` is true for the whole slot, so both
/// Cells alike answer Diagnostic — `paint.rs`'s blank-glyph fallback is what
/// keeps the blank one's colour from ever being drawn, not a different
/// verdict for it. An entirely blank slot is Pending instead, and keeps
/// `colour` rather than turning Diagnostic — a distinction with no visible
/// effect today, since a Pending Cell's content is always the blank glyph
/// regardless of foreground, but the Claim answers it once, as it is built,
/// rather than leaving the console to infer it (ADR 0044/0052).
///
fn operand_paint(
    colour: Color32,
    background: Color32,
    state: OperandState,
    theme: &Theme,
) -> (Color32, Color32) {
    match state {
        OperandState::Pending | OperandState::Valid => (colour, background),
        OperandState::Invalid => (
            blend_channel(colour, theme.diagnostic_foreground),
            blend_channel(background, theme.diagnostic_background),
        ),
    }
}

///
/// Attenuates `base` — the resolved Theme's `sector.seam` — by
/// `strength_percent`: a pure reading of the Theme colour a Sector Seam is
/// drawn with, not a lookup of its own. `base` is a plain `Color32` argument
/// rather than a `&Theme` so the per-Cell call sites in `paint.rs` read one
/// field once, before their loop, and pass the same value to every Cell's
/// seam instead of indexing `theme.sector_seam` again per Cell.
///
pub(crate) fn sector_line(strength_percent: u8, base: Color32) -> Color32 {
    let [red, green, blue, base_alpha] = base.to_srgba_unmultiplied();
    let alpha = u16::from(base_alpha) * u16::from(strength_percent.min(100)) / 100;
    Color32::from_rgba_unmultiplied(red, green, blue, alpha as u8)
}

///
/// The chrome style the console opens with.
///
/// Set once at `Console::new` rather than read every frame — unlike the
/// Source Grid, egui's own `extreme_bg_color`, `faint_bg_color`,
/// `error_fg_color` and `warn_fg_color` are not consulted per Cell, so there
/// is no seam that would make them track a live Theme the way `show_source`
/// does. They start at the Okabe–Ito built-in's own values so the chrome and
/// the Grid agree on first paint. Deriving this baseline from the resolved
/// Theme, rather than from the fixed constants above, is
/// `.scratch/theming/issues/03`'s job, not this slice's.
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

///
/// Registers [`style`] for both [`egui::Theme::Dark`] and [`egui::Theme::Light`]
/// and touches nothing else on `ctx` — in particular it never calls
/// [`egui::Context::set_theme`].
///
/// `ThemePreference` is part of the `Options` eframe restores into egui
/// memory before it constructs the application, but eframe restores memory
/// without reinstalling a style. `Console::new` calls this every launch so
/// both theme slots hold the one console style before the first frame,
/// whatever the preference resolves to; a `set_theme` call here would
/// overwrite the very value just restored. One palette exists, so both
/// slots take it — a viewer whose preference resolves to Light must not see
/// egui's own default light style beside a Grid still painted from the dark
/// `PALETTE`.
///
pub(crate) fn install_style(ctx: &egui::Context) {
    let style = Arc::new(style());
    ctx.set_style_of(egui::Theme::Dark, Arc::clone(&style));
    ctx.set_style_of(egui::Theme::Light, style);
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::{
        CellVisuals, ConsolePalette, PALETTE, cell_visuals_with_cursor_colour, install_style,
        sector_line, style,
    };
    use crate::theme::{Theme, okabe_ito};
    use egui::{Color32, Stroke};
    use orcvs::source::{OperandState, SourcePaint, Token};

    ///
    /// [`cell_visuals_with_cursor_colour`] with no Cursor on the Cell:
    /// unselected, with the Cursor's own colour set the way `Paint::derive`
    /// sets it in production, so a test that does not exercise Cursor
    /// precedence need not repeat it.
    ///
    /// `paint` is the per-Cell language fact (ADR 0052), fed in as
    /// itself. Deriving it here from a hand-built claim would restate
    /// `RenderCell::source_paint` — a restatement
    /// nothing compares against, free to drift from the mapping it copies
    /// while every test here stays green. Which claim a written Source
    /// answers with which [`SourcePaint`] is proven where a Source can be
    /// written: `orcvs::render_frame`'s
    /// `every_cell_of_a_claim_reads_the_written_answer_its_claim_was_built_with`,
    /// `a_partly_written_operand_is_invalid_on_every_cell` and
    /// `a_bound_number_sits_beside_an_unbound_blank_operand_claim`, and
    /// again through the console's own Render Frame in `paint::tests`. This
    /// layer's subject is the colour a finished fact paints, not the fact.
    ///
    fn painted(paint: SourcePaint, output_portal: bool, theme: &Theme) -> CellVisuals {
        cell_visuals_with_cursor_colour(
            paint,
            output_portal,
            false,
            false,
            Some(PALETTE.selection_fill),
            theme,
        )
    }

    ///
    /// A declared operand slot's finished fact: its Token and its state.
    ///
    /// Which pairs a Source can actually produce is a fact about the Parser
    /// rather than about this decision function, so the fixtures below state
    /// the pair they mean and
    /// `paint::tests::an_operand_cell_of_every_token_a_source_can_claim_is_
    /// tinted_with_its_own_colour` drives `.+`, `:#C4D4`, `:&` and `:<XY`
    /// through the Render Frame to prove those pairs are the real ones.
    /// `Token::Atom` and `Token::Sequence` are "declarations no Cells spell"
    /// (`lang/src/expression.rs`'s own words on `Token`): `Token::decode`
    /// refuses both outright, so the only thing that can satisfy either slot
    /// is a nested Function — and when one stands there,
    /// `take_language_unit`'s `is_function_next()` branch records the entry
    /// under `Token::Function` (`lang/src/parser.rs`), so nothing ever binds
    /// in an `Atom` or `Sequence` slot that reaches this layer. Both are
    /// therefore Pending where their Cells are blank and Invalid where they
    /// are written, never Valid, which is why the fixtures below claim them
    /// Pending and reach for Valid only where a Source can bind — `:#C4D4`
    /// binds both Note slots, `.+0102` both Number ones.
    ///
    fn operand(token: Token, state: OperandState) -> SourcePaint {
        SourcePaint::Operand { token, state }
    }

    ///
    /// `theme.md` is the decided record for the fixed palette. These literals
    /// are that record in `Color32` form: a later change to the chrome, grid
    /// geometry, or Cursor/selection colours fails here, and the same commit
    /// must change the document. The Source background and every Token's
    /// glyph colour are pinned the same way in `theme::tests`, against
    /// [`okabe_ito`] rather than this constant.
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
    /// Covers every Source Paint fact that draws its own colour, plus an
    /// Unclaimed Cell for Ordinary — the role a Leftover Char shares with
    /// it, since a Leftover Char is Unclaimed too
    /// (`SourceRevision::token_at`'s fallback stands in for a Cell no
    /// Expression claimed).
    ///
    #[test]
    fn semantic_glyph_colours_are_distinct_and_read_from_the_theme() {
        let theme = okabe_ito();
        let function = painted(SourcePaint::Function, false, &theme);
        let number = painted(operand(Token::Number, OperandState::Valid), false, &theme);
        let note = painted(operand(Token::Note, OperandState::Valid), false, &theme);
        let ordinary = painted(SourcePaint::Unclaimed, false, &theme);
        let bang = painted(SourcePaint::Bang, false, &theme);
        let comment = painted(SourcePaint::Comment, false, &theme);
        // Atom and Sequence are Pending because that is the only shape a
        // Source produces for them — see [`operand`]. Neither assertion
        // turns on it: a Pending slot draws its declared colour.
        let sequence = painted(
            operand(Token::Sequence, OperandState::Pending),
            false,
            &theme,
        );

        assert_eq!(function.foreground, theme.source_function);
        assert_eq!(number.foreground, theme.source_number);
        assert_eq!(note.foreground, theme.source_note);
        assert_eq!(ordinary.foreground, theme.source_ordinary);
        assert_eq!(bang.foreground, theme.source_bang);
        assert_eq!(comment.foreground, theme.source_comment);
        assert_eq!(sequence.foreground, theme.source_sequence);
        assert_ne!(number.foreground, function.foreground);
        assert_ne!(number.foreground, note.foreground);
        assert_ne!(number.foreground, ordinary.foreground);
        assert_ne!(comment.foreground, ordinary.foreground);
        // Atom follows Ordinary: a Cell painting no glyph of its own has
        // nothing to colour differently.
        assert_eq!(
            painted(operand(Token::Atom, OperandState::Pending), false, &theme).foreground,
            ordinary.foreground
        );
        // Sequence does not share Ordinary's colour: it has its own field
        // and its own default, distinct from every other role including
        // Ordinary.
        assert_ne!(sequence.foreground, ordinary.foreground);
        assert_ne!(sequence.foreground, comment.foreground);

        // A changed Theme reaches `cell_visuals_with_cursor_colour` on the
        // very next call — the Source Grid paints from the resolved Theme,
        // not from a fixed palette, so a custom Theme's colours would preview
        // immediately once loading exists (`.scratch/theming/issues/07`).
        let retuned = Theme {
            source_function: Color32::from_rgb(1, 2, 3),
            ..okabe_ito()
        };
        assert_eq!(
            painted(SourcePaint::Function, false, &retuned).foreground,
            Color32::from_rgb(1, 2, 3)
        );
    }

    #[test]
    fn sector_line_strength_only_attenuates_the_base_colours_alpha() {
        let base = PALETTE.sector_line;
        assert_eq!(sector_line(100, base), base);
        let [red, green, blue, alpha] = sector_line(50, base).to_srgba_unmultiplied();
        assert!(red.abs_diff(55) <= 2);
        assert!(green.abs_diff(101) <= 2);
        assert!(blue.abs_diff(86) <= 2);
        assert_eq!(alpha, 55);
        assert_eq!(sector_line(8, base).a(), 8);
        assert_eq!(sector_line(255, base), base);
    }

    ///
    /// A custom Theme's `sector.seam` reaches [`sector_line`] as the base
    /// colour it attenuates, not [`PALETTE`]'s fixed one — the paint-layer
    /// half of defect 1: Sector Seams read `theme.sector_seam`, never the
    /// chrome-only constant.
    ///
    #[test]
    fn sector_line_attenuates_the_theme_seam_colour_not_the_fixed_palette() {
        let retuned = Color32::from_rgba_unmultiplied(1, 2, 3, 200);

        let attenuated = sector_line(100, retuned);

        assert_eq!(attenuated, retuned);
        assert_ne!(attenuated, PALETTE.sector_line);
    }

    #[test]
    fn cursor_and_selection_override_the_ambient_field() {
        let theme = okabe_ito();
        let cursor_colour = Some(PALETTE.selection_fill);
        let ordinary = cell_visuals_with_cursor_colour(
            SourcePaint::Unclaimed,
            false,
            false,
            false,
            cursor_colour,
            &theme,
        );
        let selected = cell_visuals_with_cursor_colour(
            SourcePaint::Unclaimed,
            false,
            true,
            false,
            cursor_colour,
            &theme,
        );
        let cursor = cell_visuals_with_cursor_colour(
            SourcePaint::Unclaimed,
            false,
            true,
            true,
            cursor_colour,
            &theme,
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

    ///
    /// Defect 1: a Cell's border reads `theme.grid_border`,
    /// `theme.selection_border` and `theme.selection_border_rest` — the
    /// resolved Theme, not the fixed [`PALETTE`] constants Okabe–Ito happens
    /// to share their default values with. Retuning each of the three
    /// changes the matching `CellVisuals::border` on the next call, the way
    /// `semantic_glyph_colours_are_distinct_and_read_from_the_theme` already
    /// proves for glyph colours.
    ///
    #[test]
    fn each_border_channel_reads_its_own_theme_colour_not_the_fixed_palette() {
        let retuned_grid = Theme {
            grid_border: Color32::from_rgb(11, 22, 33),
            ..okabe_ito()
        };
        let ordinary = cell_visuals_with_cursor_colour(
            SourcePaint::Unclaimed,
            false,
            false,
            false,
            None,
            &retuned_grid,
        );
        assert_eq!(ordinary.border, Color32::from_rgb(11, 22, 33));
        assert_ne!(ordinary.border, PALETTE.grid_line);

        let retuned_rest = Theme {
            selection_border_rest: Color32::from_rgb(44, 55, 66),
            ..okabe_ito()
        };
        let selected = cell_visuals_with_cursor_colour(
            SourcePaint::Unclaimed,
            false,
            true,
            false,
            None,
            &retuned_rest,
        );
        assert_eq!(selected.border, Color32::from_rgb(44, 55, 66));
        assert_ne!(selected.border, PALETTE.selection_stroke_rest);

        let retuned_visible = Theme {
            selection_border: Color32::from_rgb(77, 88, 99),
            ..okabe_ito()
        };
        let cursor = cell_visuals_with_cursor_colour(
            SourcePaint::Unclaimed,
            false,
            true,
            true,
            None,
            &retuned_visible,
        );
        assert_eq!(cursor.border, Color32::from_rgb(77, 88, 99));
        assert_ne!(cursor.border, PALETTE.selection_stroke);
    }

    ///
    /// The three optional-fill states an unselected `cursor.background`
    /// exercises once resolved onto a Cell: omission/inheritance already
    /// collapses to one of the other two by `theme::resolve`
    /// (`theme::tests::omitted_optional_fills_inherit_the_parent`), so what
    /// this proves is the remaining pair at the paint layer — `None`
    /// (cleared, "supplies no Cursor fill") answers no background, and
    /// `Some`, including a fully transparent colour, is a supplied value that
    /// paints exactly what it names.
    ///
    #[test]
    fn cursor_fill_is_optional_and_defaults_to_the_uniform_cell_background() {
        let colour = Color32::from_rgb(1, 2, 3);
        let theme = okabe_ito();
        assert_eq!(
            super::cell_visuals_with_cursor_colour(
                SourcePaint::Unclaimed,
                false,
                true,
                true,
                None,
                &theme,
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
                &theme,
            )
            .background,
            Some(colour)
        );
        // An explicit but fully transparent Cursor fill is still a supplied
        // value: it replaces the role fill outright, painting nothing extra
        // over the transparent uniform Cell base — distinct in kind from
        // `None` above, though both compose to no visible paint here.
        assert_eq!(
            super::cell_visuals_with_cursor_colour(
                SourcePaint::Unclaimed,
                false,
                true,
                true,
                Some(Color32::TRANSPARENT),
                &theme,
            )
            .background,
            None
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
        let theme = okabe_ito();
        let function = painted(SourcePaint::Function, false, &theme);

        assert_eq!(function.background, Some(theme.source_function_background));
        // The literal `.scratch/theming/schema.md` records, independent of
        // both the Theme field above and the composition that surfaces it —
        // what `syntax-highlighting/02` computed as a 16% Fill tint of
        // Function over black before this Theme replaced that formula with a
        // stored value.
        assert_eq!(
            function.background,
            Some(Color32::from_rgba_unmultiplied(0x00, 0x19, 0x12, 0xFF))
        );
    }

    ///
    /// An Operand Cell is tinted with its declared Token's colour: Number,
    /// Note, Atom and Sequence. `Token::Char` is not one of them —
    /// [`SourcePaint::Operand`] carries only a declared operand Token, which
    /// is why `source_paint_visuals`'s `Char` arm is unreachable.
    ///
    /// Each Token is stated in a shape a Source can produce — Valid for
    /// Number and Note, Pending for Atom and Sequence, per [`operand`] — so
    /// this reads as four arms of `source_paint_visuals` rather than as a
    /// rule for a pair the Parser never mints. The tint itself is indifferent
    /// to the binding state by design (`operand_paint`: a Pending, Valid or
    /// Invalid operand tints alike), which is why the distinction costs the
    /// assertion nothing and is worth making anyway.
    /// `paint::tests::an_operand_cell_of_every_token_a_source_can_claim_is_
    /// tinted_with_its_own_colour` is the same rule from written Source.
    ///
    #[test]
    fn an_operand_cell_of_each_declared_token_is_tinted_with_its_own_colour() {
        let theme = okabe_ito();

        for (paint, background) in [
            (
                operand(Token::Number, OperandState::Valid),
                theme.source_number_background,
            ),
            (
                operand(Token::Note, OperandState::Valid),
                theme.source_note_background,
            ),
            (
                operand(Token::Atom, OperandState::Pending),
                theme.source_atom_background,
            ),
            (
                operand(Token::Sequence, OperandState::Pending),
                theme.source_sequence_background,
            ),
        ] {
            let visuals = painted(paint, false, &theme);
            assert_eq!(
                visuals.background,
                Some(background),
                "{paint:?} was not tinted with its own colour",
            );
        }
    }

    ///
    /// A role background set fully transparent in a custom Theme paints no
    /// background at all on a Function or Operand Cell, the same `None` an
    /// untinted role answers — not `Some` of a transparent colour, which
    /// `background_runs` would still have to walk as a Cell wanting a fill of
    /// its own. `.scratch/theming/schema.md`'s role backgrounds replace
    /// `syntax-highlighting/02`'s runtime Fill tint percentage with stored
    /// values, so this is that ticket's "0% tints nothing" rule restated for
    /// the Theme model: a role whose background Theme value is transparent,
    /// not merely a strength of zero.
    ///
    /// The Function arm is [`SourcePaint::Function`] and stays that: text
    /// that spells no Function is Unclaimed and answers `None` whatever the
    /// role's background, so it would pass this test for a reason that has
    /// nothing to do with the Theme value under test. Atom and Sequence are
    /// Pending because that is the only shape a Source produces for them
    /// (see [`operand`]).
    ///
    #[test]
    fn a_transparent_role_background_paints_no_tint() {
        let transparent = Theme {
            source_function_background: Color32::TRANSPARENT,
            source_number_background: Color32::TRANSPARENT,
            source_note_background: Color32::TRANSPARENT,
            source_atom_background: Color32::TRANSPARENT,
            source_sequence_background: Color32::TRANSPARENT,
            ..okabe_ito()
        };

        for paint in [
            SourcePaint::Function,
            operand(Token::Number, OperandState::Valid),
            operand(Token::Note, OperandState::Valid),
            operand(Token::Atom, OperandState::Pending),
            operand(Token::Sequence, OperandState::Pending),
        ] {
            let visuals = painted(paint, false, &transparent);
            assert_eq!(
                visuals.background, None,
                "{paint:?} was tinted despite a transparent role background"
            );
        }
    }

    ///
    /// Comment, Bang and an empty unclaimed Cell are not tinted at the
    /// Okabe–Ito built-in's own (nonzero-alpha-elsewhere) values — their
    /// three role backgrounds are transparent by definition
    /// (`.scratch/theming/schema.md`: "ordinary/Char/comment/Bang default to
    /// transparent"). A Leftover Char is not a fourth case here: it is
    /// Unclaimed, so it is exactly the [`SourcePaint::Unclaimed`] case below,
    /// not a fact of its own.
    ///
    #[test]
    fn comment_bang_and_empty_unclaimed_cells_are_not_tinted() {
        let theme = okabe_ito();

        for paint in [
            SourcePaint::Comment,
            SourcePaint::Bang,
            SourcePaint::Unclaimed,
        ] {
            let visuals = painted(paint, false, &theme);
            assert_eq!(visuals.background, None, "{paint:?} was tinted");
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
        let theme = okabe_ito();
        let cursor_colour = Color32::from_rgb(9, 8, 7);
        let tinted_function = theme.source_function_background;
        let unselected = super::cell_visuals_with_cursor_colour(
            SourcePaint::Function,
            false,
            false,
            false,
            Some(cursor_colour),
            &theme,
        );
        let selected = super::cell_visuals_with_cursor_colour(
            SourcePaint::Function,
            false,
            true,
            false,
            Some(cursor_colour),
            &theme,
        );
        let cursor = super::cell_visuals_with_cursor_colour(
            SourcePaint::Function,
            false,
            true,
            true,
            Some(cursor_colour),
            &theme,
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
    /// [`OperandState::Invalid`], which the slot's shared Claim decides for
    /// the whole slot: at least one Cell of it holds content that failed to
    /// bind. `.+0`'s second operand is the same shape with only one of its
    /// two Cells written, and its Claim answers Invalid for that
    /// whole slot too, so its blank Cell answers Diagnostic exactly as its
    /// written one does — `paint.rs`'s blank-glyph fallback is what keeps a
    /// Diagnostic foreground from ever being drawn on a Cell with no
    /// content, not a different verdict for it (ADR 0044). A Valid Number is
    /// unaffected either way.
    ///
    #[test]
    fn an_invalid_operand_draws_diagnostic_but_keeps_its_declared_tint() {
        let theme = okabe_ito();

        let invalid = painted(operand(Token::Number, OperandState::Invalid), false, &theme);
        let valid = painted(operand(Token::Number, OperandState::Valid), false, &theme);

        assert_eq!(invalid.foreground, theme.diagnostic_foreground);
        assert_eq!(
            invalid.background,
            Some(theme.source_number_background),
            "an Invalid Number Cell still tints as Number: Okabe–Ito's \
             diagnostic.background is transparent, so it reveals the Number \
             role background unchanged"
        );
        assert_eq!(valid.foreground, theme.source_number);
        assert_eq!(valid.background, invalid.background);
    }

    ///
    /// A transparent `diagnostic.foreground` — unlike Okabe–Ito's opaque one
    /// — reveals the declared role's own foreground instead of replacing it:
    /// `.scratch/theming/schema.md`: "A transparent fact channel reveals the
    /// underlying Token channel with alpha compositing."
    ///
    #[test]
    fn a_transparent_diagnostic_foreground_reveals_the_declared_role() {
        let theme = Theme {
            diagnostic_foreground: Color32::TRANSPARENT,
            ..okabe_ito()
        };

        let invalid = painted(operand(Token::Number, OperandState::Invalid), false, &theme);

        assert_eq!(invalid.foreground, theme.source_number);
    }

    ///
    /// A partial-alpha `diagnostic.background` blends over the declared
    /// role's own background rather than fully replacing or fully revealing
    /// it — the middle case between Okabe–Ito's transparent default (reveals
    /// the role unchanged, pinned above) and an opaque one (would replace it
    /// outright, the same way `diagnostic.foreground` does when opaque).
    ///
    #[test]
    fn a_partial_alpha_diagnostic_background_blends_over_the_role_background() {
        let theme = okabe_ito();
        let half = Color32::from_rgba_unmultiplied(255, 0, 0, 128);
        let blended = Theme {
            diagnostic_background: half,
            ..theme.clone()
        };

        let invalid = painted(
            operand(Token::Number, OperandState::Invalid),
            false,
            &blended,
        );
        let expected = theme.source_number_background.blend(half);

        assert_eq!(invalid.background, Some(expected));
        assert_ne!(invalid.background, Some(theme.source_number_background));
        assert_ne!(invalid.background, Some(half));
    }

    ///
    /// An entirely blank Pending slot — an operand a Function has claimed but
    /// nothing yet fills — draws its declared Token colour rather than
    /// Diagnostic: no Cell of the slot holds content, so nothing there has
    /// failed to bind yet. The tint is unaffected either way:
    /// `syntax-highlighting/02` tints a Pending, Valid, or Invalid operand
    /// alike. This has no visible effect today — `paint.rs`'s blank-glyph
    /// fallback draws no character on a Pending Cell regardless of
    /// foreground — but it is the distinction [`OperandState`] exists to
    /// answer correctly rather than by coincidence (ADR 0044).
    ///
    #[test]
    fn a_pending_operand_keeps_its_declared_colour_rather_than_diagnostic() {
        let theme = okabe_ito();

        let pending = painted(operand(Token::Number, OperandState::Pending), false, &theme);

        assert_eq!(pending.foreground, theme.source_number);
        assert_eq!(pending.background, Some(theme.source_number_background));
    }

    ///
    /// `output_portal` overrides an Unclaimed Cell and a Valid Operand
    /// alike: each draws the Output Portal colour on the Output Portal's own
    /// Fill tint instead of whatever its own fact would answer. A written
    /// scalar or Sequence answer is not a third arm —
    /// `RenderCell::source_paint` answers the refused Function spelling it
    /// parses as with Unclaimed (`.scratch/syntax-highlighting/issues/05`'s
    /// Answer, pinned from written Source by `orcvs::render_frame`'s
    /// `a_written_07_is_two_one_cell_unbound_function_claims` and
    /// `every_cell_of_a_claim_reads_the_written_answer_its_claim_was_built_with`),
    /// so it is the Unclaimed arm below.
    ///
    #[test]
    fn output_portal_paints_over_an_unclaimed_and_a_bound_operand_cell() {
        let theme = okabe_ito();

        for paint in [
            SourcePaint::Unclaimed,
            operand(Token::Number, OperandState::Valid),
        ] {
            let visuals = painted(paint, true, &theme);
            assert_eq!(
                visuals.foreground, theme.output_portal_foreground,
                "{paint:?} did not draw the Output Portal colour"
            );
            assert_eq!(
                visuals.background,
                Some(theme.output_portal_background),
                "{paint:?} did not carry the Output Portal tint"
            );
        }
    }

    ///
    /// Invalid operand plus Output Portal: Portal precedence over Diagnostic
    /// — `.scratch/theming/schema.md`'s composition step 4, "Portal channels
    /// take precedence over other non-Function facts, including Diagnostic."
    /// The Invalid Number's Diagnostic-blended answer is still `role_and_
    /// portal`'s starting point, so Portal composites over *that*, not over
    /// the plain Number channel — with both `diagnostic.foreground` and
    /// `output_portal.foreground` opaque in Okabe–Ito, the Portal colour
    /// still wins outright, exactly as it does for a Valid operand.
    ///
    #[test]
    fn invalid_operand_under_output_portal_takes_the_portal_colour() {
        let theme = okabe_ito();

        let invalid = painted(operand(Token::Number, OperandState::Invalid), true, &theme);

        assert_eq!(invalid.foreground, theme.output_portal_foreground);
        assert_eq!(invalid.background, Some(theme.output_portal_background));
    }

    ///
    /// A Bang answer keeps its own Bang glyph colour rather than blending
    /// with the Output Portal foreground — a Bang is what a Producer emits,
    /// not a value it writes — but its background still takes the Output
    /// Portal's own channel in place of its usual transparent one, so it
    /// still reads as an Output Portal Cell.
    ///
    #[test]
    fn output_portal_keeps_the_bang_glyph_colour_but_takes_its_own_tint() {
        let theme = okabe_ito();

        let ordinary_bang = painted(SourcePaint::Bang, false, &theme);
        let portal_bang = painted(SourcePaint::Bang, true, &theme);

        assert_eq!(
            ordinary_bang.background, None,
            "Bang is untinted ordinarily"
        );
        assert_eq!(portal_bang.foreground, theme.source_bang);
        assert_eq!(portal_bang.background, Some(theme.output_portal_background));
    }

    ///
    /// A Cell that is a bound Function's own two-Cell spelling keeps its
    /// Function paint outright when `output_portal` is also true —
    /// `.scratch/syntax-highlighting/issues/06`'s precedence decision,
    /// restated by `.scratch/theming/schema.md` as "A bound Function retains
    /// all its own paint inside a Portal." This is the smallest rule that
    /// both lets a producer's Output Portal show through a consumer's
    /// operand Cells while still letting a reader find the Function that
    /// stands on a Cell; this layer cannot and need not tell a root's own
    /// spelling from a nested one to apply it, since every Function's own
    /// two-Cell spelling carries `Token::Function` regardless of nesting.
    ///
    #[test]
    fn a_bound_function_spelling_wins_over_output_portal() {
        let theme = okabe_ito();

        let ordinary = painted(SourcePaint::Function, false, &theme);
        let overlapped = painted(SourcePaint::Function, true, &theme);

        assert_eq!(
            overlapped, ordinary,
            "Output Portal changed a Function's own paint"
        );
        assert_eq!(overlapped.foreground, theme.source_function);
        assert_eq!(
            overlapped.background,
            Some(theme.source_function_background)
        );
    }

    ///
    /// Defect 3: `diagnostic.border` was never read. An Invalid Number
    /// operand's border composites `theme.diagnostic_border` over
    /// `theme.grid_border` — the same fact this Cell's foreground and
    /// background already use
    /// (`an_invalid_operand_draws_diagnostic_but_keeps_its_declared_tint`) —
    /// at `theme.diagnostic_border_width`, not `theme.grid_border_width`. An
    /// unaffected Cell (no Diagnostic fact) keeps the ordinary Grid border.
    ///
    #[test]
    fn a_diagnostic_border_with_a_non_transparent_colour_shows() {
        let visible = Color32::from_rgb(200, 30, 30);
        let theme = Theme {
            diagnostic_border: visible,
            ..okabe_ito()
        };

        let invalid = painted(operand(Token::Number, OperandState::Invalid), false, &theme);
        let valid = painted(operand(Token::Number, OperandState::Valid), false, &theme);

        assert_eq!(invalid.border, theme.grid_border.blend(visible));
        assert_ne!(invalid.border, theme.grid_border);
        assert_eq!(invalid.border_width, theme.diagnostic_border_width.points());
        assert_eq!(valid.border, theme.grid_border);
        assert_eq!(valid.border_width, theme.grid_border_width.points());
    }

    ///
    /// Portal precedence over Diagnostic — schema composition step 4,
    /// "Portal channels take precedence over other non-Function facts,
    /// including Diagnostic" — reaches the border channel too: an Invalid
    /// operand inside an Output Portal takes the Output Portal's own border
    /// colour and width outright, the same as it already does for foreground
    /// and background
    /// (`invalid_operand_under_output_portal_takes_the_portal_colour`).
    ///
    #[test]
    fn output_portal_border_takes_precedence_over_diagnostic_border() {
        let theme = Theme {
            diagnostic_border: Color32::from_rgb(200, 30, 30),
            output_portal_border: Color32::from_rgb(30, 30, 200),
            ..okabe_ito()
        };

        let invalid_in_portal =
            painted(operand(Token::Number, OperandState::Invalid), true, &theme);

        assert_eq!(invalid_in_portal.border, theme.output_portal_border);
        assert_eq!(
            invalid_in_portal.border_width,
            theme.output_portal_border_width.points()
        );
    }

    ///
    /// Okabe–Ito's transparent `diagnostic.border`/`output_portal.border`
    /// defaults leave the ordinary Grid border visible unchanged: blending a
    /// fully transparent colour on top is the identity, so this Theme's
    /// Invalid/Portal Cells still show only `theme.grid_border` — which is
    /// what keeps `okabe_ito_reproduces_the_pre_refactor_cell_visuals_
    /// exactly` passing unchanged even though every such Cell now runs
    /// through `ordinary_border`.
    ///
    #[test]
    fn transparent_diagnostic_and_portal_borders_leave_the_grid_border_visible() {
        let theme = okabe_ito();

        let invalid_in_portal =
            painted(operand(Token::Number, OperandState::Invalid), true, &theme);

        assert_eq!(invalid_in_portal.border, theme.grid_border);
        assert_eq!(
            invalid_in_portal.border_width,
            theme.grid_border_width.points()
        );
    }

    ///
    /// Width zero suppresses the stroke a Diagnostic border would otherwise
    /// draw, even with an opaque colour — `.scratch/theming/schema.md`'s
    /// "Width 0 hides the stroke," extended to whichever channel
    /// `ordinary_border` picks.
    /// `console::tests::zero_width_suppresses_only_its_own_border_stroke`
    /// proves the geometry step drops the `Shape` outright once this reaches
    /// it as `0.0`.
    ///
    #[test]
    fn zero_diagnostic_border_width_hides_the_stroke_despite_an_opaque_colour() {
        let theme = Theme {
            diagnostic_border: Color32::from_rgb(200, 30, 30),
            diagnostic_border_width: crate::theme::GridWidth::from_points(0.0)
                .expect("0.0 is within 0..=1"),
            ..okabe_ito()
        };

        let invalid = painted(operand(Token::Number, OperandState::Invalid), false, &theme);

        assert_eq!(invalid.border_width, 0.0);
    }

    ///
    /// The Cursor's own fill still wins outright over Output Portal, the
    /// same precedence it already holds over a role's own channel
    /// (`the_cursors_own_fill_wins_over_the_tint_on_its_cell`).
    ///
    #[test]
    fn the_cursors_own_fill_wins_over_output_portal_too() {
        let theme = okabe_ito();
        let cursor_colour = Color32::from_rgb(9, 8, 7);

        let cursor = cell_visuals_with_cursor_colour(
            SourcePaint::Unclaimed,
            true,
            true,
            true,
            Some(cursor_colour),
            &theme,
        );

        assert_eq!(cursor.background, Some(cursor_colour));
    }

    ///
    /// The Cursor's own fill wins outright over an Invalid operand's
    /// Diagnostic-tinted background too, the same unconditional precedence —
    /// only the background changes: the foreground is unaffected by Cursor
    /// selection at this layer (`crate::paint`'s per-Cell loop is what
    /// decides whether a Cell is drawn with a glyph at all).
    ///
    #[test]
    fn the_cursors_own_fill_wins_over_an_invalid_operands_diagnostic_background() {
        let theme = okabe_ito();
        let cursor_colour = Color32::from_rgb(9, 8, 7);

        let unselected = cell_visuals_with_cursor_colour(
            operand(Token::Number, OperandState::Invalid),
            false,
            false,
            false,
            Some(cursor_colour),
            &theme,
        );
        let cursor = cell_visuals_with_cursor_colour(
            operand(Token::Number, OperandState::Invalid),
            false,
            true,
            true,
            Some(cursor_colour),
            &theme,
        );

        assert_eq!(unselected.background, Some(theme.source_number_background));
        assert_eq!(cursor.background, Some(cursor_colour));
        assert_eq!(
            cursor.foreground, theme.diagnostic_foreground,
            "the Cursor's own fill must not change the Diagnostic glyph colour"
        );
    }

    ///
    /// A role's own background alpha composites over a nontransparent
    /// `cell.background` rather than replacing it outright —
    /// `.scratch/theming/schema.md`: "Composite uniform `cell.background`
    /// over the Grid inside every Cell" and role backgrounds "composite over
    /// the uniform `cell.background`." Function's background is opaque in
    /// Okabe–Ito, so this Theme instead gives Number a partial-alpha
    /// background to exercise a real blend.
    ///
    #[test]
    fn explicit_background_alpha_composites_over_the_cell_base() {
        let base = Color32::from_rgba_unmultiplied(10, 20, 30, 200);
        let role = Color32::from_rgba_unmultiplied(200, 100, 50, 128);
        let theme = Theme {
            cell_background: base,
            source_number_background: role,
            ..okabe_ito()
        };

        let valid = painted(operand(Token::Number, OperandState::Valid), false, &theme);

        assert_eq!(valid.background, Some(base.blend(role)));
        assert_ne!(
            valid.background,
            Some(role),
            "the Cell base was not composited under it"
        );
    }

    ///
    /// The seam `theming/02-keep-the-restored-theme-preference-at-startup.md`
    /// fixes: eframe restores `ThemePreference` into egui memory before it
    /// builds the application, and `install_style` is what `Console::new`
    /// calls afterwards. A preference already on the context — standing in
    /// for one eframe just restored — must survive the call, unlike the
    /// removed `set_theme(Dark)` call it replaces.
    ///
    #[test]
    fn install_style_leaves_a_restored_theme_preference_untouched() {
        let ctx = egui::Context::default();
        ctx.set_theme(egui::ThemePreference::Light);

        install_style(&ctx);

        assert_eq!(
            ctx.options(|options| options.theme_preference),
            egui::ThemePreference::Light,
            "install_style overwrote the restored theme preference"
        );
    }

    ///
    /// One palette exists, so a preference that resolves to Light must not
    /// leave the Light slot at egui's own default style while the Dark slot
    /// — and the Source Grid, painted from `PALETTE` — carry the console's
    /// own. Both theme slots take the same style `install_style` installs.
    ///
    /// The comparison is `Visuals` and `animation_time`, the two fields
    /// [`style`] actually sets, rather than `Style`'s own `PartialEq`:
    /// `Style::number_formatter` compares by `Arc::ptr_eq`
    /// (`egui-0.36.2/src/style.rs:57-60`), so two independently built
    /// `Style::default()`s — one inside each `style()` call — never compare
    /// equal on that field alone, whatever their visible content. The two
    /// theme slots are additionally asserted to share one `Arc`, which
    /// sidesteps that field entirely by construction.
    ///
    #[test]
    fn install_style_registers_the_console_style_for_both_themes() {
        let ctx = egui::Context::default();

        install_style(&ctx);

        let dark = ctx.style_of(egui::Theme::Dark);
        let light = ctx.style_of(egui::Theme::Light);
        assert!(
            Arc::ptr_eq(&dark, &light),
            "Dark and Light were not registered from the one console style"
        );

        let expected = style();
        assert_eq!(
            light.visuals, expected.visuals,
            "the installed style's Visuals do not match style()"
        );
        assert_eq!(
            light.animation_time, expected.animation_time,
            "the installed style's animation_time does not match style()"
        );
    }

    ///
    /// Pins `cell_visuals_with_cursor_colour`'s answer for a representative
    /// fact set to the exact premultiplied byte arrays captured from
    /// `ba987f6` — the commit immediately before this refactor, running the
    /// *pre-refactor* `cell_visuals_with_cursor_colour(paint, output_portal,
    /// selected, cursor_visible, cursor_colour, SourcePaintSettings)` over
    /// the same cases with `SourcePaintSettings::default()` and a
    /// `Some(Color32::from_rgb(9, 8, 7))` Cursor fill. This is the TDD
    /// baseline `.scratch/theming/issues/06` asks for: proof the refactor
    /// reproduces today's output rather than an argument that the algebra
    /// should. Captured with a throwaway `git worktree add --detach
    /// ba987f6` and a temporary `eprintln!`-driven test run with
    /// `--nocapture`, removed afterwards; every literal below is copied
    /// verbatim from that run's output, not recomputed from this module or
    /// from `theme.rs`.
    ///
    /// Every role, every Operand binding state Diagnostic distinguishes,
    /// Output Portal over Unclaimed/a Valid operand/Bang (keeps its own
    /// glyph)/a bound Function (bypassed outright), and the Cursor's own
    /// fill winning over a role's tint, an Invalid operand's Diagnostic
    /// glyph, and the rest/visible border pair.
    ///
    /// One captured `ba987f6` case: the fact `cell_visuals_with_cursor_colour`
    /// was asked about, and the exact premultiplied byte arrays it answered.
    /// A named struct rather than an eight-element tuple, so a field is
    /// named at every use instead of counted by position.
    struct BaselineCase {
        name: &'static str,
        fact: SourcePaint,
        output_portal: bool,
        selected: bool,
        cursor_visible: bool,
        background: Option<[u8; 4]>,
        border: [u8; 4],
        foreground: [u8; 4],
    }

    #[test]
    fn okabe_ito_reproduces_the_pre_refactor_cell_visuals_exactly() {
        let theme = okabe_ito();
        let cursor_colour = Some(Color32::from_rgb(9, 8, 7));

        // Every byte array copied verbatim from the `ba987f6` capture
        // described above.
        let cases = vec![
            BaselineCase {
                name: "unclaimed",
                fact: SourcePaint::Unclaimed,
                output_portal: false,
                selected: false,
                cursor_visible: false,
                background: None,
                border: [8, 16, 14, 72],
                foreground: [234, 235, 229, 255],
            },
            BaselineCase {
                name: "comment",
                fact: SourcePaint::Comment,
                output_portal: false,
                selected: false,
                cursor_visible: false,
                background: None,
                border: [8, 16, 14, 72],
                foreground: [153, 153, 153, 255],
            },
            BaselineCase {
                name: "bang",
                fact: SourcePaint::Bang,
                output_portal: false,
                selected: false,
                cursor_visible: false,
                background: None,
                border: [8, 16, 14, 72],
                foreground: [204, 121, 167, 255],
            },
            BaselineCase {
                name: "bang_portal",
                fact: SourcePaint::Bang,
                output_portal: true,
                selected: false,
                cursor_visible: false,
                background: Some([37, 25, 0, 255]),
                border: [8, 16, 14, 72],
                foreground: [204, 121, 167, 255],
            },
            BaselineCase {
                name: "function",
                fact: SourcePaint::Function,
                output_portal: false,
                selected: false,
                cursor_visible: false,
                background: Some([0, 25, 18, 255]),
                border: [8, 16, 14, 72],
                foreground: [0, 158, 115, 255],
            },
            BaselineCase {
                name: "function_portal",
                fact: SourcePaint::Function,
                output_portal: true,
                selected: false,
                cursor_visible: false,
                background: Some([0, 25, 18, 255]),
                border: [8, 16, 14, 72],
                foreground: [0, 158, 115, 255],
            },
            BaselineCase {
                name: "number_pending",
                fact: operand(Token::Number, OperandState::Pending),
                output_portal: false,
                selected: false,
                cursor_visible: false,
                background: Some([14, 29, 37, 255]),
                border: [8, 16, 14, 72],
                foreground: [86, 180, 233, 255],
            },
            BaselineCase {
                name: "number_valid",
                fact: operand(Token::Number, OperandState::Valid),
                output_portal: false,
                selected: false,
                cursor_visible: false,
                background: Some([14, 29, 37, 255]),
                border: [8, 16, 14, 72],
                foreground: [86, 180, 233, 255],
            },
            BaselineCase {
                name: "number_invalid",
                fact: operand(Token::Number, OperandState::Invalid),
                output_portal: false,
                selected: false,
                cursor_visible: false,
                background: Some([14, 29, 37, 255]),
                border: [8, 16, 14, 72],
                foreground: [213, 94, 0, 255],
            },
            BaselineCase {
                name: "number_valid_portal",
                fact: operand(Token::Number, OperandState::Valid),
                output_portal: true,
                selected: false,
                cursor_visible: false,
                background: Some([37, 25, 0, 255]),
                border: [8, 16, 14, 72],
                foreground: [230, 159, 0, 255],
            },
            BaselineCase {
                name: "note_valid",
                fact: operand(Token::Note, OperandState::Valid),
                output_portal: false,
                selected: false,
                cursor_visible: false,
                background: Some([38, 36, 11, 255]),
                border: [8, 16, 14, 72],
                foreground: [240, 228, 66, 255],
            },
            BaselineCase {
                name: "atom_pending",
                fact: operand(Token::Atom, OperandState::Pending),
                output_portal: false,
                selected: false,
                cursor_visible: false,
                background: Some([37, 38, 37, 255]),
                border: [8, 16, 14, 72],
                foreground: [234, 235, 229, 255],
            },
            BaselineCase {
                name: "sequence_pending",
                fact: operand(Token::Sequence, OperandState::Pending),
                output_portal: false,
                selected: false,
                cursor_visible: false,
                background: Some([0, 18, 28, 255]),
                border: [8, 16, 14, 72],
                foreground: [0, 114, 178, 255],
            },
            BaselineCase {
                name: "sequence_invalid",
                fact: operand(Token::Sequence, OperandState::Invalid),
                output_portal: false,
                selected: false,
                cursor_visible: false,
                background: Some([0, 18, 28, 255]),
                border: [8, 16, 14, 72],
                foreground: [213, 94, 0, 255],
            },
            BaselineCase {
                name: "unclaimed_portal",
                fact: SourcePaint::Unclaimed,
                output_portal: true,
                selected: false,
                cursor_visible: false,
                background: Some([37, 25, 0, 255]),
                border: [8, 16, 14, 72],
                foreground: [230, 159, 0, 255],
            },
            BaselineCase {
                name: "unclaimed_selected",
                fact: SourcePaint::Unclaimed,
                output_portal: false,
                selected: true,
                cursor_visible: false,
                background: Some([9, 8, 7, 255]),
                border: [82, 195, 163, 255],
                foreground: [234, 235, 229, 255],
            },
            BaselineCase {
                name: "unclaimed_cursor_visible",
                fact: SourcePaint::Unclaimed,
                output_portal: false,
                selected: true,
                cursor_visible: true,
                background: Some([9, 8, 7, 255]),
                border: [101, 230, 190, 255],
                foreground: [234, 235, 229, 255],
            },
            BaselineCase {
                name: "function_selected",
                fact: SourcePaint::Function,
                output_portal: false,
                selected: true,
                cursor_visible: false,
                background: Some([9, 8, 7, 255]),
                border: [82, 195, 163, 255],
                foreground: [0, 158, 115, 255],
            },
            BaselineCase {
                name: "function_cursor_visible",
                fact: SourcePaint::Function,
                output_portal: false,
                selected: true,
                cursor_visible: true,
                background: Some([9, 8, 7, 255]),
                border: [101, 230, 190, 255],
                foreground: [0, 158, 115, 255],
            },
            BaselineCase {
                name: "number_invalid_selected",
                fact: operand(Token::Number, OperandState::Invalid),
                output_portal: false,
                selected: true,
                cursor_visible: true,
                background: Some([9, 8, 7, 255]),
                border: [101, 230, 190, 255],
                foreground: [213, 94, 0, 255],
            },
        ];

        for case in cases {
            let visuals = cell_visuals_with_cursor_colour(
                case.fact,
                case.output_portal,
                case.selected,
                case.cursor_visible,
                cursor_colour,
                &theme,
            );
            assert_eq!(
                visuals.background.map(|c| c.to_array()),
                case.background,
                "{}: background",
                case.name
            );
            assert_eq!(
                visuals.border.to_array(),
                case.border,
                "{}: border",
                case.name
            );
            assert_eq!(
                visuals.foreground.to_array(),
                case.foreground,
                "{}: foreground",
                case.name
            );
        }
    }
}
