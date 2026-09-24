use std::sync::Arc;

use eframe::egui;

use egui::{Color32, CornerRadius, Shadow, Stroke, Style, Visuals, style::Selection};

use orcvs::source::{OperandState, SourcePaint, Token};

use crate::theme::{Appearance, Theme};

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
/// Every Source Paint fact a Cell can carry, in [`SourcePaintVisuals`]'s
/// index order: the four whole-Cell facts, then each declared operand Token
/// in each [`OperandState`].
///
const SOURCE_PAINT_FACTS: [SourcePaint; 16] = {
    use OperandState::{Invalid, Pending, Valid};
    use SourcePaint::{Bang, Comment, Function, Unclaimed};
    use Token::{Atom, Note, Number, Sequence};
    const fn operand(token: Token, state: OperandState) -> SourcePaint {
        SourcePaint::Operand { token, state }
    }
    [
        Unclaimed,
        Function,
        Bang,
        Comment,
        operand(Number, Pending),
        operand(Number, Valid),
        operand(Number, Invalid),
        operand(Note, Pending),
        operand(Note, Valid),
        operand(Note, Invalid),
        operand(Atom, Pending),
        operand(Atom, Valid),
        operand(Atom, Invalid),
        operand(Sequence, Pending),
        operand(Sequence, Valid),
        operand(Sequence, Invalid),
    ]
};

///
/// [`cell_visuals_with_cursor_colour`]'s answer for every Cell that is not
/// the single-Cell Cursor, resolved from `theme` once per Paint rather than
/// once per Cell — `.scratch/theming/issues/06`'s "resolve the Theme once per
/// frame into a flat lookup" constraint.
///
/// An unselected Cell's visuals depend only on its Source Paint fact and its
/// Output Portal flag (`selected`, `cursor_visible` and the Cursor fill are
/// all inert when `selected` is false), so the 16 facts × 2 flags are every
/// answer the walk can need. Each entry is built by calling
/// [`cell_visuals_with_cursor_colour`] itself, so the table cannot drift from
/// the per-Cell definition the tests pin; the per-Cell walk is left with one
/// indexed load instead of the role match, the Diagnostic/Output Portal
/// blends, the `cell.background` composite and the border priority match.
///
/// Entries are filled on first ask rather than up front. Resolving all 32
/// costs about 570 ns, which a full Grid amortises to nothing but a Paint of
/// a scrolled-away or barely visible Grid — the console derives one per frame
/// at whatever the viewport culls to — would pay in full for the handful of
/// facts it reads; `paint_derive/empty` priced that at 16 ns eager against
/// 587. A Cell that misses pays one branch and the resolution it would have
/// paid anyway, and the facts a Source actually carries are few, so the walk
/// resolves each of them once whatever its size.
///
pub(crate) struct SourcePaintVisuals<'a> {
    theme: &'a Theme,
    entries: [Option<CellVisuals>; SOURCE_PAINT_FACTS.len() * 2],
}

impl<'a> SourcePaintVisuals<'a> {
    pub(crate) fn new(theme: &'a Theme) -> Self {
        Self {
            theme,
            entries: [None; SOURCE_PAINT_FACTS.len() * 2],
        }
    }

    ///
    /// The visuals of an unselected Cell carrying `paint`, inside a root
    /// Function's Output Portal Reservation when `output_portal` is true.
    ///
    #[inline]
    pub(crate) fn unselected(&mut self, paint: SourcePaint, output_portal: bool) -> CellVisuals {
        let index = fact_index(paint) * 2 + usize::from(output_portal);
        match self.entries[index] {
            Some(visuals) => visuals,
            None => {
                let visuals = cell_visuals_with_cursor_colour(
                    paint,
                    output_portal,
                    false,
                    false,
                    None,
                    self.theme,
                );
                self.entries[index] = Some(visuals);
                visuals
            }
        }
    }
}

///
/// `paint`'s position in [`SOURCE_PAINT_FACTS`].
///
#[inline]
fn fact_index(paint: SourcePaint) -> usize {
    match paint {
        SourcePaint::Unclaimed => 0,
        SourcePaint::Function => 1,
        SourcePaint::Bang => 2,
        SourcePaint::Comment => 3,
        SourcePaint::Operand { token, state } => {
            let token = match token {
                Token::Number => 0,
                Token::Note => 1,
                Token::Atom => 2,
                Token::Sequence => 3,
                Token::Bang | Token::Comment | Token::Function | Token::Char => {
                    unreachable!("SourcePaint::Operand carries only a declared operand Token")
                }
            };
            let state = match state {
                OperandState::Pending => 0,
                OperandState::Valid => 1,
                OperandState::Invalid => 2,
            };
            4 + token * 3 + state
        }
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
/// One Cell's final background: `role_background` (a Cell's own role/
/// Diagnostic/Output-Portal answer, from [`cell_visuals_with_cursor_colour`])
/// combined with the Region/Cursor fallback chain — extracted from
/// [`crate::paint::Paint::derive_with_theme`]'s per-Cell loop so that
/// function and [`crate::contrast::painted`]'s standalone per-state answer
/// read the exact same decision and cannot independently drift.
///
/// `is_region_cursor` is `crate::paint::Paint::derive_with_theme`'s own
/// `is_cursor && region_spans`: the Cursor's own Cell inside a Region larger
/// than one Cell, which takes `region_cursor_fill` outright regardless of
/// `role_background` — the same "Cursor's own fill wins outright" rule
/// [`cell_visuals_with_cursor_colour`] applies to the single-Cell Cursor.
/// Every other Cell keeps `role_background` when it answered one, and falls
/// back to `region_fill` inside a spanning Region (`in_region`) or
/// `base_fill` outside one.
///
pub(crate) fn cell_background(
    role_background: Option<Color32>,
    is_region_cursor: bool,
    in_region: bool,
    region_cursor_fill: Option<Color32>,
    region_fill: Option<Color32>,
    base_fill: Option<Color32>,
    cell_background: Color32,
) -> Option<Color32> {
    if is_region_cursor {
        compose_cell_fill(cell_background, region_cursor_fill)
    } else {
        role_background.or(if in_region { region_fill } else { base_fill })
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
/// The chrome `Style` for one resolved Theme.
///
/// Every `Visuals` field the console sets — explicitly assigned fields and
/// the toolkit's own inherited defaults alike — reads one of the named
/// chrome keys `.scratch/theming/schema.md`'s Chrome mapping table lists:
/// `panel.background`, `panel.border` (+ `.width`), `text`, `text.active`,
/// `text.muted`, `input.background`, `selection.background`,
/// `selection.border` (+ `.width`), `selection.border.rest`,
/// `widget.inactive.border` (+ `.width`), `widget.border.width`, `link`,
/// `code.background`, `input.cursor` (+ `.width`), `error` and `warning` —
/// so a custom Theme's chrome follows its own colours and widths the same
/// way the Source Grid already does (`.scratch/theming/issues/06`). After
/// this, nothing the console draws is a compiled constant, and no toolkit
/// default is a second, hidden fixed palette beside it.
///
/// `visuals.weak_text_color` is set explicitly from `theme.text_muted`
/// rather than left `None`: egui's own default derives a weak text colour
/// from `text_color() * weak_text_alpha` (`egui-0.36.2/src/style.rs`'s
/// `Visuals::weak_text_color`, `weak_text_alpha` defaulting to `0.6`), and
/// the Okabe–Ito built-in's own `text_muted` value is `text`'s colour at
/// that same `0.6` gamma multiply, so Okabe–Ito's chrome is unchanged while
/// a custom Theme's `text.muted` is honoured instead of silently recomputed
/// from `text` whatever the Theme actually declares.
///
/// `visuals.hyperlink_color`/`code_bg_color` read `link`/`code.background`
/// directly: egui's own dark defaults for both already equal Okabe–Ito's
/// values, which is exactly the "second fixed palette hidden in `Visuals`
/// defaults" the schema's audit line exists to catch — a custom Theme's
/// `link`/`code.background` would otherwise never reach these two fields.
///
/// `visuals.text_cursor.stroke` and `visuals.ime_composition`'s active
/// underline both read `input.cursor`/`.width` — "Input caret and active
/// IME underline" share one property pair. The inactive underline keeps
/// egui's own half-linear attenuation (`Color32::linear_multiply(0.5)`,
/// the same pattern `ImeComposition::dark`/`::light` already use) over that
/// same colour rather than a second hardcoded one, and `legacy_visuals`
/// stays whatever platform default `Visuals::dark`/`::light` chose — it is
/// not a themed property.
///
/// `visuals.widgets.hovered`/`.active.fg_stroke` read `text.active`, not
/// `selection.border`: Okabe–Ito's own values for the two happen to
/// coincide, which is why reading the wrong key would still reproduce
/// today's appearance — `hovered_and_active_foreground_reads_text_active_
/// not_selection_border` retunes them apart to prove the wiring.
///
/// `Visuals::dark_mode`, and every other field `Visuals::dark`/`Visuals::light`
/// set that this function does not override — the text colour-transfer
/// function among them — follows `theme.appearance` by starting from the
/// matching constructor rather than by flipping the flag alone:
/// `docs/research/egui-theming.md`'s own reading of the upstream docs,
/// "Merely changing `Visuals::dark_mode` does not convert a palette."
///
/// Typography, spacing, corner radii, shadows and gradients stay at
/// `Visuals::dark`/`::light`'s own literal values — square corners
/// (`CornerRadius::ZERO`), no shadows (`Shadow::NONE` on windows and
/// popups) and no animation (`animation_time: 0.0`) among them — per
/// `.scratch/theming/schema.md`: "These are not additional independently
/// configurable controls." None of them reads `theme`.
///
/// Every chrome border and the input caret read their own bounded
/// `ChromeWidth` (`0` to `2` display points inclusive, `0` hiding the
/// stroke — `.scratch/theming/schema.md`'s width catalogue) rather than a
/// literal `1.0`: `panel.border.width` for the panel/window/noninteractive
/// border, `widget.border.width` for the hovered/active/open widget border,
/// `widget.inactive.border.width` for the idle widget border (replacing the
/// literal `Stroke::NONE` it used to be — the same absence today, since the
/// default width is `0`, but now a Theme's own value rather than a
/// hardcoded one), `selection.border.width` for text selection, and
/// `input.cursor.width` for the caret and the active IME underline.
///
pub fn style(theme: &Theme) -> Style {
    // Pairs a bounded `ChromeWidth` with its colour — every chrome border,
    // the caret and the IME underlines are one of these pairs, so this
    // replaces a repeated inline `Stroke::new(theme.x_width.points(), theme.x)`
    // at each of the seven call sites below with the pairing itself.
    fn bounded_stroke(width: crate::theme::ChromeWidth, colour: Color32) -> Stroke {
        Stroke::new(width.points(), colour)
    }

    let mut visuals = match theme.appearance {
        Appearance::Dark => Visuals::dark(),
        Appearance::Light => Visuals::light(),
    };
    visuals.panel_fill = theme.panel_background;
    visuals.window_fill = theme.panel_background;
    visuals.extreme_bg_color = theme.input_background;
    visuals.faint_bg_color = theme.input_background;
    visuals.error_fg_color = theme.error;
    visuals.warn_fg_color = theme.warning;
    visuals.weak_text_color = Some(theme.text_muted);
    visuals.hyperlink_color = theme.link;
    visuals.code_bg_color = theme.code_background;
    let caret = bounded_stroke(theme.input_cursor_width, theme.input_cursor);
    visuals.text_cursor.stroke = caret;
    visuals.ime_composition.active_underline_stroke = caret;
    visuals.ime_composition.inactive_underline_stroke = Stroke {
        width: caret.width,
        color: caret.color.linear_multiply(0.5),
    };
    visuals.selection = Selection {
        bg_fill: theme.selection_background,
        stroke: bounded_stroke(theme.selection_border_width, theme.selection_border),
    };
    visuals.window_corner_radius = CornerRadius::ZERO;
    visuals.menu_corner_radius = CornerRadius::ZERO;
    visuals.window_shadow = Shadow::NONE;
    visuals.popup_shadow = Shadow::NONE;
    // `Frame::window` and `Panel`'s separator both read these — `panel.border`/
    // `panel.border.width`, the "Panel/window/noninteractive border" row.
    let chrome = bounded_stroke(theme.panel_border_width, theme.panel_border);
    visuals.window_stroke = chrome;
    visuals.widgets.noninteractive.bg_fill = theme.panel_background;
    visuals.widgets.noninteractive.weak_bg_fill = theme.panel_background;
    visuals.widgets.noninteractive.bg_stroke = chrome;
    visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, theme.text);
    visuals.widgets.inactive.bg_fill = theme.panel_background;
    visuals.widgets.inactive.weak_bg_fill = theme.panel_background;
    // "Inactive widget border" — `widget.inactive.border`, `.width`: a real
    // colour at width 0 by default, not `Stroke::NONE`, so a custom Theme
    // that raises the width shows this colour instead of staying absent.
    visuals.widgets.inactive.bg_stroke = bounded_stroke(
        theme.widget_inactive_border_width,
        theme.widget_inactive_border,
    );
    visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, theme.text);
    // "Hovered/active strong and weak fill" — `selection.background`.
    visuals.widgets.hovered.bg_fill = theme.selection_background;
    visuals.widgets.hovered.weak_bg_fill = theme.selection_background;
    // "Hovered/open widget border" — `selection.border.rest`, `widget.border.width`.
    visuals.widgets.hovered.bg_stroke =
        bounded_stroke(theme.widget_border_width, theme.selection_border_rest);
    // "Hovered/active widget foreground" — `text.active`, not `selection.border`:
    // Okabe–Ito's own values for the two coincide, which is why reading the
    // wrong key would still look right until a custom Theme set them apart.
    visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, theme.text_active);
    visuals.widgets.active.bg_fill = theme.selection_background;
    visuals.widgets.active.weak_bg_fill = theme.selection_background;
    // "Active widget border" — `selection.border`, `widget.border.width`.
    visuals.widgets.active.bg_stroke =
        bounded_stroke(theme.widget_border_width, theme.selection_border);
    visuals.widgets.active.fg_stroke = Stroke::new(1.0, theme.text_active);
    // "Open widget weak fill" — `panel.background`; the strong fill and the
    // border share the hovered/open row above.
    visuals.widgets.open.bg_fill = theme.selection_background;
    visuals.widgets.open.weak_bg_fill = theme.panel_background;
    visuals.widgets.open.bg_stroke =
        bounded_stroke(theme.widget_border_width, theme.selection_border_rest);
    visuals.widgets.open.fg_stroke = Stroke::new(1.0, theme.text);
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
/// Registers [`style`] of `dark` for [`egui::Theme::Dark`] and of `light`
/// for [`egui::Theme::Light`] through [`egui::Context::set_style_of`], and
/// touches nothing else on `ctx` — in particular it never calls
/// [`egui::Context::set_theme`].
///
/// `ThemePreference` is part of the `Options` eframe restores into egui
/// memory before it constructs the application, but eframe restores memory
/// without reinstalling a style. `Console::new` calls this every launch so
/// both slots hold the console's style before the first frame, whatever the
/// preference resolves to; a `set_theme` call here would overwrite the very
/// value just restored (`.scratch/theming/issues/02`). The console calls it
/// again when a viewer picks a different Theme for either appearance.
///
/// egui then chooses the slot each frame from the preference and the
/// operating system's appearance, and the console paints the Source from
/// the Theme of the same appearance (`Console::ui`), so chrome and Source
/// never come from different Themes.
///
pub(crate) fn install(ctx: &egui::Context, dark: &Theme, light: &Theme) {
    ctx.set_style_of(egui::Theme::Dark, Arc::new(style(dark)));
    ctx.set_style_of(egui::Theme::Light, Arc::new(style(light)));
}

#[cfg(test)]
mod tests {

    use super::{
        CellVisuals, SOURCE_PAINT_FACTS, SourcePaintVisuals, cell_visuals_with_cursor_colour,
        fact_index, install, sector_line, style,
    };
    use crate::theme::{Theme, okabe_ito, orcvs_light};
    use egui::{Color32, CornerRadius, Shadow, Stroke, Visuals};
    use orcvs::source::{OperandState, SourcePaint, Token};

    ///
    /// `fact_index` and `SOURCE_PAINT_FACTS` name the same order, so no two
    /// facts share a `SourcePaintVisuals` entry and none reads another's.
    ///
    #[test]
    fn every_source_paint_fact_indexes_its_own_table_entry() {
        for (index, fact) in SOURCE_PAINT_FACTS.into_iter().enumerate() {
            assert_eq!(fact_index(fact), index, "{fact:?}");
        }
    }

    ///
    /// The once-per-Paint table answers exactly what the per-Cell definition
    /// answers for every unselected fact and Output Portal flag, under the
    /// built-in and under a Theme whose Cell base and fact channels are all
    /// partly transparent — the case where every composite in
    /// `source_paint_visuals`, `role_and_portal` and `ordinary_border` shows.
    ///
    #[test]
    fn the_per_paint_table_matches_the_per_cell_visuals_for_every_fact() {
        let translucent = Theme {
            cell_background: Color32::from_rgba_unmultiplied(10, 20, 30, 128),
            diagnostic_foreground: Color32::from_rgba_unmultiplied(200, 40, 0, 100),
            diagnostic_background: Color32::from_rgba_unmultiplied(90, 0, 0, 60),
            diagnostic_border: Color32::from_rgba_unmultiplied(255, 0, 0, 70),
            output_portal_foreground: Color32::from_rgba_unmultiplied(230, 160, 0, 90),
            output_portal_background: Color32::from_rgba_unmultiplied(40, 30, 0, 80),
            output_portal_border: Color32::from_rgba_unmultiplied(230, 160, 0, 50),
            ..okabe_ito()
        };

        for theme in [okabe_ito(), translucent] {
            let mut table = SourcePaintVisuals::new(&theme);
            for fact in SOURCE_PAINT_FACTS {
                for output_portal in [false, true] {
                    assert_eq!(
                        table.unselected(fact, output_portal),
                        cell_visuals_with_cursor_colour(
                            fact,
                            output_portal,
                            false,
                            false,
                            theme.cursor_background,
                            &theme,
                        ),
                        "{fact:?}, output_portal: {output_portal}"
                    );
                }
            }
        }
    }

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
            Some(theme.selection_background),
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
    /// `theme.md` is the decided record for the chrome. These literals are
    /// that record in `Color32` form, read through [`style`] rather than a
    /// fixed palette constant: a later change to the chrome fails here, and
    /// the same commit must change the document. The Source background and
    /// every Token's glyph colour are pinned the same way in `theme::tests`,
    /// against [`okabe_ito`] directly.
    ///
    /// Every field `style` sets is asserted here, field by field rather than
    /// one `Visuals` equality, so a regression names the one field that
    /// moved instead of a giant struct diff — the practical equivalent of
    /// comparing the whole produced `Visuals` against the pre-`03` baseline,
    /// since every value below is what that baseline already painted
    /// (`weak_text_color`, `hyperlink_color`, `code_bg_color`, `text_cursor`
    /// and the IME underlines were egui's own inherited defaults then, not
    /// yet Theme-read, but numerically identical — that is the audit's own
    /// point). **One exception, noted rather than hidden**:
    /// `widgets.inactive.bg_stroke` was the literal `Stroke::NONE` before
    /// this issue and is now `Stroke::new(0.0, theme.widget_inactive_border)`
    /// — a real `Color32`, not `TRANSPARENT`, at zero width. Both paint
    /// nothing (`chrome_border_widths_come_from_the_theme` pins the zero
    /// width; the geometry step, not this one, drops a zero-width stroke),
    /// so the *visible* chrome is unchanged, but the value itself is not
    /// byte-identical to the old constant, which is why this is called out
    /// explicitly instead of asserted as `Stroke::NONE`.
    ///
    #[test]
    fn okabe_ito_chrome_matches_the_decided_record() {
        let theme = okabe_ito();
        let visuals = style(&theme).visuals;
        let page = Color32::from_rgb(11, 17, 18); // #0B1112
        let panel_border = Color32::from_rgba_unmultiplied_const(29, 55, 49, 72).to_opaque();
        let ordinary = Color32::from_rgb(234, 235, 229); // #EAEBE5
        let selection_fill = Color32::from_rgb(10, 42, 34); // #0A2A22
        let selection_stroke_rest = Color32::from_rgb(82, 195, 163); // #52C3A3
        let selection_stroke = Color32::from_rgb(101, 230, 190); // #65E6BE
        let bang = Color32::from_rgb(204, 121, 167); // #CC79A7
        let hyperlink = Color32::from_rgb(90, 170, 255); // #5AAAFF
        let code_background = Color32::from_rgb(64, 64, 64); // #404040
        let input_cursor = Color32::from_rgb(192, 222, 255); // #C0DEFF
        let widget_inactive_border = Color32::from_rgb(28, 57, 50); // #1C3932

        assert!(visuals.dark_mode, "Okabe–Ito is a dark Theme");

        // panel.background: page fill, everywhere it applies.
        assert_eq!(visuals.panel_fill, page);
        assert_eq!(visuals.window_fill, page);
        assert_eq!(visuals.widgets.noninteractive.bg_fill, page);
        assert_eq!(visuals.widgets.noninteractive.weak_bg_fill, page);
        assert_eq!(visuals.widgets.inactive.bg_fill, page);
        assert_eq!(visuals.widgets.inactive.weak_bg_fill, page);
        assert_eq!(visuals.widgets.open.weak_bg_fill, page);

        // panel.border/.width: the chrome stroke, at its 1-point default.
        assert_eq!(visuals.window_stroke, Stroke::new(1.0, panel_border));
        assert_eq!(
            visuals.widgets.noninteractive.bg_stroke,
            Stroke::new(1.0, panel_border)
        );

        // widget.inactive.border/.width: a real colour at width 0 — see this
        // test's own doc for why that is not `Stroke::NONE` and still paints
        // nothing.
        assert_eq!(visuals.widgets.inactive.bg_stroke.width, 0.0);
        assert_eq!(
            visuals.widgets.inactive.bg_stroke.color,
            widget_inactive_border
        );

        // text: noninteractive/inactive/open foreground.
        assert_eq!(visuals.widgets.noninteractive.fg_stroke.color, ordinary);
        assert_eq!(visuals.widgets.inactive.fg_stroke.color, ordinary);
        assert_eq!(visuals.widgets.open.fg_stroke.color, ordinary);

        // text.active: hovered/active foreground — Okabe–Ito's own value
        // coincides with selection.border, which is exactly why this is
        // asserted against the literal rather than `theme.selection_border`.
        assert_eq!(visuals.widgets.hovered.fg_stroke.color, selection_stroke);
        assert_eq!(visuals.widgets.active.fg_stroke.color, selection_stroke);

        // text.muted: egui's own weak-text formula (`text_color() *
        // weak_text_alpha`, `weak_text_alpha` defaulting to 0.6) restated as
        // an explicit value — the exact check the deleted `theme::tests::
        // okabe_ito_matches_todays_style_and_palette_constants` carried.
        assert_eq!(
            visuals.weak_text_color,
            Some(ordinary.gamma_multiply(0.6)),
            "weak_text_color must equal text at egui's own 0.6 gamma multiply"
        );

        // input.background: extreme/faint backgrounds.
        assert_eq!(visuals.extreme_bg_color, Color32::BLACK);
        assert_eq!(visuals.faint_bg_color, Color32::BLACK);

        // selection.background/.border/.border.width: the selection highlight.
        assert_eq!(visuals.selection.bg_fill, selection_fill);
        assert_eq!(visuals.selection.stroke, Stroke::new(1.0, selection_stroke));

        // selection.background: hovered/active/open strong fill.
        assert_eq!(visuals.widgets.hovered.bg_fill, selection_fill);
        assert_eq!(visuals.widgets.hovered.weak_bg_fill, selection_fill);
        assert_eq!(visuals.widgets.active.bg_fill, selection_fill);
        assert_eq!(visuals.widgets.active.weak_bg_fill, selection_fill);
        assert_eq!(visuals.widgets.open.bg_fill, selection_fill);

        // selection.border/selection.border.rest + widget.border.width:
        // hovered/active/open border.
        assert_eq!(
            visuals.widgets.hovered.bg_stroke,
            Stroke::new(1.0, selection_stroke_rest)
        );
        assert_eq!(
            visuals.widgets.active.bg_stroke,
            Stroke::new(1.0, selection_stroke)
        );
        assert_eq!(
            visuals.widgets.open.bg_stroke,
            Stroke::new(1.0, selection_stroke_rest)
        );

        // error, warning.
        assert_eq!(visuals.error_fg_color, bang);
        assert_eq!(visuals.warn_fg_color, bang);

        // link, code.background.
        assert_eq!(visuals.hyperlink_color, hyperlink);
        assert_eq!(visuals.code_bg_color, code_background);

        // input.cursor/.width: the text-edit caret and the active IME
        // underline share one property pair.
        assert_eq!(visuals.text_cursor.stroke, Stroke::new(2.0, input_cursor));
        assert_eq!(
            visuals.ime_composition.active_underline_stroke,
            Stroke::new(2.0, input_cursor)
        );
        // The inactive IME underline: egui's own existing half-linear
        // attenuation of that same colour, not a second hardcoded one.
        assert_eq!(
            visuals.ime_composition.inactive_underline_stroke,
            Stroke {
                width: 2.0,
                color: input_cursor.linear_multiply(0.5),
            },
            "the inactive IME underline must be input.cursor's own half-linear attenuation"
        );
        assert_eq!(
            visuals.ime_composition.legacy_visuals,
            Visuals::dark().ime_composition.legacy_visuals,
            "legacy_visuals is a platform default, not a themed property"
        );
    }

    ///
    /// Retuning each of the ten named chrome keys `style` reads changes the
    /// matching `Visuals` field on the next call — proof it reads `theme`
    /// live rather than merely reproducing Okabe–Ito's own values by
    /// construction, the chrome counterpart of
    /// `glyph_colours_are_distinct_and_read_from_the_theme`'s retuned check.
    ///
    #[test]
    fn each_named_chrome_key_changes_style_when_retuned() {
        let base = okabe_ito();
        let retuned = Color32::from_rgb(9, 9, 9);

        let panel_background = Theme {
            panel_background: retuned,
            ..base.clone()
        };
        assert_eq!(style(&panel_background).visuals.panel_fill, retuned);
        assert_eq!(style(&panel_background).visuals.window_fill, retuned);

        let panel_border = Theme {
            panel_border: retuned,
            ..base.clone()
        };
        assert_eq!(style(&panel_border).visuals.window_stroke.color, retuned);

        let text = Theme {
            text: retuned,
            ..base.clone()
        };
        assert_eq!(
            style(&text).visuals.widgets.noninteractive.fg_stroke.color,
            retuned
        );

        let text_muted = Theme {
            text_muted: retuned,
            ..base.clone()
        };
        assert_eq!(style(&text_muted).visuals.weak_text_color, Some(retuned));

        let input_background = Theme {
            input_background: retuned,
            ..base.clone()
        };
        assert_eq!(style(&input_background).visuals.extreme_bg_color, retuned);
        assert_eq!(style(&input_background).visuals.faint_bg_color, retuned);

        let selection_background = Theme {
            selection_background: retuned,
            ..base.clone()
        };
        assert_eq!(
            style(&selection_background).visuals.selection.bg_fill,
            retuned
        );

        let selection_border = Theme {
            selection_border: retuned,
            ..base.clone()
        };
        assert_eq!(
            style(&selection_border).visuals.selection.stroke.color,
            retuned
        );

        let selection_border_rest = Theme {
            selection_border_rest: retuned,
            ..base.clone()
        };
        assert_eq!(
            style(&selection_border_rest)
                .visuals
                .widgets
                .hovered
                .bg_stroke
                .color,
            retuned
        );

        let error = Theme {
            error: retuned,
            ..base.clone()
        };
        assert_eq!(style(&error).visuals.error_fg_color, retuned);

        let warning = Theme {
            warning: retuned,
            ..base
        };
        assert_eq!(style(&warning).visuals.warn_fg_color, retuned);
    }

    ///
    /// `.scratch/theming/schema.md`'s Chrome mapping table: "Hovered/active
    /// widget foreground | `text.active`" — a dedicated key, not a reuse of
    /// `selection.border`. Okabe–Ito's own `text_active` and
    /// `selection_border` happen to share one value (`#65E6BE`), which is
    /// why reading the wrong key would still have passed
    /// `okabe_ito_chrome_matches_the_decided_record`; retuning `text_active`
    /// alone, away from `selection_border`, is what proves the wiring.
    ///
    #[test]
    fn hovered_and_active_foreground_reads_text_active_not_selection_border() {
        let theme = Theme {
            text_active: Color32::from_rgb(9, 9, 9),
            ..okabe_ito()
        };
        let visuals = style(&theme).visuals;

        assert_eq!(visuals.widgets.hovered.fg_stroke.color, theme.text_active);
        assert_eq!(visuals.widgets.active.fg_stroke.color, theme.text_active);
        assert_ne!(theme.text_active, theme.selection_border);
    }

    ///
    /// `.scratch/theming/schema.md`'s Chrome mapping table: "Links; code
    /// spans | `link`; `code.background`". Neither was read by `style`
    /// before this Theme: egui's own defaults for `hyperlink_color` and
    /// `code_bg_color` happened to equal Okabe–Ito's `link`/`code.background`
    /// values, which is exactly the "second fixed palette hidden in
    /// `Visuals` defaults" the audit line exists to catch.
    ///
    #[test]
    fn hyperlinks_and_code_spans_read_the_theme() {
        let theme = Theme {
            link: Color32::from_rgb(9, 9, 9),
            code_background: Color32::from_rgb(8, 8, 8),
            ..okabe_ito()
        };
        let visuals = style(&theme).visuals;

        assert_eq!(visuals.hyperlink_color, theme.link);
        assert_eq!(visuals.code_bg_color, theme.code_background);
    }

    ///
    /// `.scratch/theming/schema.md`'s Chrome mapping table: "Input caret and
    /// active IME underline | `input.cursor`, `input.cursor.width`" and
    /// "Inactive IME underline | Same width; existing half-linear colour
    /// attenuation." Both strokes derive from `theme.input_cursor`/`.width`;
    /// the inactive one keeps egui's own `Color32::linear_multiply(0.5)`
    /// pattern over that same colour. `legacy_visuals` is untouched — a
    /// platform default, not a themed property.
    ///
    #[test]
    fn text_cursor_and_ime_underlines_read_input_cursor() {
        let theme = Theme {
            input_cursor: Color32::from_rgb(9, 8, 7),
            ..okabe_ito()
        };
        let visuals = style(&theme).visuals;
        let width = theme.input_cursor_width.points();

        assert_eq!(
            visuals.text_cursor.stroke,
            Stroke::new(width, theme.input_cursor)
        );
        assert_eq!(
            visuals.ime_composition.active_underline_stroke,
            Stroke::new(width, theme.input_cursor)
        );
        assert_eq!(
            visuals.ime_composition.inactive_underline_stroke,
            Stroke {
                width,
                color: theme.input_cursor.linear_multiply(0.5),
            }
        );
        assert_eq!(
            visuals.ime_composition.legacy_visuals,
            Visuals::dark().ime_composition.legacy_visuals,
            "the platform default for legacy IME visuals must be untouched"
        );
    }

    ///
    /// `.scratch/theming/schema.md`'s width catalogue: chrome border widths
    /// are finite display points 0 to 2 inclusive, and "Zero width
    /// suppresses that stroke, not fills or other strokes." Retuning each of
    /// the five chrome width keys changes the matching `Stroke`'s width, and
    /// a zeroed width still leaves the stroke's own colour (and every other
    /// field) untouched — the geometry step, not this one, is what turns a
    /// zero-width `Stroke` into nothing drawn.
    ///
    #[test]
    fn chrome_border_widths_come_from_the_theme() {
        let base = okabe_ito();

        let panel_border_width = Theme {
            panel_border_width: crate::theme::ChromeWidth::from_points(1.75)
                .expect("1.75 is within 0..=2"),
            ..base.clone()
        };
        let visuals = style(&panel_border_width).visuals;
        assert_eq!(visuals.window_stroke.width, 1.75);
        assert_eq!(visuals.widgets.noninteractive.bg_stroke.width, 1.75);

        let widget_border_width = Theme {
            widget_border_width: crate::theme::ChromeWidth::from_points(1.25)
                .expect("1.25 is within 0..=2"),
            ..base.clone()
        };
        let visuals = style(&widget_border_width).visuals;
        assert_eq!(visuals.widgets.hovered.bg_stroke.width, 1.25);
        assert_eq!(visuals.widgets.active.bg_stroke.width, 1.25);
        assert_eq!(visuals.widgets.open.bg_stroke.width, 1.25);

        let widget_inactive_border_width = Theme {
            widget_inactive_border_width: crate::theme::ChromeWidth::from_points(1.5)
                .expect("1.5 is within 0..=2"),
            ..base.clone()
        };
        let visuals = style(&widget_inactive_border_width).visuals;
        assert_eq!(visuals.widgets.inactive.bg_stroke.width, 1.5);
        assert_eq!(
            visuals.widgets.inactive.bg_stroke.color,
            widget_inactive_border_width.widget_inactive_border,
            "a nonzero widget.inactive.border.width must show widget.inactive.border, not stay Stroke::NONE"
        );

        let selection_border_width = Theme {
            selection_border_width: crate::theme::ChromeWidth::from_points(0.25)
                .expect("0.25 is within 0..=2"),
            ..base.clone()
        };
        assert_eq!(
            style(&selection_border_width)
                .visuals
                .selection
                .stroke
                .width,
            0.25
        );

        // Zero a width the base Theme holds above zero, so the case can fail:
        // only the two panel strokes' widths may move.
        assert_eq!(base.panel_border_width.points(), 1.0);
        let mut expected = style(&base).visuals;
        expected.window_stroke.width = 0.0;
        expected.widgets.noninteractive.bg_stroke.width = 0.0;
        let zeroed = Theme {
            panel_border_width: crate::theme::ChromeWidth::from_points(0.0)
                .expect("0.0 is within 0..=2"),
            ..base
        };
        assert_eq!(
            style(&zeroed).visuals,
            expected,
            "a zeroed panel.border.width must suppress only that stroke's width"
        );
    }

    ///
    /// `theme.md` is the decided record for Orcvs Light's chrome as well,
    /// pinned the same way `okabe_ito_chrome_matches_the_decided_record`
    /// pins the dark one — the extension `.scratch/console-testing/issues/03`
    /// asked for, written so a third Theme extends this suite again rather
    /// than rewriting it. Every chrome key is asserted at a literal, so a
    /// retune of the light definition fails here and the same commit must
    /// move `console/src/theme.md`.
    ///
    #[test]
    fn orcvs_light_chrome_matches_the_decided_record() {
        let theme = orcvs_light();
        let visuals = style(&theme).visuals;
        let page = Color32::from_rgb(239, 244, 242); // #EFF4F2
        let source = Color32::from_rgb(250, 252, 251); // #FAFCFB
        let panel_border = Color32::from_rgb(192, 206, 201); // #C0CEC9
        let ordinary = Color32::from_rgb(48, 63, 59); // #303F3B
        let selection_fill = Color32::from_rgb(204, 235, 226); // #CCEBE2
        let selection_stroke_rest = Color32::from_rgb(24, 126, 96); // #187E60
        let selection_stroke = Color32::from_rgb(7, 98, 71); // #076247
        let bang = Color32::from_rgb(144, 66, 111); // #90426F
        let hyperlink = Color32::from_rgb(11, 98, 184); // #0B62B8
        let code_background = Color32::from_rgb(230, 230, 230); // #E6E6E6
        let input_cursor = Color32::from_rgb(0, 83, 125); // #00537D

        assert!(!visuals.dark_mode, "Orcvs Light is a light Theme");

        // panel.background: page fill, everywhere it applies.
        assert_eq!(visuals.panel_fill, page);
        assert_eq!(visuals.window_fill, page);
        assert_eq!(visuals.widgets.noninteractive.bg_fill, page);
        assert_eq!(visuals.widgets.noninteractive.weak_bg_fill, page);
        assert_eq!(visuals.widgets.inactive.bg_fill, page);
        assert_eq!(visuals.widgets.inactive.weak_bg_fill, page);
        assert_eq!(visuals.widgets.open.weak_bg_fill, page);

        // panel.border/.width and widget.inactive.border/.width — one
        // colour, as in Okabe–Ito, the inactive one at width 0.
        assert_eq!(visuals.window_stroke, Stroke::new(1.0, panel_border));
        assert_eq!(
            visuals.widgets.noninteractive.bg_stroke,
            Stroke::new(1.0, panel_border)
        );
        assert_eq!(visuals.widgets.inactive.bg_stroke.width, 0.0);
        assert_eq!(visuals.widgets.inactive.bg_stroke.color, panel_border);

        // text, text.active.
        assert_eq!(visuals.widgets.noninteractive.fg_stroke.color, ordinary);
        assert_eq!(visuals.widgets.inactive.fg_stroke.color, ordinary);
        assert_eq!(visuals.widgets.open.fg_stroke.color, ordinary);
        assert_eq!(visuals.widgets.hovered.fg_stroke.color, selection_stroke);
        assert_eq!(visuals.widgets.active.fg_stroke.color, selection_stroke);

        // text.muted: the ink at 75%, not egui's own light-mode weak text.
        // Okabe–Ito's value happens to equal egui's 0.6 gamma multiply of
        // `text`; this one deliberately does not, because attenuating dark
        // ink toward a near-white page loses contrast far faster than
        // attenuating near-white toward a near-black one.
        assert_eq!(
            visuals.weak_text_color,
            Some(Color32::from_rgba_unmultiplied(48, 63, 59, 191))
        );
        assert_ne!(
            visuals.weak_text_color,
            Some(ordinary.gamma_multiply(0.6)),
            "the light Theme's muted text is a recorded value, not egui's own attenuation"
        );

        // input.background: extreme/faint backgrounds, borrowed from the
        // Source ground exactly as the dark Theme borrows its own.
        assert_eq!(visuals.extreme_bg_color, source);
        assert_eq!(visuals.faint_bg_color, source);

        // selection.background/.border/.border.width.
        assert_eq!(visuals.selection.bg_fill, selection_fill);
        assert_eq!(visuals.selection.stroke, Stroke::new(1.0, selection_stroke));
        assert_eq!(visuals.widgets.hovered.bg_fill, selection_fill);
        assert_eq!(visuals.widgets.hovered.weak_bg_fill, selection_fill);
        assert_eq!(visuals.widgets.active.bg_fill, selection_fill);
        assert_eq!(visuals.widgets.active.weak_bg_fill, selection_fill);
        assert_eq!(visuals.widgets.open.bg_fill, selection_fill);
        assert_eq!(
            visuals.widgets.hovered.bg_stroke,
            Stroke::new(1.0, selection_stroke_rest)
        );
        assert_eq!(
            visuals.widgets.active.bg_stroke,
            Stroke::new(1.0, selection_stroke)
        );
        assert_eq!(
            visuals.widgets.open.bg_stroke,
            Stroke::new(1.0, selection_stroke_rest)
        );

        // error, warning: borrowed from Bang, as in Okabe–Ito.
        assert_eq!(visuals.error_fg_color, bang);
        assert_eq!(visuals.warn_fg_color, bang);

        // link, code.background, input.cursor/.width.
        assert_eq!(visuals.hyperlink_color, hyperlink);
        assert_eq!(visuals.code_bg_color, code_background);
        assert_eq!(visuals.text_cursor.stroke, Stroke::new(2.0, input_cursor));
        assert_eq!(
            visuals.ime_composition.active_underline_stroke,
            Stroke::new(2.0, input_cursor)
        );
        assert_eq!(
            visuals.ime_composition.inactive_underline_stroke,
            Stroke {
                width: 2.0,
                color: input_cursor.linear_multiply(0.5),
            }
        );
        assert_eq!(
            visuals.ime_composition.legacy_visuals,
            Visuals::light().ime_composition.legacy_visuals,
            "legacy_visuals is a platform default, not a themed property"
        );
    }

    ///
    /// ADR 0053's own reading of the upstream docs: "Merely changing
    /// `Visuals::dark_mode` does not convert a palette." `style` follows
    /// that by starting from `Visuals::dark()`/`Visuals::light()` rather
    /// than flipping the flag alone, so `dark_mode` still tracks
    /// `theme.appearance` exactly.
    ///
    #[test]
    fn dark_mode_follows_the_themes_declared_appearance() {
        assert!(style(&okabe_ito()).visuals.dark_mode, "Dark appearance");
        assert!(!style(&orcvs_light()).visuals.dark_mode, "Light appearance");
    }

    ///
    /// `restyle-egui-console/02`'s four prohibitions — no gradients, no
    /// rounded tiles, no shadows, no animation — hold for every Theme
    /// `style` produces, not only Okabe–Ito's: none of the four is
    /// themeable, so `style` never reads `theme` to decide them.
    ///
    #[test]
    fn the_four_prohibitions_hold_for_every_theme() {
        for theme in [okabe_ito(), orcvs_light()] {
            let built = style(&theme);
            let visuals = &built.visuals;
            assert_eq!(visuals.window_corner_radius, CornerRadius::ZERO);
            assert_eq!(visuals.menu_corner_radius, CornerRadius::ZERO);
            assert_eq!(visuals.window_shadow, Shadow::NONE);
            assert_eq!(visuals.popup_shadow, Shadow::NONE);
            for widget in [
                &visuals.widgets.noninteractive,
                &visuals.widgets.inactive,
                &visuals.widgets.hovered,
                &visuals.widgets.active,
                &visuals.widgets.open,
            ] {
                assert_eq!(widget.corner_radius, CornerRadius::ZERO);
                assert_eq!(widget.expansion, 0.0);
            }
            assert_eq!(built.animation_time, 0.0);
        }
    }

    ///
    /// Chrome tracks a Theme's own declared appearance and colours, not a
    /// single hardcoded look: two Themes that disagree on `panel.background`
    /// and `appearance` produce visibly different chrome. The Source Grid's
    /// own half of that claim — its colours changing with the resolved
    /// Theme — is proven separately by `glyph_colours_are_distinct_and_read_
    /// from_the_theme` and `each_border_channel_reads_its_own_theme_colour_
    /// not_a_fixed_default`.
    ///
    #[test]
    fn chrome_colours_change_with_the_resolved_theme() {
        let dark = style(&okabe_ito()).visuals;
        let light = style(&orcvs_light()).visuals;

        assert_ne!(dark.dark_mode, light.dark_mode);
        assert_ne!(dark.panel_fill, light.panel_fill);
        assert_ne!(
            dark.widgets.noninteractive.fg_stroke.color,
            light.widgets.noninteractive.fg_stroke.color
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
    fn glyph_colours_are_distinct_and_read_from_the_theme() {
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
        // not from a fixed palette, so a loaded custom Theme's colours
        // preview immediately once `.scratch/theming/issues/04` lets a
        // selection paint.
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
        let base = okabe_ito().sector_seam;
        assert_eq!(sector_line(100, base), base);
        let [red, green, blue, alpha] = sector_line(50, base).to_srgba_unmultiplied();
        assert!(red.abs_diff(55) <= 2);
        assert!(green.abs_diff(101) <= 2);
        assert!(blue.abs_diff(86) <= 2);
        assert_eq!(alpha, 55);
        assert_eq!(sector_line(8, base).a(), 8);
        assert_eq!(sector_line(255, base), base);
    }

    #[test]
    fn cursor_and_selection_override_the_ambient_field() {
        let theme = okabe_ito();
        let cursor_colour = Some(theme.selection_background);
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
        assert_eq!(ordinary.border, theme.grid_border);
        assert_eq!(selected.background, Some(theme.selection_background));
        assert_eq!(selected.border, theme.selection_border_rest);
        assert_eq!(cursor.background, Some(theme.selection_background));
        assert_eq!(cursor.border, theme.selection_border);
        assert_ne!(cursor, selected);
    }

    ///
    /// Defect 1: a Cell's border reads `theme.grid_border`,
    /// `theme.selection_border` and `theme.selection_border_rest` — the
    /// resolved Theme, never a fixed constant. Retuning each of the three
    /// changes the matching `CellVisuals::border` on the next call, the way
    /// `glyph_colours_are_distinct_and_read_from_the_theme` already proves
    /// for glyph colours.
    ///
    #[test]
    fn each_border_channel_reads_its_own_theme_colour_not_a_fixed_default() {
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
        assert_ne!(ordinary.border, okabe_ito().grid_border);

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
        assert_ne!(selected.border, okabe_ito().selection_border_rest);

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
        assert_ne!(cursor.border, okabe_ito().selection_border);
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
        let theme = okabe_ito();
        let style = super::style(&theme);
        let chrome = Stroke::new(1.0, theme.panel_border);
        assert_eq!(style.visuals.window_stroke, chrome);
        assert_eq!(
            style.visuals.widgets.noninteractive.bg_stroke, chrome,
            "Panel::show_separator_line reads noninteractive.bg_stroke"
        );
    }

    ///
    /// Okabe–Ito's `widget.inactive.border.width` is 0, so the idle border
    /// paints nothing (`okabe_ito_chrome_matches_the_decided_record`'s own
    /// doc has the full width-vs-colour distinction).
    ///
    #[test]
    fn idle_widgets_have_no_rest_outline() {
        let theme = okabe_ito();
        let style = super::style(&theme);
        assert_eq!(style.visuals.widgets.inactive.bg_stroke.width, 0.0);
        assert_eq!(
            style.visuals.widgets.active.bg_stroke.color,
            theme.selection_border
        );
        assert_eq!(style.visuals.selection.stroke.color, theme.selection_border);
    }

    #[test]
    fn caret_border_change_is_visible_but_restrained() {
        let theme = okabe_ito();
        let resting = theme.selection_border_rest;
        let visible = theme.selection_border;
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
        // both the Theme field above and the composition that surfaces it:
        // Function's own foreground colour at 10% alpha, the uniform tint
        // opacity the user's 2026-09-22 retune gives every tinted role.
        assert_eq!(
            function.background,
            Some(Color32::from_rgba_unmultiplied(0x00, 0x9E, 0x73, 0x1A))
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

        // The Output Portal channel composites *over* each fact's own raw
        // role background, not in place of it (`role_and_portal`'s
        // `background.blend(theme.output_portal_background)`) — Unclaimed's
        // own background is fully transparent, so the composite collapses
        // to the Output Portal tint alone; Number's is not, since the
        // user's 2026-09-22 retune made every tinted role background
        // translucent rather than opaque, so its own hue still shows
        // through under the Portal tint.
        for (paint, role_background) in [
            (SourcePaint::Unclaimed, theme.source_ordinary_background),
            (
                operand(Token::Number, OperandState::Valid),
                theme.source_number_background,
            ),
        ] {
            let visuals = painted(paint, true, &theme);
            assert_eq!(
                visuals.foreground, theme.output_portal_foreground,
                "{paint:?} did not draw the Output Portal colour"
            );
            assert_eq!(
                visuals.background,
                Some(
                    theme
                        .cell_background
                        .blend(role_background.blend(theme.output_portal_background))
                ),
                "{paint:?} did not carry its own role background composited with the Output \
                 Portal tint"
            );
        }
    }

    ///
    /// Invalid operand plus Output Portal: Portal precedence over Diagnostic
    /// — `.scratch/theming/schema.md`'s composition step 4, "Portal channels
    /// take precedence over other non-Function facts, including Diagnostic."
    /// The Invalid Number's Diagnostic-blended answer is still `role_and_
    /// portal`'s starting point, so Portal composites over *that*, not over
    /// the plain Number channel — with `diagnostic.foreground` opaque in
    /// Okabe–Ito, the Portal colour still wins outright for the foreground,
    /// exactly as it does for a Valid operand. The background is Number's
    /// own translucent tint (Diagnostic's own background is transparent, so
    /// it leaves Number's tint unchanged) composited with the Output
    /// Portal's translucent tint on top, not the Portal channel alone.
    ///
    #[test]
    fn invalid_operand_under_output_portal_takes_the_portal_colour() {
        let theme = okabe_ito();

        let invalid = painted(operand(Token::Number, OperandState::Invalid), true, &theme);

        assert_eq!(invalid.foreground, theme.output_portal_foreground);
        assert_eq!(
            invalid.background,
            Some(
                theme.cell_background.blend(
                    theme
                        .source_number_background
                        .blend(theme.output_portal_background)
                )
            )
        );
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
    /// builds the application, and `install` is what `Console::new` calls
    /// afterwards. A preference already on the context — standing in for one
    /// eframe just restored — must survive the call, unlike the removed
    /// `set_theme(Dark)` call it replaces.
    ///
    #[test]
    fn install_leaves_a_restored_theme_preference_untouched() {
        let ctx = egui::Context::default();
        ctx.set_theme(egui::ThemePreference::Light);

        install(&ctx, &okabe_ito(), &orcvs_light());

        assert_eq!(
            ctx.options(|options| options.theme_preference),
            egui::ThemePreference::Light,
            "install overwrote the restored theme preference"
        );
    }

    ///
    /// `.scratch/theming/issues/04`: `style()` of the selected dark Theme
    /// for `egui::Theme::Dark` and of the selected light Theme for
    /// `egui::Theme::Light`, each registered through `set_style_of`.
    ///
    /// The comparison is `Visuals` and `animation_time`, the two fields
    /// [`style`] actually sets, rather than `Style`'s own `PartialEq`:
    /// `Style::number_formatter` compares by `Arc::ptr_eq`
    /// (`egui-0.36.2/src/style.rs:57-60`), so two independently built
    /// `Style::default()`s — one inside each `style()` call — never compare
    /// equal on that field alone, whatever their visible content.
    ///
    #[test]
    fn install_registers_each_appearances_own_theme() {
        let ctx = egui::Context::default();

        install(&ctx, &okabe_ito(), &orcvs_light());

        for (slot, theme) in [
            (egui::Theme::Dark, okabe_ito()),
            (egui::Theme::Light, orcvs_light()),
        ] {
            let installed = ctx.style_of(slot);
            let expected = style(&theme);
            assert_eq!(
                installed.visuals, expected.visuals,
                "the {slot:?} slot does not hold style(&{})",
                theme.identity
            );
            assert_eq!(installed.animation_time, expected.animation_time);
        }
    }

    ///
    /// Pins `cell_visuals_with_cursor_colour`'s answer for a representative
    /// fact set to exact premultiplied byte arrays, captured — not
    /// recomputed from this module or `theme.rs` — by a temporary
    /// `eprintln!`-driven test run with `--nocapture`, removed afterwards.
    /// Originally captured from `ba987f6`, the commit immediately before
    /// `.scratch/theming/issues/06`'s refactor, as a TDD baseline proving
    /// the refactor reproduced pre-refactor output rather than arguing the
    /// algebra should. Every role-background literal was recaptured
    /// 2026-09-22 against the user's uniform-10%-opacity retune of the
    /// tinted roles; the values below are current output, not the original
    /// `ba987f6` capture.
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
                background: Some([23, 16, 0, 26]),
                border: [8, 16, 14, 72],
                foreground: [204, 121, 167, 255],
            },
            BaselineCase {
                name: "function",
                fact: SourcePaint::Function,
                output_portal: false,
                selected: false,
                cursor_visible: false,
                background: Some([0, 16, 12, 26]),
                border: [8, 16, 14, 72],
                foreground: [0, 158, 115, 255],
            },
            BaselineCase {
                name: "function_portal",
                fact: SourcePaint::Function,
                output_portal: true,
                selected: false,
                cursor_visible: false,
                background: Some([0, 16, 12, 26]),
                border: [8, 16, 14, 72],
                foreground: [0, 158, 115, 255],
            },
            BaselineCase {
                name: "number_pending",
                fact: operand(Token::Number, OperandState::Pending),
                output_portal: false,
                selected: false,
                cursor_visible: false,
                background: Some([9, 18, 24, 26]),
                border: [8, 16, 14, 72],
                foreground: [86, 180, 233, 255],
            },
            BaselineCase {
                name: "number_valid",
                fact: operand(Token::Number, OperandState::Valid),
                output_portal: false,
                selected: false,
                cursor_visible: false,
                background: Some([9, 18, 24, 26]),
                border: [8, 16, 14, 72],
                foreground: [86, 180, 233, 255],
            },
            BaselineCase {
                name: "number_invalid",
                fact: operand(Token::Number, OperandState::Invalid),
                output_portal: false,
                selected: false,
                cursor_visible: false,
                background: Some([9, 18, 24, 26]),
                border: [8, 16, 14, 72],
                foreground: [213, 94, 0, 255],
            },
            BaselineCase {
                name: "number_valid_portal",
                fact: operand(Token::Number, OperandState::Valid),
                output_portal: true,
                selected: false,
                cursor_visible: false,
                background: Some([31, 32, 22, 49]),
                border: [8, 16, 14, 72],
                foreground: [230, 159, 0, 255],
            },
            BaselineCase {
                name: "note_valid",
                fact: operand(Token::Note, OperandState::Valid),
                output_portal: false,
                selected: false,
                cursor_visible: false,
                background: Some([24, 23, 7, 26]),
                border: [8, 16, 14, 72],
                foreground: [240, 228, 66, 255],
            },
            BaselineCase {
                name: "atom_pending",
                fact: operand(Token::Atom, OperandState::Pending),
                output_portal: false,
                selected: false,
                cursor_visible: false,
                background: Some([24, 24, 23, 26]),
                border: [8, 16, 14, 72],
                foreground: [234, 235, 229, 255],
            },
            BaselineCase {
                name: "sequence_pending",
                fact: operand(Token::Sequence, OperandState::Pending),
                output_portal: false,
                selected: false,
                cursor_visible: false,
                background: Some([0, 12, 18, 26]),
                border: [8, 16, 14, 72],
                foreground: [0, 114, 178, 255],
            },
            BaselineCase {
                name: "sequence_invalid",
                fact: operand(Token::Sequence, OperandState::Invalid),
                output_portal: false,
                selected: false,
                cursor_visible: false,
                background: Some([0, 12, 18, 26]),
                border: [8, 16, 14, 72],
                foreground: [213, 94, 0, 255],
            },
            BaselineCase {
                name: "unclaimed_portal",
                fact: SourcePaint::Unclaimed,
                output_portal: true,
                selected: false,
                cursor_visible: false,
                background: Some([23, 16, 0, 26]),
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
