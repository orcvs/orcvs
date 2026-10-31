//!
//! Validates a resolved Theme's *composited* text contrast: the effective
//! foreground and background each reachable painted text state actually
//! displays, not raw Theme colour pairs read in isolation.
//!
//! [`painted`] asks [`crate::style::CellColours::colours`] — the decision
//! [`crate::paint::Paint::derive_with_theme`] asks per Cell — for every
//! Source Grid colour, so the two cannot independently drift.
//! [`chrome`] does the same for the console's chrome: it reads every colour out of
//! [`crate::style::style`], the `Style` [`crate::style::install`] registers,
//! from the `Visuals` fields egui's default widget styles paint from, and
//! composites each fill over
//! the surface it is painted on with [`egui::Color32::blend`], the same
//! primitive every reused function above is itself built from.
//!
//! # Scope
//!
//! [`validate`] measures text contrast only: an effective foreground against
//! the effective background it is actually painted on. It does not measure
//! pairwise Token-colour distinguishability or border/focus visibility. A
//! Source Grid state whose glyph is fully transparent, such as one a
//! transparent `cursor.foreground` hides, draws no text and is not measured.
//!
//! Its chrome states are `text`, `text.muted`, `text.active`, selected
//! text, `error`, `warning` and `link`, each on the surfaces egui paints it
//! on (see [`State`]), and a Panel button's glyph over its own fill, at rest
//! and pressed. Five chrome cases are left out, each for a reason:
//!
//! - A disabled widget: egui fades it to half opacity, and WCAG 2.1 SC 1.4.3
//!   exempts the text of an inactive user-interface component.
//! - `code.background`: nothing in the console paints a code span.
//! - A state's strong `bg_fill`: egui 0.37.0 paints no text on one — it is a
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
//! Comment is measured in every state like any Source role, but a Comment
//! below the floor does not fail ([`Role::held_to_floor`]).
//!
//! Pairwise colour-vision separation of the Source glyph channels is a
//! test-only check on the built-in Themes, in the `colour_vision` child
//! module. It is not applied to a loaded Theme document.
//! The bar edge's separation from the Grid's lines is likewise a test-only
//! check on the built-ins.

use std::fmt;

use egui::Color32;
use egui::style::WidgetVisuals;
use orcvs::render_frame::{PortalRole, PortalRoles};
use orcvs::source::{OperandState, SourcePaint, Token};

use crate::style::{CellColours, CursorPlacement, style};
use crate::theme::Theme;

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
    /// A Panel button's glyph at rest, `button.foreground`.
    Button,
    /// A pressed Panel button's glyph, `button.pressed.foreground`.
    ButtonPressed,
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

    ///
    /// Whether a state of this role below [`CONTRAST_FLOOR`] fails. Comment is
    /// measured and reported like every role, but is not held to the floor: a
    /// Theme sets it apart from the code around it by drawing it faint, so the
    /// floor would take away the contrast that marks it as a Comment.
    ///
    pub(crate) const fn held_to_floor(self) -> bool {
        !matches!(self, Self::Comment)
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
            Self::Text
            | Self::TextMuted
            | Self::TextActive
            | Self::SelectionBorder
            | Self::Error
            | Self::Warning
            | Self::Link
            | Self::Button
            | Self::ButtonPressed => None,
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
            Self::Button => "button.foreground",
            Self::ButtonPressed => "button.pressed.foreground",
        })
    }
}

impl CursorPlacement {
    const ALL: [Self; 4] = [Self::Plain, Self::Cursor, Self::Region, Self::RegionCursor];
}

///
/// Every set of Portal roles a Cell can play: none, Input Portal alone,
/// Output Portal alone, and both, each with and without Locked.
///
pub(crate) const PORTAL_ROLE_SETS: [PortalRoles; 8] = {
    const INPUT: PortalRoles = PortalRoles::NONE.with(PortalRole::Input);
    const OUTPUT: PortalRoles = PortalRoles::NONE.with(PortalRole::Output);
    const BOTH: PortalRoles = OUTPUT.with(PortalRole::Input);
    [
        PortalRoles::NONE,
        INPUT,
        OUTPUT,
        BOTH,
        PortalRoles::NONE.with(PortalRole::Locked),
        INPUT.with(PortalRole::Locked),
        OUTPUT.with(PortalRole::Locked),
        BOTH.with(PortalRole::Locked),
    ]
};

///
/// Every set of Portal roles a Cell carrying `fact` can play:
/// [`PORTAL_ROLE_SETS`], less a set holding Locked unless `fact` is a
/// Function's own spelling. A lock marks only the two Cells of the Function of
/// the Expression root it withheld, a Bang Function's among them. Each set is
/// measured, including those the precedence rule paints alike, so that the
/// matrix covers every set a Cell can carry rather than the sets that rule
/// happens to tell apart.
///
pub(super) fn reachable_portal_sets(fact: SourcePaint) -> impl Iterator<Item = PortalRoles> {
    PORTAL_ROLE_SETS.into_iter().filter(move |portals| {
        !portals.contains(PortalRole::Locked)
            || matches!(fact, SourcePaint::Function | SourcePaint::Bang)
    })
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
/// [`Self::SourceGrid`] crosses [`CursorPlacement`] with the Portal roles the
/// Cell plays: the two are
/// independent — the Cursor can sit on a Cell a Portal covers, and a
/// Region can span over one, per `.scratch/theming/schema.md`'s own Overlap
/// rule ("covers every Cell of the Reservation whatever else claims it") and
/// its Cursor-precedence exception ("The Cursor's own fill still wins
/// outright over everything above, on its own Cell").
///
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum State {
    SourceGrid {
        cursor: CursorPlacement,
        portals: PortalRoles,
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
    /// `button.background` over the panel: a Panel Cell button at rest.
    Button,
    /// `button.pressed.background` over the panel: a pressed Cell button.
    ButtonPressed,
}

impl fmt::Display for State {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SourceGrid { cursor, portals } => {
                match (
                    portals.contains(PortalRole::Output),
                    portals.contains(PortalRole::Input),
                ) {
                    (false, false) => write!(f, "{cursor}")?,
                    (true, false) => write!(f, "{cursor}, Output Portal")?,
                    (false, true) => write!(f, "{cursor}, Input Portal")?,
                    (true, true) => write!(f, "{cursor}, Output and Input Portal")?,
                }
                if portals.contains(PortalRole::Locked) {
                    f.write_str(", Locked")?;
                }
                Ok(())
            }
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
            Self::Button => f.write_str("vs button.background over panel.background"),
            Self::ButtonPressed => {
                f.write_str("vs button.pressed.background over panel.background")
            }
        }
    }
}

///
/// The effective foreground and *opaque* background a `(fact, cursor,
/// portals)` triple actually paints, composited exactly as
/// [`crate::paint::Paint::derive_with_theme`] composes one Cell — the same
/// decision, standalone rather than read off one Cell of a whole Render
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
    portals: PortalRoles,
    theme: &Theme,
) -> (Color32, Color32) {
    let displayed = CellColours::new(theme).displayed(fact, portals, true, cursor);

    (displayed.foreground, displayed.background)
}

/// How many console-chrome states [`chrome`] measures.
const CHROME_STATES: usize = 17;

///
/// Every console-chrome text state, as the effective foreground and opaque
/// background egui paints it for `theme`.
///
/// Every colour is read out of [`crate::style::style`] — the `Style`
/// [`crate::style::install`] registers for both egui appearance slots — from
/// the `Visuals` fields egui's default widget styles paint from
/// (`egui-0.37.0/src/theme/default_style.rs`): a button's text is its state's
/// [`egui::style::WidgetVisuals::text_color`] on the state's `weak_bg_fill`;
/// a selected button swaps in `Visuals::selection`'s stroke colour and fill;
/// a checkbox's frameless label is its state's `text_color`;
/// `Visuals::text_color`, `weak_text_color` and `text_edit_bg_color` are read
/// as they are; and `Visuals::selection` is what
/// `text_selection::paint_text_selection` recolours selected glyphs and fills
/// their background from. A `style` mapping that changes which Theme key a
/// `Visuals` field reads therefore changes what this measures with it.
///
/// Every text-bearing widget fill in egui 0.37.0 is the state's
/// `weak_bg_fill` — a button's frame, `ComboBox`, and the active window's
/// title bar alike; a state's strong `bg_fill` carries no text (a checkbox's
/// box, a slider's rail, a colour swatch), which is why only the weak fill is
/// measured. An open widget is measured the way `SubMenuButton::ui` paints
/// one: it swaps `widgets.open` in for `widgets.inactive` while its button
/// draws.
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
    let widgets = &visuals.widgets;

    let panel = theme.window_background.blend(visuals.panel_fill);
    let input = panel.blend(visuals.text_edit_bg_color());
    let button = |state: &WidgetVisuals| (state.text_color(), panel.blend(state.weak_bg_fill));

    let (inactive_text, inactive_fill) = button(&widgets.inactive);
    let (hovered_text, hovered_fill) = button(&widgets.hovered);
    let (active_text, active_fill) = button(&widgets.active);
    let (open_text, open_fill) = button(&widgets.open);
    let (selected_text, selected_fill) = (
        visuals.selection.stroke.color,
        panel.blend(visuals.selection.bg_fill),
    );
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
            widgets.hovered.text_color(),
            panel,
        ),
        (
            Role::TextActive,
            State::ActiveFrameless,
            widgets.active.text_color(),
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
        (
            Role::Button,
            State::Button,
            theme.button_foreground,
            panel.blend(theme.button_background),
        ),
        (
            Role::ButtonPressed,
            State::ButtonPressed,
            theme.button_pressed_foreground,
            panel.blend(theme.button_pressed_background),
        ),
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
    /// [`Self::background`].
    pub(crate) foreground: Color32,
    /// The opaque surface the text is painted on, resolved down to the
    /// window backdrop.
    pub(crate) background: Color32,
    pub(crate) ratio: f32,
}

impl ContrastResult {
    /// Whether [`Self::ratio`] clears [`CONTRAST_FLOOR`].
    pub(crate) fn passes(&self) -> bool {
        self.ratio >= CONTRAST_FLOOR
    }

    ///
    /// Whether this state counts against the Theme: below the floor, and of
    /// a role [held to it](Role::held_to_floor).
    ///
    pub(crate) fn fails(&self) -> bool {
        !self.passes() && self.role.held_to_floor()
    }
}

///
/// [`validate`]'s complete answer for one Theme: the floor every
/// [`ContrastResult`] is measured against, plus the results.
///
pub(crate) struct ContrastReport {
    pub(crate) floor: f32,
    pub(crate) results: Vec<ContrastResult>,
}

///
/// Measures every reachable painted text state against `theme` and reports
/// each one's role, state, effective foreground and background, measured
/// WCAG ratio.
///
/// Never refuses `theme`: a scheme that fails a state is measured and
/// reported exactly like one that passes, because the floor is a fact about
/// the scheme worth seeing rather than a gate a Theme must clear before it
/// loads. `crate::theme_registry` shows the states below the floor as a
/// Theme notice when a custom Theme loads; a failing one loads anyway,
/// because the viewer chose it.
///
pub(crate) fn validate(theme: &Theme) -> ContrastReport {
    let source_states: usize = Role::SOURCE_ROLES
        .iter()
        .filter_map(|role| role.source_paint_fact())
        .map(|fact| reachable_portal_sets(fact).count())
        .sum();
    let mut results =
        Vec::with_capacity(source_states * CursorPlacement::ALL.len() + CHROME_STATES);

    for &role in &Role::SOURCE_ROLES {
        let fact = role
            .source_paint_fact()
            .expect("Role::SOURCE_ROLES holds only roles with a Source Paint fact");
        for cursor in CursorPlacement::ALL {
            for portals in reachable_portal_sets(fact) {
                let state = State::SourceGrid { cursor, portals };
                let (foreground, background) = painted(fact, cursor, portals, theme);
                // A fully transparent glyph draws nothing, so there is no
                // text to measure: a hidden Cursor glyph would otherwise
                // report 1:1 against its own background.
                if foreground.a() == 0 {
                    continue;
                }
                results.push(measure(role, state, foreground, background));
            }
        }
    }

    for (role, state, foreground, background) in chrome(theme) {
        results.push(measure(role, state, foreground, background));
    }

    ContrastReport {
        floor: CONTRAST_FLOOR,
        results,
    }
}

///
/// Composites `foreground` over the already-opaque `background` and measures
/// [`contrast`] between the two.
///
fn measure(role: Role, state: State, foreground: Color32, background: Color32) -> ContrastResult {
    let displayed = background.blend(foreground);
    let ratio = contrast(displayed, background);
    ContrastResult {
        role,
        state,
        foreground,
        background,
        ratio,
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

#[cfg(test)]
pub(crate) mod colour_vision;

#[cfg(test)]
mod tests {
    use egui::Color32;
    use orcvs::render_frame::{PortalRole, PortalRoles};
    use orcvs::source::SourcePaint;

    use super::{
        ContrastReport, ContrastResult, CursorPlacement, Role, State, contrast, painted, validate,
    };
    use crate::theme::{Appearance, Theme};
    use crate::theme_registry::ThemeRegistry;
    use crate::theme_registry::tests_support::{built_ins, default_theme};

    /// The roles of a Cell only an Output Portal covers.
    const OUTPUT: PortalRoles = PortalRoles::NONE.with(PortalRole::Output);

    /// The roles of a Cell only an Input Portal covers.
    const INPUT: PortalRoles = PortalRoles::NONE.with(PortalRole::Input);

    fn find(report: &ContrastReport, role: Role, state: State) -> ContrastResult {
        report
            .results
            .iter()
            .find(|result| result.role == role && result.state == state)
            .unwrap_or_else(|| panic!("validate did not report {role} / {state}"))
            .clone()
    }

    /// The results of `report` that [fail](ContrastResult::fails) — the
    /// shipped-Theme gate's comparison.
    pub(super) fn failures(report: &ContrastReport) -> Vec<&ContrastResult> {
        report
            .results
            .iter()
            .filter(|result| result.fails())
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
    fn validate_reports_the_floor_in_the_returned_data() {
        assert_eq!(validate(&default_theme(Appearance::Dark)).floor, 4.5);
    }

    #[test]
    fn validate_reports_one_result_per_role_cursor_placement_and_portal_role_set_plus_seventeen_chrome_states()
     {
        let report = validate(&default_theme(Appearance::Dark));
        // 8 Source Grid roles x 4 CursorPlacements x 4 Portal role sets
        // = 128, the Function and Bang roles x 4 CursorPlacements x the 4
        // sets holding Locked = 32, plus 17 chrome states: text on panel, input and the
        // inactive and open fills (4); text.muted on panel and input (2);
        // text.active on the hovered and active fills, framed and frameless
        // (4); selected text on selection.background over panel and input
        // (2); error, warning and link on panel (3); a Panel button's glyph
        // at rest and pressed (2). Stated as one literal
        // rather than recomputed from the same lengths `validate` sizes its
        // `Vec` from, so a change to either count is caught by an
        // independent number.
        assert_eq!(report.results.len(), 177);
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
        let base = default_theme(Appearance::Dark);
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
        let ordinary = find(
            &report,
            Role::Ordinary,
            State::SourceGrid {
                cursor: CursorPlacement::Plain,
                portals: PortalRoles::NONE,
            },
        );
        assert!(
            ordinary.passes(),
            "Ordinary was not retuned and should still clear the floor: {:.2}:1",
            ordinary.ratio
        );
    }

    // === Chrome states beyond text/text.muted on panel and input ===

    /// Asserts `result` measures `foreground` against itself — 1:1 — and so
    /// fails, and that no Source Grid state and neither `text` nor
    /// `text.muted` on panel and input fails, so the failure is one only the
    /// chrome state under test can see.
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
            .filter(|result| result.fails())
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
        let base = default_theme(Appearance::Dark);
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
        let base = default_theme(Appearance::Dark);
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
        let base = default_theme(Appearance::Dark);
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
    /// A Panel button's glyph is measured over its own fill on the panel, at
    /// rest and pressed, and fails where it matches that fill.
    ///
    #[test]
    fn a_button_glyph_matching_its_fill_fails() {
        let base = default_theme(Appearance::Dark);
        for (role, state, theme) in [
            (
                Role::Button,
                State::Button,
                Theme {
                    button_foreground: base.button_background,
                    ..base.clone()
                },
            ),
            (
                Role::ButtonPressed,
                State::ButtonPressed,
                Theme {
                    button_pressed_foreground: base.button_pressed_background,
                    ..base.clone()
                },
            ),
        ] {
            let report = validate(&theme);
            fails_only_in_an_added_state(&report, &find(&report, role, state));
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
        let base = default_theme(Appearance::Dark);
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
    /// A role whose own background is transparent (Ordinary's, in the dark
    /// default) reads fine against the bare Grid in its `plain` state, but a
    /// custom Theme's opaque `region.background` set to the same colour as
    /// the glyph itself makes the `Region` state of that same role unreadable.
    /// Region fallback only applies because Ordinary's own background stays
    /// fully transparent in this Theme; a role background with any nonzero
    /// alpha keeps winning over the Region fill regardless
    /// (`region_fallback_only_reaches_a_fully_transparent_role_background`
    /// pins that distinction directly).
    ///
    #[test]
    fn a_role_passing_plain_fails_against_its_painted_region_tint() {
        let theme = Theme {
            region_background: default_theme(Appearance::Dark).source_ordinary,
            ..default_theme(Appearance::Dark)
        };

        let report = validate(&theme);

        let plain = find(
            &report,
            Role::Ordinary,
            State::SourceGrid {
                cursor: CursorPlacement::Plain,
                portals: PortalRoles::NONE,
            },
        );
        assert!(
            plain.passes(),
            "Ordinary/plain should still read against the bare Grid: {:.2}:1",
            plain.ratio
        );

        let region = find(
            &report,
            Role::Ordinary,
            State::SourceGrid {
                cursor: CursorPlacement::Region,
                portals: PortalRoles::NONE,
            },
        );
        assert!(
            !region.passes(),
            "Ordinary/Region should fail once the Region tint matches the glyph: {:.2}:1",
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
    /// `cursor.background` set to Ordinary's own colour leaves `plain`
    /// passing while `Cursor` fails.
    ///
    #[test]
    fn a_role_passing_plain_fails_against_its_selected_cursor_background() {
        let theme = Theme {
            cursor_background: Some(default_theme(Appearance::Dark).source_ordinary),
            // The Cursor's glyph colour is cleared, so the Cursor's Cell
            // draws Ordinary's own glyph.
            cursor_foreground: None,
            ..default_theme(Appearance::Dark)
        };

        let report = validate(&theme);

        let plain = find(
            &report,
            Role::Ordinary,
            State::SourceGrid {
                cursor: CursorPlacement::Plain,
                portals: PortalRoles::NONE,
            },
        );
        assert!(
            plain.passes(),
            "Ordinary/plain should still read against the bare Grid: {:.2}:1",
            plain.ratio
        );

        let cursor = find(
            &report,
            Role::Ordinary,
            State::SourceGrid {
                cursor: CursorPlacement::Cursor,
                portals: PortalRoles::NONE,
            },
        );
        assert!(
            !cursor.passes(),
            "Ordinary/Cursor should fail once the Cursor fill matches the glyph: {:.2}:1",
            cursor.ratio
        );
    }

    ///
    /// The Cursor state measures the glyph `cursor.foreground` draws, not
    /// the role's: a glyph colour equal to the Cursor fill is reported below
    /// the floor for every Source role, and one that contrasts with it is
    /// not. The plain state keeps the role's own glyph either way.
    ///
    #[test]
    fn the_cursor_state_measures_the_cursor_foreground_against_the_cursor_fill() {
        let fill = Color32::from_rgb(20, 20, 20);
        let report = |glyph| {
            validate(&Theme {
                cursor_background: Some(fill),
                cursor_foreground: Some(glyph),
                ..default_theme(Appearance::Dark)
            })
        };
        let (failing, passing) = (report(fill), report(Color32::WHITE));

        for role in Role::SOURCE_ROLES {
            for portals in [PortalRoles::NONE, OUTPUT, INPUT] {
                let cursor = State::SourceGrid {
                    cursor: CursorPlacement::Cursor,
                    portals,
                };
                let failed = find(&failing, role, cursor);
                assert_eq!(failed.foreground, fill, "{role} / {cursor}");
                assert!(!failed.passes(), "{role} / {cursor}: {:.2}:1", failed.ratio);
                let passed = find(&passing, role, cursor);
                assert_eq!(passed.foreground, Color32::WHITE, "{role} / {cursor}");
                assert!(passed.passes(), "{role} / {cursor}: {:.2}:1", passed.ratio);
            }
            let plain = State::SourceGrid {
                cursor: CursorPlacement::Plain,
                portals: PortalRoles::NONE,
            };
            assert_ne!(
                find(&failing, role, plain).foreground,
                fill,
                "{role} / {plain}"
            );
        }
    }

    ///
    /// The Region's Cursor state measures the glyph the Region's Cursor
    /// fallback resolves: `cursor.foreground` equal to the Region's Cursor
    /// fill is reported below the floor while `region.cursor.foreground` is
    /// cleared, and a contrasting `region.cursor.foreground` passes over it.
    ///
    #[test]
    fn the_region_cursor_state_measures_the_resolved_region_cursor_glyph() {
        let fill = Color32::from_rgb(20, 20, 20);
        let report = |region_cursor_foreground| {
            validate(&Theme {
                region_cursor_background: Some(fill),
                cursor_foreground: Some(fill),
                region_cursor_foreground,
                ..default_theme(Appearance::Dark)
            })
        };
        let (failing, passing) = (report(None), report(Some(Color32::WHITE)));

        for role in Role::SOURCE_ROLES {
            for portals in [PortalRoles::NONE, OUTPUT, INPUT] {
                let state = State::SourceGrid {
                    cursor: CursorPlacement::RegionCursor,
                    portals,
                };
                let failed = find(&failing, role, state);
                assert_eq!(failed.foreground, fill, "{role} / {state}");
                assert!(!failed.passes(), "{role} / {state}: {:.2}:1", failed.ratio);
                let passed = find(&passing, role, state);
                assert_eq!(passed.foreground, Color32::WHITE, "{role} / {state}");
                assert!(passed.passes(), "{role} / {state}: {:.2}:1", passed.ratio);
            }
        }
    }

    ///
    /// A transparent Cursor glyph draws no glyph, so the Cursor states it
    /// hides are not measured rather than reported at 1:1, while every
    /// other state of the same role still is.
    ///
    #[test]
    fn a_hidden_cursor_glyph_is_not_measured() {
        let report = validate(&Theme {
            cursor_foreground: Some(Color32::TRANSPARENT),
            ..default_theme(Appearance::Dark)
        });

        for role in Role::SOURCE_ROLES {
            for cursor in [CursorPlacement::Cursor, CursorPlacement::RegionCursor] {
                assert!(
                    !report.results.iter().any(|result| result.role == role
                        && matches!(result.state, State::SourceGrid { cursor: c, .. } if c == cursor)),
                    "{role} under {cursor:?} is measured"
                );
            }
            let plain = State::SourceGrid {
                cursor: CursorPlacement::Plain,
                portals: PortalRoles::NONE,
            };
            assert!(find(&report, role, plain).foreground.a() != 0);
        }
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
            ..default_theme(Appearance::Dark)
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
            ..default_theme(Appearance::Dark)
        };

        let report = validate(&theme);
        let invalid_number = find(
            &report,
            Role::NumberInvalid,
            State::SourceGrid {
                cursor: CursorPlacement::Plain,
                portals: PortalRoles::NONE,
            },
        );

        assert_eq!(
            invalid_number.foreground, theme.source_number,
            "a transparent diagnostic.foreground must reveal source.number, not vanish"
        );
    }

    // === `painted`'s reuse of `style`'s own composition ===

    ///
    /// `painted` with no Portal role and with the Output Portal role for
    /// `Function` are identical — "A bound Function retains all its own
    /// paint inside a Portal" (`style::role_and_portal`'s own doc), the one
    /// named bypass the reused composition functions apply. This is a real
    /// reachable state (a Function standing inside another's Output Portal
    /// Reservation) whose *answer* happens to equal the non-Portal case,
    /// which is exactly what the bypass rule says should happen.
    ///
    #[test]
    fn function_is_unaffected_by_output_portal_placement() {
        let theme = default_theme(Appearance::Dark);
        let plain = painted(
            SourcePaint::Function,
            CursorPlacement::Plain,
            PortalRoles::NONE,
            &theme,
        );
        let portal = painted(
            SourcePaint::Function,
            CursorPlacement::Plain,
            OUTPUT,
            &theme,
        );
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
    /// Portal role set is actually threaded through every
    /// `CursorPlacement`, not only `Plain`.
    ///
    #[test]
    fn cursor_and_region_placements_still_answer_the_output_portal_channel() {
        // The Cursor's glyph colours are cleared, so each placement draws
        // the glyph its channels compose.
        let theme = Theme {
            cursor_foreground: None,
            region_cursor_foreground: None,
            ..default_theme(Appearance::Dark)
        };
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
                    portals: PortalRoles::NONE,
                },
            );
            let portal = find(
                &report,
                Role::NumberValid,
                State::SourceGrid {
                    cursor,
                    portals: OUTPUT,
                },
            );
            assert_ne!(
                plain.foreground, portal.foreground,
                "{cursor}: Output Portal must still blend its foreground channel over Number's"
            );
        }
    }

    ///
    /// The Input Portal channel reaches every Cursor placement too: under the
    /// Cursor, a Region and the Region's Cursor Cell, `Number, Valid` draws
    /// the Input Portal foreground rather than Number's own.
    ///
    #[test]
    fn cursor_and_region_placements_still_answer_the_input_portal_channel() {
        // The Cursor's glyph colours are cleared, so each placement draws
        // the glyph its channels compose.
        let theme = Theme {
            cursor_foreground: None,
            region_cursor_foreground: None,
            ..default_theme(Appearance::Dark)
        };
        let report = validate(&theme);

        for cursor in CursorPlacement::ALL {
            let input = find(
                &report,
                Role::NumberValid,
                State::SourceGrid {
                    cursor,
                    portals: INPUT,
                },
            );
            assert_eq!(
                input.foreground, theme.input_portal_foreground,
                "{cursor}: Input Portal must blend its foreground channel over Number's"
            );
        }
    }

    ///
    /// Every Source role, at every Cursor placement, under Input Portal alone
    /// and under both Portal roles, is measured and clears the floor in every
    /// built-in. A Cell playing both roles measures exactly what an Output
    /// Portal Cell does, because Output Portal wins.
    ///
    #[test]
    fn every_source_role_under_input_portal_clears_the_floor_in_every_built_in() {
        let both_roles = OUTPUT.with(PortalRole::Input);
        let registry = ThemeRegistry::built_in();
        for theme in built_ins(&registry) {
            let report = validate(theme);
            for &role in &Role::SOURCE_ROLES {
                for cursor in CursorPlacement::ALL {
                    let input = find(
                        &report,
                        role,
                        State::SourceGrid {
                            cursor,
                            portals: INPUT,
                        },
                    );
                    assert!(
                        input.passes(),
                        "{}: {role} / {} at {:.2}:1",
                        theme.identity,
                        input.state,
                        input.ratio
                    );
                    let output = find(
                        &report,
                        role,
                        State::SourceGrid {
                            cursor,
                            portals: OUTPUT,
                        },
                    );
                    let both = find(
                        &report,
                        role,
                        State::SourceGrid {
                            cursor,
                            portals: both_roles,
                        },
                    );
                    assert_eq!(
                        (both.foreground, both.background),
                        (output.foreground, output.background),
                        "{}: {role} / {}",
                        theme.identity,
                        both.state
                    );
                }
            }
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
        let theme = default_theme(Appearance::Dark);
        let report = validate(&theme);

        let nonzero_alpha_role_plain = find(
            &report,
            Role::NumberInvalid,
            State::SourceGrid {
                cursor: CursorPlacement::Plain,
                portals: PortalRoles::NONE,
            },
        );
        let nonzero_alpha_role_region = find(
            &report,
            Role::NumberInvalid,
            State::SourceGrid {
                cursor: CursorPlacement::Region,
                portals: PortalRoles::NONE,
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
                portals: PortalRoles::NONE,
            },
        );
        let transparent_region = find(
            &transparent_report,
            Role::Comment,
            State::SourceGrid {
                cursor: CursorPlacement::Region,
                portals: PortalRoles::NONE,
            },
        );
        assert_ne!(
            transparent_plain.background, transparent_region.background,
            "a transparent role background must let an opaque Region fill show"
        );
    }

    ///
    /// The single-Cell Cursor always replaces the role background outright
    /// — `style::CellColours::colours`'s own "The Cursor's own
    /// fill wins outright on its Cell" — regardless of whether that role
    /// background was opaque. This fixture sets `source_number_background`
    /// explicitly opaque to exercise the claim the test's name makes.
    ///
    #[test]
    fn cursors_own_fill_replaces_even_an_opaque_role_background() {
        let theme = Theme {
            source_number_background: Color32::from_rgb(14, 29, 37),
            cursor_background: Some(Color32::from_rgb(1, 2, 3)),
            ..default_theme(Appearance::Dark)
        };

        let report = validate(&theme);

        let plain = find(
            &report,
            Role::NumberInvalid,
            State::SourceGrid {
                cursor: CursorPlacement::Plain,
                portals: PortalRoles::NONE,
            },
        );
        let cursor = find(
            &report,
            Role::NumberInvalid,
            State::SourceGrid {
                cursor: CursorPlacement::Cursor,
                portals: PortalRoles::NONE,
            },
        );
        assert_ne!(
            plain.background, cursor.background,
            "the Cursor's own fill must replace even an opaque role background"
        );
        assert_eq!(cursor.background, Color32::from_rgb(1, 2, 3));
    }

    ///
    /// A Comment drawn in its own background's colour measures 1:1 and is
    /// reported, yet does not fail; the same glyph colour on an Ordinary Cell
    /// does.
    ///
    #[test]
    fn a_comment_below_the_floor_is_reported_and_does_not_fail() {
        let background = default_theme(Appearance::Dark).grid_background;
        let theme = Theme {
            source_comment: background,
            source_ordinary: background,
            ..default_theme(Appearance::Dark)
        };
        let report = validate(&theme);
        let plain = State::SourceGrid {
            cursor: CursorPlacement::Plain,
            portals: PortalRoles::NONE,
        };

        let comment = find(&report, Role::Comment, plain);
        assert!(!comment.passes(), "Comment measures {:.2}:1", comment.ratio);
        assert!(!comment.fails());
        assert!(find(&report, Role::Ordinary, plain).fails());
    }

    // === The shipped-Theme gate ===

    ///
    /// No shipped Theme has a state that [fails](ContrastResult::fails), or
    /// this test fails and lists them: every state of a role held to the
    /// floor clears it, and a Comment below it is reported and does not fail. `shipped` is the Theme registry's own built-ins, so a
    /// built-in added there is gated here without a second list to keep.
    ///
    #[test]
    fn shipped_theme_gate() {
        let registry = ThemeRegistry::built_in();
        let shipped = built_ins(&registry);

        for theme in shipped {
            let report = validate(theme);
            let failures = failures(&report);
            assert!(
                failures.is_empty(),
                "{}'s contrast failures: {failures:?}",
                theme.identity
            );
        }
    }

    ///
    /// Each built-in's bar edge stands apart from the Grid's lines it runs
    /// beside, so a bar does not read as one more line of the Grid: the
    /// separator, over the panel, against the sector seam and the Grid
    /// border, each over the Grid. A test-only check on the built-ins, as the
    /// colour-vision check is; a loaded Theme document is not held to it.
    ///
    #[test]
    fn every_built_in_bar_edge_stands_apart_from_the_grid_lines() {
        /// The least ratio a bar edge keeps to each Grid line.
        const APART: f32 = 1.3;

        let registry = ThemeRegistry::built_in();
        for theme in built_ins(&registry) {
            let over = |surface: Color32, colour: Color32| {
                theme.window_background.blend(surface).blend(colour)
            };
            let edge = over(theme.panel_background, theme.panel_separator);
            for (line, colour) in [
                ("sector.seam", theme.sector_seam),
                ("grid.border", theme.grid_border),
            ] {
                let ratio = contrast(edge, over(theme.grid_background, colour));
                assert!(
                    ratio >= APART,
                    "{}'s panel.separator is {ratio}:1 against its {line}, under {APART}:1",
                    theme.identity
                );
            }
        }
    }
}
