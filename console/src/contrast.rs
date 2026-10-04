//!
//! Validates a resolved Theme's *composited* text contrast: the effective
//! foreground and background each reachable painted text state actually
//! displays, not raw Theme colour pairs read in isolation.
//!
//! [`painted`] calls [`crate::style::cell_visuals_with_cursor_colour`] and
//! [`crate::style::cell_background`] — the same functions
//! [`crate::paint::Paint::derive_with_theme`] calls per Cell — for every
//! Source Grid colour operation, so the two cannot independently drift.
//! [`chrome`] does the same for the console's chrome: it reads every colour out of
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
//! Pairwise colour-vision separation of the Source glyph channels is a
//! test-only check on the built-in Themes, in the `colour_vision` child
//! module. It is not applied to a loaded Theme document.
//!
//! It does not account for the Cursor Effect's animated `area` field
//! (`theme.cursor_area`), which `console::shapes`'s `SourceShapes::into_shapes`
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
/// Number and Note admit Valid and Invalid.
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

impl Role {
    /// Every Source Grid role [`validate`] measures, in this enum's
    /// declaration order.
    const SOURCE_ROLES: [Self; 8] = [
        Self::Ordinary,
        Self::Function,
        Self::Bang,
        Self::Comment,
        Self::NumberValid,
        Self::NumberInvalid,
        Self::NoteValid,
        Self::NoteInvalid,
    ];

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
/// also lies in a value Function's Output Portal Reservation: the two are
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
/// backdrop: the console's own paint order draws `window.background` as the
/// `eframe` clear colour first, `grid.background` as the Source panel's fill
/// over it, then each Cell's own composited fill on top
/// (`console::source_view::source_panel_frame`, `Console::clear_color`). A translucent
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
/// for a button's fill and text colour; [`egui::Style::widget_style`],
/// which `Checkbox` paints its label from; `Visuals::text_color`,
/// `weak_text_color` and `text_edit_bg_color`; and `Visuals::selection`, which
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
/// `SubMenuButton::ui` paints one: it swaps `widgets.open` in for
/// `widgets.inactive` while its button draws.
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
    /// The Theme's own effective foreground before compositing, which may be
    /// translucent. [`Self::ratio`] measures it composited over
    /// [`Self::background`]. Do not store the composited colour here:
    /// accepted exceptions match on this exact colour, and a composited one
    /// no longer matches them.
    pub(crate) foreground: Color32,
    /// The opaque surface the text is painted on, resolved down to the
    /// window backdrop.
    pub(crate) background: Color32,
    pub(crate) ratio: f32,
    /// Whether this exact `(role, state, foreground, background)` is recorded
    /// as an explicitly accepted exception. Never turns [`Self::passes`] into
    /// `true`: acceptance annotates a failure, it does not hide or pass it.
    /// Keyed on the measured colour pair as well as role/state, so a retune
    /// that changes `foreground`/`background` while keeping the same
    /// role/state label does not stay silently accepted.
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
pub(crate) struct ContrastReport {
    pub(crate) floor: f32,
    /// This module's own `# Scope` section, restated as data a caller can
    /// show beside the report rather than only read in source.
    // On wasm the whole report is unreachable, which `lib.rs`'s expectation
    // on `mod contrast` covers, so this one is native-only.
    #[cfg_attr(
        all(not(test), not(target_arch = "wasm32")),
        expect(
            dead_code,
            reason = "the report carries its scope for any caller to show, and the one shipped \
                      caller, the Theme notice, names the floor and the failing states only"
        )
    )]
    pub(crate) scope: &'static str,
    pub(crate) results: Vec<ContrastResult>,
}

/// [`ContrastReport::scope`]'s exact text.
const SCOPE: &str = "Text contrast only: an effective foreground against the effective \
background it is actually painted on. Not pairwise Token-colour distinguishability, \
border/focus visibility, or the Cursor Effect's animated area field. Chrome leaves out \
disabled widgets (faded by egui, and exempt as inactive components under WCAG 2.1 SC 1.4.3), \
code.background (no code span is painted), a widget state's strong fill (egui paints no text \
on it), a popup or window floating over another surface, the Source Grid or a panel \
(measured as one panel over the window backdrop; the two differ only for a translucent \
panel.background), and text.active on the open widget fill (egui paints an open widget's text \
in widgets.open.fg_stroke, which the Theme maps from text, so the open fill is measured with \
text instead).";

///
/// Measures every reachable painted text state against `theme` and reports
/// each one's role, state, effective foreground and background, measured
/// WCAG ratio, and whether it is an explicitly accepted exception.
///
/// Never refuses `theme`: a scheme that fails a state is measured and
/// reported exactly like one that passes, because the floor is a fact about
/// the scheme worth seeing rather than a gate a Theme must clear before it
/// loads. `crate::theme_registry` shows the states below the floor as a
/// Theme notice when a custom Theme loads; a failing one loads anyway,
/// because the viewer chose it.
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
/// One below-floor `(role, state)` result recorded as explicitly accepted
/// for a shipped Theme, at the exact colour pair it was accepted at.
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
/// Okabe–Ito's list is empty: it has no reachable painted failure
/// (`tests::okabe_ito_has_nothing_to_except` pins that).
///
/// Called from [`measure`], which every [`validate`] result passes through —
/// not test-only: `theme_registry` validates every Theme file it loads.
/// `ContrastResult::accepted` has to come from
/// somewhere for the report to carry its acceptance annotation in the
/// returned data rather than only in test code, and this table is that
/// somewhere. `tests::unaccepted`, the shipped-Theme *gate*'s comparison, has
/// no such production reason and is test code.
///
fn accepted_failures(identity: &ThemeIdentity) -> &'static [AcceptedFailure] {
    match identity {
        id if *id == OKABE_ITO_IDENTITY => &[],
        // Orcvs Light's list is empty too: every reachable state clears the
        // floor, so it has no exception to record.
        // `console/src/theme.md` states the measured figures.
        id if *id == ORCVS_LIGHT_IDENTITY => &[],
        _ => &[],
    }
}

#[cfg(test)]
mod colour_vision;

#[cfg(test)]
mod tests {
    use egui::Color32;
    use orcvs::source::SourcePaint;

    use super::{
        AcceptedFailure, ContrastReport, ContrastResult, CursorPlacement, Role, State, contrast,
        painted, validate,
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
    /// — the shipped-Theme gate's comparison. A plain filter
    /// over `report.results` rather than a separately parameterized
    /// function: acceptance is already baked into each `ContrastResult` by
    /// `validate`, so there is nothing left for a test-only helper to do but
    /// filter.
    pub(super) fn unaccepted(report: &ContrastReport) -> Vec<&ContrastResult> {
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
        // Every chrome case left out is named in the returned scope, not only
        // in rustdoc.
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
        // 8 Source Grid roles x 4 CursorPlacements x 2 output_portal states
        // = 64, plus 15 chrome states: text on panel, input and the
        // inactive and open fills (4); text.muted on panel and input (2);
        // text.active on the hovered and active fills, framed and frameless
        // (4); selected text on selection.background over panel and input
        // (2); error, warning and link on panel (3). Stated as one literal
        // rather than recomputed from the same lengths `validate` sizes its
        // `Vec` from, so a change to either count is caught by an
        // independent number.
        assert_eq!(report.results.len(), 79);
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

    // === Chrome states beyond text/text.muted on panel and input ===

    /// Asserts `result` measures `foreground` against itself — 1:1 — and so
    /// fails, and that every Source Grid state and `text`/`text.muted` on
    /// panel and input still passes, so the failure is one only the chrome
    /// state under test can see.
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
    /// input stay untouched.
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
    /// reachable state (a Function standing inside another's Output Portal
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
    /// (`source.number.background`) is translucent but nonzero-alpha, so its
    /// `Region` state equals its `plain` state exactly, while `Comment`'s
    /// fully transparent background lets the Region tint show once
    /// `region.background` is not itself transparent. The rule is about zero
    /// versus nonzero alpha, not opaque versus translucent.
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
    /// is a translucent 10% tint, so this fixture sets
    /// `source_number_background` explicitly opaque to exercise the claim the
    /// test's name makes, rather than relying on a shipped Theme to happen to
    /// have one.
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
            role: Role::NoteInvalid,
            state,
            foreground: Color32::from_rgb(1, 2, 3),
            background: Color32::from_rgb(4, 5, 6),
        };

        assert!(entry.matches(Role::NoteInvalid, state, entry.foreground, entry.background));
        assert!(
            !entry.matches(
                Role::NoteInvalid,
                state,
                Color32::from_rgb(9, 9, 9),
                entry.background
            ),
            "a retuned foreground must not match a stale accepted entry"
        );
        assert!(
            !entry.matches(
                Role::NoteInvalid,
                state,
                entry.foreground,
                Color32::from_rgb(9, 9, 9)
            ),
            "a retuned background must not match a stale accepted entry"
        );
    }

    ///
    /// Okabe–Ito's accepted-exception list is empty for the stronger of the
    /// two possible reasons: nothing fails. This pins that premise separately
    /// from `shipped_theme_gate`, which would also pass on a list full of
    /// exceptions.
    ///
    #[test]
    fn okabe_ito_has_nothing_to_except() {
        let report = validate(&okabe_ito());

        assert!(
            super::accepted_failures(&OKABE_ITO_IDENTITY).is_empty(),
            "Okabe-Ito ships with no accepted contrast exception"
        );
        let below_floor: Vec<_> = report
            .results
            .iter()
            .filter(|result| !result.passes())
            .collect();
        assert!(
            below_floor.is_empty(),
            "Okabe-Ito must clear the floor unaided: {below_floor:?}"
        );
    }

    ///
    /// Orcvs Light's accepted-exception list is empty for the stronger of
    /// the two possible reasons: nothing fails. This pins that premise
    /// separately from `shipped_theme_gate`, which would also pass on a list full of
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
    /// `#[ignore]`d. Every tinted role background is a uniform 10% opacity
    /// (that role's own foreground colour at alpha `0x1A`), and every
    /// measured ratio is at or above the 4.5:1 floor through this real
    /// shipped composition — `console/src/theme.md` records the exact
    /// figures. There is therefore nothing to except: `accepted_failures`
    /// for `okabe-ito` is empty, and this test's own run is what proves that
    /// emptiness is correct rather than merely convenient.
    ///
    /// `orcvs_light()` clears the floor in every reachable state the same
    /// way rather than by exception. `shipped` is a hand-kept array rather
    /// than the Theme registry's built-ins, which are private to
    /// `crate::theme_registry`: a built-in added there is not gated until it
    /// is added here too.
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
}
