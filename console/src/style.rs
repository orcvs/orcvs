use std::sync::Arc;

use eframe::egui;

use egui::{Color32, CornerRadius, Shadow, Stroke, Style, Visuals, style::Selection};

use orcvs::render_frame::{PortalRole, PortalRoles};
use orcvs::source::{OperandState, SourcePaint, Token};

use crate::paint::{EdgeLine, EdgeRank};
use crate::theme::{Appearance, GridWidth, Theme};

///
/// `border` is the Cell's own border fact, its width in display points: the
/// fact-priority pick [`ordinary_border`] answers. `None` when the Cell
/// carries no border fact, so its edges show the Grid line or a neighbour's
/// fact. The selection outline is not a Cell's fact; the Paint draws it on the
/// edges it encloses. Never `Eq`, since a `Stroke` carries an `f32`.
///
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct CellVisuals {
    pub background: Option<Color32>,
    pub border: Option<EdgeLine>,
    pub foreground: Color32,
}

///
/// How an unselected Cell is coloured: fill, border and Token.
///
/// `background` is `None` when the Cell needs no fill of its own beyond the
/// live `theme.grid_background` — the `source_panel_frame` behind the Grid
/// has already painted that colour across the console, so a Cell with
/// nothing more to show answers `None` instead of repainting it. `Some`
/// answers the resolved Theme's role/Diagnostic/Portal channel for the
/// Cell's fact, composited over `theme.cell_background` (see
/// [`source_paint_visuals`]). [`CellColours::colours`] applies Cursor and
/// Region precedence over this answer.
///
/// Every Cell's border is [`ordinary_border`]'s: the Cursor's outline is not
/// a Cell's border but the edges it owns ([`crate::paint::Paint::edges`]).
/// `paint` is the per-Cell language fact `RenderCell::source_paint` answers
/// from the shared Claim, including Pending, Valid, or Invalid for an
/// Operand. `portal_roles` is an independent fact: the Portal roles this Cell
/// plays (`RenderCell::portal_roles`). `occupied` is whether the Cell holds
/// content, which decides only whether an unclaimed Cell's Ordinary border
/// can show.
///
fn cell_visuals(
    paint: SourcePaint,
    portal_roles: PortalRoles,
    occupied: bool,
    theme: &Theme,
) -> CellVisuals {
    let portal = painting_portal(portal_roles);
    let (foreground, background) = source_paint_visuals(paint, portal, theme);
    CellVisuals {
        background,
        border: ordinary_border(paint, portal, occupied, theme),
        foreground,
    }
}

///
/// Where a Source Grid Cell sits relative to the Cursor and a Region spanning
/// more than one Cell: the axis [`CellColours::colours`] decides a Cell's
/// fill by, independently of the Portal roles that Cell plays.
///
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CursorPlacement {
    /// Not the Cursor, not inside a Region: the ordinary Cell.
    Plain,
    /// The single-Cell Cursor, selected and visible.
    Cursor,
    /// A Region Cell other than the Cursor's own, inside a Region spanning
    /// more than one Cell.
    Region,
    /// The Cursor's own Cell inside a Region spanning more than one Cell.
    RegionCursor,
}

///
/// Every Source Paint fact a Cell can carry, in [`CellColours`]'s
/// index order: the four whole-Cell facts, then each declared operand Token
/// in each [`OperandState`].
///
const SOURCE_PAINT_FACTS: [SourcePaint; 10] = {
    use OperandState::{Invalid, Pending, Valid};
    use SourcePaint::{Bang, Comment, Function, Unclaimed};
    use Token::{Note, Number};
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
    ]
};

///
/// A Cell's colours as [`CellColours::displayed`] answers them: the opaque
/// background it shows, and the foreground the Theme draws over it.
///
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct DisplayedColours {
    pub(crate) background: Color32,
    pub(crate) foreground: Color32,
}

impl DisplayedColours {
    /// The foreground composited over the background: what a viewer sees of
    /// an occupied Cell's Token.
    pub(crate) fn displayed_foreground(self) -> Color32 {
        self.background.blend(self.foreground)
    }
}

///
/// A Cell's whole colour decision for one Theme, built once per Paint: the
/// Cursor, Region-Cursor, Region and base fills, each already composited over
/// `theme.cell_background`, the Cursor's and the Region Cursor's glyph
/// colours, and [`cell_visuals`]'s answer for each fact and painting Portal,
/// so painting and the contrast check read one decision.
///
/// An unselected Cell's visuals depend only on its Source Paint fact, the
/// Portal that paints it, [`painting_portal`]'s answer, and whether it is
/// occupied. Occupancy decides only whether an unclaimed Cell's Ordinary
/// border shows, so it splits the Unclaimed fact alone: the 11 keys
/// [`visuals_key`] answers × 4 Portal answers are every answer the walk can
/// need, and the table stays small enough that zeroing it is cheap. Each
/// entry is built by
/// calling [`cell_visuals`] itself, so the table cannot drift from the
/// per-Cell definition the tests pin; the per-Cell walk is left with one
/// indexed load instead of the role match, the Diagnostic/Portal blends, the
/// `cell.background` composite and the border priority match.
///
/// Entries are filled on first ask rather than up front. Resolving every
/// entry costs hundreds of nanoseconds, which a full Grid amortises to
/// nothing but a Paint of a scrolled-away or barely visible Grid — the
/// console derives one per frame at whatever the viewport culls to — would
/// pay in full for the handful of facts it reads; `paint_derive/empty` is the
/// benchmark that prices it. A Cell that misses pays one branch and the
/// resolution it would have paid anyway, and the facts a Source actually
/// carries are few, so the walk resolves each of them once whatever its size.
///
pub(crate) struct CellColours<'a> {
    theme: &'a Theme,
    /// `grid.background` composited over `window.background`, the console's
    /// paint order under the Source panel. Opaque, because a Theme's
    /// `window.background` always is.
    surface: Color32,
    cursor_fill: Option<Color32>,
    region_cursor_fill: Option<Color32>,
    region_fill: Option<Color32>,
    base_fill: Option<Color32>,
    cursor_glyph: Option<Color32>,
    region_cursor_glyph: Option<Color32>,
    entries: [Option<CellVisuals>; VISUALS_KEYS * PORTAL_ANSWERS],
}

/// How many answers [`painting_portal`] gives: none, Input, Output or Locked.
const PORTAL_ANSWERS: usize = 4;

/// How many keys [`visuals_key`] answers: every Source Paint fact, plus an
/// occupied unclaimed Cell.
const VISUALS_KEYS: usize = SOURCE_PAINT_FACTS.len() + 1;

impl<'a> CellColours<'a> {
    pub(crate) fn new(theme: &'a Theme) -> Self {
        Self {
            theme,
            surface: theme.window_background.blend(theme.grid_background),
            cursor_fill: compose_cell_fill(theme.cell_background, theme.cursor_background),
            region_cursor_fill: compose_cell_fill(
                theme.cell_background,
                theme.region_cursor_fill(),
            ),
            region_fill: compose_cell_fill(theme.cell_background, Some(theme.region_background)),
            base_fill: compose_cell_fill(theme.cell_background, None),
            cursor_glyph: theme.cursor_foreground,
            region_cursor_glyph: theme.region_cursor_glyph(),
            entries: [None; VISUALS_KEYS * PORTAL_ANSWERS],
        }
    }

    ///
    /// The colours of a Cell carrying `paint`, playing `portal_roles`,
    /// holding content when `occupied`, and at `placement`.
    ///
    /// The Cursor's own fill wins outright on its Cell, discarding whatever
    /// the role/Diagnostic/Portal channels answered: `cursor.background` on
    /// the single-Cell Cursor, and [`Theme::region_cursor_fill`] on the
    /// Cursor's Cell inside a spanning Region. Every other Cell keeps its
    /// role answer when it has one, and falls back to `region.background`
    /// inside a spanning Region or to the uniform Cell base outside one, so
    /// the Cell base does not count as a fact fill when evaluating Region
    /// fallback.
    ///
    /// The Cursor's glyph colour replaces the fact's on the Cursor's own Cell
    /// when it holds one: `cursor.foreground` on the single-Cell Cursor, and
    /// [`Theme::region_cursor_glyph`] inside a spanning Region. A transparent
    /// glyph colour hides the glyph; a cleared one keeps the fact's. The
    /// border is the fact's at every placement.
    ///
    #[inline]
    pub(crate) fn colours(
        &mut self,
        paint: SourcePaint,
        portal_roles: PortalRoles,
        occupied: bool,
        placement: CursorPlacement,
    ) -> CellVisuals {
        let visuals = self.fact_visuals(paint, portal_roles, occupied);
        let (background, glyph) = match placement {
            CursorPlacement::Plain => (visuals.background.or(self.base_fill), None),
            CursorPlacement::Region => (visuals.background.or(self.region_fill), None),
            CursorPlacement::Cursor => (self.cursor_fill, self.cursor_glyph),
            CursorPlacement::RegionCursor => (self.region_cursor_fill, self.region_cursor_glyph),
        };
        CellVisuals {
            background,
            foreground: glyph.unwrap_or(visuals.foreground),
            ..visuals
        }
    }

    ///
    /// The colours of a Cell carrying `paint`, playing `portal_roles`, holding
    /// content when `occupied`, and at `placement`, as the console displays
    /// them: its background composited over the opaque surface every Cell is
    /// displayed on — `grid.background` composited over `window.background` —
    /// so it is opaque, and its foreground as the Theme gives it, drawn over
    /// that background.
    ///
    pub(crate) fn displayed(
        &mut self,
        paint: SourcePaint,
        portal_roles: PortalRoles,
        occupied: bool,
        placement: CursorPlacement,
    ) -> DisplayedColours {
        let visuals = self.colours(paint, portal_roles, occupied, placement);
        DisplayedColours {
            background: self
                .surface
                .blend(visuals.background.unwrap_or(Color32::TRANSPARENT)),
            foreground: visuals.foreground,
        }
    }

    #[inline]
    fn fact_visuals(
        &mut self,
        paint: SourcePaint,
        portal_roles: PortalRoles,
        occupied: bool,
    ) -> CellVisuals {
        let portal = match painting_portal(portal_roles) {
            None => 0,
            Some(PortalRole::Input) => 1,
            Some(PortalRole::Output) => 2,
            Some(PortalRole::Locked) => 3,
        };
        let index = visuals_key(paint, occupied) * PORTAL_ANSWERS + portal;
        match self.entries[index] {
            Some(visuals) => visuals,
            None => {
                let visuals = cell_visuals(paint, portal_roles, occupied, self.theme);
                self.entries[index] = Some(visuals);
                visuals
            }
        }
    }
}

///
/// [`CellColours`]'s key for a Cell carrying `paint`: its fact's position, or
/// the one past every fact for an occupied unclaimed Cell, the only fact
/// whose visuals occupancy changes.
///
#[inline]
fn visuals_key(paint: SourcePaint, occupied: bool) -> usize {
    if occupied && paint == SourcePaint::Unclaimed {
        SOURCE_PAINT_FACTS.len()
    } else {
        fact_index(paint)
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
                Token::Comment
                | Token::Payload
                | Token::Function
                | Token::Char
                | Token::Untyped => {
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
/// A Cell's own border: the Diagnostic and Portal border channels, each
/// composited over the Grid line's colour, where a Grid line is drawn, by the
/// same fact priority
/// [`role_and_portal`] applies to foreground and background —
/// `.scratch/theming/schema.md`'s composition step 4, "Border channels use
/// the same fact priority and compose over the ordinary Grid border": the
/// painting Portal over Diagnostic over the plain Grid border, with a bound
/// Function and a Bang bypassed exactly as they are there.
///
/// Colour composites with [`blend_channel`], equivalent to the pinned [`Color32::blend`] operation
/// every other channel in this module uses, so a transparent
/// `diagnostic.border` or Portal border leaves `theme.grid_border` showing
/// unchanged.
///
/// Width is *picked* rather than blended — a stroke has one width, and
/// composing two would not mean alpha compositing — so whichever channel
/// wins the same priority decides the stroke's own display-point width. A
/// width of zero withdraws its fact: the border falls to the fact below it,
/// and a Portal withdrawn over a Diagnostic shows the Diagnostic border.
///
/// Beneath Diagnostic and the Portals sits the Cell's role border, over the
/// Grid line, when that border is visible. A Cell a role claims draws its
/// role's border whether it holds content or not, so a Comment or a Pending
/// operand slot is framed as one span. An unclaimed Cell draws the Ordinary
/// border only while `occupied`: an empty Cell keeps the plain Grid line, so
/// a visible Ordinary border never frames the whole Grid. A transparent role
/// border holds no edge, so the Grid line at `grid.border.width` keeps every
/// edge no role border is drawn on. A Bang's border ranks above every other
/// role's, so a Bang keeps all four of its edges beside another role.
///
/// `None` for a Cell carrying no fact with a width: its edges show the Grid
/// line, or a neighbour's fact.
///
/// Called for every Cell, the Cursor's included: the Cursor's outline is
/// the edges [`crate::paint::Paint::edges`] gives it, not a Cell's border.
///
fn ordinary_border(
    paint: SourcePaint,
    portal: Option<PortalRole>,
    occupied: bool,
    theme: &Theme,
) -> Option<EdgeLine> {
    let invalid_operand = matches!(
        paint,
        SourcePaint::Operand {
            state: OperandState::Invalid,
            ..
        }
    );
    // What a border composites over: the Grid line, unless a zero
    // `grid.border.width` draws none, when the border shows its own colour.
    let grid_line = EdgeLine::grid(theme).map_or(Color32::TRANSPARENT, |line| line.stroke.color);
    let own = RoleChannels::of(paint, theme);
    let framed = occupied || paint != SourcePaint::Unclaimed;
    let role = (framed && own.border.a() != 0)
        .then(|| {
            EdgeLine::drawn(
                if paint == SourcePaint::Bang {
                    EdgeRank::Bang
                } else {
                    EdgeRank::Role
                },
                own.border_width,
                blend_channel(grid_line, own.border),
            )
        })
        .flatten();
    let below_diagnostic = role.map_or(grid_line, |line| line.stroke.color);
    let diagnostic = invalid_operand
        .then(|| {
            EdgeLine::drawn(
                EdgeRank::Diagnostic,
                theme.diagnostic_border_width,
                blend_channel(below_diagnostic, theme.diagnostic_border),
            )
        })
        .flatten();
    // The same overlay `role_and_portal` composites the background with.
    let portal = fill_overlay(paint, portal).and_then(|role| {
        let channels = PortalChannels::of(role, theme);
        let below = diagnostic.map_or(below_diagnostic, |line| line.stroke.color);
        EdgeLine::drawn(
            match role {
                PortalRole::Output => EdgeRank::OutputPortal,
                PortalRole::Input => EdgeRank::InputPortal,
                PortalRole::Locked => EdgeRank::Locked,
            },
            channels.border_width,
            blend_channel(below, channels.border),
        )
    });

    portal.or(diagnostic).or(role)
}

///
/// The role whose channels paint a Cell playing `portal_roles`: Output
/// Portal over Input Portal over Locked, so a Cell a Function both reads and
/// writes shows the write, and a locked Cell a Function reads or writes
/// paints as that read or write would, as Orca draws a port over a locked
/// Cell. A locked Function's own spelling keeps its own paint inside a
/// Portal, so the lock does not show on a Cell a Function also reads or
/// writes.
///
fn painting_portal(portal_roles: PortalRoles) -> Option<PortalRole> {
    if portal_roles.contains(PortalRole::Output) {
        Some(PortalRole::Output)
    } else if portal_roles.contains(PortalRole::Input) {
        Some(PortalRole::Input)
    } else if portal_roles.contains(PortalRole::Locked) {
        Some(PortalRole::Locked)
    } else {
        None
    }
}

///
/// The role whose channels composite over the fill and border of a Cell
/// carrying `paint`, which [`painting_portal`] answered `portal` for.
///
/// A bound Function keeps all its own paint inside a Portal, so a reader
/// can find the Function standing on a Cell, and a Bang keeps all of its,
/// because a Bang is what a Producer emits rather than a value it writes.
/// Locked paints both all the same, because a lock marks nothing but a
/// Function's own spelling, a Bang Function's among them.
///
fn fill_overlay(paint: SourcePaint, portal: Option<PortalRole>) -> Option<PortalRole> {
    portal.filter(|role| {
        *role == PortalRole::Locked || !matches!(paint, SourcePaint::Function | SourcePaint::Bang)
    })
}

///
/// The role whose foreground shows over the glyph of a Cell carrying `paint`,
/// which [`painting_portal`] answered `portal` for, or `None` where the
/// fact's own foreground shows alone: [`fill_overlay`]'s answer, except where
/// that role's foreground is transparent, which composites to the fact's own.
///
fn glyph_overlay(
    paint: SourcePaint,
    portal: Option<PortalRole>,
    theme: &Theme,
) -> Option<PortalRole> {
    fill_overlay(paint, portal)
        .filter(|role| PortalChannels::of(*role, theme).foreground != Color32::TRANSPARENT)
}

///
/// The role whose foreground shows over the glyph of a Cell carrying `paint`
/// and playing `portal_roles`, or `None` where the fact's own foreground
/// shows alone: the decision [`role_and_portal`] paints the glyph with, for
/// a check that names the channel a glyph is drawn in.
///
#[cfg(test)]
pub(crate) fn glyph_role(
    paint: SourcePaint,
    portal_roles: PortalRoles,
    theme: &Theme,
) -> Option<PortalRole> {
    glyph_overlay(paint, painting_portal(portal_roles), theme)
}

///
/// One Portal role's Theme channels: `output_portal.*`, `input_portal.*` or
/// `locked.*`.
///
struct PortalChannels {
    foreground: Color32,
    background: Color32,
    border: Color32,
    border_width: GridWidth,
}

impl PortalChannels {
    fn of(role: PortalRole, theme: &Theme) -> Self {
        match role {
            PortalRole::Output => Self {
                foreground: theme.output_portal_foreground,
                background: theme.output_portal_background,
                border: theme.output_portal_border,
                border_width: theme.output_portal_border_width,
            },
            PortalRole::Input => Self {
                foreground: theme.input_portal_foreground,
                background: theme.input_portal_background,
                border: theme.input_portal_border,
                border_width: theme.input_portal_border_width,
            },
            PortalRole::Locked => Self {
                foreground: theme.locked_foreground,
                background: theme.locked_background,
                border: theme.locked_border,
                border_width: theme.locked_border_width,
            },
        }
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
fn compose_cell_fill(cell_background: Color32, fill: Option<Color32>) -> Option<Color32> {
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
/// fact and the Portal that paints it,
/// resolved from `theme` — the flat, once-per-frame lookup every field below
/// is a direct read of, never a per-Cell walk, match over a table, or hash.
///
/// # Channel composition
///
/// Every composited channel below uses [`compose_cell_fill`] (background) or
/// [`blend_channel`] (foreground): a transparent Theme channel
/// therefore reveals whatever sits beneath it, and an opaque one replaces it
/// outright, so a Theme's opaque, translucent and transparent channels all
/// compose as its record states. [`role_and_portal`] answers the
/// foreground and the *raw*, not-yet-composited background (a role's own
/// channel, blended with Diagnostic and the painting Portal as those apply); this
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
/// | any other fact, Portal | above ▸ Portal foreground | above ▸ Portal background |
/// | Bang, Portal | `source.bang` (bypassed) | `source.bang.background` (bypassed) |
/// | Function, Portal | `source.function` (bypassed) | `source.function.background` (bypassed) |
/// | any fact, Locked | above ▸ `locked.foreground` | above ▸ `locked.background` |
///
/// "Portal" is the painting Portal's channels, `output_portal.*` or
/// `input_portal.*`, and "Locked" is `locked.*` (see the next section).
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
/// # Portal precedence
///
/// `portal` is [`painting_portal`]'s answer from `RenderCell::portal_roles()`:
/// Output Portal when the Cell plays that role, else Input Portal when it
/// plays that one, else Locked when it plays that one. Only one role's
/// channels paint a Cell; the first of them wins outright rather than
/// compositing over the others'.
///
/// Locked paints every fact, a bound Function and a Bang included, because
/// the Cells a lock marks are always the two Cells of the Function of the
/// Expression root it withheld, a Bang Function's among them: a bypass for
/// either would leave a lock nothing to paint. Where a read or a write also
/// covers those Cells, the Portal answers instead, and a Function or a Bang
/// keeps its own paint inside it, so neither the Portal nor the lock shows
/// there.
///
/// The Portal is read first, because the Cell it names is a Portal Cell
/// whatever else claims it — a written answer, an empty Cell still waiting for
/// one, or another Expression's operand slot the Portal happens to land on;
/// Diagnostic included, so an Invalid operand inside a Portal draws the Portal
/// foreground. The one exception is a bound Function claim: a Cell that is
/// itself a Function's own two-Cell spelling keeps its Function paint outright,
/// root or nested, the same as when no Portal covers it. That
/// is the smallest rule that both lets a producer's Output Portal show
/// through a consumer's operand Cells (the written value is the producer's
/// output) and still lets a reader find the Function that stands on a Cell:
/// distinguishing a root's own spelling from a nested one would need the
/// Expression itself, which this decision does not have and does not need.
/// A Bang is the one further exception: it keeps all its own paint inside a
/// Portal, as a Function does, because a Bang is what a Producer emits
/// rather than a value it writes.
///
/// # Otherwise, from the Source Paint fact
///
/// An Unclaimed Cell answers Ordinary. Bang and Comment always answer their
/// own channel. Function — root or nested — draws its own channel. Text that
/// spells no Function where an Expression could start (`hi`, both Cells of a
/// written `07`, a lone `|`) answers Ordinary, as an unclaimed Cell does:
/// `RenderCell::source_paint` answers it as Unclaimed because the Parser's
/// attempted Function classification is not a Paint distinction. Diagnostic
/// belongs to an Invalid operand slot. Number and Note Operand facts read
/// [`operand_paint`].
///
fn source_paint_visuals(
    paint: SourcePaint,
    portal: Option<PortalRole>,
    theme: &Theme,
) -> (Color32, Option<Color32>) {
    let (foreground, background) = role_and_portal(paint, portal, theme);

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
fn role_and_portal(
    paint: SourcePaint,
    portal: Option<PortalRole>,
    theme: &Theme,
) -> (Color32, Color32) {
    let (foreground, background) = role(paint, theme);
    let foreground = glyph_overlay(paint, portal, theme).map_or(foreground, |role| {
        blend_channel(foreground, PortalChannels::of(role, theme).foreground)
    });
    let background = fill_overlay(paint, portal).map_or(background, |role| {
        blend_channel(background, PortalChannels::of(role, theme).background)
    });
    (foreground, background)
}

///
/// The declared role's own foreground and raw background for `paint`, before
/// Portal composition — Diagnostic already applied for an Invalid
/// Operand by [`operand_paint`].
///
fn role(paint: SourcePaint, theme: &Theme) -> (Color32, Color32) {
    let own = RoleChannels::of(paint, theme);
    match paint {
        SourcePaint::Operand { state, .. } => {
            operand_paint(own.foreground, own.background, state, theme)
        }
        _ => (own.foreground, own.background),
    }
}

///
/// One Source role's Theme channels: `source.<role>`, its `.background`,
/// its `.border` and its `.border.width`.
///
struct RoleChannels {
    foreground: Color32,
    background: Color32,
    border: Color32,
    border_width: GridWidth,
}

impl RoleChannels {
    ///
    /// The channels of the role `paint` draws in, the one lookup the glyph,
    /// fill and border all read: Unclaimed draws as Ordinary, a Bang as
    /// Bang rather than as a Function, and an operand as its declared Token
    /// in every state.
    ///
    fn of(paint: SourcePaint, theme: &Theme) -> Self {
        let channels = |foreground, background, border, border_width| Self {
            foreground,
            background,
            border,
            border_width,
        };
        match paint {
            SourcePaint::Unclaimed => channels(
                theme.source_ordinary,
                theme.source_ordinary_background,
                theme.source_ordinary_border,
                theme.source_ordinary_border_width,
            ),
            SourcePaint::Comment => channels(
                theme.source_comment,
                theme.source_comment_background,
                theme.source_comment_border,
                theme.source_comment_border_width,
            ),
            SourcePaint::Function => channels(
                theme.source_function,
                theme.source_function_background,
                theme.source_function_border,
                theme.source_function_border_width,
            ),
            SourcePaint::Bang => channels(
                theme.source_bang,
                theme.source_bang_background,
                theme.source_bang_border,
                theme.source_bang_border_width,
            ),
            SourcePaint::Operand { token, .. } => match token {
                Token::Number => channels(
                    theme.source_number,
                    theme.source_number_background,
                    theme.source_number_border,
                    theme.source_number_border_width,
                ),
                Token::Note => channels(
                    theme.source_note,
                    theme.source_note_background,
                    theme.source_note_border,
                    theme.source_note_border_width,
                ),
                Token::Comment
                | Token::Payload
                | Token::Function
                | Token::Char
                | Token::Untyped => {
                    unreachable!("SourcePaint::Operand carries only a declared operand Token")
                }
            },
        }
    }
}

///
/// An Operand Cell's foreground and raw background, from its declared
/// `colour`/`background` (`Number` or `Note`) and its
/// finished state.
///
/// The Parser labels an operand slot with its signature's declared Token
/// whether or not what stands there binds, so Valid, Pending, and Invalid all
/// keep the declared role's own background — `.scratch/theming/schema.md`:
/// "Declared Number/Note operands retain their role background in Pending,
/// Valid and Invalid states." The foreground differs: a Valid or
/// Pending slot draws `colour` outright, and an Invalid one — Cells whose
/// written content failed to bind — blends the Diagnostic foreground channel
/// over it, so a transparent `diagnostic.foreground` still reveals the
/// declared role while an opaque one replaces it. `.+0`'s
/// second operand is this Invalid case: one Cell holds `0`,
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
/// way the Source Grid does. After
/// this, nothing the console draws is a compiled constant, and no toolkit
/// default is a second, hidden fixed palette beside it.
///
/// `visuals.weak_text_color` is set explicitly from `theme.text_muted`
/// rather than left `None`: egui's own default derives a weak text colour
/// from `text_color() * weak_text_alpha` (`egui-0.36.2/src/style.rs`'s
/// `Visuals::weak_text_color`, `weak_text_alpha` defaulting to `0.6`), so
/// a Theme's `text.muted` is honoured instead of silently recomputed from
/// `text` whatever the Theme actually declares.
///
/// `visuals.hyperlink_color`/`code_bg_color` read `link`/`code.background`
/// directly: egui's own defaults for both are the "second fixed palette
/// hidden in `Visuals` defaults" the schema's audit line exists to catch,
/// and a Theme's `link`/`code.background` would otherwise never reach these
/// two fields.
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
/// `selection.border`: a Theme may set the two alike, and reading the
/// wrong key would then still look right —
/// `hovered_and_active_foreground_reads_text_active_
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
/// `widget.inactive.border.width` for the idle widget border,
/// `selection.border.width` for text selection, and
/// `input.cursor.width` for the caret and the active IME underline.
///
pub fn style(theme: &Theme) -> Style {
    // Pairs a bounded `ChromeWidth` with its colour — every chrome border,
    // the caret and the IME underlines are one of these pairs.
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
    // `Frame::window`, `Ui::separator` and a group's frame read these —
    // `panel.border`/`panel.border.width`, the "Panel/window/noninteractive
    // border" row. The bars draw `bar_separator` in place of `Panel`'s own
    // separator line, which reads them too.
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
    // a Theme may set the two alike, and reading the wrong key would then
    // still look right until a custom Theme set them apart.
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
/// The line along each bar's edge that faces the Source, and along the
/// Overview's edges that do: `panel.separator` at `panel.separator.width`.
/// The console draws it in place of egui's own panel separator line, so
/// `panel.border` keeps windows, menus, popups and `Ui::separator`.
///
pub(crate) fn bar_separator(theme: &Theme) -> Stroke {
    Stroke::new(theme.panel_separator_width.points(), theme.panel_separator)
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
/// value just restored. The console calls it
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
        CellColours, CellVisuals, CursorPlacement, EdgeLine, EdgeRank, SOURCE_PAINT_FACTS,
        bar_separator, cell_visuals, fact_index, install, style,
    };
    use crate::contrast::PORTAL_ROLE_SETS;
    use crate::theme::{Appearance, Theme};
    use crate::theme_registry::tests_support::default_theme;
    use egui::{Color32, CornerRadius, Shadow, Stroke, Visuals};
    use orcvs::render_frame::{PortalRole, PortalRoles};
    use orcvs::source::{OperandState, SourcePaint, Token};

    ///
    /// The dark default with every Source role, Diagnostic and Output Portal
    /// border transparent, the last two at a drawn width, so a test about one
    /// border fact sees only the border it sets, composited over the Grid
    /// line.
    ///
    fn transparent_borders() -> Theme {
        let drawn = crate::theme::GridWidth::from_points(0.5).expect("0.5 is within 0..=1");
        Theme {
            source_number_border: Color32::TRANSPARENT,
            source_note_border: Color32::TRANSPARENT,
            source_function_border: Color32::TRANSPARENT,
            source_bang_border: Color32::TRANSPARENT,
            diagnostic_border: Color32::TRANSPARENT,
            diagnostic_border_width: drawn,
            output_portal_border: Color32::TRANSPARENT,
            output_portal_border_width: drawn,
            ..default_theme(Appearance::Dark)
        }
    }

    /// The roles of a Cell only an Output Portal covers.
    const OUTPUT: PortalRoles = PortalRoles::NONE.with(PortalRole::Output);

    /// The roles of a Cell only an Input Portal covers.
    const INPUT: PortalRoles = PortalRoles::NONE.with(PortalRole::Input);

    /// The roles of a Cell a Function both reads and writes.
    const BOTH: PortalRoles = OUTPUT.with(PortalRole::Input);

    /// The roles of a Cell only a lock marks.
    const LOCKED: PortalRoles = PortalRoles::NONE.with(PortalRole::Locked);

    /// [`OUTPUT`] when `output_portal` is true, and no role otherwise.
    fn output_portal_roles(output_portal: bool) -> PortalRoles {
        if output_portal {
            OUTPUT
        } else {
            PortalRoles::NONE
        }
    }

    ///
    /// `fact_index` and `SOURCE_PAINT_FACTS` name the same order, so no two
    /// facts share a `CellColours` entry and none reads another's.
    ///
    #[test]
    fn every_source_paint_fact_indexes_its_own_table_entry() {
        for (index, fact) in SOURCE_PAINT_FACTS.into_iter().enumerate() {
            assert_eq!(fact_index(fact), index, "{fact:?}");
        }
    }

    ///
    /// The once-per-Paint table answers exactly what the per-Cell definition
    /// answers for every unselected fact and Portal role set, under the dark
    /// default and under a Theme whose Cell base and fact channels are all
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
            input_portal_foreground: Color32::from_rgba_unmultiplied(170, 120, 250, 110),
            input_portal_background: Color32::from_rgba_unmultiplied(30, 20, 60, 70),
            input_portal_border: Color32::from_rgba_unmultiplied(170, 120, 250, 40),
            locked_foreground: Color32::from_rgba_unmultiplied(0, 0, 0, 100),
            locked_background: Color32::from_rgba_unmultiplied(120, 120, 120, 50),
            locked_border: Color32::from_rgba_unmultiplied(120, 120, 120, 30),
            ..default_theme(Appearance::Dark)
        };

        for theme in [default_theme(Appearance::Dark), translucent] {
            let mut table = CellColours::new(&theme);
            for fact in SOURCE_PAINT_FACTS {
                for (roles, occupied) in PORTAL_ROLE_SETS
                    .into_iter()
                    .flat_map(|roles| [(roles, false), (roles, true)])
                {
                    assert_eq!(
                        table.fact_visuals(fact, roles, occupied),
                        cell_visuals(fact, roles, occupied, &theme),
                        "{fact:?}, {roles:?}, occupied {occupied}"
                    );
                }
            }
        }
    }

    ///
    /// [`cell_visuals`] for a Cell carrying `paint`: unselected, before Cursor
    /// and Region precedence, so a test that does not exercise them need not
    /// name a placement.
    ///
    /// `paint` is the per-Cell language fact, fed in as itself. Deriving it
    /// here from a hand-built claim would restate
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
        painted_in(paint, output_portal_roles(output_portal), theme)
    }

    /// [`painted`] for a Cell playing any set of Portal roles.
    fn painted_in(paint: SourcePaint, portal_roles: PortalRoles, theme: &Theme) -> CellVisuals {
        cell_visuals(paint, portal_roles, true, theme)
    }

    ///
    /// [`CellColours::colours`] for one Cell at `placement`, under `theme`
    /// with `cursor_fill` as its `cursor.background`.
    ///
    fn coloured(
        paint: SourcePaint,
        portal_roles: PortalRoles,
        placement: CursorPlacement,
        cursor_fill: Option<Color32>,
        theme: &Theme,
    ) -> CellVisuals {
        let theme = Theme {
            cursor_background: cursor_fill,
            ..theme.clone()
        };
        CellColours::new(&theme).colours(paint, portal_roles, true, placement)
    }

    ///
    /// A Cell's whole colour decision at every placement, over an ordinary
    /// (transparent), a tinted (opaque) and a translucent fact background:
    /// the Cursor's and the Region Cursor's fills win outright, a Region Cell
    /// keeps a fact fill and falls back to the Region fill only where it has
    /// none, and the foreground and border are the fact's at every placement.
    ///
    #[test]
    fn a_cells_colours_cross_every_placement_and_kind_of_background() {
        let cursor = Color32::from_rgb(1, 2, 3);
        let region_cursor = Color32::from_rgb(4, 5, 6);
        let region = Color32::from_rgb(7, 8, 9);
        let tint = Color32::from_rgb(10, 20, 30);
        let translucent = Color32::from_rgba_unmultiplied(200, 100, 50, 128);
        let theme = Theme {
            cursor_background: Some(cursor),
            region_cursor_background: Some(region_cursor),
            region_background: region,
            source_ordinary_background: Color32::TRANSPARENT,
            source_function_background: tint,
            source_number_background: translucent,
            // The Cursor's glyph colours are cleared, so the foreground is
            // the fact's at every placement.
            cursor_foreground: None,
            region_cursor_foreground: None,
            ..default_theme(Appearance::Dark)
        };
        let ordinary = SourcePaint::Unclaimed;
        let tinted = SourcePaint::Function;
        let see_through = operand(Token::Number, OperandState::Valid);

        let mut colours = CellColours::new(&theme);
        for (placement, expected) in [
            (
                CursorPlacement::Plain,
                [None, Some(tint), Some(translucent)],
            ),
            (
                CursorPlacement::Region,
                [Some(region), Some(tint), Some(translucent)],
            ),
            (CursorPlacement::Cursor, [Some(cursor); 3]),
            (CursorPlacement::RegionCursor, [Some(region_cursor); 3]),
        ] {
            for (fact, background) in [ordinary, tinted, see_through].into_iter().zip(expected) {
                let unselected = cell_visuals(fact, PortalRoles::NONE, true, &theme);
                let visuals = colours.colours(fact, PortalRoles::NONE, true, placement);
                assert_eq!(visuals.background, background, "{placement:?}, {fact:?}");
                assert_eq!(
                    visuals.foreground, unselected.foreground,
                    "{placement:?}, {fact:?}"
                );
                assert_eq!(visuals.border, unselected.border, "{placement:?}, {fact:?}");
            }
        }
    }

    ///
    /// A single-Cell Cursor with no fill of its own, over a transparent Cell
    /// base, paints no fill: a Region fill reaches only a Region spanning more
    /// than one Cell, and the single-Cell Cursor is not one.
    ///
    #[test]
    fn a_cursor_with_no_fill_paints_none_even_with_a_region_fill() {
        let theme = Theme {
            cursor_background: None,
            cell_background: Color32::TRANSPARENT,
            region_background: Color32::from_rgb(7, 8, 9),
            ..default_theme(Appearance::Dark)
        };
        assert_eq!(
            CellColours::new(&theme)
                .colours(
                    SourcePaint::Unclaimed,
                    PortalRoles::NONE,
                    true,
                    CursorPlacement::Cursor
                )
                .background,
            None
        );
    }

    ///
    /// A declared operand slot's finished fact: its Token and its state.
    ///
    /// Which pairs a Source can actually produce is a fact about the Parser
    /// rather than about this decision function, so the fixtures below state
    /// the pair they mean and
    /// `paint::tests::an_operand_cell_of_every_token_a_source_can_claim_is_
    /// tinted_with_its_own_colour` drives `.+` and `.v` through the Render
    /// Frame to prove those pairs are the real ones.
    ///
    fn operand(token: Token, state: OperandState) -> SourcePaint {
        SourcePaint::Operand { token, state }
    }

    ///
    /// `theme.md` is the decided record for the chrome. For every built-in
    /// the registry holds, its table there holds the Theme's values, and
    /// every chrome field `style` sets reads the Theme key the record maps it
    /// to, so a change to the chrome fails here and the same commit must
    /// change the record. Each field is asserted on its own rather than as
    /// one `Visuals` equality, so a regression names the one field that moved
    /// instead of a giant struct diff. `widgets.inactive.bg_stroke` is a real
    /// colour at the Theme's inactive border width, not `Stroke::NONE`, even
    /// where that width is zero: both paint nothing, but the values differ.
    ///
    #[test]
    fn every_built_in_chrome_matches_its_record() {
        let registry = crate::theme_registry::ThemeRegistry::built_in();
        for theme in crate::theme_registry::tests_support::built_ins(&registry) {
            crate::theme::record::assert_matches_its_table(theme);
            assert_chrome_reads_its_keys(theme);
        }
    }

    fn assert_chrome_reads_its_keys(theme: &Theme) {
        let name = &theme.name;
        let visuals = style(theme).visuals;
        let page = theme.panel_background;
        let selection_fill = theme.selection_background;
        let widget = theme.widget_border_width.points();

        assert_eq!(
            visuals.dark_mode,
            theme.appearance == Appearance::Dark,
            "{name}"
        );

        // panel.background: page fill, everywhere it applies.
        for (field, fill) in [
            ("panel_fill", visuals.panel_fill),
            ("window_fill", visuals.window_fill),
            (
                "noninteractive.bg_fill",
                visuals.widgets.noninteractive.bg_fill,
            ),
            (
                "noninteractive.weak_bg_fill",
                visuals.widgets.noninteractive.weak_bg_fill,
            ),
            ("inactive.bg_fill", visuals.widgets.inactive.bg_fill),
            (
                "inactive.weak_bg_fill",
                visuals.widgets.inactive.weak_bg_fill,
            ),
            ("open.weak_bg_fill", visuals.widgets.open.weak_bg_fill),
        ] {
            assert_eq!(fill, page, "{name}'s {field}");
        }

        // panel.border/.width: the chrome stroke.
        let panel_border = Stroke::new(theme.panel_border_width.points(), theme.panel_border);
        assert_eq!(visuals.window_stroke, panel_border, "{name}");
        assert_eq!(
            visuals.widgets.noninteractive.bg_stroke, panel_border,
            "{name}"
        );

        // widget.inactive.border/.width.
        assert_eq!(
            visuals.widgets.inactive.bg_stroke,
            Stroke::new(
                theme.widget_inactive_border_width.points(),
                theme.widget_inactive_border
            ),
            "{name}"
        );

        // text: noninteractive/inactive/open foreground; text.active:
        // hovered/active foreground; text.muted: weak text.
        assert_eq!(
            visuals.widgets.noninteractive.fg_stroke.color, theme.text,
            "{name}"
        );
        assert_eq!(
            visuals.widgets.inactive.fg_stroke.color, theme.text,
            "{name}"
        );
        assert_eq!(visuals.widgets.open.fg_stroke.color, theme.text, "{name}");
        assert_eq!(
            visuals.widgets.hovered.fg_stroke.color, theme.text_active,
            "{name}"
        );
        assert_eq!(
            visuals.widgets.active.fg_stroke.color, theme.text_active,
            "{name}"
        );
        assert_eq!(visuals.weak_text_color, Some(theme.text_muted), "{name}");

        // input.background: extreme/faint backgrounds.
        assert_eq!(visuals.extreme_bg_color, theme.input_background, "{name}");
        assert_eq!(visuals.faint_bg_color, theme.input_background, "{name}");

        // selection.background/.border/.border.width: the selection
        // highlight, and the hovered/active/open strong fill.
        assert_eq!(visuals.selection.bg_fill, selection_fill, "{name}");
        assert_eq!(
            visuals.selection.stroke,
            Stroke::new(
                theme.selection_border_width.points(),
                theme.selection_border
            ),
            "{name}"
        );
        for (field, fill) in [
            ("hovered.bg_fill", visuals.widgets.hovered.bg_fill),
            ("hovered.weak_bg_fill", visuals.widgets.hovered.weak_bg_fill),
            ("active.bg_fill", visuals.widgets.active.bg_fill),
            ("active.weak_bg_fill", visuals.widgets.active.weak_bg_fill),
            ("open.bg_fill", visuals.widgets.open.bg_fill),
        ] {
            assert_eq!(fill, selection_fill, "{name}'s {field}");
        }

        // selection.border/selection.border.rest at widget.border.width:
        // hovered/active/open border.
        assert_eq!(
            visuals.widgets.hovered.bg_stroke,
            Stroke::new(widget, theme.selection_border_rest),
            "{name}"
        );
        assert_eq!(
            visuals.widgets.active.bg_stroke,
            Stroke::new(widget, theme.selection_border),
            "{name}"
        );
        assert_eq!(
            visuals.widgets.open.bg_stroke,
            Stroke::new(widget, theme.selection_border_rest),
            "{name}"
        );

        // error, warning, link, code.background.
        assert_eq!(visuals.error_fg_color, theme.error, "{name}");
        assert_eq!(visuals.warn_fg_color, theme.warning, "{name}");
        assert_eq!(visuals.hyperlink_color, theme.link, "{name}");
        assert_eq!(visuals.code_bg_color, theme.code_background, "{name}");

        // input.cursor/.width: the text-edit caret and the active IME
        // underline share one property pair; the inactive underline is
        // egui's own half-linear attenuation of that colour.
        let input_cursor = Stroke::new(theme.input_cursor_width.points(), theme.input_cursor);
        assert_eq!(visuals.text_cursor.stroke, input_cursor, "{name}");
        assert_eq!(
            visuals.ime_composition.active_underline_stroke, input_cursor,
            "{name}"
        );
        assert_eq!(
            visuals.ime_composition.inactive_underline_stroke,
            Stroke {
                width: input_cursor.width,
                color: theme.input_cursor.linear_multiply(0.5),
            },
            "{name}"
        );
        let platform = match theme.appearance {
            Appearance::Dark => Visuals::dark(),
            Appearance::Light => Visuals::light(),
        };
        assert_eq!(
            visuals.ime_composition.legacy_visuals, platform.ime_composition.legacy_visuals,
            "legacy_visuals is a platform default, not a themed property"
        );
    }

    ///
    /// Retuning each of the ten named chrome keys `style` reads changes the
    /// matching `Visuals` field on the next call — proof it reads `theme`
    /// live rather than merely reproducing one Theme's own values by
    /// construction, the chrome counterpart of
    /// `glyph_colours_are_distinct_and_read_from_the_theme`'s retuned check.
    ///
    #[test]
    fn each_named_chrome_key_changes_style_when_retuned() {
        let base = default_theme(Appearance::Dark);
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
    /// `selection.border`. A Theme may give `text_active` and
    /// `selection_border` one value, as the dark default does, and reading
    /// the wrong key would then still pass
    /// `every_built_in_chrome_matches_its_record`; retuning `text_active`
    /// alone, away from `selection_border`, is what proves the wiring.
    ///
    #[test]
    fn hovered_and_active_foreground_reads_text_active_not_selection_border() {
        let theme = Theme {
            text_active: Color32::from_rgb(9, 9, 9),
            ..default_theme(Appearance::Dark)
        };
        let visuals = style(&theme).visuals;

        assert_eq!(visuals.widgets.hovered.fg_stroke.color, theme.text_active);
        assert_eq!(visuals.widgets.active.fg_stroke.color, theme.text_active);
        assert_ne!(theme.text_active, theme.selection_border);
    }

    ///
    /// `.scratch/theming/schema.md`'s Chrome mapping table: "Links; code
    /// spans | `link`; `code.background`". Retuning both away from any
    /// Theme's values proves `style` reads them rather than leaving egui's
    /// own defaults, the "second fixed palette hidden in `Visuals` defaults"
    /// the audit line exists to catch.
    ///
    #[test]
    fn hyperlinks_and_code_spans_read_the_theme() {
        let theme = Theme {
            link: Color32::from_rgb(9, 9, 9),
            code_background: Color32::from_rgb(8, 8, 8),
            ..default_theme(Appearance::Dark)
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
            ..default_theme(Appearance::Dark)
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
    /// suppresses that stroke, not fills or other strokes." Retuning four of
    /// the five chrome width keys changes the matching `Stroke`'s width
    /// (`text_cursor_and_ime_underlines_read_input_cursor` reads the fifth,
    /// `input.cursor.width`, at its default), and
    /// a zeroed width still leaves the stroke's own colour (and every other
    /// field) untouched — the geometry step, not this one, is what turns a
    /// zero-width `Stroke` into nothing drawn.
    ///
    #[test]
    fn chrome_border_widths_come_from_the_theme() {
        let base = default_theme(Appearance::Dark);

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
    /// `panel.separator` and its width reach only [`bar_separator`]: windows,
    /// menus, popups and `Ui::separator` keep `panel.border`.
    ///
    #[test]
    fn the_panel_separator_reaches_only_the_bar_separator() {
        let base = default_theme(Appearance::Dark);
        let retuned = Theme {
            panel_separator: Color32::from_rgb(0x12, 0x34, 0x56),
            panel_separator_width: crate::theme::ChromeWidth::from_points(0.5)
                .expect("0.5 is within 0..=2"),
            ..base.clone()
        };
        assert_ne!(bar_separator(&retuned), bar_separator(&base));
        assert_eq!(
            bar_separator(&retuned),
            Stroke::new(0.5, Color32::from_rgb(0x12, 0x34, 0x56))
        );
        assert_eq!(
            style(&retuned).visuals,
            style(&base).visuals,
            "panel.separator reached egui's Visuals"
        );
    }

    ///
    /// `docs/research/egui-theming.md`'s reading of the upstream docs:
    /// "Merely changing `Visuals::dark_mode` does not convert a palette."
    /// `style` follows
    /// that by starting from `Visuals::dark()`/`Visuals::light()` rather
    /// than flipping the flag alone, so `dark_mode` still tracks
    /// `theme.appearance` exactly.
    ///
    #[test]
    fn dark_mode_follows_the_themes_declared_appearance() {
        assert!(
            style(&default_theme(Appearance::Dark)).visuals.dark_mode,
            "Dark appearance"
        );
        assert!(
            !style(&default_theme(Appearance::Light)).visuals.dark_mode,
            "Light appearance"
        );
    }

    ///
    /// The four prohibitions — no gradients, no
    /// rounded tiles, no shadows, no animation — hold for every Theme
    /// `style` produces, every built-in among them: none of the four is
    /// themeable, so `style` never reads `theme` to decide them.
    ///
    #[test]
    fn the_four_prohibitions_hold_for_every_theme() {
        let registry = crate::theme_registry::ThemeRegistry::built_in();
        for theme in crate::theme_registry::tests_support::built_ins(&registry) {
            let built = style(theme);
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
        let dark = style(&default_theme(Appearance::Dark)).visuals;
        let light = style(&default_theme(Appearance::Light)).visuals;

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
        let theme = default_theme(Appearance::Dark);
        let function = painted(SourcePaint::Function, false, &theme);
        let number = painted(operand(Token::Number, OperandState::Valid), false, &theme);
        let note = painted(operand(Token::Note, OperandState::Valid), false, &theme);
        let ordinary = painted(SourcePaint::Unclaimed, false, &theme);
        let bang = painted(SourcePaint::Bang, false, &theme);
        let comment = painted(SourcePaint::Comment, false, &theme);

        assert_eq!(function.foreground, theme.source_function);
        assert_eq!(number.foreground, theme.source_number);
        assert_eq!(note.foreground, theme.source_note);
        assert_eq!(ordinary.foreground, theme.source_ordinary);
        assert_eq!(bang.foreground, theme.source_bang);
        assert_eq!(comment.foreground, theme.source_comment);
        assert_ne!(number.foreground, function.foreground);
        assert_ne!(number.foreground, note.foreground);
        assert_ne!(number.foreground, ordinary.foreground);
        assert_ne!(comment.foreground, ordinary.foreground);

        // A changed Theme reaches `cell_visuals` on the
        // very next call — the Source Grid paints from the resolved Theme,
        // not from a fixed palette, so a selected custom Theme's colours
        // paint immediately.
        let retuned = Theme {
            source_function: Color32::from_rgb(1, 2, 3),
            ..default_theme(Appearance::Dark)
        };
        assert_eq!(
            painted(SourcePaint::Function, false, &retuned).foreground,
            Color32::from_rgb(1, 2, 3)
        );
    }

    #[test]
    fn cursor_and_selection_override_the_ambient_field() {
        let theme = default_theme(Appearance::Dark);
        let cursor_colour = Some(theme.selection_background);
        let ordinary = cell_visuals(SourcePaint::Unclaimed, PortalRoles::NONE, true, &theme);
        let selected = coloured(
            SourcePaint::Unclaimed,
            PortalRoles::NONE,
            CursorPlacement::Cursor,
            cursor_colour,
            &theme,
        );

        // `None`: the panel behind the Grid has already painted the Source colour.
        assert_eq!(ordinary.background, None);
        // No border of its own: the Grid line shows on its edges.
        assert_eq!(ordinary.border, None);
        assert_eq!(selected.background, Some(theme.selection_background));
        // The selected Cell keeps its fact's border; its outline is the
        // edges the Paint gives it.
        assert_eq!(selected.border, None);
    }

    ///
    /// A Cell's border reads `theme.grid_border` (under a transparent
    /// Diagnostic border) — the resolved Theme, never a fixed
    /// constant. Retuning it changes `CellVisuals::border` on the next call,
    /// the way `glyph_colours_are_distinct_and_read_from_the_theme` already
    /// proves for glyph colours.
    ///
    #[test]
    fn each_border_channel_reads_its_own_theme_colour_not_a_fixed_default() {
        let retuned_grid = Theme {
            grid_border: Color32::from_rgb(11, 22, 33),
            ..transparent_borders()
        };
        let invalid = painted(
            operand(Token::Number, OperandState::Invalid),
            false,
            &retuned_grid,
        );
        let colour = |visuals: CellVisuals| visuals.border.map(|border| border.stroke.color);
        assert_eq!(colour(invalid), Some(Color32::from_rgb(11, 22, 33)));
    }

    ///
    /// The three optional-fill states an unselected `cursor.background`
    /// exercises once resolved onto a Cell: omission/inheritance already
    /// collapses to one of the other two by `theme_document::resolve`
    /// (`theme_document::tests::omitted_optional_colours_inherit_the_parent`), so what
    /// this proves is the remaining pair at the paint layer — `None`
    /// (cleared, "supplies no Cursor fill") answers no background, and
    /// `Some`, including a fully transparent colour, is a supplied value that
    /// paints exactly what it names.
    ///
    #[test]
    fn cursor_fill_is_optional_and_defaults_to_the_uniform_cell_background() {
        let colour = Color32::from_rgb(1, 2, 3);
        let theme = default_theme(Appearance::Dark);
        assert_eq!(
            coloured(
                SourcePaint::Unclaimed,
                PortalRoles::NONE,
                CursorPlacement::Cursor,
                None,
                &theme
            )
            .background,
            None
        );
        assert_eq!(
            coloured(
                SourcePaint::Unclaimed,
                PortalRoles::NONE,
                CursorPlacement::Cursor,
                Some(colour),
                &theme
            )
            .background,
            Some(colour)
        );
        // An explicit but fully transparent Cursor fill is still a supplied
        // value: it replaces the role fill outright, painting nothing extra
        // over the transparent uniform Cell base — distinct in kind from
        // `None` above, though both compose to no visible paint here.
        assert_eq!(
            coloured(
                SourcePaint::Unclaimed,
                PortalRoles::NONE,
                CursorPlacement::Cursor,
                Some(Color32::TRANSPARENT),
                &theme
            )
            .background,
            None
        );
    }

    #[test]
    fn panel_separator_uses_the_grid_line() {
        let theme = default_theme(Appearance::Dark);
        let style = super::style(&theme);
        let chrome = Stroke::new(1.0, theme.panel_border);
        assert_eq!(style.visuals.window_stroke, chrome);
        assert_eq!(
            style.visuals.widgets.noninteractive.bg_stroke, chrome,
            "Panel::show_separator_line reads noninteractive.bg_stroke"
        );
    }

    ///
    /// The dark default's `widget.inactive.border.width` is 0, so the idle
    /// border paints nothing (`every_built_in_chrome_matches_its_record`'s
    /// own doc has the full width-vs-colour distinction).
    ///
    #[test]
    fn idle_widgets_have_no_rest_outline() {
        let theme = default_theme(Appearance::Dark);
        let style = super::style(&theme);
        assert_eq!(style.visuals.widgets.inactive.bg_stroke.width, 0.0);
        assert_eq!(
            style.visuals.widgets.active.bg_stroke.color,
            theme.selection_border
        );
        assert_eq!(style.visuals.selection.stroke.color, theme.selection_border);
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
        let theme = default_theme(Appearance::Dark);
        let function = painted(SourcePaint::Function, false, &theme);

        assert_eq!(function.background, Some(theme.source_function_background));
    }

    ///
    /// An Operand Cell is tinted with its declared Token's colour: Number or
    /// Note. `Token::Char` is not one of them — [`SourcePaint::Operand`]
    /// carries only a declared operand Token, which is why
    /// `source_paint_visuals`'s `Char` arm is unreachable. The tint is
    /// indifferent to the binding state by design (`operand_paint`: a
    /// Pending, Valid or Invalid operand tints alike).
    /// `paint::tests::an_operand_cell_of_every_token_a_source_can_claim_is_
    /// tinted_with_its_own_colour` is the same rule from written Source.
    ///
    #[test]
    fn an_operand_cell_of_each_declared_token_is_tinted_with_its_own_colour() {
        let theme = default_theme(Appearance::Dark);

        for (paint, background) in [
            (
                operand(Token::Number, OperandState::Valid),
                theme.source_number_background,
            ),
            (
                operand(Token::Note, OperandState::Valid),
                theme.source_note_background,
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
    /// its own. `.scratch/theming/schema.md`'s role backgrounds are stored
    /// values rather than a tint strength, so a role that tints nothing is
    /// one whose background Theme value is transparent.
    ///
    /// The Function arm is [`SourcePaint::Function`] and stays that: text
    /// that spells no Function is Unclaimed and answers `None` whatever the
    /// role's background, so it would pass this test for a reason that has
    /// nothing to do with the Theme value under test.
    ///
    #[test]
    fn a_transparent_role_background_paints_no_tint() {
        let transparent = Theme {
            source_function_background: Color32::TRANSPARENT,
            source_number_background: Color32::TRANSPARENT,
            source_note_background: Color32::TRANSPARENT,
            ..default_theme(Appearance::Dark)
        };

        for paint in [
            SourcePaint::Function,
            operand(Token::Number, OperandState::Valid),
            operand(Token::Note, OperandState::Valid),
        ] {
            let visuals = painted(paint, false, &transparent);
            assert_eq!(
                visuals.background, None,
                "{paint:?} was tinted despite a transparent role background"
            );
        }
    }

    ///
    /// Comment, Bang and an empty unclaimed Cell are not tinted while their
    /// three role backgrounds are transparent, which the dark default's Bang
    /// is not, so the test clears all three. A Leftover Char is not a fourth
    /// case here: it is
    /// Unclaimed, so it is exactly the [`SourcePaint::Unclaimed`] case below,
    /// not a fact of its own.
    ///
    #[test]
    fn comment_bang_and_empty_unclaimed_cells_are_not_tinted() {
        let theme = Theme {
            source_comment_background: Color32::TRANSPARENT,
            source_bang_background: Color32::TRANSPARENT,
            source_ordinary_background: Color32::TRANSPARENT,
            ..default_theme(Appearance::Dark)
        };

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
        let theme = default_theme(Appearance::Dark);
        let cursor_colour = Color32::from_rgb(9, 8, 7);
        let tinted_function = theme.source_function_background;
        let unselected = cell_visuals(SourcePaint::Function, PortalRoles::NONE, true, &theme);
        let selected = coloured(
            SourcePaint::Function,
            PortalRoles::NONE,
            CursorPlacement::Cursor,
            Some(cursor_colour),
            &theme,
        );
        let cursor = coloured(
            SourcePaint::Function,
            PortalRoles::NONE,
            CursorPlacement::Cursor,
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
    /// is why the tint stays. This is
    /// [`OperandState::Invalid`], which the slot's shared Claim decides for
    /// the whole slot: at least one Cell of it holds content that failed to
    /// bind; `operand_paint` states the partly written case. A Valid Number
    /// is unaffected either way.
    ///
    #[test]
    fn an_invalid_operand_draws_diagnostic_but_keeps_its_declared_tint() {
        let theme = default_theme(Appearance::Dark);

        let invalid = painted(operand(Token::Number, OperandState::Invalid), false, &theme);
        let valid = painted(operand(Token::Number, OperandState::Valid), false, &theme);

        assert_eq!(invalid.foreground, theme.diagnostic_foreground);
        assert_eq!(
            invalid.background,
            Some(theme.source_number_background),
            "an Invalid Number Cell still tints as Number: a transparent \
             diagnostic.background reveals the Number role background unchanged"
        );
        assert_eq!(valid.foreground, theme.source_number);
        assert_eq!(valid.background, invalid.background);
    }

    ///
    /// A transparent `diagnostic.foreground` — unlike the dark default's
    /// opaque one — reveals the declared role's own foreground instead of replacing it:
    /// `.scratch/theming/schema.md`: "A transparent fact channel reveals the
    /// underlying Token channel with alpha compositing."
    ///
    #[test]
    fn a_transparent_diagnostic_foreground_reveals_the_declared_role() {
        let theme = Theme {
            diagnostic_foreground: Color32::TRANSPARENT,
            ..default_theme(Appearance::Dark)
        };

        let invalid = painted(operand(Token::Number, OperandState::Invalid), false, &theme);

        assert_eq!(invalid.foreground, theme.source_number);
    }

    ///
    /// A partial-alpha `diagnostic.background` blends over the declared
    /// role's own background rather than fully replacing or fully revealing
    /// it — the middle case between a transparent one (reveals the role
    /// unchanged, pinned above) and an opaque one (would replace it
    /// outright, the same way `diagnostic.foreground` does when opaque).
    ///
    #[test]
    fn a_partial_alpha_diagnostic_background_blends_over_the_role_background() {
        let theme = default_theme(Appearance::Dark);
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
    /// failed to bind yet. The tint is unaffected either way: a Pending,
    /// Valid, or Invalid operand is tinted alike.
    ///
    #[test]
    fn a_pending_operand_keeps_its_declared_colour_rather_than_diagnostic() {
        let theme = default_theme(Appearance::Dark);

        let pending = painted(operand(Token::Number, OperandState::Pending), false, &theme);

        assert_eq!(pending.foreground, theme.source_number);
        assert_eq!(pending.background, Some(theme.source_number_background));
    }

    ///
    /// `output_portal` overrides an Unclaimed Cell and a Valid Operand
    /// alike: each draws the Output Portal colour on the Output Portal's own
    /// Fill tint instead of whatever its own fact would answer. A written
    /// answer is not a third arm —
    /// `RenderCell::source_paint` answers the refused Function spelling it
    /// parses as with Unclaimed (pinned from written Source by
    /// `orcvs::render_frame`'s
    /// `a_written_07_is_two_one_cell_unbound_function_claims` and
    /// `every_cell_of_a_claim_reads_the_written_answer_its_claim_was_built_with`),
    /// so it is the Unclaimed arm below.
    ///
    #[test]
    fn output_portal_paints_over_an_unclaimed_and_a_bound_operand_cell() {
        let theme = crate::theme_registry::tests_support::filled_output_portal_theme();

        // The Output Portal channel composites *over* each fact's own raw
        // role background, not in place of it (`role_and_portal`'s
        // `background.blend(theme.output_portal_background)`) — Unclaimed's
        // own background is fully transparent, so the composite collapses
        // to the Output Portal fill alone; Number's is not, so its own fill
        // shows through under the translucent Output Portal fill.
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
    /// the plain Number channel — with `diagnostic.foreground` opaque in the
    /// dark default, the Portal colour still wins outright for the
    /// foreground, exactly as it does for a Valid operand. The background is
    /// Number's own tint (Diagnostic's own background is transparent, so it
    /// leaves Number's tint unchanged) composited with the Output Portal's
    /// tint on top.
    ///
    #[test]
    fn invalid_operand_under_output_portal_takes_the_portal_colour() {
        let theme = default_theme(Appearance::Dark);

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
    /// A Cell that is a bound Function's own two-Cell spelling keeps its
    /// Function paint outright when `output_portal` is also true, root or
    /// nested; `source_paint_visuals` states why.
    ///
    #[test]
    fn a_bound_function_spelling_wins_over_output_portal() {
        let theme = default_theme(Appearance::Dark);

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
    /// An Invalid Number operand's border composites `theme.diagnostic_border` over
    /// `theme.grid_border` — the same fact this Cell's foreground and
    /// background already use
    /// (`an_invalid_operand_draws_diagnostic_but_keeps_its_declared_tint`) —
    /// at `theme.diagnostic_border_width`, not `theme.grid_border_width`. An
    /// unaffected Cell (no Diagnostic fact) has no border of its own.
    ///
    #[test]
    fn a_diagnostic_border_with_a_non_transparent_colour_shows() {
        let visible = Color32::from_rgb(200, 30, 30);
        let theme = Theme {
            diagnostic_border: visible,
            ..transparent_borders()
        };

        let invalid = painted(operand(Token::Number, OperandState::Invalid), false, &theme);
        let valid = painted(operand(Token::Number, OperandState::Valid), false, &theme);

        assert_eq!(
            invalid.border.map(|border| border.stroke),
            Some(Stroke::new(
                theme.diagnostic_border_width.points(),
                theme.grid_border.blend(visible)
            ))
        );
        assert_eq!(valid.border, None);
    }

    ///
    /// A Function's own border sits beneath the Locked border: a locked
    /// Function shows `locked.border` over its Function border, at
    /// `locked.border.width`.
    ///
    #[test]
    fn the_locked_border_composes_over_a_functions_own_border() {
        let function_border = Color32::from_rgba_unmultiplied(30, 200, 30, 128);
        let locked_border = Color32::from_rgba_unmultiplied(200, 200, 30, 64);
        let theme = Theme {
            source_function_border: function_border,
            source_function_border_width: crate::theme::GridWidth::from_points(1.0)
                .expect("within 0..=1"),
            locked_border,
            ..default_theme(Appearance::Dark)
        };

        let locked = painted_in(
            SourcePaint::Function,
            PortalRoles::NONE.with(PortalRole::Locked),
            &theme,
        );

        assert_eq!(
            locked.border.map(|border| border.stroke),
            Some(Stroke::new(
                theme.locked_border_width.points(),
                theme
                    .grid_border
                    .blend(function_border)
                    .blend(locked_border)
            ))
        );
    }

    ///
    /// A role border frames every Cell its role claims, blank or not: an
    /// empty Function Cell and a Pending operand slot take their role's
    /// border. An empty unclaimed Cell keeps the plain Grid line even with a
    /// visible Ordinary border, which frames only an unclaimed Cell holding
    /// content.
    ///
    #[test]
    fn a_role_border_frames_every_claimed_cell_and_the_ordinary_border_only_content() {
        let function_border = Color32::from_rgb(30, 200, 30);
        let number_border = Color32::from_rgb(30, 30, 200);
        let ordinary_border = Color32::from_rgb(200, 30, 30);
        let theme = Theme {
            source_function_border: function_border,
            source_number_border: number_border,
            source_ordinary_border: ordinary_border,
            ..default_theme(Appearance::Dark)
        };
        let empty = |paint| cell_visuals(paint, PortalRoles::NONE, false, &theme);

        let function = empty(SourcePaint::Function)
            .border
            .map(|border| border.stroke);
        assert_eq!(
            function,
            Some(Stroke::new(
                theme.source_function_border_width.points(),
                theme.grid_border.blend(function_border)
            ))
        );

        let pending = empty(operand(Token::Number, OperandState::Pending))
            .border
            .map(|border| border.stroke);
        assert_eq!(
            pending,
            Some(Stroke::new(
                theme.source_number_border_width.points(),
                theme.grid_border.blend(number_border)
            ))
        );

        assert_eq!(empty(SourcePaint::Unclaimed).border, None);

        let written = painted(SourcePaint::Unclaimed, false, &theme)
            .border
            .map(|border| border.stroke);
        assert_eq!(
            written,
            Some(Stroke::new(
                theme.source_ordinary_border_width.points(),
                theme.grid_border.blend(ordinary_border)
            ))
        );
    }

    ///
    /// Each Source role strokes its own border and no other role's: a Bang
    /// takes the Bang border rather than the Function one, a Valid or Invalid
    /// operand takes its declared Token's border, and an Invalid operand
    /// shows the Diagnostic border over its Token's. A Pending operand slot
    /// is blank, so
    /// [`a_role_border_frames_every_claimed_cell_and_the_ordinary_border_only_content`]
    /// covers it.
    ///
    #[test]
    fn each_role_strokes_its_own_border() {
        let width = |points| crate::theme::GridWidth::from_points(points).expect("within 0..=1");
        let border = |shade| Color32::from_rgb(shade, 255 - shade, 128);
        let diagnostic_border = Color32::from_rgba_unmultiplied(200, 30, 30, 96);
        let theme = Theme {
            source_ordinary_border: border(10),
            source_ordinary_border_width: width(0.1),
            source_comment_border: border(20),
            source_comment_border_width: width(0.2),
            source_number_border: border(30),
            source_number_border_width: width(0.3),
            source_note_border: border(40),
            source_note_border_width: width(0.4),
            source_function_border: border(50),
            source_function_border_width: width(0.5),
            source_bang_border: border(60),
            source_bang_border_width: width(0.6),
            diagnostic_border,
            ..transparent_borders()
        };
        let role = |shade| {
            (
                theme.grid_border.blend(border(shade)),
                f32::from(shade) / 100.0,
            )
        };

        for (fact, (expected, expected_width)) in [
            (SourcePaint::Unclaimed, role(10)),
            (SourcePaint::Comment, role(20)),
            (operand(Token::Number, OperandState::Valid), role(30)),
            (operand(Token::Note, OperandState::Valid), role(40)),
            (SourcePaint::Function, role(50)),
            (SourcePaint::Bang, role(60)),
            (
                operand(Token::Number, OperandState::Invalid),
                (
                    role(30).0.blend(diagnostic_border),
                    theme.diagnostic_border_width.points(),
                ),
            ),
            (
                operand(Token::Note, OperandState::Invalid),
                (
                    role(40).0.blend(diagnostic_border),
                    theme.diagnostic_border_width.points(),
                ),
            ),
        ] {
            let stroke = painted(fact, false, &theme)
                .border
                .expect("every role draws its border")
                .stroke;
            assert_eq!(stroke.color, expected, "{fact:?}");
            assert!(
                (stroke.width - expected_width).abs() < 1e-6,
                "{fact:?}: {} is not {expected_width}",
                stroke.width
            );
        }
    }

    ///
    /// Portal precedence over Diagnostic reaches the border channel too: an
    /// Invalid operand inside an Output Portal takes the Output Portal's own border
    /// colour and width outright, the same as it already does for foreground
    /// and background
    /// (`invalid_operand_under_output_portal_takes_the_portal_colour`).
    ///
    #[test]
    fn output_portal_border_takes_precedence_over_diagnostic_border() {
        let theme = Theme {
            diagnostic_border: Color32::from_rgb(200, 30, 30),
            output_portal_border: Color32::from_rgb(30, 30, 200),
            ..transparent_borders()
        };

        let invalid_in_portal =
            painted(operand(Token::Number, OperandState::Invalid), true, &theme);

        assert_eq!(
            invalid_in_portal.border.map(|border| border.stroke),
            Some(Stroke::new(
                theme.output_portal_border_width.points(),
                theme.output_portal_border
            ))
        );
    }

    ///
    /// Transparent `diagnostic.border`/`output_portal.border` colours leave
    /// the ordinary Grid border visible unchanged: blending a fully
    /// transparent colour on top is the identity, so an Invalid operand in a
    /// Portal still shows only `theme.grid_border` although it runs through
    /// `ordinary_border`.
    ///
    #[test]
    fn transparent_diagnostic_and_portal_borders_leave_the_grid_border_visible() {
        let theme = transparent_borders();

        let invalid_in_portal =
            painted(operand(Token::Number, OperandState::Invalid), true, &theme);

        assert_eq!(
            invalid_in_portal.border.map(|border| border.stroke),
            Some(Stroke::new(
                theme.output_portal_border_width.points(),
                theme.grid_border
            ))
        );
    }

    ///
    /// A width of zero withdraws a border fact rather than hiding the edge:
    /// a zero-width Diagnostic leaves the Cell with no fact, so its edges show
    /// the Grid line, and a zero-width Output Portal over an Invalid operand
    /// leaves the Diagnostic border, composited over the Grid line alone.
    ///
    #[test]
    fn a_zero_width_border_falls_to_the_fact_below_it() {
        let diagnostic = Color32::from_rgb(200, 30, 30);
        let zero = crate::theme::GridWidth::from_points(0.0).expect("0.0 is within 0..=1");
        let invalid = operand(Token::Number, OperandState::Invalid);

        let no_diagnostic = Theme {
            diagnostic_border: diagnostic,
            diagnostic_border_width: zero,
            ..transparent_borders()
        };
        assert_eq!(painted(invalid, false, &no_diagnostic).border, None);

        let no_portal = Theme {
            diagnostic_border: diagnostic,
            output_portal_border: Color32::from_rgb(30, 30, 200),
            output_portal_border_width: zero,
            ..transparent_borders()
        };
        assert_eq!(
            painted(invalid, true, &no_portal).border,
            Some(EdgeLine {
                rank: EdgeRank::Diagnostic,
                stroke: Stroke::new(
                    no_portal.diagnostic_border_width.points(),
                    no_portal.grid_border.blend(diagnostic)
                ),
                seamed: false,
            })
        );
        assert_eq!(
            painted(SourcePaint::Unclaimed, true, &no_portal).border,
            None
        );
    }

    ///
    /// Each Portal role's border withdraws at zero width, so a Cell playing
    /// only that role carries no border fact.
    ///
    #[test]
    fn every_portal_border_withdraws_at_zero_width() {
        let zero = crate::theme::GridWidth::from_points(0.0).expect("0.0 is within 0..=1");
        let theme = Theme {
            output_portal_border_width: zero,
            input_portal_border_width: zero,
            locked_border_width: zero,
            ..default_theme(Appearance::Dark)
        };
        for roles in [OUTPUT, INPUT, LOCKED] {
            assert_eq!(
                painted_in(SourcePaint::Unclaimed, roles, &theme).border,
                None,
                "{roles:?}"
            );
        }
    }

    ///
    /// With no Grid line drawn, a border composites over nothing: a
    /// translucent Diagnostic border shows its own colour rather than one
    /// blended over a `grid.border` colour that is never on screen.
    ///
    #[test]
    fn a_border_over_no_grid_line_shows_its_own_colour() {
        let diagnostic = Color32::from_rgba_unmultiplied(200, 30, 30, 96);
        let theme = Theme {
            grid_border_width: crate::theme::GridWidth::from_points(0.0)
                .expect("0.0 is within 0..=1"),
            grid_border: Color32::from_rgb(0, 200, 0),
            diagnostic_border: diagnostic,
            ..transparent_borders()
        };
        assert_eq!(
            painted(operand(Token::Number, OperandState::Invalid), false, &theme).border,
            Some(EdgeLine {
                rank: EdgeRank::Diagnostic,
                stroke: Stroke::new(theme.diagnostic_border_width.points(), diagnostic),
                seamed: false,
            })
        );
    }

    ///
    /// The Cursor's own fill still wins outright over Output Portal, the
    /// same precedence it already holds over a role's own channel
    /// (`the_cursors_own_fill_wins_over_the_tint_on_its_cell`).
    ///
    #[test]
    fn the_cursors_own_fill_wins_over_output_portal_too() {
        let theme = default_theme(Appearance::Dark);
        let cursor_colour = Color32::from_rgb(9, 8, 7);

        let cursor = coloured(
            SourcePaint::Unclaimed,
            OUTPUT,
            CursorPlacement::Cursor,
            Some(cursor_colour),
            &theme,
        );

        assert_eq!(cursor.background, Some(cursor_colour));
    }

    ///
    /// The Cursor's Cell draws its glyph in the Cursor's glyph colour over
    /// every fact and every Portal role set, Locked included, which Source
    /// cannot always place under the Cursor: `cursor.foreground` on the
    /// single-Cell Cursor, and the Region's Cursor's fallback while a Region
    /// spans. Off the Cursor, and with both cleared, the glyph is the fact's.
    ///
    #[test]
    fn the_cursor_glyph_replaces_every_fact_and_portal_on_the_cursors_cell() {
        let glyph = Color32::from_rgb(1, 200, 3);
        let region_glyph = Color32::from_rgb(3, 1, 200);
        let theme = Theme {
            cursor_foreground: Some(glyph),
            region_cursor_foreground: Some(region_glyph),
            ..default_theme(Appearance::Dark)
        };
        let cleared = Theme {
            cursor_foreground: None,
            region_cursor_foreground: None,
            ..default_theme(Appearance::Dark)
        };
        let mut colours = CellColours::new(&theme);
        let mut cleared_colours = CellColours::new(&cleared);
        for paint in SOURCE_PAINT_FACTS {
            for portals in PORTAL_ROLE_SETS {
                let role = cell_visuals(paint, portals, true, &theme).foreground;
                let context = format!("{paint:?} under {portals:?}");
                for (placement, expected, expected_cleared) in [
                    (CursorPlacement::Cursor, glyph, role),
                    (CursorPlacement::RegionCursor, region_glyph, role),
                    (CursorPlacement::Plain, role, role),
                    (CursorPlacement::Region, role, role),
                ] {
                    assert_eq!(
                        colours.colours(paint, portals, true, placement).foreground,
                        expected,
                        "{context} at {placement:?}"
                    );
                    assert_eq!(
                        cleared_colours
                            .colours(paint, portals, true, placement)
                            .foreground,
                        expected_cleared,
                        "{context} at {placement:?}, cleared"
                    );
                }
            }
        }
    }

    // === Input Portal, and Output Portal over it ===

    ///
    /// Input Portal takes precedence over every fact but a Function and a Bang,
    /// Diagnostic included: each draws the Input Portal foreground over its
    /// own, on its own raw role background with the Input Portal background
    /// composited on top — the same rule Output Portal follows
    /// (`output_portal_paints_over_an_unclaimed_and_a_bound_operand_cell`,
    /// `invalid_operand_under_output_portal_takes_the_portal_colour`).
    ///
    #[test]
    fn input_portal_paints_over_every_non_function_fact_including_diagnostic() {
        for theme in [
            default_theme(Appearance::Dark),
            default_theme(Appearance::Light),
        ] {
            for (paint, role_background) in [
                (SourcePaint::Unclaimed, theme.source_ordinary_background),
                (SourcePaint::Comment, theme.source_comment_background),
                (
                    operand(Token::Number, OperandState::Valid),
                    theme.source_number_background,
                ),
                (
                    operand(Token::Number, OperandState::Invalid),
                    theme.source_number_background,
                ),
                (
                    operand(Token::Note, OperandState::Valid),
                    theme.source_note_background,
                ),
                (
                    operand(Token::Note, OperandState::Invalid),
                    theme.source_note_background,
                ),
            ] {
                let visuals = painted_in(paint, INPUT, &theme);
                assert_eq!(
                    visuals.foreground, theme.input_portal_foreground,
                    "{}: {paint:?} did not draw the Input Portal colour",
                    theme.identity
                );
                assert_eq!(
                    visuals.background,
                    Some(
                        theme
                            .cell_background
                            .blend(role_background.blend(theme.input_portal_background))
                    ),
                    "{}: {paint:?} did not carry its role background under the Input Portal tint",
                    theme.identity
                );
            }
        }
    }

    ///
    /// A Bang keeps all its own paint whichever Portal it stands in, borders
    /// included, as a bound Function does.
    ///
    #[test]
    fn a_bang_keeps_its_own_paint_inside_every_portal_role() {
        let theme = Theme {
            output_portal_border: Color32::from_rgb(30, 30, 200),
            input_portal_border: Color32::from_rgb(200, 30, 200),
            ..default_theme(Appearance::Dark)
        };
        let ordinary = painted_in(SourcePaint::Bang, PortalRoles::NONE, &theme);

        for roles in [INPUT, OUTPUT, BOTH] {
            assert_eq!(
                painted_in(SourcePaint::Bang, roles, &theme),
                ordinary,
                "{roles:?} changed a Bang's own paint"
            );
        }
    }

    ///
    /// A bound Function keeps all its own paint whichever Portal it reads or
    /// writes through, borders included.
    ///
    #[test]
    fn a_bound_function_spelling_wins_over_every_portal_role() {
        let theme = Theme {
            output_portal_border: Color32::from_rgb(30, 30, 200),
            input_portal_border: Color32::from_rgb(200, 30, 200),
            ..default_theme(Appearance::Dark)
        };
        let ordinary = painted_in(SourcePaint::Function, PortalRoles::NONE, &theme);

        for roles in [INPUT, OUTPUT, BOTH] {
            assert_eq!(
                painted_in(SourcePaint::Function, roles, &theme),
                ordinary,
                "{roles:?} changed a Function's own paint"
            );
        }
    }

    ///
    /// An Invalid operand inside an Input Portal takes the Input Portal's own
    /// border colour and width over the Diagnostic border, as foreground and
    /// background do.
    ///
    #[test]
    fn input_portal_border_takes_precedence_over_diagnostic_border() {
        let theme = Theme {
            diagnostic_border: Color32::from_rgb(200, 30, 30),
            input_portal_border: Color32::from_rgb(200, 30, 200),
            input_portal_border_width: crate::theme::GridWidth::from_points(0.25)
                .expect("0.25 is within 0..=1"),
            ..default_theme(Appearance::Dark)
        };

        let invalid = painted_in(operand(Token::Number, OperandState::Invalid), INPUT, &theme);

        assert_eq!(
            invalid.border.map(|border| border.stroke),
            Some(Stroke::new(0.25, theme.input_portal_border))
        );
    }

    ///
    /// A Cell that plays both Portal roles paints exactly as an Output Portal
    /// Cell, for every fact: Output Portal's channels win outright over Input
    /// Portal's, foreground, background and border alike, rather than
    /// compositing over them. The fixture gives each Portal a distinct,
    /// translucent value on every channel, so a composite would show.
    ///
    #[test]
    fn output_portal_wins_over_input_portal_on_every_channel() {
        let theme = Theme {
            output_portal_foreground: Color32::from_rgba_unmultiplied(230, 160, 0, 200),
            output_portal_background: Color32::from_rgba_unmultiplied(230, 160, 0, 60),
            output_portal_border: Color32::from_rgba_unmultiplied(230, 160, 0, 120),
            output_portal_border_width: crate::theme::GridWidth::from_points(0.75)
                .expect("0.75 is within 0..=1"),
            input_portal_foreground: Color32::from_rgba_unmultiplied(170, 120, 250, 200),
            input_portal_background: Color32::from_rgba_unmultiplied(170, 120, 250, 60),
            input_portal_border: Color32::from_rgba_unmultiplied(170, 120, 250, 120),
            input_portal_border_width: crate::theme::GridWidth::from_points(0.25)
                .expect("0.25 is within 0..=1"),
            ..default_theme(Appearance::Dark)
        };

        for fact in SOURCE_PAINT_FACTS {
            assert_eq!(
                painted_in(fact, BOTH, &theme),
                painted_in(fact, OUTPUT, &theme),
                "{fact:?}"
            );
        }
        let both = painted_in(SourcePaint::Unclaimed, BOTH, &theme);
        let input = painted_in(SourcePaint::Unclaimed, INPUT, &theme);
        assert_ne!(both.foreground, input.foreground);
        assert_ne!(both.background, input.background);
        assert_ne!(both.border, input.border);
        assert_eq!(both.border.map(|border| border.stroke.width), Some(0.75));
    }

    // === Locked ===

    ///
    /// A lock marks a Function's own spelling, so Locked paints a bound
    /// Function and a Bang in its own channels on every channel, composited
    /// over the fact's own paint. The fixture gives Locked a translucent
    /// value on every channel, so a bypass or an outright replacement would
    /// show.
    ///
    #[test]
    fn locked_paints_a_function_and_a_bang_over_their_own_paint() {
        let theme = Theme {
            locked_foreground: Color32::from_rgba_unmultiplied(0, 0, 0, 100),
            locked_background: Color32::from_rgba_unmultiplied(120, 120, 120, 50),
            locked_border: Color32::from_rgba_unmultiplied(120, 120, 120, 90),
            locked_border_width: crate::theme::GridWidth::from_points(0.25)
                .expect("0.25 is within 0..=1"),
            ..transparent_borders()
        };

        for (paint, foreground, background) in [
            (
                SourcePaint::Function,
                theme.source_function,
                theme.source_function_background,
            ),
            (
                SourcePaint::Bang,
                theme.source_bang,
                theme.source_bang_background,
            ),
        ] {
            let locked = painted_in(paint, LOCKED, &theme);
            assert_eq!(
                locked.foreground,
                foreground.blend(theme.locked_foreground),
                "{paint:?}"
            );
            assert_eq!(
                locked.background,
                Some(
                    theme
                        .cell_background
                        .blend(background.blend(theme.locked_background))
                ),
                "{paint:?}"
            );
            assert_eq!(
                locked.border.map(|border| border.stroke),
                Some(Stroke::new(
                    0.25,
                    theme.grid_border.blend(theme.locked_border)
                )),
                "{paint:?}"
            );
        }
    }

    ///
    /// Every built-in draws a locked Function and a locked Bang on the Source
    /// background, washing out a Function's own tint, frame each Cell, and
    /// keep the glyph colour, so a locked root still reads as what it is.
    ///
    #[test]
    fn the_built_ins_wash_out_and_frame_a_locked_function_and_keep_its_glyph() {
        let registry = crate::theme_registry::ThemeRegistry::built_in();
        for theme in crate::theme_registry::tests_support::built_ins(&registry)
            .iter()
            .cloned()
        {
            for paint in [SourcePaint::Function, SourcePaint::Bang] {
                let plain = painted_in(paint, PortalRoles::NONE, &theme);
                let locked = painted_in(paint, LOCKED, &theme);
                assert_eq!(locked.foreground, plain.foreground, "{}", theme.identity);
                assert_ne!(locked.border, plain.border, "{}", theme.identity);
            }
            let plain = painted_in(SourcePaint::Function, PortalRoles::NONE, &theme);
            let locked = painted_in(SourcePaint::Function, LOCKED, &theme);
            assert_eq!(
                locked.background,
                Some(
                    theme.cell_background.blend(
                        theme
                            .source_function_background
                            .blend(theme.locked_background)
                    )
                ),
                "{}",
                theme.identity
            );
            assert_ne!(locked.background, plain.background, "{}", theme.identity);
        }
    }

    ///
    /// A Cell a lock marks that a Function also reads or writes paints as
    /// that read or write alone, for every fact, as Orca draws a port over a
    /// locked Cell. A locked Function's own spelling therefore keeps its own
    /// paint there, as inside any Portal, and shows no lock.
    ///
    #[test]
    fn a_read_or_a_write_paints_a_locked_cell_as_an_unlocked_one() {
        let theme = Theme {
            locked_foreground: Color32::from_rgba_unmultiplied(0, 0, 0, 100),
            locked_background: Color32::from_rgba_unmultiplied(120, 120, 120, 50),
            locked_border: Color32::from_rgba_unmultiplied(120, 120, 120, 90),
            ..default_theme(Appearance::Dark)
        };

        for fact in SOURCE_PAINT_FACTS {
            for roles in [INPUT, OUTPUT, BOTH] {
                assert_eq!(
                    painted_in(fact, roles.with(PortalRole::Locked), &theme),
                    painted_in(fact, roles, &theme),
                    "{fact:?}, {roles:?}"
                );
            }
        }
        let function = painted_in(SourcePaint::Function, PortalRoles::NONE, &theme);
        for roles in [INPUT, OUTPUT, BOTH] {
            assert_eq!(
                painted_in(
                    SourcePaint::Function,
                    roles.with(PortalRole::Locked),
                    &theme
                ),
                function,
                "{roles:?}"
            );
        }
    }

    ///
    /// Cursor and Region keep their precedence over Input Portal: the
    /// single-Cell Cursor's own fill wins on its Cell, which keeps the
    /// Portal's border for its outline to outrank, and the
    /// Cursor's Cell inside a spanning Region takes the Region Cursor fill.
    /// Every other Region Cell keeps the Input Portal background, because a
    /// visible fact fill suppresses the Region fill.
    ///
    #[test]
    fn cursor_and_region_keep_their_precedence_over_input_portal() {
        let theme = Theme {
            region_cursor_background: Some(Color32::from_rgb(4, 5, 6)),
            ..default_theme(Appearance::Dark)
        };
        let cursor_colour = Color32::from_rgb(9, 8, 7);

        let cursor = coloured(
            SourcePaint::Unclaimed,
            INPUT,
            CursorPlacement::Cursor,
            Some(cursor_colour),
            &theme,
        );
        assert_eq!(cursor.background, Some(cursor_colour));
        assert_eq!(
            cursor.border.map(|border| border.rank),
            Some(EdgeRank::InputPortal)
        );

        let region_cursor = coloured(
            SourcePaint::Unclaimed,
            INPUT,
            CursorPlacement::RegionCursor,
            Some(cursor_colour),
            &theme,
        );
        assert_eq!(region_cursor.background, Some(Color32::from_rgb(4, 5, 6)));

        let region_cell = coloured(
            SourcePaint::Unclaimed,
            INPUT,
            CursorPlacement::Region,
            Some(cursor_colour),
            &theme,
        )
        .background;
        assert_eq!(region_cell, Some(theme.input_portal_background));
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
        // The Cursor's glyph colour is cleared, so its Cell's glyph is the
        // one the operand draws.
        let theme = Theme {
            cursor_foreground: None,
            ..default_theme(Appearance::Dark)
        };
        let cursor_colour = Color32::from_rgb(9, 8, 7);

        let unselected = cell_visuals(
            operand(Token::Number, OperandState::Invalid),
            PortalRoles::NONE,
            true,
            &theme,
        );
        let cursor = coloured(
            operand(Token::Number, OperandState::Invalid),
            PortalRoles::NONE,
            CursorPlacement::Cursor,
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
    /// the uniform `cell.background`." This Theme gives Number a
    /// partial-alpha background to exercise a real blend.
    ///
    #[test]
    fn explicit_background_alpha_composites_over_the_cell_base() {
        let base = Color32::from_rgba_unmultiplied(10, 20, 30, 200);
        let role = Color32::from_rgba_unmultiplied(200, 100, 50, 128);
        let theme = Theme {
            cell_background: base,
            source_number_background: role,
            ..default_theme(Appearance::Dark)
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
    /// eframe restores `ThemePreference` into egui memory before it builds
    /// the application, and `install` is what `Console::new` calls
    /// afterwards. A preference already on the context — standing in for one
    /// eframe just restored — must survive the call.
    ///
    #[test]
    fn install_leaves_a_restored_theme_preference_untouched() {
        let ctx = egui::Context::default();
        ctx.set_theme(egui::ThemePreference::Light);

        install(
            &ctx,
            &default_theme(Appearance::Dark),
            &default_theme(Appearance::Light),
        );

        assert_eq!(
            ctx.options(|options| options.theme_preference),
            egui::ThemePreference::Light,
            "install overwrote the restored theme preference"
        );
    }

    ///
    /// `style()` of the selected dark Theme for `egui::Theme::Dark` and of
    /// the selected light Theme for `egui::Theme::Light`, each registered
    /// through `set_style_of`.
    ///
    /// The comparison is `Visuals` and `animation_time`, the two fields
    /// [`style`] actually sets, rather than `Style`'s own `PartialEq`:
    /// `Style::number_formatter` compares by `Arc::ptr_eq`
    /// (`NumberFormatter`'s `PartialEq`), so two independently built
    /// `Style::default()`s — one inside each `style()` call — never compare
    /// equal on that field alone, whatever their visible content.
    ///
    #[test]
    fn install_registers_each_appearances_own_theme() {
        let ctx = egui::Context::default();

        install(
            &ctx,
            &default_theme(Appearance::Dark),
            &default_theme(Appearance::Light),
        );

        for (slot, theme) in [
            (egui::Theme::Dark, default_theme(Appearance::Dark)),
            (egui::Theme::Light, default_theme(Appearance::Light)),
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
}
