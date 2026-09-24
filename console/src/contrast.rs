//!
//! Validates a resolved Theme's *composited* text contrast: the effective
//! foreground and background each reachable painted text state actually
//! displays, not raw Theme colour pairs read in isolation.
//! `.scratch/theming/issues/08`.
//!
//! [`painted`] calls [`crate::style::cell_visuals_with_cursor_colour`] and
//! [`crate::style::cell_background`] — the same functions
//! [`crate::paint::Paint::derive_with_theme`] calls per Cell — for every
//! Source Grid colour operation, so the two cannot independently drift.
//! [`chrome`] does the same for the console's chrome
//! (`.scratch/theming/issues/11`): it reads every colour out of
//! [`crate::style::style`], the `Style` [`crate::style::install`] registers,
//! through the egui accessors the widgets themselves paint from —
//! [`egui::Style::button_style`] among them — and composites each fill over
//! the surface it is painted on with [`egui::Color32::blend`], the same
//! primitive every reused function above is itself built from.
//!
//! # Scope
//!
//! [`validate`] measures text contrast only: an effective foreground against
//! the effective background it is actually painted on. It does not measure
//! pairwise Token-colour distinguishability or border/focus visibility.
//!
//! Its chrome states are `text`, `text.muted`, `text.active`, selected
//! text, `error`, `warning` and `link`, each on the surfaces egui paints it
//! on (see [`State`]). Five chrome cases are left out, each for a reason
//! [`ContrastReport::scope`] also carries:
//!
//! - A disabled widget: egui fades it to half opacity, and WCAG 2.1 SC 1.4.3
//!   exempts the text of an inactive user-interface component.
//! - `code.background`: nothing in the console paints a code span.
//! - A state's strong `bg_fill`: egui 0.36.2 paints no text on one — it is a
//!   checkbox's box, a slider's rail, a colour swatch — so only the weak fill
//!   every text-bearing widget uses is measured.
//! - A popup, tooltip or window floating over another surface — the Source
//!   Grid, or a panel: its fill is `panel.background` like a panel's, and it
//!   is measured as one panel over the window backdrop. What really lies
//!   beneath it differs from that only where `panel.background` is
//!   translucent, and which surface a floating area overlaps is decided by
//!   layout, not by the Theme.
//! - `text.active` on the open widget fill: egui paints an open widget's
//!   text in `widgets.open.fg_stroke`, which [`crate::style::style`] maps
//!   from `text`, so the open fill is measured with `text` — the pair
//!   actually painted.
//!
//! [`distinguish`] measures the second question, added by
//! `.scratch/theming/issues/04` after a review of the light Theme's glyph
//! colours went looking for a confusion [`validate`] structurally cannot
//! see: two channels that each clear the floor against their own background
//! and still read as one colour to a dichromat. It simulates dichromatic
//! vision over the same composited colours [`validate`] measures, and
//! reports how far apart each pair of Source glyph channels stays. The
//! rejected light definition failed it at 0.70 — Diagnostic against Output
//! Portal under protanopia — while passing every contrast state;
//! `the_rejected_light_glyph_definition_fails_this_gate` keeps that as a
//! regression.
//!
//! Its own boundaries are stated at [`CONFUSION_FLOOR`] and
//! [`ColourVision`]: the floor is gated for the two red–green dichromacies
//! and measured-but-ungated for tritanopia, and neither function measures
//! background tints against each other, only the glyphs painted on them.
//!
//! It does not account for the Cursor Effect's animated `area` field
//! (`theme.cursor_area`), which `console.rs`'s `SourceShapes::into_shapes`
//! draws beneath every Cell's own background. Where a Cell's own background
//! is fully transparent, the real console can show a translucent tint of
//! `cursor_area` at that Cell, up to `cursor_area`'s own configured alpha —
//! but which Cells, and how much, is decided by Glitch amount and Glitch
//! frequency (viewer settings, not Theme properties — ADR 0053: "A Theme
//! decides how things look, never how much it moves") and by continuous,
//! real-time animation. No single sample derived from the Theme alone would
//! represent that faithfully, so this validator measures the plain Grid
//! surface instead. `contrast::tests::painted_background_is_unaffected_by_
//! the_cursor_area_effect` pins this as a checked boundary rather than an
//! unnoticed gap.

use std::fmt;

use egui::widget_style::{Classes, HasClasses as _, SELECTED_CLASS, WidgetState};
use egui::{Color32, Style};
use orcvs::source::{OperandState, SourcePaint, Token};

use crate::style::{cell_background, cell_visuals_with_cursor_colour, compose_cell_fill, style};
use crate::theme::{OKABE_ITO_IDENTITY, ORCVS_LIGHT_IDENTITY, Theme, ThemeIdentity};

///
/// The WCAG 2.1 Success Criterion 1.4.3 ("Contrast (Minimum)") ratio every
/// [`ContrastResult`] is measured against: 4.5:1, the floor normal-size text
/// must clear. Stated once, here, with its source, and again in
/// [`ContrastReport::floor`] for a caller reading the report rather than
/// this source file.
///
pub(crate) const CONTRAST_FLOOR: f32 = 4.5;

///
/// A named text role [`validate`] measures, and the Source Paint fact it
/// reads from — `None` for the console-chrome roles, from `text` onward,
/// which are not Source Grid facts. Each chrome role is named for the Theme
/// key `crate::style::style` maps the painted foreground from.
///
/// Number and Note admit Valid and Invalid. Atom and Sequence admit only
/// Invalid: `Token::decode` refuses both outright (`lang/src/expression.rs`'s
/// own words on `Token`: "declarations no Cells spell"), so the only thing
/// that can satisfy either slot is a nested Function, which
/// `take_language_unit`'s `is_function_next()` branch records under
/// `Token::Function` instead — never under `Atom` or `Sequence`. Neither
/// Token ever reaches a *Valid* state through any Source, only Invalid
/// (written, unbound) or Pending (blank).
///
/// Pending is not a role this validator measures at all: a Pending Cell
/// draws no glyph (`paint::tests::every_pending_operand_token_draws_no_
/// glyph_through_the_real_paint_path` proves this through the real paint
/// path), so there is no foreground for a Pending state to measure. That
/// guard test's own failure message says to restore Pending roles here if
/// painting ever changes to draw one.
///
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Role {
    Ordinary,
    Function,
    Bang,
    Comment,
    NumberValid,
    NumberInvalid,
    NoteValid,
    NoteInvalid,
    AtomInvalid,
    SequenceInvalid,
    Text,
    TextMuted,
    TextActive,
    /// Selected text: egui recolours a selection's glyphs to
    /// `Visuals::selection.stroke`, which `style` maps from
    /// `selection.border`.
    SelectionBorder,
    Error,
    Warning,
    Link,
}

#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "SOURCE_ROLES is read by validate, but the glyph-channel mapping here only by \
                  distinguish, which no shipped path runs; theming/07 shows validate's report"
    )
)]
impl Role {
    /// Every Source Grid role [`validate`] measures, in this enum's
    /// declaration order.
    const SOURCE_ROLES: [Self; 10] = [
        Self::Ordinary,
        Self::Function,
        Self::Bang,
        Self::Comment,
        Self::NumberValid,
        Self::NumberInvalid,
        Self::NoteValid,
        Self::NoteInvalid,
        Self::AtomInvalid,
        Self::SequenceInvalid,
    ];

    ///
    /// Which Theme foreground channel this role's glyph actually draws in,
    /// given whether the Cell also lies in an Output Portal Reservation —
    /// the *meaning* a reader has to read off the colour, which is what
    /// [`distinguish`] compares rather than the `(role, state)` label.
    ///
    /// Several roles share one channel, by design rather than by accident:
    /// every Invalid operand draws `diagnostic.foreground` whatever Token
    /// declared the slot, and every non-Function, non-Bang Cell inside a
    /// Reservation draws `output_portal.foreground` whatever it would
    /// otherwise have been. Those collapses are the painting rules working;
    /// comparing two states that share a channel would measure a distinction
    /// the console never intended to draw.
    ///
    /// `source.sequence` is absent from [`GlyphChannel`] for a different
    /// reason: no reachable state paints a glyph in it. `Token::Sequence`
    /// never binds, so its only role here is `Sequence, Invalid`, which
    /// draws Diagnostic — Sequence's own colour reaches the screen as the
    /// `source.sequence.background` tint alone, which is a background and
    /// therefore outside this module's scope.
    ///
    /// A fact channel whose foreground is exactly transparent paints nothing
    /// over the glyph — `style::blend_channel` leaves the Token's colour
    /// showing, which is ADR 0053's way to carry a fact on the background
    /// alone — so the glyph is measured as the channel it reveals. An
    /// Invalid Sequence then reveals `source.sequence`, which is not a
    /// [`GlyphChannel`], and is left out. An Invalid Atom reveals
    /// `source.ordinary`, the colour `style::role` draws an Atom operand in.
    ///
    fn glyph_channel(self, output_portal: bool, theme: &Theme) -> Option<GlyphChannel> {
        let portal = output_portal && theme.output_portal_foreground != Color32::TRANSPARENT;
        let diagnostic = theme.diagnostic_foreground != Color32::TRANSPARENT;
        match self {
            // "A bound Function retains all its own paint inside a Portal",
            // and "Bang retains its glyph foreground" — both answered before
            // the Portal, exactly as `style::role_and_portal` bypasses them.
            Self::Function => Some(GlyphChannel::Function),
            Self::Bang => Some(GlyphChannel::Bang),
            Self::Text
            | Self::TextMuted
            | Self::TextActive
            | Self::SelectionBorder
            | Self::Error
            | Self::Warning
            | Self::Link => None,
            _ if portal => Some(GlyphChannel::OutputPortal),
            Self::NumberInvalid | Self::NoteInvalid | Self::AtomInvalid | Self::SequenceInvalid
                if diagnostic =>
            {
                Some(GlyphChannel::Diagnostic)
            }
            Self::Ordinary | Self::AtomInvalid => Some(GlyphChannel::Ordinary),
            Self::Comment => Some(GlyphChannel::Comment),
            Self::NumberValid | Self::NumberInvalid => Some(GlyphChannel::Number),
            Self::NoteValid | Self::NoteInvalid => Some(GlyphChannel::Note),
            Self::SequenceInvalid => None,
        }
    }

    /// The Source Paint fact this role reads, or `None` for a console-chrome
    /// role.
    fn source_paint_fact(self) -> Option<SourcePaint> {
        match self {
            Self::Ordinary => Some(SourcePaint::Unclaimed),
            Self::Function => Some(SourcePaint::Function),
            Self::Bang => Some(SourcePaint::Bang),
            Self::Comment => Some(SourcePaint::Comment),
            Self::NumberValid => Some(operand(Token::Number, OperandState::Valid)),
            Self::NumberInvalid => Some(operand(Token::Number, OperandState::Invalid)),
            Self::NoteValid => Some(operand(Token::Note, OperandState::Valid)),
            Self::NoteInvalid => Some(operand(Token::Note, OperandState::Invalid)),
            Self::AtomInvalid => Some(operand(Token::Atom, OperandState::Invalid)),
            Self::SequenceInvalid => Some(operand(Token::Sequence, OperandState::Invalid)),
            Self::Text
            | Self::TextMuted
            | Self::TextActive
            | Self::SelectionBorder
            | Self::Error
            | Self::Warning
            | Self::Link => None,
        }
    }
}

const fn operand(token: Token, state: OperandState) -> SourcePaint {
    SourcePaint::Operand { token, state }
}

impl fmt::Display for Role {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Ordinary => "Ordinary",
            Self::Function => "Function",
            Self::Bang => "Bang",
            Self::Comment => "Comment",
            Self::NumberValid => "Number, Valid",
            Self::NumberInvalid => "Number, Invalid",
            Self::NoteValid => "Note, Valid",
            Self::NoteInvalid => "Note, Invalid",
            Self::AtomInvalid => "Atom, Invalid",
            Self::SequenceInvalid => "Sequence, Invalid",
            Self::Text => "text",
            Self::TextMuted => "text.muted",
            Self::TextActive => "text.active",
            Self::SelectionBorder => "selection.border",
            Self::Error => "error",
            Self::Warning => "warning",
            Self::Link => "link",
        })
    }
}

///
/// Where a Source Grid role's Cell sits relative to the Cursor and a Region
/// spanning more than one Cell — the axis
/// [`crate::paint::Paint::derive_with_theme`] decides once per Cell,
/// independently of whether that Cell also lies in an Output Portal
/// Reservation (see [`State::SourceGrid`]).
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

impl CursorPlacement {
    const ALL: [Self; 4] = [Self::Plain, Self::Cursor, Self::Region, Self::RegionCursor];
}

impl fmt::Display for CursorPlacement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Plain => "plain",
            Self::Cursor => "Cursor",
            Self::Region => "Region",
            Self::RegionCursor => "Region, Cursor's Cell",
        })
    }
}

///
/// The reachable painted state a [`ContrastResult`] was measured in.
///
/// [`Self::SourceGrid`] crosses [`CursorPlacement`] with whether the Cell
/// also lies in a root Function's Output Portal Reservation: the two are
/// independent — the Cursor can sit on a Cell an Output Portal covers, and a
/// Region can span over one, per `.scratch/theming/schema.md`'s own Overlap
/// rule ("covers every Cell of the Reservation whatever else claims it") and
/// its Cursor-precedence exception ("The Cursor's own fill still wins
/// outright over everything above, on its own Cell").
///
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum State {
    SourceGrid {
        cursor: CursorPlacement,
        output_portal: bool,
    },
    /// Against `panel.background` over the window backdrop: a label and the
    /// `error`/`warning`/`link` foregrounds.
    Panel,
    /// Against `input.background` over the panel: a `TextEdit`'s text and
    /// hint.
    Input,
    /// An inactive button's own fill, over the panel.
    Inactive,
    /// A hovered button's own fill, over the panel.
    Hovered,
    /// An active (pressed or focused) button's own fill, over the panel.
    Active,
    /// A hovered widget egui paints without a fill of its own — a checkbox
    /// label — straight over the panel.
    HoveredFrameless,
    /// An active widget painted without a fill of its own, over the panel.
    ActiveFrameless,
    /// An open widget's fill — a sub-menu button whose menu is open, an
    /// open combo box, an active window's title bar — over the panel.
    Open,
    /// `selection.background` over the panel: selected label text, or a
    /// selected `selectable_value`.
    Selection,
    /// `selection.background` over the input: a `TextEdit` selection.
    InputSelection,
}

impl fmt::Display for State {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SourceGrid {
                cursor,
                output_portal: false,
            } => write!(f, "{cursor}"),
            Self::SourceGrid {
                cursor,
                output_portal: true,
            } => write!(f, "{cursor}, Output Portal"),
            Self::Panel => f.write_str("vs panel.background"),
            Self::Input => f.write_str("vs input.background"),
            Self::Inactive => f.write_str("vs the inactive widget fill"),
            Self::Hovered => f.write_str("vs the hovered widget fill"),
            Self::Active => f.write_str("vs the active widget fill"),
            Self::HoveredFrameless => f.write_str("hovered, frameless, vs panel.background"),
            Self::ActiveFrameless => f.write_str("active, frameless, vs panel.background"),
            Self::Open => f.write_str("vs the open widget fill"),
            Self::Selection => f.write_str("vs selection.background over panel.background"),
            Self::InputSelection => f.write_str("vs selection.background over input.background"),
        }
    }
}

///
/// The effective foreground and *opaque* background a `(fact, cursor,
/// output_portal)` triple actually paints, composited exactly as
/// [`crate::paint::Paint::derive_with_theme`] composes one Cell — the same
/// functions, standalone rather than read off one Cell of a whole Render
/// Frame.
///
/// The background is resolved all the way down to the opaque window
/// backdrop: `console.rs`'s own paint order draws `window.background` as the
/// `eframe` clear colour first, `grid.background` as the Source panel's fill
/// over it, then each Cell's own composited fill on top
/// (`console.rs::source_panel_frame`, `Console::clear_color`). A translucent
/// background therefore resolves against its underlying surface rather than
/// against itself.
///
fn painted(
    fact: SourcePaint,
    cursor: CursorPlacement,
    output_portal: bool,
    theme: &Theme,
) -> (Color32, Color32) {
    let is_cursor = matches!(
        cursor,
        CursorPlacement::Cursor | CursorPlacement::RegionCursor
    );
    let region_spans = matches!(
        cursor,
        CursorPlacement::Region | CursorPlacement::RegionCursor
    );
    let selected = is_cursor && !region_spans;
    let cursor_fill = theme.cursor_background;

    let visuals = cell_visuals_with_cursor_colour(
        fact,
        output_portal,
        selected,
        // `cursor_visible` decides only the Cell's border, never its
        // foreground or background (`cell_visuals_with_cursor_colour`'s own
        // body) — `selected` is a faithful value here rather than an unused
        // placeholder.
        selected,
        cursor_fill,
        theme,
    );

    let region_cursor_fill = theme.region_cursor_background.or(cursor_fill);
    let region_fill = compose_cell_fill(theme.cell_background, Some(theme.region_background));
    let base_fill = compose_cell_fill(theme.cell_background, None);
    let in_region = region_spans && !is_cursor;

    let painted_background = cell_background(
        visuals.background,
        is_cursor && region_spans,
        in_region,
        region_cursor_fill,
        region_fill,
        base_fill,
        theme.cell_background,
    );

    let grid_surface = theme.window_background.blend(theme.grid_background);
    let effective_background =
        grid_surface.blend(painted_background.unwrap_or(Color32::TRANSPARENT));

    (visuals.foreground, effective_background)
}

/// How many console-chrome states [`chrome`] measures.
const CHROME_STATES: usize = 15;

///
/// Every console-chrome text state, as the effective foreground and opaque
/// background egui paints it for `theme`.
///
/// Every colour is read out of [`crate::style::style`] — the `Style`
/// [`crate::style::install`] registers for both egui appearance slots — and
/// through the accessors egui's own widgets call when they paint it:
/// [`egui::Style::button_style`], the function `Button::ui` itself calls
/// for a button's fill and text colour (`egui-0.36.2/src/widgets/
/// button.rs:331`); [`egui::Style::widget_style`], which `Checkbox` paints
/// its label from; `Visuals::text_color`, `weak_text_color` and
/// `text_edit_bg_color`; and `Visuals::selection`, which
/// `text_selection::paint_text_selection` recolours selected glyphs and
/// fills their background from. A `style` mapping that changes which Theme
/// key a `Visuals` field reads therefore changes what this measures with
/// it, rather than leaving a second, hand-kept copy of the mapping here to
/// drift from the one painting uses.
///
/// Every text-bearing widget fill in egui 0.36.2 is the state's
/// `weak_bg_fill` — `button_style`'s frame, `ComboBox`, and the active
/// window's title bar alike; a state's strong `bg_fill` carries no text
/// (a checkbox's box, a slider's rail, a colour swatch), which is why only
/// the weak fill is measured. An open widget is measured the way
/// `SubMenuButton::ui` paints one (`egui-0.36.2/src/containers/menu.rs:383`):
/// `widgets.open` standing in for `widgets.inactive`.
///
/// Each fill is composited in paint order over the opaque window backdrop
/// `Console::clear_color` clears to — the panel over the backdrop, the
/// input or widget fill over the panel, the selection over whichever of the
/// two it highlights — each by [`Color32::blend`], the premultiplied
/// source-over egui's painter applies and [`painted`] composites the
/// Source Grid with.
///
fn chrome(theme: &Theme) -> [(Role, State, Color32, Color32); CHROME_STATES] {
    let installed = style(theme);
    let visuals = &installed.visuals;
    let plain = Classes::default();

    let panel = theme.window_background.blend(visuals.panel_fill);
    let input = panel.blend(visuals.text_edit_bg_color());
    let button = |painting: &Style, classes: &Classes, state: WidgetState| {
        let drawn = painting.button_style(classes, state);
        (drawn.text_style.color, panel.blend(drawn.frame.fill))
    };
    let frameless = |state: WidgetState| installed.widget_style(&plain, state).text.color;

    let (inactive_text, inactive_fill) = button(&installed, &plain, WidgetState::Inactive);
    let (hovered_text, hovered_fill) = button(&installed, &plain, WidgetState::Hovered);
    let (active_text, active_fill) = button(&installed, &plain, WidgetState::Active);
    let mut open_style = installed.clone();
    open_style.visuals.widgets.inactive = open_style.visuals.widgets.open;
    let (open_text, open_fill) = button(&open_style, &plain, WidgetState::Inactive);
    let selected = Classes::default().with_class(SELECTED_CLASS);
    let (selected_text, selected_fill) = button(&installed, &selected, WidgetState::Inactive);
    let selection = visuals.selection;

    [
        (Role::Text, State::Panel, visuals.text_color(), panel),
        (Role::Text, State::Input, visuals.text_color(), input),
        (Role::Text, State::Inactive, inactive_text, inactive_fill),
        (Role::Text, State::Open, open_text, open_fill),
        (
            Role::TextMuted,
            State::Panel,
            visuals.weak_text_color(),
            panel,
        ),
        (
            Role::TextMuted,
            State::Input,
            visuals.weak_text_color(),
            input,
        ),
        (Role::TextActive, State::Hovered, hovered_text, hovered_fill),
        (Role::TextActive, State::Active, active_text, active_fill),
        (
            Role::TextActive,
            State::HoveredFrameless,
            frameless(WidgetState::Hovered),
            panel,
        ),
        (
            Role::TextActive,
            State::ActiveFrameless,
            frameless(WidgetState::Active),
            panel,
        ),
        (
            Role::SelectionBorder,
            State::Selection,
            selected_text,
            selected_fill,
        ),
        (
            Role::SelectionBorder,
            State::InputSelection,
            selection.stroke.color,
            input.blend(selection.bg_fill),
        ),
        (Role::Error, State::Panel, visuals.error_fg_color, panel),
        (Role::Warning, State::Panel, visuals.warn_fg_color, panel),
        (Role::Link, State::Panel, visuals.hyperlink_color, panel),
    ]
}

///
/// One reachable painted text state's measured result.
///
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ContrastResult {
    pub(crate) role: Role,
    pub(crate) state: State,
    pub(crate) foreground: Color32,
    pub(crate) background: Color32,
    pub(crate) ratio: f32,
    /// Whether `.scratch/theming/issues/08`'s comments record this exact
    /// `(role, state, foreground, background)` as an explicitly accepted
    /// exception. Never turns [`Self::passes`] into `true`: acceptance
    /// annotates a failure, it does not hide or pass it. Keyed on the
    /// measured colour pair as well as role/state, so a retune that changes
    /// `foreground`/`background` while keeping the same role/state label
    /// does not stay silently accepted.
    pub(crate) accepted: bool,
}

impl ContrastResult {
    /// Whether [`Self::ratio`] clears [`CONTRAST_FLOOR`].
    pub(crate) fn passes(&self) -> bool {
        self.ratio >= CONTRAST_FLOOR
    }
}

///
/// [`validate`]'s complete answer for one Theme: the floor and scope every
/// [`ContrastResult`] is measured against, carried in the data itself rather
/// than only in this module's rustdoc, plus the results.
///
#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "scope is read only by tests: theming/07's Theme notice names the floor and the \
                  failing states, not the scope text"
    )
)]
pub(crate) struct ContrastReport {
    pub(crate) floor: f32,
    /// This module's own `# Scope` section, restated as data a caller can
    /// show beside the report rather than only read in source.
    pub(crate) scope: &'static str,
    pub(crate) results: Vec<ContrastResult>,
}

/// [`ContrastReport::scope`]'s exact text.
const SCOPE: &str = "Text contrast only: an effective foreground against the effective \
background it is actually painted on. Not pairwise Token-colour distinguishability, \
border/focus visibility, or the Cursor Effect's animated area field. Colour vision is \
measured separately, by distinguish. Chrome leaves out disabled widgets (faded by egui, and \
exempt as inactive components under WCAG 2.1 SC 1.4.3), code.background (no code span is \
painted), a widget state's strong fill (egui paints no text on it), a popup or window \
floating over another surface, the Source Grid or a panel (measured as one panel over the \
window backdrop; the two differ only for a translucent panel.background), and text.active on \
the open widget fill (egui paints an open widget's text in widgets.open.fg_stroke, which the \
Theme maps from text, so the open fill is measured with text instead).";

///
/// Measures every reachable painted text state against `theme` and reports
/// each one's role, state, effective foreground and background, measured
/// WCAG ratio, and whether it is an explicitly accepted exception.
///
/// Never refuses `theme`: a scheme that fails a state is measured and
/// reported exactly like one that passes, because the floor is a fact about
/// the scheme worth seeing rather than a gate a Theme must clear before it
/// loads. `crate::theme_registry` shows the states below the floor as a
/// Theme notice when a custom Theme loads (`.scratch/theming/issues/07`); a
/// failing one loads anyway, because the viewer chose it.
///
pub(crate) fn validate(theme: &Theme) -> ContrastReport {
    let mut results = Vec::with_capacity(
        Role::SOURCE_ROLES.len() * CursorPlacement::ALL.len() * 2 + CHROME_STATES,
    );

    for &role in &Role::SOURCE_ROLES {
        let fact = role
            .source_paint_fact()
            .expect("Role::SOURCE_ROLES holds only roles with a Source Paint fact");
        for cursor in CursorPlacement::ALL {
            for output_portal in [false, true] {
                let state = State::SourceGrid {
                    cursor,
                    output_portal,
                };
                let (foreground, background) = painted(fact, cursor, output_portal, theme);
                results.push(measure(
                    role,
                    state,
                    foreground,
                    background,
                    &theme.identity,
                ));
            }
        }
    }

    for (role, state, foreground, background) in chrome(theme) {
        results.push(measure(
            role,
            state,
            foreground,
            background,
            &theme.identity,
        ));
    }

    ContrastReport {
        floor: CONTRAST_FLOOR,
        scope: SCOPE,
        results,
    }
}

///
/// Composites `foreground` over the already-opaque `background`, measures
/// [`contrast`] between the two, and looks `identity`'s accepted exceptions
/// up for a matching entry.
///
fn measure(
    role: Role,
    state: State,
    foreground: Color32,
    background: Color32,
    identity: &ThemeIdentity,
) -> ContrastResult {
    let displayed = background.blend(foreground);
    let ratio = contrast(displayed, background);
    let accepted = accepted_failures(identity)
        .iter()
        .any(|entry| entry.matches(role, state, foreground, background));
    ContrastResult {
        role,
        state,
        foreground,
        background,
        ratio,
        accepted,
    }
}

///
/// The WCAG 2.1 relative-luminance contrast ratio between two colours. Both
/// arguments must already be opaque: `.r()`/`.g()`/`.b()` read
/// [`Color32`]'s premultiplied bytes directly, which equal straight sRGB
/// only at full alpha. Every caller in this module guarantees this by
/// compositing down to the opaque window backdrop first.
///
fn contrast(foreground: Color32, background: Color32) -> f32 {
    let luminance = |colour: Color32| {
        let channel = |value: u8| {
            let value = f32::from(value) / 255.0;
            if value <= 0.03928 {
                value / 12.92
            } else {
                ((value + 0.055) / 1.055).powf(2.4)
            }
        };
        0.2126 * channel(colour.r()) + 0.7152 * channel(colour.g()) + 0.0722 * channel(colour.b())
    };
    let (first, second) = (luminance(foreground), luminance(background));
    (first.max(second) + 0.05) / (first.min(second) + 0.05)
}

///
/// One below-floor `(role, state)` result `.scratch/theming/issues/08`'s
/// comments record as explicitly accepted for a shipped Theme, at the exact
/// colour pair it was accepted at.
///
#[derive(Clone, Copy, Debug, PartialEq)]
struct AcceptedFailure {
    role: Role,
    state: State,
    foreground: Color32,
    background: Color32,
}

impl AcceptedFailure {
    fn matches(&self, role: Role, state: State, foreground: Color32, background: Color32) -> bool {
        self.role == role
            && self.state == state
            && self.foreground == foreground
            && self.background == background
    }
}

///
/// The accepted exceptions for a shipped Theme, keyed by identity.
///
/// Okabe–Ito's list is empty. Its only candidate was Sequence: before
/// Pending roles were dropped, `Sequence, Pending`'s `Cursor`/`Region,
/// Cursor's Cell` states measured `#0072B2` against the bare Grid
/// background, `#000000`, and `.scratch/theming/issues/08`'s acceptance
/// line named exactly that pair as an accepted exception. Dropping Pending
/// removes that role from the report entirely — it has no foreground to
/// measure — and Sequence's one remaining role, `Sequence, Invalid`, passes
/// the floor on its own (Diagnostic's foreground replaces Sequence's outright
/// once it is Invalid), so Sequence has no reachable failing state left to
/// except. This is recorded as "no reachable painted failure," not as the
/// exception having been withdrawn.
///
/// `#0072B2` and the 4.5:1 floor are unchanged; nothing here retunes a
/// colour or lowers the floor to reach this empty list.
///
/// Called from [`measure`], which every [`validate`] result passes through —
/// not test-only: `theme_registry` validates every Theme file it loads.
/// `ContrastResult::accepted` has to come from
/// somewhere for the report to carry its acceptance annotation in the
/// returned data rather than only in test code, and this table is that
/// somewhere. [`unaccepted`], the shipped-Theme *gate*'s comparison, has no
/// such production reason and lives in `mod tests`.
///
fn accepted_failures(identity: &ThemeIdentity) -> &'static [AcceptedFailure] {
    match identity {
        id if *id == OKABE_ITO_IDENTITY => &[],
        // Orcvs Light's list is empty for the same reason, and for a
        // stronger one: `.scratch/theming/issues/04` tuned the light
        // definition against this validator until every reachable state
        // cleared the floor, so it ships with no exception to record.
        // `console/src/theme.md` states the measured figures.
        id if *id == ORCVS_LIGHT_IDENTITY => &[],
        _ => &[],
    }
}

// === Colour vision ===

///
/// The CIEDE2000 colour difference every pair of distinct [`GlyphChannel`]s
/// must keep under a simulated red–green dichromacy: 5.0, the separation at
/// which two colours are ordinarily taken to be clearly distinct rather than
/// merely measurably different.
///
/// It is a floor this repository's own colour authority already clears with
/// margin rather than one fitted to the light built-in:
/// [`crate::theme::okabe_ito`]'s worst red–green pair measures 6.65
/// (Bang against Comment under deuteranopia), and it is the published
/// colour-blind-safe Okabe–Ito assignment, unretuned.
/// `console/src/theme.md` records both built-ins' figures.
///
#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "read only by distinguish, a review gate over the built-ins that no shipped \
                  path runs; theming/07 shows validate's report alone"
    )
)]
pub(crate) const CONFUSION_FLOOR: f32 = 5.0;

///
/// A dichromacy [`distinguish`] simulates.
///
/// Protanopia and deuteranopia — the two red–green dichromacies, together
/// roughly 8% of men — are gated against [`CONFUSION_FLOOR`].
///
/// Tritanopia is measured and reported, and deliberately **not** gated. The
/// Okabe–Ito assignment this console paints from is published as safe for
/// red–green deficiency and makes no tritan claim, and it does not hold
/// under one: the shipped dark built-in's Bang (`#CC79A7`, reddish purple)
/// and Diagnostic (`#D55E00`, vermillion) measure 0.60 apart under
/// tritanopia, because the axis that separates them is exactly the one
/// tritanopia removes. Gating tritanopia would therefore fail the shipped
/// dark Theme, and no lightness arrangement inside the contrast ceiling a
/// near-white ground imposes rescues it for the light one either. It is
/// reported so the number is a measured, checked boundary rather than an
/// unexamined gap — `shipped_theme_colour_vision_gate` pins both built-ins'
/// tritan figures — and so a future palette decision can see what it costs.
///
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "read only by distinguish, a review gate over the built-ins that no shipped \
                  path runs; theming/07 shows validate's report alone"
    )
)]
pub(crate) enum ColourVision {
    Protanopia,
    Deuteranopia,
    Tritanopia,
}

#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "read only by distinguish, a review gate over the built-ins that no shipped \
                  path runs; theming/07 shows validate's report alone"
    )
)]
impl ColourVision {
    const ALL: [Self; 3] = [Self::Protanopia, Self::Deuteranopia, Self::Tritanopia];

    /// Whether [`CONFUSION_FLOOR`] is a gate for this dichromacy or only a
    /// reference line beside a reported measurement — see this enum's own
    /// documentation for why tritanopia is the latter.
    pub(crate) fn gated(self) -> bool {
        matches!(self, Self::Protanopia | Self::Deuteranopia)
    }
}

impl fmt::Display for ColourVision {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Protanopia => "protanopia",
            Self::Deuteranopia => "deuteranopia",
            Self::Tritanopia => "tritanopia",
        })
    }
}

///
/// A Theme foreground channel some reachable Cell actually draws a glyph in
/// — the distinction a reader reads off the colour. [`Role::glyph_channel`]
/// maps each `(role, output_portal)` pair onto one, and says why the set is
/// this one rather than one entry per Theme colour key.
///
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "read only by distinguish, a review gate over the built-ins that no shipped \
                  path runs; theming/07 shows validate's report alone"
    )
)]
pub(crate) enum GlyphChannel {
    Ordinary,
    Comment,
    Number,
    Note,
    Function,
    Bang,
    Diagnostic,
    OutputPortal,
}

impl fmt::Display for GlyphChannel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Ordinary => "Ordinary",
            Self::Comment => "Comment",
            Self::Number => "Number",
            Self::Note => "Note",
            Self::Function => "Function",
            Self::Bang => "Bang",
            Self::Diagnostic => "Diagnostic",
            Self::OutputPortal => "Output Portal",
        })
    }
}

///
/// How far apart one pair of [`GlyphChannel`]s stays under one simulated
/// dichromacy, at the [`CursorPlacement`] where they are closest.
///
/// The two colours are the *displayed* glyph colours — each channel's
/// effective foreground already composited over the effective background it
/// paints on, the same pair [`measure`] takes a contrast ratio between — so
/// a translucent channel, a role tint, the Region wash and the doubled
/// Portal-over-role tint all count, exactly as they do on screen.
///
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "distinguish's report, which no shipped path runs: theming/07 shows \
                  validate's report (08's validator) on load, and nothing yet shows this one"
    )
)]
pub(crate) struct ConfusionResult {
    pub(crate) vision: ColourVision,
    pub(crate) first: GlyphChannel,
    pub(crate) second: GlyphChannel,
    /// Where the two were closest. Both colours are read at this same
    /// placement: two Cells a reader compares are on one screen under one
    /// Cursor and Region state, so comparing a `plain` Cell's glyph with a
    /// `Region` Cell's would measure a difference the reader can already see
    /// from the wash.
    pub(crate) placement: CursorPlacement,
    pub(crate) first_colour: Color32,
    pub(crate) second_colour: Color32,
    /// CIEDE2000, between the two colours *after* simulation.
    pub(crate) distance: f32,
}

#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "read only by distinguish, a review gate over the built-ins that no shipped \
                  path runs; theming/07 shows validate's report alone"
    )
)]
impl ConfusionResult {
    /// Whether [`Self::distance`] clears [`CONFUSION_FLOOR`]. Answered for
    /// every dichromacy, including the ungated one — whether a failure is a
    /// gate failure is [`ColourVision::gated`]'s question, not this one's.
    pub(crate) fn passes(&self) -> bool {
        self.distance >= CONFUSION_FLOOR
    }
}

///
/// [`distinguish`]'s complete answer for one Theme, shaped like
/// [`ContrastReport`]: the floor and scope carried in the data, then one
/// result per `(dichromacy, channel pair)`.
///
#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "distinguish's report, which no shipped path runs: theming/07 shows \
                  validate's report (08's validator) on load, and nothing yet shows this one"
    )
)]
pub(crate) struct ConfusionReport {
    pub(crate) floor: f32,
    pub(crate) scope: &'static str,
    pub(crate) results: Vec<ConfusionResult>,
}

/// [`ConfusionReport::scope`]'s exact text.
#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "read only by distinguish, a review gate over the built-ins that no shipped \
                  path runs; theming/07 shows validate's report alone"
    )
)]
const CONFUSION_SCOPE: &str = "Pairwise separation of the Source glyph channels under \
simulated dichromacy, measured on the composited colours each one actually displays. The \
floor gates protanopia and deuteranopia; tritanopia is measured and reported, not gated. Not \
background tints against each other, not text contrast, not border/focus visibility, and not \
anomalous trichromacy, which is a continuum this reports the endpoint of.";

///
/// Measures, for every pair of distinct [`GlyphChannel`]s and every
/// [`ColourVision`], the smallest CIEDE2000 separation the two keep once
/// both are simulated — over the same reachable painted states [`validate`]
/// measures contrast for, and over the colours those states actually
/// display.
///
/// Never refuses `theme`, for [`validate`]'s reason: a scheme that confuses
/// two channels is measured and reported exactly like one that does not.
///
#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "read only by distinguish, a review gate over the built-ins that no shipped \
                  path runs; theming/07 shows validate's report alone"
    )
)]
pub(crate) fn distinguish(theme: &Theme) -> ConfusionReport {
    // Every reachable painted glyph, as (channel, placement, displayed
    // colour). `painted` answers the same effective pair `validate` measures,
    // and `measure`'s own `background.blend(foreground)` is what a viewer
    // sees, so this compares displayed colours rather than raw Theme values.
    let mut displayed: Vec<(GlyphChannel, CursorPlacement, Color32)> = Vec::new();
    for &role in &Role::SOURCE_ROLES {
        let fact = role
            .source_paint_fact()
            .expect("Role::SOURCE_ROLES holds only roles with a Source Paint fact");
        for placement in CursorPlacement::ALL {
            for output_portal in [false, true] {
                let Some(channel) = role.glyph_channel(output_portal, theme) else {
                    continue;
                };
                let (foreground, background) = painted(fact, placement, output_portal, theme);
                displayed.push((channel, placement, background.blend(foreground)));
            }
        }
    }

    let mut results: Vec<ConfusionResult> = Vec::new();
    for vision in ColourVision::ALL {
        let simulated: Vec<_> = displayed
            .iter()
            .map(|&(channel, placement, colour)| {
                (channel, placement, colour, lab(simulate(colour, vision)))
            })
            .collect();

        for (index, &(first, placement, first_colour, first_lab)) in simulated.iter().enumerate() {
            for &(second, other_placement, second_colour, second_lab) in &simulated[index + 1..] {
                if first == second || placement != other_placement {
                    continue;
                }
                let candidate = ConfusionResult {
                    vision,
                    first,
                    second,
                    placement,
                    first_colour,
                    second_colour,
                    distance: difference(first_lab, second_lab),
                };
                let existing = results.iter_mut().find(|result| {
                    result.vision == vision
                        && ((result.first == first && result.second == second)
                            || (result.first == second && result.second == first))
                });
                match existing {
                    Some(result) if candidate.distance < result.distance => *result = candidate,
                    Some(_) => {}
                    None => results.push(candidate),
                }
            }
        }
    }

    ConfusionReport {
        floor: CONFUSION_FLOOR,
        scope: CONFUSION_SCOPE,
        results,
    }
}

///
/// `colour` as a dichromat of `vision` sees it, by the Viénot, Brettel &
/// Mollon (1999) transform: linearize, convert to Smith–Pokorny LMS, replace
/// the missing cone's response with the plane through the two anchor
/// stimuli, convert back, re-encode.
///
/// `colour` must already be opaque — every caller composites down to the
/// window backdrop first, as [`contrast`]'s callers do, because
/// `.r()`/`.g()`/`.b()` read premultiplied bytes.
///
/// [`egui::ecolor`]'s own `linear_f32_from_gamma_u8` and
/// `gamma_u8_from_linear_f32` do the transfer function, rather than a second
/// copy of it here: they are the pinned colour operations everything else in
/// this module and in [`crate::style`] already composites with.
///
/// The protanopia and deuteranopia planes are Viénot, Brettel & Mollon's
/// published ones, not fitted. That paper gives no tritan plane: the
/// tritanopia row is the widely used companion projection in the same LMS
/// space, so the ungated tritan figures rest on a weaker model than the two
/// gated ones. `linearize` here is applied to the same sRGB primaries
/// `Color32` stores.
/// `simulating_a_dichromacy_is_idempotent` and
/// `simulation_leaves_neutrals_alone` check the two properties that follow
/// from this being a projection onto a plane, which is what makes the
/// arithmetic wrong-detectable rather than merely plausible.
///
#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "read only by distinguish, a review gate over the built-ins that no shipped \
                  path runs; theming/07 shows validate's report alone"
    )
)]
fn simulate(colour: Color32, vision: ColourVision) -> Color32 {
    let linear = |value: u8| egui::ecolor::linear_f32_from_gamma_u8(value);
    let (red, green, blue) = (linear(colour.r()), linear(colour.g()), linear(colour.b()));

    // Smith–Pokorny cone fundamentals, in the normalization Viénot, Brettel
    // & Mollon (1999) state them in.
    let long = 17.8824 * red + 43.5161 * green + 4.11935 * blue;
    let medium = 3.45565 * red + 27.1554 * green + 3.86714 * blue;
    let short = 0.029_956_6 * red + 0.184_309 * green + 1.46709 * blue;

    let (long, medium, short) = match vision {
        ColourVision::Protanopia => (2.02344 * medium - 2.52581 * short, medium, short),
        ColourVision::Deuteranopia => (long, 0.494_207 * long + 1.24827 * short, short),
        ColourVision::Tritanopia => (long, medium, -0.395_913 * long + 0.801_109 * medium),
    };

    let back = |first: f32, second: f32, third: f32| {
        egui::ecolor::gamma_u8_from_linear_f32(
            (first * long + second * medium + third * short).clamp(0.0, 1.0),
        )
    };
    Color32::from_rgb(
        back(0.080_944_45, -0.130_504_41, 0.116_721_07),
        back(-0.010_248_534, 0.054_019_33, -0.113_614_71),
        back(-0.000_365_296_94, -0.004_121_614_7, 0.693_511_4),
    )
}

///
/// `colour` in CIE L\*a\*b\* against the D65 white point — the space
/// [`difference`] is defined in. `colour` must already be opaque, for
/// [`simulate`]'s reason.
///
#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "read only by distinguish, a review gate over the built-ins that no shipped \
                  path runs; theming/07 shows validate's report alone"
    )
)]
fn lab(colour: Color32) -> [f32; 3] {
    let linear = |value: u8| egui::ecolor::linear_f32_from_gamma_u8(value);
    let (red, green, blue) = (linear(colour.r()), linear(colour.g()), linear(colour.b()));

    // sRGB to CIE XYZ (D65), then XYZ to L*a*b* against the same white.
    let x = (0.412_456_4 * red + 0.357_576_1 * green + 0.180_437_5 * blue) / 0.950_47;
    let y = 0.212_672_9 * red + 0.715_152_2 * green + 0.072_175_0 * blue;
    let z = (0.019_333_9 * red + 0.119_192 * green + 0.950_304_1 * blue) / 1.088_83;

    let f = |value: f32| {
        if value > 216.0 / 24389.0 {
            value.cbrt()
        } else {
            (841.0 / 108.0) * value + 4.0 / 29.0
        }
    };
    let (fx, fy, fz) = (f(x), f(y), f(z));
    [116.0 * fy - 16.0, 500.0 * (fx - fy), 200.0 * (fy - fz)]
}

///
/// CIEDE2000 (CIE 142-2001) between two [`lab`] colours, at the default
/// parametric weights `kL = kC = kH = 1`.
///
/// Stated over L\*a\*b\* rather than over [`Color32`] so that
/// `the_colour_difference_matches_the_published_test_vectors` can check it
/// against Sharma, Wu & Dalal's published dataset directly — those vectors
/// are L\*a\*b\* pairs, several of which no 8-bit sRGB colour reaches, so a
/// `Color32`-only entry point could not be checked against them at all.
///
#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "read only by distinguish, a review gate over the built-ins that no shipped \
                  path runs; theming/07 shows validate's report alone"
    )
)]
fn difference(first: [f32; 3], second: [f32; 3]) -> f32 {
    let [first_l, first_a, first_b] = first;
    let [second_l, second_a, second_b] = second;

    let first_chroma = first_a.hypot(first_b);
    let second_chroma = second_a.hypot(second_b);
    let mean_chroma = (first_chroma + second_chroma) / 2.0;
    let seventh = mean_chroma.powi(7);
    let g = 0.5 * (1.0 - (seventh / (seventh + 25f32.powi(7))).sqrt());

    let first_a = (1.0 + g) * first_a;
    let second_a = (1.0 + g) * second_a;
    let first_chroma = first_a.hypot(first_b);
    let second_chroma = second_a.hypot(second_b);

    let hue = |a: f32, b: f32| {
        if a == 0.0 && b == 0.0 {
            0.0
        } else {
            b.atan2(a).to_degrees().rem_euclid(360.0)
        }
    };
    let first_hue = hue(first_a, first_b);
    let second_hue = hue(second_a, second_b);

    let delta_l = second_l - first_l;
    let delta_chroma = second_chroma - first_chroma;
    let chroma_product = first_chroma * second_chroma;
    let delta_hue = if chroma_product == 0.0 {
        0.0
    } else {
        let raw = second_hue - first_hue;
        if raw.abs() <= 180.0 {
            raw
        } else if raw > 180.0 {
            raw - 360.0
        } else {
            raw + 360.0
        }
    };
    let delta_capital_hue = 2.0 * chroma_product.sqrt() * (delta_hue.to_radians() / 2.0).sin();

    let mean_l = (first_l + second_l) / 2.0;
    let mean_chroma = (first_chroma + second_chroma) / 2.0;
    let mean_hue = if chroma_product == 0.0 {
        first_hue + second_hue
    } else if (first_hue - second_hue).abs() <= 180.0 {
        (first_hue + second_hue) / 2.0
    } else if first_hue + second_hue < 360.0 {
        (first_hue + second_hue + 360.0) / 2.0
    } else {
        (first_hue + second_hue - 360.0) / 2.0
    };

    let t = 1.0 - 0.17 * (mean_hue - 30.0).to_radians().cos()
        + 0.24 * (2.0 * mean_hue).to_radians().cos()
        + 0.32 * (3.0 * mean_hue + 6.0).to_radians().cos()
        - 0.20 * (4.0 * mean_hue - 63.0).to_radians().cos();

    let mean_seventh = mean_chroma.powi(7);
    let rotation_chroma = 2.0 * (mean_seventh / (mean_seventh + 25f32.powi(7))).sqrt();
    let rotation_angle = 30.0 * (-(((mean_hue - 275.0) / 25.0).powi(2))).exp();
    let rotation = -(2.0 * rotation_angle).to_radians().sin() * rotation_chroma;

    let lightness_weight =
        1.0 + (0.015 * (mean_l - 50.0).powi(2)) / (20.0 + (mean_l - 50.0).powi(2)).sqrt();
    let chroma_weight = 1.0 + 0.045 * mean_chroma;
    let hue_weight = 1.0 + 0.015 * mean_chroma * t;

    let lightness_term = delta_l / lightness_weight;
    let chroma_term = delta_chroma / chroma_weight;
    let hue_term = delta_capital_hue / hue_weight;

    (lightness_term.powi(2)
        + chroma_term.powi(2)
        + hue_term.powi(2)
        + rotation * chroma_term * hue_term)
        .sqrt()
}

#[cfg(test)]
mod tests {
    use egui::Color32;
    use orcvs::source::SourcePaint;

    use super::{
        AcceptedFailure, CONFUSION_FLOOR, ColourVision, ConfusionReport, ConfusionResult,
        ContrastReport, ContrastResult, CursorPlacement, GlyphChannel, Role, State, contrast,
        difference, distinguish, lab, painted, simulate, validate,
    };
    use crate::theme::{OKABE_ITO_IDENTITY, ORCVS_LIGHT_IDENTITY, Theme, okabe_ito, orcvs_light};

    fn find(report: &ContrastReport, role: Role, state: State) -> ContrastResult {
        report
            .results
            .iter()
            .find(|result| result.role == role && result.state == state)
            .unwrap_or_else(|| panic!("validate did not report {role} / {state}"))
            .clone()
    }

    /// The below-floor results of `report` that are not accepted exceptions
    /// — `.scratch/theming/issues/08`'s shipped-Theme gate. A plain filter
    /// over `report.results` rather than a separately parameterized
    /// function: acceptance is already baked into each `ContrastResult` by
    /// `validate`, so there is nothing left for a test-only helper to do but
    /// filter.
    fn unaccepted(report: &ContrastReport) -> Vec<&ContrastResult> {
        report
            .results
            .iter()
            .filter(|result| !result.passes() && !result.accepted)
            .collect()
    }

    // === The contrast formula itself, on synthetic colours with
    // independently known WCAG answers ===

    ///
    /// The WCAG spec's own stated extreme: black text on a white background
    /// is the maximum contrast ratio, 21:1 ("Understanding Success Criterion
    /// 1.4.3") — a fact about the ratio's own defined range, not a figure
    /// this module's arithmetic could drift into by coincidence.
    ///
    #[test]
    fn contrast_of_black_and_white_is_the_wcag_maximum_21_to_1() {
        let ratio = contrast(Color32::WHITE, Color32::BLACK);
        assert!((ratio - 21.0).abs() < 0.01, "got {ratio:.2}:1");
    }

    ///
    /// The ratio's own stated minimum: two identical colours are 1:1.
    ///
    #[test]
    fn contrast_of_identical_colours_is_the_wcag_minimum_1_to_1() {
        let mid_tone = Color32::from_rgb(96, 140, 60);
        let ratio = contrast(mid_tone, mid_tone);
        assert!((ratio - 1.0).abs() < 0.001, "got {ratio:.2}:1");
    }

    ///
    /// A worked mid pair, computed by hand from the WCAG 2.1
    /// relative-luminance formula rather than by running this module's own
    /// code under test: pure red's relative luminance is `0.2126 * 1.0 =
    /// 0.2126` (its own channel fully saturated, the other two at zero);
    /// white's is `1.0` (the three coefficients sum to `1.0` exactly). Their
    /// ratio is `(1.0 + 0.05) / (0.2126 + 0.05) = 1.05 / 0.2626 ≈ 4.00:1`.
    ///
    #[test]
    fn contrast_of_a_worked_mid_pair_matches_the_hand_computed_ratio() {
        let red = Color32::from_rgb(255, 0, 0);
        let ratio = contrast(Color32::WHITE, red);
        assert!(
            (ratio - 4.00).abs() < 0.01,
            "pure red on white: got {ratio:.2}:1, expected ~4.00:1"
        );
    }

    // === validate's shape ===

    #[test]
    fn validate_reports_the_floor_and_scope_in_the_returned_data() {
        let report = validate(&okabe_ito());
        assert_eq!(report.floor, 4.5);
        assert!(
            report.scope.contains("Text contrast only"),
            "the report must carry its own scope, not only rustdoc: {:?}",
            report.scope
        );
        // `.scratch/theming/issues/11`: every chrome case left out is named
        // in the returned scope, not only in rustdoc.
        for excluded in [
            "disabled widgets",
            "code.background",
            "strong fill",
            "floating over another surface",
            "text.active on the open widget fill",
        ] {
            assert!(
                report.scope.contains(excluded),
                "the scope must name the excluded {excluded:?}: {:?}",
                report.scope
            );
        }
    }

    #[test]
    fn validate_reports_one_result_per_role_cursor_placement_and_output_portal_plus_fifteen_chrome_states()
     {
        let report = validate(&okabe_ito());
        // 10 Source Grid roles x 4 CursorPlacements x 2 output_portal states
        // = 80, plus 15 chrome states: text on panel, input and the
        // inactive and open fills (4); text.muted on panel and input (2);
        // text.active on the hovered and active fills, framed and frameless
        // (4); selected text on selection.background over panel and input
        // (2); error, warning and link on panel (3). Stated as one literal
        // rather than recomputed from the same lengths `validate` sizes its
        // `Vec` from, so a change to either count is caught by an
        // independent number.
        assert_eq!(report.results.len(), 95);
    }

    ///
    /// The validator against a Theme built to fail — proof `validate`
    /// discriminates a pass from a failure rather than reporting every
    /// state as passing regardless of the colours it is given, and that a
    /// failing Theme is reported, never refused: `validate` returns a plain
    /// `ContrastReport`, not a `Result`, and the failing entry sits in
    /// `results` beside every passing one.
    ///
    #[test]
    fn validate_reports_a_theme_built_to_fail_rather_than_a_vacuous_pass() {
        let base = okabe_ito();
        let theme = Theme {
            text: base.panel_background,
            ..base
        };

        let report = validate(&theme);

        let text = find(&report, Role::Text, State::Panel);
        assert!(
            !text.passes(),
            "text equal to panel.background should fail: {:.2}:1",
            text.ratio
        );
        assert!(
            (text.ratio - 1.0).abs() < 0.01,
            "identical colours measure 1:1, got {:.2}:1",
            text.ratio
        );

        // A state untouched by the failing override still passes, so the
        // failure above is specific to the retuned property rather than
        // `validate` reporting every state as failing once one does.
        let comment = find(
            &report,
            Role::Comment,
            State::SourceGrid {
                cursor: CursorPlacement::Plain,
                output_portal: false,
            },
        );
        assert!(
            comment.passes(),
            "Comment was not retuned and should still clear the floor: {:.2}:1",
            comment.ratio
        );
    }

    // === Chrome states beyond text/text.muted on panel and input
    // (`.scratch/theming/issues/11`) ===

    /// Asserts `result` measures `foreground` against itself — 1:1 — and so
    /// fails, and that every state the pre-`11` report already measured
    /// still passes, so the failure is one only the added state can see.
    fn fails_only_in_an_added_state(report: &ContrastReport, result: &ContrastResult) {
        assert!(
            !result.passes() && (result.ratio - 1.0).abs() < 0.01,
            "{} / {} should measure 1:1, got {:.2}:1",
            result.role,
            result.state,
            result.ratio
        );
        let old_report_failures: Vec<_> = report
            .results
            .iter()
            .filter(|result| {
                matches!(result.state, State::SourceGrid { .. })
                    || matches!(
                        (result.role, result.state),
                        (Role::Text | Role::TextMuted, State::Panel | State::Input)
                    )
            })
            .filter(|result| !result.passes())
            .collect();
        assert!(
            old_report_failures.is_empty(),
            "the fixture must be invisible to the states the old report measured: \
             {old_report_failures:?}"
        );
    }

    ///
    /// `text.active` set to `selection.background` is unreadable on every
    /// hovered or active button — egui paints both from
    /// `selection.background` — while `text` and `text.muted` on panel and
    /// input, the only chrome the old report measured, stay untouched.
    ///
    #[test]
    fn text_active_matching_the_hovered_and_active_fill_fails_only_there() {
        let base = okabe_ito();
        let theme = Theme {
            text_active: base.selection_background,
            ..base
        };

        let report = validate(&theme);

        for state in [State::Hovered, State::Active] {
            fails_only_in_an_added_state(&report, &find(&report, Role::TextActive, state));
        }
        // The frameless hover and press — a checkbox label, which egui
        // paints in the same foreground straight onto the panel — measure
        // that foreground against the panel rather than the widget fill.
        let panel = find(&report, Role::Text, State::Panel).background;
        for state in [State::HoveredFrameless, State::ActiveFrameless] {
            let frameless = find(&report, Role::TextActive, state);
            assert_eq!(
                (frameless.foreground, frameless.background),
                (theme.text_active, panel),
                "{state}"
            );
        }
    }

    ///
    /// Selected text — egui recolours it to `selection.border` over
    /// `selection.background` — set to the fill it sits on fails over both
    /// the panel (a selectable label or a selected `selectable_value`) and
    /// the input (a `TextEdit` selection).
    ///
    #[test]
    fn selected_text_matching_selection_background_fails_only_there() {
        let base = okabe_ito();
        let theme = Theme {
            selection_border: base.selection_background,
            ..base
        };

        let report = validate(&theme);

        for state in [State::Selection, State::InputSelection] {
            fails_only_in_an_added_state(&report, &find(&report, Role::SelectionBorder, state));
        }
    }

    ///
    /// `error`, `warning` and `link` each fail on the panel when set to
    /// `panel.background`, one at a time, so each result is shown to read
    /// its own property rather than a shared one.
    ///
    #[test]
    fn error_warning_and_link_matching_the_panel_each_fail_there() {
        let base = okabe_ito();
        for (role, theme) in [
            (
                Role::Error,
                Theme {
                    error: base.panel_background,
                    ..base.clone()
                },
            ),
            (
                Role::Warning,
                Theme {
                    warning: base.panel_background,
                    ..base.clone()
                },
            ),
            (
                Role::Link,
                Theme {
                    link: base.panel_background,
                    ..base.clone()
                },
            ),
        ] {
            let report = validate(&theme);
            fails_only_in_an_added_state(&report, &find(&report, role, State::Panel));
            for other in [Role::Error, Role::Warning, Role::Link] {
                if other != role {
                    assert!(
                        find(&report, other, State::Panel).passes(),
                        "{other} was not retuned alongside {role}"
                    );
                }
            }
        }
    }

    ///
    /// An open widget paints `text` on `widgets.open.weak_bg_fill`, which is
    /// `panel.background` — so a translucent panel is measured through the
    /// open fill composited over the panel over the window backdrop, not
    /// against `panel.background`'s raw bytes.
    ///
    #[test]
    fn the_open_widget_state_composites_its_fill_over_the_panel() {
        let base = okabe_ito();
        let theme = Theme {
            window_background: Color32::WHITE,
            panel_background: Color32::from_rgba_unmultiplied(0, 0, 0, 128),
            ..base
        };

        let report = validate(&theme);
        let open = find(&report, Role::Text, State::Open);
        let panel = Color32::WHITE.blend(theme.panel_background);
        assert_eq!(open.background, panel.blend(theme.panel_background));
        assert_ne!(
            open.background,
            find(&report, Role::Text, State::Panel).background
        );
    }

    // === Fixtures: text passing on the bare Grid but failing on its
    // painted tint or selected background ===

    ///
    /// A role whose own background is transparent (Comment's, in Okabe–Ito)
    /// reads fine against the bare Grid in its `plain` state, but a custom
    /// Theme's opaque `region.background` set to the same colour as the
    /// glyph itself makes the `Region` state of that same role unreadable.
    /// Region fallback only applies because Comment's own background stays
    /// fully transparent in this Theme; a role background with any nonzero
    /// alpha keeps winning over the Region fill regardless
    /// (`region_fallback_only_reaches_a_fully_transparent_role_background`
    /// pins that distinction directly).
    ///
    #[test]
    fn a_role_passing_plain_fails_against_its_painted_region_tint() {
        let theme = Theme {
            region_background: okabe_ito().source_comment,
            ..okabe_ito()
        };

        let report = validate(&theme);

        let plain = find(
            &report,
            Role::Comment,
            State::SourceGrid {
                cursor: CursorPlacement::Plain,
                output_portal: false,
            },
        );
        assert!(
            plain.passes(),
            "Comment/plain should still read against the bare Grid: {:.2}:1",
            plain.ratio
        );

        let region = find(
            &report,
            Role::Comment,
            State::SourceGrid {
                cursor: CursorPlacement::Region,
                output_portal: false,
            },
        );
        assert!(
            !region.passes(),
            "Comment/Region should fail once the Region tint matches the glyph: {:.2}:1",
            region.ratio
        );
        assert!(
            (region.ratio - 1.0).abs() < 0.05,
            "the glyph and its Region tint are nearly identical, got {:.2}:1",
            region.ratio
        );
    }

    ///
    /// The same fixture shape for the single-Cell Cursor: a custom Theme's
    /// `cursor.background` set to Comment's own colour leaves `plain`
    /// passing while `Cursor` fails.
    ///
    #[test]
    fn a_role_passing_plain_fails_against_its_selected_cursor_background() {
        let theme = Theme {
            cursor_background: Some(okabe_ito().source_comment),
            ..okabe_ito()
        };

        let report = validate(&theme);

        let plain = find(
            &report,
            Role::Comment,
            State::SourceGrid {
                cursor: CursorPlacement::Plain,
                output_portal: false,
            },
        );
        assert!(
            plain.passes(),
            "Comment/plain should still read against the bare Grid: {:.2}:1",
            plain.ratio
        );

        let cursor = find(
            &report,
            Role::Comment,
            State::SourceGrid {
                cursor: CursorPlacement::Cursor,
                output_portal: false,
            },
        );
        assert!(
            !cursor.passes(),
            "Comment/Cursor should fail once the Cursor fill matches the glyph: {:.2}:1",
            cursor.ratio
        );
    }

    // === Fixtures: alpha compositing ===

    ///
    /// A partial-alpha foreground over a partial-alpha background, checked
    /// against literal, hand-worked bytes rather than a recomposition that
    /// reuses the same `Color32::blend`/`contrast` this module's own code
    /// calls: `panel.background` at `(40, 40, 40, 128)` straight
    /// premultiplies to `(20, 20, 20, 128)` (`ecolor`'s `mul_frac_round(40,
    /// 128) = 20`) and composites over opaque black to `(20, 20, 20, 255)`
    /// unchanged — black contributes nothing at any alpha. `text` at
    /// 50%-alpha white premultiplies to `(128, 128, 128, 128)` and
    /// composites over that surface to `(138, 138, 138, 255)`. Both figures
    /// were computed independently (by hand, following `ecolor`'s published
    /// integer arithmetic) rather than by calling this module's own
    /// functions.
    ///
    #[test]
    fn a_partial_alpha_foreground_over_a_partial_alpha_background_composites_correctly() {
        let translucent_panel = Color32::from_rgba_unmultiplied(40, 40, 40, 128);
        let translucent_text = Color32::from_rgba_unmultiplied(255, 255, 255, 128);
        let theme = Theme {
            window_background: Color32::BLACK,
            panel_background: translucent_panel,
            text: translucent_text,
            ..okabe_ito()
        };

        let report = validate(&theme);
        let text = find(&report, Role::Text, State::Panel);

        assert_eq!(
            text.background,
            Color32::from_rgb(20, 20, 20),
            "the composited panel surface"
        );
        assert!(
            (text.ratio - 5.3364).abs() < 0.001,
            "got {:.4}:1, expected 5.3364:1, hand-worked from the composited \
             (138, 138, 138) foreground against the (20, 20, 20) background",
            text.ratio
        );
    }

    // === Fixture: a transparent fact foreground exposes the underlying
    // Token ===

    ///
    /// `diagnostic.foreground` set fully transparent leaves an Invalid
    /// Number operand showing its own declared Token colour rather than
    /// Diagnostic's — `.scratch/theming/schema.md`'s "A transparent fact
    /// channel reveals the underlying Token channel with alpha
    /// compositing." The reported `foreground` is the Number Token's own
    /// colour, not a disabled Diagnostic channel treated as though it were
    /// still the text.
    ///
    #[test]
    fn a_transparent_diagnostic_foreground_exposes_the_declared_tokens_own_colour() {
        let theme = Theme {
            diagnostic_foreground: Color32::TRANSPARENT,
            ..okabe_ito()
        };

        let report = validate(&theme);
        let invalid_number = find(
            &report,
            Role::NumberInvalid,
            State::SourceGrid {
                cursor: CursorPlacement::Plain,
                output_portal: false,
            },
        );

        assert_eq!(
            invalid_number.foreground, theme.source_number,
            "a transparent diagnostic.foreground must reveal source.number, not vanish"
        );
    }

    // === `painted`'s reuse of `style`'s own composition ===

    ///
    /// `painted`'s `output_portal: false` and `output_portal: true` for
    /// `Function` are identical — "A bound Function retains all its own
    /// paint inside a Portal" (`style::role_and_portal`'s own doc), the one
    /// named bypass the reused composition functions apply. This is a real
    /// reachable state (a nested Function inside a root's Output Portal
    /// Reservation) whose *answer* happens to equal the non-Portal case,
    /// which is exactly what the bypass rule says should happen.
    ///
    #[test]
    fn function_is_unaffected_by_output_portal_placement() {
        let theme = okabe_ito();
        let plain = painted(SourcePaint::Function, CursorPlacement::Plain, false, &theme);
        let portal = painted(SourcePaint::Function, CursorPlacement::Plain, true, &theme);
        assert_eq!(plain, portal);
    }

    ///
    /// The Cursor can sit on a Cell an Output Portal Reservation covers, and
    /// a Region can span over one — `.scratch/theming/schema.md`'s Overlap
    /// rule applies to every non-Function fact regardless of Cursor/Region
    /// state, so these are reachable states this validator measures rather
    /// than an impossible cross-product. `Number, Valid`'s foreground
    /// differs between `plain` and `Output Portal` (the Output Portal
    /// foreground channel blends over the Token's own), proving the
    /// `output_portal` flag is actually threaded through every
    /// `CursorPlacement`, not only `Plain`.
    ///
    #[test]
    fn cursor_and_region_placements_still_answer_the_output_portal_channel() {
        let theme = okabe_ito();
        let report = validate(&theme);

        for cursor in [
            CursorPlacement::Cursor,
            CursorPlacement::Region,
            CursorPlacement::RegionCursor,
        ] {
            let plain = find(
                &report,
                Role::NumberValid,
                State::SourceGrid {
                    cursor,
                    output_portal: false,
                },
            );
            let portal = find(
                &report,
                Role::NumberValid,
                State::SourceGrid {
                    cursor,
                    output_portal: true,
                },
            );
            assert_ne!(
                plain.foreground, portal.foreground,
                "{cursor}: Output Portal must still blend its foreground channel over Number's"
            );
        }
    }

    ///
    /// Region fallback only reaches a fact whose own background is fully
    /// transparent (alpha `0`): `Number, Invalid`'s background
    /// (`source.number.background`) is nonzero-alpha — translucent since
    /// the user's 2026-09-22 retune, not the opaque value it was before,
    /// but nonzero either way — so its `Region` state equals its `plain`
    /// state exactly, while `Comment`'s fully transparent background lets
    /// the Region tint show once `region.background` is not itself
    /// transparent. The rule is about zero versus nonzero alpha, not
    /// opaque versus translucent.
    ///
    #[test]
    fn region_fallback_only_reaches_a_fully_transparent_role_background() {
        let theme = okabe_ito();
        let report = validate(&theme);

        let nonzero_alpha_role_plain = find(
            &report,
            Role::NumberInvalid,
            State::SourceGrid {
                cursor: CursorPlacement::Plain,
                output_portal: false,
            },
        );
        let nonzero_alpha_role_region = find(
            &report,
            Role::NumberInvalid,
            State::SourceGrid {
                cursor: CursorPlacement::Region,
                output_portal: false,
            },
        );
        assert_eq!(
            nonzero_alpha_role_plain.background, nonzero_alpha_role_region.background,
            "a role background with any nonzero alpha must win over Region fallback"
        );

        let transparent_role_theme = Theme {
            region_background: Color32::from_rgba_unmultiplied(255, 255, 255, 255),
            ..theme
        };
        let transparent_report = validate(&transparent_role_theme);
        let transparent_plain = find(
            &transparent_report,
            Role::Comment,
            State::SourceGrid {
                cursor: CursorPlacement::Plain,
                output_portal: false,
            },
        );
        let transparent_region = find(
            &transparent_report,
            Role::Comment,
            State::SourceGrid {
                cursor: CursorPlacement::Region,
                output_portal: false,
            },
        );
        assert_ne!(
            transparent_plain.background, transparent_region.background,
            "a transparent role background must let an opaque Region fill show"
        );
    }

    ///
    /// The single-Cell Cursor always replaces the role background outright
    /// — `style::cell_visuals_with_cursor_colour`'s own "The Cursor's own
    /// fill wins outright on its Cell" — regardless of whether that role
    /// background was opaque. Every one of Okabe–Ito's own role backgrounds
    /// is translucent since the user's 2026-09-22 retune to a uniform 10%
    /// tint opacity, so this fixture sets `source_number_background`
    /// explicitly opaque to exercise the claim the test's name makes,
    /// rather than relying on a shipped Theme to happen to have one.
    ///
    #[test]
    fn cursors_own_fill_replaces_even_an_opaque_role_background() {
        let theme = Theme {
            source_number_background: Color32::from_rgb(14, 29, 37),
            cursor_background: Some(Color32::from_rgb(1, 2, 3)),
            ..okabe_ito()
        };

        let report = validate(&theme);

        let plain = find(
            &report,
            Role::NumberInvalid,
            State::SourceGrid {
                cursor: CursorPlacement::Plain,
                output_portal: false,
            },
        );
        let cursor = find(
            &report,
            Role::NumberInvalid,
            State::SourceGrid {
                cursor: CursorPlacement::Cursor,
                output_portal: false,
            },
        );
        assert_ne!(
            plain.background, cursor.background,
            "the Cursor's own fill must replace even an opaque role background"
        );
        assert_eq!(cursor.background, Color32::from_rgb(1, 2, 3));
    }

    // === The Cursor Effect's animated area field is out of scope ===

    ///
    /// This module's documented assumption, checked: `theme.cursor_area`
    /// does not change a measured background, at a state whose own painted
    /// background is fully transparent (Comment's `plain` state) — the one
    /// case where the real console's animated area field could show
    /// through. If this assertion ever fails, `painted` has started
    /// accounting for `cursor_area` and this module's `# Scope` section
    /// needs updating to match, not the other way around.
    ///
    #[test]
    fn painted_background_is_unaffected_by_the_cursor_area_effect() {
        let base = okabe_ito();
        let distinct_area = Theme {
            cursor_area: Color32::from_rgb(250, 10, 10),
            ..base.clone()
        };

        let report_base = validate(&base);
        let report_distinct = validate(&distinct_area);

        let state = State::SourceGrid {
            cursor: CursorPlacement::Plain,
            output_portal: false,
        };
        let comment_base = find(&report_base, Role::Comment, state);
        let comment_distinct = find(&report_distinct, Role::Comment, state);
        assert_eq!(
            comment_base.background, comment_distinct.background,
            "cursor_area must not change a measured background"
        );
    }

    // === Acceptance is keyed on the measured colour pair, not only the label ===

    ///
    /// [`AcceptedFailure::matches`] requires the same foreground/background
    /// as well as the same role/state: a retune that keeps a role's and
    /// state's label the same but changes the colour it measures must not
    /// stay silently accepted.
    ///
    #[test]
    fn accepted_failure_match_requires_the_same_colour_pair_not_only_role_and_state() {
        let state = State::SourceGrid {
            cursor: CursorPlacement::Plain,
            output_portal: false,
        };
        let entry = AcceptedFailure {
            role: Role::SequenceInvalid,
            state,
            foreground: Color32::from_rgb(1, 2, 3),
            background: Color32::from_rgb(4, 5, 6),
        };

        assert!(entry.matches(
            Role::SequenceInvalid,
            state,
            entry.foreground,
            entry.background
        ));
        assert!(
            !entry.matches(
                Role::SequenceInvalid,
                state,
                Color32::from_rgb(9, 9, 9),
                entry.background
            ),
            "a retuned foreground must not match a stale accepted entry"
        );
        assert!(
            !entry.matches(
                Role::SequenceInvalid,
                state,
                entry.foreground,
                Color32::from_rgb(9, 9, 9)
            ),
            "a retuned background must not match a stale accepted entry"
        );
    }

    // === Sequence has no reachable failing state left to except ===

    ///
    /// Dropping Pending removes `Sequence, Pending` from the report
    /// entirely (`Role` has no such variant any more), and `Sequence,
    /// Invalid` — the only Sequence role left — passes the floor on its own:
    /// `diagnostic.foreground` replaces Sequence's own colour outright once
    /// it is Invalid. Okabe–Ito's accepted-exception list is therefore
    /// empty, which this test pins as "no reachable painted failure,"
    /// distinct from an exception having been withdrawn.
    ///
    #[test]
    fn sequence_has_no_reachable_failing_state() {
        let report = validate(&okabe_ito());

        let sequence_results: Vec<_> = report
            .results
            .iter()
            .filter(|result| result.role == Role::SequenceInvalid)
            .collect();
        assert!(
            !sequence_results.is_empty(),
            "Sequence, Invalid must still be reported"
        );
        for result in sequence_results {
            assert!(
                result.passes(),
                "Sequence, Invalid / {} unexpectedly fails at {:.2}:1 — Sequence's accepted \
                 exception is empty because it has no failing state, not because the \
                 exception was withdrawn; if this fails, that premise no longer holds",
                result.state,
                result.ratio
            );
        }

        assert!(
            super::accepted_failures(&OKABE_ITO_IDENTITY).is_empty(),
            "Okabe-Ito's accepted-exception list should be empty: nothing currently fails \
             that this issue's comments record as accepted"
        );
    }

    ///
    /// Orcvs Light's accepted-exception list is empty for the stronger of
    /// the two possible reasons: nothing fails, because
    /// `.scratch/theming/issues/04` tuned the definition against this
    /// validator until nothing did. This pins that premise separately from
    /// `shipped_theme_gate`, which would also pass on a list full of
    /// exceptions.
    ///
    #[test]
    fn orcvs_light_has_nothing_to_except() {
        let report = validate(&orcvs_light());

        assert!(
            super::accepted_failures(&ORCVS_LIGHT_IDENTITY).is_empty(),
            "Orcvs Light ships with no accepted contrast exception"
        );
        let below_floor: Vec<_> = report
            .results
            .iter()
            .filter(|result| !result.passes())
            .collect();
        assert!(
            below_floor.is_empty(),
            "Orcvs Light must clear the floor unaided: {below_floor:?}"
        );
    }

    // === The shipped-Theme gate ===

    ///
    /// Every below-floor state of every shipped Theme must be an accepted
    /// exception, or this test fails and lists them — a real gate, not
    /// `#[ignore]`d: the user's 2026-09-22 retune of every tinted role
    /// background to a uniform 10% opacity (expressed as that role's own
    /// foreground colour at alpha `0x1A`, replacing the previous
    /// precomputed opaque tints) raised every measured ratio at or above
    /// the 4.5:1 floor, confirmed through this real shipped composition —
    /// `console/src/theme.md` records the exact figures. There is
    /// therefore nothing left to except: `accepted_failures` for
    /// `okabe-ito` is empty, and this test's own run is what proves that
    /// emptiness is correct rather than merely convenient.
    ///
    /// `orcvs_light()` joins it under `.scratch/theming/issues/04`, and
    /// clears the floor in every reachable state for the same reason rather
    /// than by exception: its colours were tuned against this validator
    /// until they did. `shipped` is a hand-kept array rather than the
    /// Theme registry's built-ins, which `.scratch/theming/issues/07` added
    /// privately to `crate::theme_registry` — a known weakness recorded in
    /// `.scratch/theming/issues/08`'s comments.
    ///
    #[test]
    fn shipped_theme_gate() {
        let shipped = [okabe_ito(), orcvs_light()];

        for theme in &shipped {
            let report = validate(theme);
            let failures = unaccepted(&report);
            assert!(
                failures.is_empty(),
                "{}'s unrecorded contrast failures: {failures:?}",
                theme.identity
            );
        }
    }

    // === Colour vision ===

    /// The closest reported pair for one dichromacy.
    fn closest(report: &ConfusionReport, vision: ColourVision) -> &ConfusionResult {
        report
            .results
            .iter()
            .filter(|result| result.vision == vision)
            .min_by(|first, second| {
                first
                    .distance
                    .partial_cmp(&second.distance)
                    .expect("CIEDE2000 of finite colours is finite")
            })
            .unwrap_or_else(|| panic!("distinguish reported nothing for {vision}"))
    }

    ///
    /// `difference` against Sharma, Wu & Dalal's published CIEDE2000 test
    /// data ("The CIEDE2000 Color-Difference Formula: Implementation Notes,
    /// Supplementary Test Data, and Mathematical Observations", 2005) — the
    /// dataset written precisely to catch the arc-crossing, hue-mean and
    /// rotation-term mistakes a plausible-looking implementation makes. Each
    /// pair is a published input and a published answer, neither derived
    /// from this module.
    ///
    #[test]
    fn the_colour_difference_matches_the_published_test_vectors() {
        for (first, second, expected) in [
            ([50.0, 2.6772, -79.7751], [50.0, 0.0, -82.7485], 2.0425),
            ([50.0, 3.1571, -77.2803], [50.0, 0.0, -82.7485], 2.8615),
            ([50.0, 2.8361, -74.0200], [50.0, 0.0, -82.7485], 3.4412),
            ([50.0, -1.3802, -84.2814], [50.0, 0.0, -82.7485], 1.0000),
            ([50.0, 2.5, 0.0], [50.0, 0.0, -2.5], 4.3065),
            // One colour achromatic: the zero-chroma-product branches.
            ([50.0, 0.0, 0.0], [50.0, -1.0, 2.0], 2.3669),
            // Hue difference either side of 180°, which moves the hue mean.
            ([50.0, 2.49, -0.001], [50.0, -2.49, 0.0009], 7.1792),
            ([50.0, 2.49, -0.001], [50.0, -2.49, 0.0011], 7.2195),
            (
                [60.2574, -34.0099, 36.2677],
                [60.4626, -34.1751, 39.4387],
                1.2644,
            ),
        ] {
            let measured = difference(first, second);
            assert!(
                (measured - expected).abs() < 0.002,
                "{first:?} vs {second:?}: got {measured:.4}, published {expected:.4}"
            );
            assert!(
                (difference(second, first) - expected).abs() < 0.002,
                "CIEDE2000 is symmetric"
            );
        }
    }

    ///
    /// The two properties that follow from a dichromacy simulation being a
    /// projection onto a plane, checked rather than assumed, because a
    /// transposed or mistyped matrix coefficient breaks both while still
    /// producing plausible-looking colours.
    ///
    /// Neutral greys carry no chromatic signal, so every dichromat sees them
    /// unchanged; and projecting an already-projected colour changes
    /// nothing, so the transform is idempotent. Both hold here only up to
    /// 8-bit re-encoding, which is what the tolerances allow for.
    ///
    #[test]
    fn simulation_leaves_neutrals_alone() {
        for vision in ColourVision::ALL {
            for level in [0u8, 1, 17, 64, 128, 191, 254, 255] {
                let neutral = Color32::from_gray(level);
                let seen = simulate(neutral, vision);
                for (channel, value) in [("r", seen.r()), ("g", seen.g()), ("b", seen.b())] {
                    assert!(
                        value.abs_diff(level) <= 1,
                        "{vision}: gray {level} moved to {value} on {channel}"
                    );
                }
            }
        }
    }

    #[test]
    fn simulating_a_dichromacy_is_idempotent() {
        // A deterministic spread over the cube rather than a random sample:
        // this is a property of the arithmetic, so it should be checked at
        // the same inputs every run.
        for red in (0..=255).step_by(51) {
            for green in (0..=255).step_by(51) {
                for blue in (0..=255).step_by(51) {
                    let colour = Color32::from_rgb(red as u8, green as u8, blue as u8);
                    for vision in ColourVision::ALL {
                        let once = simulate(colour, vision);
                        let twice = simulate(once, vision);
                        for (channel, first, second) in [
                            ("r", once.r(), twice.r()),
                            ("g", once.g(), twice.g()),
                            ("b", once.b(), twice.b()),
                        ] {
                            assert!(
                                first.abs_diff(second) <= 3,
                                "{vision} on {colour:?}: {channel} {first} then {second}"
                            );
                        }
                    }
                }
            }
        }
    }

    ///
    /// Each dichromacy removes the axis it is named for and leaves the other
    /// one, which is what makes `simulate` a simulation rather than merely a
    /// colour transform. Measured as the fraction of a pair's normal-vision
    /// separation that survives, so the claim is about the axis rather than
    /// about any absolute figure these four particular colours happen to
    /// have: a red–green dichromacy must lose most of the red-against-green
    /// separation and keep essentially all of the blue-against-yellow one,
    /// and tritanopia must do neither.
    ///
    /// A red and a green at *different* lightness stay far apart under
    /// protanopia even so, because lightness survives every dichromacy —
    /// which is exactly why `.scratch/theming/issues/04` had to separate the
    /// light Theme's Diagnostic from its Output Portal by lightness, and why
    /// this test measures the collapse as a ratio rather than asserting the
    /// remainder falls under [`CONFUSION_FLOOR`].
    ///
    #[test]
    fn each_dichromacy_removes_its_own_axis_and_leaves_the_other() {
        let red = Color32::from_rgb(200, 30, 30);
        let green = Color32::from_rgb(30, 160, 30);
        let blue = Color32::from_rgb(30, 30, 200);
        let yellow = Color32::from_rgb(220, 210, 40);

        let apart = |first, second, vision| {
            difference(lab(simulate(first, vision)), lab(simulate(second, vision)))
        };
        let red_green = difference(lab(red), lab(green));
        let blue_yellow = difference(lab(blue), lab(yellow));

        for vision in [ColourVision::Protanopia, ColourVision::Deuteranopia] {
            let surviving = apart(red, green, vision) / red_green;
            assert!(
                surviving < 0.5,
                "{vision} should lose most of red against green, {surviving:.2} survived"
            );
            let kept = apart(blue, yellow, vision) / blue_yellow;
            assert!(
                (0.9..1.1).contains(&kept),
                "{vision} should keep blue against yellow, {kept:.2} survived"
            );
        }

        let vision = ColourVision::Tritanopia;
        let kept = apart(red, green, vision) / red_green;
        assert!(
            (0.9..1.1).contains(&kept),
            "{vision} should keep red against green, {kept:.2} survived"
        );
    }

    #[test]
    fn distinguish_reports_the_floor_and_scope_in_the_returned_data() {
        let report = distinguish(&okabe_ito());
        assert_eq!(report.floor, CONFUSION_FLOOR);
        assert!(
            report.scope.contains("not gated"),
            "the report must carry its own scope, not only rustdoc: {:?}",
            report.scope
        );
    }

    ///
    /// Every unordered pair of the eight glyph channels, once per
    /// dichromacy, and nothing else: `distinguish` reports the minimum over
    /// placements rather than one row per placement, and never pairs a
    /// channel with itself.
    ///
    #[test]
    fn distinguish_reports_one_result_per_channel_pair_and_dichromacy() {
        let report = distinguish(&orcvs_light());
        // 8 channels choose 2 = 28 pairs, times 3 dichromacies. Stated as
        // one literal rather than recomputed from the same lengths the code
        // iterates.
        assert_eq!(report.results.len(), 84);
        for result in &report.results {
            assert_ne!(result.first, result.second);
        }
    }

    ///
    /// The validator against a Theme built to confuse — proof `distinguish`
    /// finds a confusion rather than reporting every pair as separated. Two
    /// channels set to the *same* colour must measure zero, and a pair left
    /// alone must still pass, so the failure is specific to the retune.
    ///
    #[test]
    fn distinguish_reports_a_theme_built_to_confuse_rather_than_a_vacuous_pass() {
        let base = okabe_ito();
        let theme = Theme {
            source_bang: base.source_function,
            source_bang_background: base.source_function_background,
            ..base
        };

        let report = distinguish(&theme);
        let confused = report
            .results
            .iter()
            .find(|result| {
                result.vision == ColourVision::Deuteranopia
                    && [result.first, result.second].iter().all(|channel| {
                        matches!(channel, GlyphChannel::Bang | GlyphChannel::Function)
                    })
            })
            .expect("Bang against Function is a reported pair");
        assert!(
            confused.distance < 0.01,
            "identical colours must measure zero, got {:.4}",
            confused.distance
        );
        assert!(!confused.passes());

        let untouched = report
            .results
            .iter()
            .find(|result| {
                result.vision == ColourVision::Deuteranopia
                    && [result.first, result.second].iter().all(|channel| {
                        matches!(channel, GlyphChannel::Ordinary | GlyphChannel::Number)
                    })
            })
            .expect("Ordinary against Number is a reported pair");
        assert!(
            untouched.passes(),
            "a pair the fixture did not touch must still pass: {:.2}",
            untouched.distance
        );
    }

    ///
    /// The glyph definition this gate rejected, as a regression: the light
    /// Theme's own colours before the user's 2026-09-23 decision, which took
    /// their hues from the `feat/egui-theming` proposal. It clears
    /// [`CONTRAST_FLOOR`] in all 84 painted states the report measured then
    /// — `.scratch/theming/issues/04` tuned it against [`validate`] until it
    /// did — and still fails here, which is the whole reason this second
    /// measurement exists.
    ///
    /// Its worst gated pair is Diagnostic against Output Portal under
    /// protanopia, at 0.70: a rust `#A34A00` and a gold `#7A5200` that a
    /// protanope sees as one colour. Its Note against Number under
    /// deuteranopia is barely better, 2.16. Both are pairs no contrast
    /// measurement can see, because each colour passes the floor against its
    /// own background.
    ///
    #[test]
    fn the_rejected_light_glyph_definition_fails_this_gate() {
        let rejected = rejected_light_glyphs();

        let contrast_report = validate(&rejected);
        assert!(
            unaccepted(&contrast_report).is_empty(),
            "the rejected definition cleared the contrast floor; if it no longer does, this \
             fixture is no longer the thing that motivated a second measurement"
        );

        let report = distinguish(&rejected);
        let worst = closest(&report, ColourVision::Protanopia);
        assert!(
            !worst.passes(),
            "the rejected definition's closest protanopia pair is {} against {} at {:.2}",
            worst.first,
            worst.second,
            worst.distance
        );
        assert!(
            (worst.distance - 0.70).abs() < 0.05,
            "recorded at 0.70, measured {:.2} ({} against {})",
            worst.distance,
            worst.first,
            worst.second
        );
    }

    ///
    /// Why this gate measures a simulated colour difference rather than the
    /// relative luminance the review that prompted it reached for. The
    /// rejected definition's Function `#077055` and Bang `#AD2A3B` are
    /// within 1.09:1 of each other in relative luminance — on a greyscale
    /// display they are one tone — and a dichromat still separates them
    /// easily, because every dichromacy keeps lightness *and* one chromatic
    /// axis. Equal luminance is therefore a fact about those two colours,
    /// not a verdict on them: the pair measures 14.60 under deuteranopia,
    /// nearly three times [`CONFUSION_FLOOR`].
    ///
    /// The corollary is the one `.scratch/theming/issues/04` acted on: since
    /// lightness survives every dichromacy, lightness is what separates two
    /// colours a dichromacy would otherwise merge — which is why Orcvs
    /// Light's Diagnostic and Output Portal are darkened past what contrast
    /// alone asks for.
    ///
    #[test]
    fn equal_luminance_alone_does_not_decide_a_colour_vision_confusion() {
        let rejected = rejected_light_glyphs();
        let (function, bang) = (rejected.source_function, rejected.source_bang);

        let luminance_ratio = contrast(function, bang);
        assert!(
            luminance_ratio < 1.1,
            "the two are meant to be one tone, got {luminance_ratio:.2}:1"
        );

        let separation = difference(
            lab(simulate(function, ColourVision::Deuteranopia)),
            lab(simulate(bang, ColourVision::Deuteranopia)),
        );
        assert!(
            separation > 2.5 * CONFUSION_FLOOR,
            "one tone, and still far apart to a deuteranope: got {separation:.2}"
        );
    }

    ///
    /// Orcvs Light's glyph channels as they stood before the user's
    /// 2026-09-23 decision: the `feat/egui-theming` proposal's hues, with
    /// `.scratch/theming/issues/04`'s own three darkenings applied and its
    /// four decided-here values. Only the glyph channels differ from the
    /// shipped light Theme — the chrome keys were kept by that decision, so
    /// holding them fixed here is what isolates the glyphs.
    ///
    fn rejected_light_glyphs() -> Theme {
        let tint =
            |[red, green, blue]: [u8; 3]| Color32::from_rgba_unmultiplied(red, green, blue, 0x1A);
        let ([number, note, function, bang, sequence, diagnostic, portal], base) = (
            [
                [0x35, 0x64, 0xA0],
                [0x75, 0x53, 0xA2],
                [0x07, 0x70, 0x55],
                [0xAD, 0x2A, 0x3B],
                [0x12, 0x38, 0x6B],
                [0xA3, 0x4A, 0x00],
                [0x7A, 0x52, 0x00],
            ],
            orcvs_light(),
        );
        let opaque = |[red, green, blue]: [u8; 3]| Color32::from_rgb(red, green, blue);
        Theme {
            source_number: opaque(number),
            source_number_background: tint(number),
            source_note: opaque(note),
            source_note_background: tint(note),
            source_function: opaque(function),
            source_function_background: tint(function),
            source_bang: opaque(bang),
            source_sequence: opaque(sequence),
            source_sequence_background: tint(sequence),
            diagnostic_foreground: opaque(diagnostic),
            output_portal_foreground: opaque(portal),
            output_portal_background: tint(portal),
            error: opaque(bang),
            warning: opaque(bang),
            ..base
        }
    }

    ///
    /// The shipped-Theme colour-vision gate, beside `shipped_theme_gate`.
    /// Every gated dichromacy's every channel pair clears
    /// [`CONFUSION_FLOOR`] for both built-ins, with no exception list: the
    /// dark built-in is the published Okabe–Ito assignment and the light one
    /// was re-picked from it under `.scratch/theming/issues/04` until it
    /// did.
    ///
    /// Tritanopia is asserted separately and much lower, at the figures both
    /// built-ins actually reach rather than at the floor, because
    /// [`ColourVision::gated`] says why no palette on this page clears the
    /// floor there. Pinning the real numbers is what keeps that a checked
    /// boundary: if a retune makes tritanopia worse, this fails and
    /// `console/src/theme.md`'s colour-vision table has to move with it.
    ///
    #[test]
    fn shipped_theme_colour_vision_gate() {
        for theme in [okabe_ito(), orcvs_light()] {
            let report = distinguish(&theme);
            let failures: Vec<_> = report
                .results
                .iter()
                .filter(|result| result.vision.gated() && !result.passes())
                .collect();
            assert!(
                failures.is_empty(),
                "{}'s confusable glyph channels: {failures:?}",
                theme.identity
            );

            let tritan = closest(&report, ColourVision::Tritanopia);
            let recorded = if theme.identity == ORCVS_LIGHT_IDENTITY {
                1.54
            } else {
                0.60
            };
            assert!(
                (tritan.distance - recorded).abs() < 0.05,
                "{}'s closest tritanopia pair is {} against {} at {:.2}, recorded as {recorded:.2}",
                theme.identity,
                tritan.first,
                tritan.second,
                tritan.distance
            );
        }
    }

    ///
    /// ADR 0053 lets a Theme carry Diagnostic and Output Portal on the
    /// background alone, by making their foregrounds transparent; the glyph
    /// then keeps the colour of the Token under it. `distinguish` must
    /// compare the channel actually painted, or it measures a Number,
    /// Invalid glyph drawn in `source.number` against Number, Valid in the
    /// same colour and reports two different meanings at a distance of 0.
    ///
    #[test]
    fn a_transparent_fact_foreground_is_measured_as_the_token_it_reveals() {
        let theme = Theme {
            diagnostic_foreground: Color32::TRANSPARENT,
            output_portal_foreground: Color32::TRANSPARENT,
            ..okabe_ito()
        };
        let report = distinguish(&theme);

        let drawn_in: Vec<_> = report
            .results
            .iter()
            .flat_map(|result| [result.first, result.second])
            .filter(|channel| {
                matches!(
                    channel,
                    GlyphChannel::Diagnostic | GlyphChannel::OutputPortal
                )
            })
            .collect();
        assert!(
            drawn_in.is_empty(),
            "no glyph is drawn in a transparent channel, yet {drawn_in:?} were measured"
        );
        let failures: Vec<_> = report
            .results
            .iter()
            .filter(|result| result.vision.gated() && !result.passes())
            .collect();
        assert!(failures.is_empty(), "{failures:?}");
    }

    ///
    /// The figures `console/src/theme.md` publishes for the two built-ins'
    /// closest red–green pairs, pinned so the document and the code move in
    /// one commit. The dark built-in's worst is what
    /// [`CONFUSION_FLOOR`]'s own documentation cites as the reason 5.0 is
    /// not a floor fitted to the light Theme.
    ///
    #[test]
    fn the_recorded_red_green_separations_are_what_the_built_ins_measure() {
        for (theme, protanopia, deuteranopia) in
            [(okabe_ito(), 14.16, 6.65), (orcvs_light(), 9.20, 7.19)]
        {
            let report = distinguish(&theme);
            for (vision, recorded) in [
                (ColourVision::Protanopia, protanopia),
                (ColourVision::Deuteranopia, deuteranopia),
            ] {
                let worst = closest(&report, vision);
                assert!(
                    (worst.distance - recorded).abs() < 0.05,
                    "{}'s closest {vision} pair is {} ({:?}) against {} ({:?}) at {:.2}, \
                     theme.md records {recorded:.2}",
                    theme.identity,
                    worst.first,
                    worst.first_colour,
                    worst.second,
                    worst.second_colour,
                    worst.distance
                );
            }
        }
    }
}
