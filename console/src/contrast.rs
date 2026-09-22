//!
//! Validates a resolved Theme's *composited* text contrast: the effective
//! foreground and background each reachable painted text state actually
//! displays, not raw Theme colour pairs read in isolation.
//! `.scratch/theming/issues/08`.
//!
//! Reuses `style`'s own composition —
//! [`crate::style::cell_visuals_with_cursor_colour`] and
//! [`crate::style::compose_cell_fill`] — for every colour operation a Source
//! Grid state produces, so painting and this validator cannot independently
//! drift. This module adds only the state *selection*
//! [`crate::paint::Paint::derive_with_theme`] performs once per Cell of a
//! whole Render Frame, mirrored here so one reachable state can be asked for
//! standalone. Console text (`text`, `text.muted`) has no such function yet —
//! `.scratch/theming/issues/03` derives chrome from the Theme — so it
//! composites directly with the same [`egui::Color32::blend`] primitive
//! every reused function above is itself built from.
//!
//! # Scope
//!
//! This module measures text contrast only: an effective foreground against
//! the effective background it is actually painted on. It does not measure
//! pairwise Token-colour distinguishability, colour-vision accessibility, or
//! border/focus visibility — [`validate`]'s own doc comment restates this,
//! since a report is exactly where a reader would otherwise assume a wider
//! claim.

#![cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "unused outside this module's own tests until theming/07 wires a loader and \
                   shows validate's report when a viewer loads a scheme"
    )
)]

use egui::Color32;
use orcvs::source::{OperandState, SourcePaint, Token};

use crate::style::{cell_visuals_with_cursor_colour, compose_cell_fill};
use crate::theme::Theme;

///
/// The WCAG 2.1 Success Criterion 1.4.3 ("Contrast (Minimum)") ratio every
/// [`ContrastResult`] is measured against: 4.5:1, the floor normal-size text
/// must clear. Stated once, here, with its source, rather than repeated as a
/// literal at each call site.
///
pub(crate) const CONTRAST_FLOOR: f32 = 4.5;

///
/// Every Source Paint fact a real Source can produce, paired with the label
/// [`ContrastResult::role`] reports it under.
///
/// Number and Note admit Pending, Valid and Invalid. Atom and Sequence admit
/// only Pending and Invalid, never Valid: `Token::decode` refuses both
/// outright (`lang/src/expression.rs`'s own words on `Token`: "declarations
/// no Cells spell"), so the only thing that can satisfy either slot is a
/// nested Function — which `take_language_unit`'s `is_function_next()`
/// branch records under `Token::Function` instead, never under `Atom` or
/// `Sequence`. Unclaimed, Function, Bang and Comment carry no operand state
/// of their own. This mirrors `style::tests`'s own `operand` fixture and its
/// documented reachability — `an_operand_cell_of_every_token_a_source_can_
/// claim_is_tinted_with_its_own_colour` drives the real pairs through the
/// Render Frame.
///
const SOURCE_PAINT_FACTS: [(&str, SourcePaint); 14] = [
    ("Ordinary", SourcePaint::Unclaimed),
    ("Function", SourcePaint::Function),
    ("Bang", SourcePaint::Bang),
    ("Comment", SourcePaint::Comment),
    (
        "Number, Pending",
        SourcePaint::Operand {
            token: Token::Number,
            state: OperandState::Pending,
        },
    ),
    (
        "Number, Valid",
        SourcePaint::Operand {
            token: Token::Number,
            state: OperandState::Valid,
        },
    ),
    (
        "Number, Invalid",
        SourcePaint::Operand {
            token: Token::Number,
            state: OperandState::Invalid,
        },
    ),
    (
        "Note, Pending",
        SourcePaint::Operand {
            token: Token::Note,
            state: OperandState::Pending,
        },
    ),
    (
        "Note, Valid",
        SourcePaint::Operand {
            token: Token::Note,
            state: OperandState::Valid,
        },
    ),
    (
        "Note, Invalid",
        SourcePaint::Operand {
            token: Token::Note,
            state: OperandState::Invalid,
        },
    ),
    (
        "Atom, Pending",
        SourcePaint::Operand {
            token: Token::Atom,
            state: OperandState::Pending,
        },
    ),
    (
        "Atom, Invalid",
        SourcePaint::Operand {
            token: Token::Atom,
            state: OperandState::Invalid,
        },
    ),
    (
        "Sequence, Pending",
        SourcePaint::Operand {
            token: Token::Sequence,
            state: OperandState::Pending,
        },
    ),
    (
        "Sequence, Invalid",
        SourcePaint::Operand {
            token: Token::Sequence,
            state: OperandState::Invalid,
        },
    ),
];

///
/// Which reachable placement — beyond the plain role/Diagnostic/Output-Portal
/// channel a [`SOURCE_PAINT_FACTS`] entry already carries — a text state is
/// measured in, mirroring [`crate::paint::Paint::derive_with_theme`]'s own
/// per-Cell decision.
///
/// Crossed with every [`SOURCE_PAINT_FACTS`] entry rather than with each
/// other: Output Portal, Cursor and Region are each asked about in isolation
/// (a Cell is not simultaneously the Cursor and inside an Output Portal
/// Reservation in these fixtures) rather than as a full three-way
/// cross-product, which `.scratch/theming/issues/08`'s "do not substitute
/// impossible cross-products" reads as a boundary to keep rather than a
/// requirement to exhaustively combine every axis with every other.
///
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Placement {
    /// Not the Cursor, not inside a Region: the ordinary Cell.
    Plain,
    /// Within a root Function's Output Portal Reservation.
    OutputPortal,
    /// The single-Cell Cursor, selected and visible.
    Cursor,
    /// A Region Cell other than the Cursor's own, inside a Region spanning
    /// more than one Cell.
    Region,
    /// The Cursor's own Cell inside a Region spanning more than one Cell.
    RegionCursor,
}

impl Placement {
    const ALL: [Self; 5] = [
        Self::Plain,
        Self::OutputPortal,
        Self::Cursor,
        Self::Region,
        Self::RegionCursor,
    ];

    fn name(self) -> &'static str {
        match self {
            Self::Plain => "plain",
            Self::OutputPortal => "Output Portal",
            Self::Cursor => "Cursor",
            Self::Region => "Region",
            Self::RegionCursor => "Region, Cursor's Cell",
        }
    }
}

///
/// The effective foreground and *opaque* background a `(fact, placement)`
/// pair actually paints, composited exactly as
/// [`crate::paint::Paint::derive_with_theme`] composes one Cell — same
/// functions, same order, standalone rather than read off one Cell of a
/// whole Render Frame.
///
/// The background is resolved all the way down to the opaque window
/// backdrop: `console.rs`'s own paint order draws `window.background` as the
/// `eframe` clear colour first, `grid.background` as the Source panel's fill
/// over it, then each Cell's own composited fill on top
/// (`console.rs::source_panel_frame`, `Console::clear_color`). A translucent
/// background therefore resolves against its underlying surface — the
/// `.scratch/theming/schema.md` composition rule this replaces the earlier
/// `Color32::to_opaque` normalisation with, which measured a translucent
/// colour against itself rather than against what is actually behind it.
///
fn painted(fact: SourcePaint, placement: Placement, theme: &Theme) -> (Color32, Color32) {
    let output_portal = placement == Placement::OutputPortal;
    let is_cursor = matches!(placement, Placement::Cursor | Placement::RegionCursor);
    let region_spans = matches!(placement, Placement::Region | Placement::RegionCursor);
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

    // `Paint::derive_with_theme`'s own three-way background decision,
    // mirrored: the Cursor's Cell inside a spanning Region takes the Region
    // Cursor fill (falling back to the ordinary Cursor fill); any other Cell
    // of that Region falls back to the Region fill when its own role/
    // Diagnostic/Output-Portal composition left nothing visible; every other
    // Cell falls back to the uniform Cell base alone.
    let region_cursor_fill = theme.region_cursor_background.or(cursor_fill);
    let region_fill = compose_cell_fill(theme.cell_background, Some(theme.region_background));
    let base_fill = compose_cell_fill(theme.cell_background, None);

    let painted_background = if is_cursor && region_spans {
        compose_cell_fill(theme.cell_background, region_cursor_fill)
    } else if region_spans {
        visuals.background.or(region_fill)
    } else {
        visuals.background.or(base_fill)
    };

    let grid_surface = theme.window_background.blend(theme.grid_background);
    let effective_background =
        grid_surface.blend(painted_background.unwrap_or(Color32::TRANSPARENT));

    (visuals.foreground, effective_background)
}

///
/// One reachable painted text state's measured result: its role, its
/// placement, the effective foreground and background it actually
/// composites to, and the WCAG ratio between them.
///
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ContrastResult {
    pub(crate) role: &'static str,
    pub(crate) state: &'static str,
    pub(crate) foreground: Color32,
    pub(crate) background: Color32,
    pub(crate) ratio: f32,
}

impl ContrastResult {
    /// Whether [`Self::ratio`] clears [`CONTRAST_FLOOR`].
    pub(crate) fn passes(&self) -> bool {
        self.ratio >= CONTRAST_FLOOR
    }
}

///
/// Measures every reachable painted text state against `theme` and reports
/// each one's role, state, effective foreground and background, and measured
/// WCAG ratio.
///
/// Never refuses `theme`: a scheme that fails a state is measured and
/// reported exactly like one that passes, because the floor is a fact about
/// the scheme worth seeing rather than a gate a Theme must clear before it
/// loads. `.scratch/theming/issues/07` shows this report when a viewer loads
/// a scheme; a failing one loads anyway, because the viewer chose it.
///
/// # Scope
///
/// This measures text contrast only. It does not measure whether two Token
/// colours are distinguishable from each other, colour-vision accessibility,
/// or border/focus visibility — a low text/background ratio and a
/// low-distinguishability pair between two Tokens are different findings,
/// and a caller reading this report should not infer the second from the
/// first.
///
pub(crate) fn validate(theme: &Theme) -> Vec<ContrastResult> {
    let mut results = Vec::with_capacity(SOURCE_PAINT_FACTS.len() * Placement::ALL.len() + 4);

    for &(role, fact) in &SOURCE_PAINT_FACTS {
        for placement in Placement::ALL {
            let (foreground, background) = painted(fact, placement, theme);
            results.push(measure(role, placement.name(), foreground, background));
        }
    }

    // Console text has no reused composition function yet
    // (`.scratch/theming/issues/03` derives chrome from the Theme), so it
    // composites directly over its own two reachable surfaces: the normal
    // widget/panel fill, and the extreme/faint input fill
    // (`.scratch/theming/schema.md`'s chrome mapping). Both are themselves
    // composited over the opaque window backdrop first, the same
    // `window.background ▸ surface` step `painted` performs for the Source
    // Grid.
    let window = theme.window_background;
    let panel = window.blend(theme.panel_background);
    let input = window.blend(theme.input_background);
    for (role, state, foreground, background) in [
        ("text", "vs panel.background", theme.text, panel),
        ("text", "vs input.background", theme.text, input),
        ("text.muted", "vs panel.background", theme.text_muted, panel),
        ("text.muted", "vs input.background", theme.text_muted, input),
    ] {
        results.push(measure(role, state, foreground, background));
    }

    results
}

///
/// Composites `foreground` over the already-opaque `background` — the
/// pinned "self behind on_top" compositing every other resolved-Theme
/// channel uses ([`compose_cell_fill`], `style.rs`'s `ordinary_border`): a
/// translucent foreground is measured as the colour it actually displays,
/// not its stored premultiplied bytes misread as straight sRGB. An opaque
/// foreground is unchanged by this. Then measures [`contrast`] between the
/// two.
///
fn measure(
    role: &'static str,
    state: &'static str,
    foreground: Color32,
    background: Color32,
) -> ContrastResult {
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
/// compositing down to the opaque window backdrop first
/// ([`painted`], [`validate`]'s console-text loop).
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
/// The failing `(role, state)` pairs `.scratch/theming/issues/08`'s comments
/// record as explicitly accepted for a shipped Theme, keyed by identity. A
/// pair absent from a Theme's entry — or a Theme absent from this list
/// entirely — must report zero failures for [`unaccepted_failures`] to
/// answer empty.
///
/// Okabe–Ito's two entries are exactly the states that measure
/// `source.sequence`'s own accepted colour, `#0072B2`, against the bare Grid
/// background, `#000000` — the issue's own acceptance line, precisely: "the
/// bare Grid background, `#000000`, approximately 4.05:1... this exception
/// does not exempt other roles, Themes or newly measured failing states."
/// That is `Cursor` and `Region, Cursor's Cell`, where Okabe–Ito's unset
/// Cursor/Region-Cursor fills let the bare background show through. `plain`
/// and `Region` measure a worse 3.67:1 against Sequence's own near-black
/// background tint (`source.sequence.background`, not `base00`) — a
/// genuinely different, newly measured failing state the composited-state
/// validator surfaces that the earlier `base00`-only check could not, and
/// the issue's own words classify it as one to record for explicit review
/// rather than fold silently into the one already-confirmed figure. It is
/// therefore pending alongside the invalid Number/Note/Atom Diagnostic
/// states below, not accepted here.
///
fn accepted_failures(identity: &str) -> &'static [(&'static str, &'static str)] {
    match identity {
        "okabe-ito" => &[
            ("Sequence, Pending", "Cursor"),
            ("Sequence, Pending", "Region, Cursor's Cell"),
        ],
        _ => &[],
    }
}

///
/// `theme`'s failing results whose `(role, state)` pair is not in `accepted`
/// — `.scratch/theming/issues/08`'s shipped-Theme gate, factored out to a
/// plain function so it can be tested against a synthetic accepted set
/// independently of whether the real shipped Theme currently clears it
/// ([`tests::shipped_theme_gate`] is `#[ignore]`d while the invalid
/// Number/Note/Atom Diagnostic failures below await a human decision).
///
fn unaccepted_failures(theme: &Theme, accepted: &[(&str, &str)]) -> Vec<ContrastResult> {
    validate(theme)
        .into_iter()
        .filter(|result| !result.passes())
        .filter(|result| {
            !accepted
                .iter()
                .any(|&(role, state)| role == result.role && state == result.state)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use egui::Color32;

    use super::{
        ContrastResult, Placement, SOURCE_PAINT_FACTS, accepted_failures, contrast, painted,
        unaccepted_failures, validate,
    };
    use crate::theme::{Theme, okabe_ito};

    fn find(report: &[ContrastResult], role: &str, state: &str) -> ContrastResult {
        report
            .iter()
            .find(|result| result.role == role && result.state == state)
            .unwrap_or_else(|| panic!("validate did not report {role} / {state}"))
            .clone()
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
    fn validate_reports_one_result_per_fact_and_placement_plus_four_console_text_states() {
        let report = validate(&okabe_ito());
        assert_eq!(
            report.len(),
            SOURCE_PAINT_FACTS.len() * Placement::ALL.len() + 4,
            "one result per (fact, placement) pair plus text/text.muted against panel and input"
        );
    }

    ///
    /// The validator against a Theme built to fail — proof `validate`
    /// discriminates a pass from a failure rather than reporting every
    /// state as passing regardless of the colours it is given, and that a
    /// failing Theme is reported, never refused: `validate` returns a plain
    /// `Vec`, not a `Result`, and the failing entry sits in it beside every
    /// passing one.
    ///
    #[test]
    fn validate_reports_a_theme_built_to_fail_rather_than_a_vacuous_pass() {
        let base = okabe_ito();
        let theme = Theme {
            text: base.panel_background,
            ..base
        };

        let report = validate(&theme);

        let text = find(&report, "text", "vs panel.background");
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
        let comment = find(&report, "Comment", "plain");
        assert!(
            comment.passes(),
            "Comment was not retuned and should still clear the floor: {:.2}:1",
            comment.ratio
        );
    }

    // === Fixtures: text passing on the bare Grid but failing on its
    // painted tint or selected background ===

    ///
    /// A role whose own background is transparent (Comment's, in Okabe–Ito)
    /// reads fine against the bare Grid in its `plain` state, but a custom
    /// Theme's opaque `region.background` set to the same colour as the
    /// glyph itself makes the `Region` state of that same role unreadable —
    /// `.scratch/theming/issues/08`'s "text that passes against the bare
    /// Grid background but fails against its painted tint" fixture. Region
    /// fallback only applies because Comment's own background stays
    /// transparent in this Theme; an opaque role background would keep
    /// winning over the Region fill (`every_check_measures_against_its_
    /// documented_background`'s sibling assertion for facts with opaque
    /// backgrounds, exercised implicitly by `Number, Invalid` below staying
    /// identical between `plain` and `Region`).
    ///
    #[test]
    fn a_role_passing_plain_fails_against_its_painted_region_tint() {
        let matching_grey = Color32::from_rgb(153, 153, 153); // Comment's own colour
        let theme = Theme {
            region_background: matching_grey,
            ..okabe_ito()
        };

        let report = validate(&theme);

        let plain = find(&report, "Comment", "plain");
        assert!(
            plain.passes(),
            "Comment/plain should still read against the bare Grid: {:.2}:1",
            plain.ratio
        );

        let region = find(&report, "Comment", "Region");
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
    /// passing while `Cursor` fails — `.scratch/theming/issues/08`'s
    /// "...or its... selected background".
    ///
    #[test]
    fn a_role_passing_plain_fails_against_its_selected_cursor_background() {
        let matching_grey = Color32::from_rgb(153, 153, 153); // Comment's own colour
        let theme = Theme {
            cursor_background: Some(matching_grey),
            ..okabe_ito()
        };

        let report = validate(&theme);

        let plain = find(&report, "Comment", "plain");
        assert!(
            plain.passes(),
            "Comment/plain should still read against the bare Grid: {:.2}:1",
            plain.ratio
        );

        let cursor = find(&report, "Comment", "Cursor");
        assert!(
            !cursor.passes(),
            "Comment/Cursor should fail once the Cursor fill matches the glyph: {:.2}:1",
            cursor.ratio
        );
    }

    // === Fixtures: alpha compositing ===

    ///
    /// A partial-alpha foreground over a partial-alpha background, checked
    /// against an independently worked answer rather than the formula this
    /// module itself computes with: 50%-alpha white composited over a
    /// 50%-alpha grey `(40, 40, 40)` panel, itself composited over the
    /// opaque black window backdrop.
    ///
    /// `panel.background` at `(40, 40, 40, 128)` straight premultiplies to
    /// `(20, 20, 20, 128)` (`ecolor`'s `mul_frac_round(40, 128) = 20`), and
    /// composites over opaque black to `(20, 20, 20, 255)` unchanged — black
    /// contributes nothing over any alpha. `text` at 50%-alpha white
    /// premultiplies to `(128, 128, 128, 128)` and composites over that
    /// surface to `(138, 138, 138, 255)`. The reported background is
    /// asserted against the first figure, so a compositing regression is
    /// caught at the pixel before it is caught at the ratio.
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
        let text = find(&report, "text", "vs panel.background");

        let expected_panel_surface = Color32::BLACK.blend(translucent_panel);
        let expected_displayed = expected_panel_surface.blend(translucent_text);
        let expected_ratio = contrast(expected_displayed, expected_panel_surface);
        assert!(
            (text.ratio - expected_ratio).abs() < 0.001,
            "got {:.4}:1, expected {:.4}:1 from an independent recomposition of the same bytes",
            text.ratio,
            expected_ratio
        );
        assert_eq!(
            text.background, expected_panel_surface,
            "the reported background must be the actual composited panel surface"
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
        let invalid_number = find(&report, "Number, Invalid", "plain");

        assert_eq!(
            invalid_number.foreground, theme.source_number,
            "a transparent diagnostic.foreground must reveal source.number, not vanish"
        );
    }

    // === `painted`'s reuse of `style`'s own composition ===

    ///
    /// `painted`'s `Placement::Plain` and `Placement::OutputPortal` for
    /// `Function` are identical — "A bound Function retains all its own
    /// paint inside a Portal" (`style::role_and_portal`'s own doc), the one
    /// named bypass the reused composition functions apply. This is not an
    /// impossible cross-product folded away by this module: it is a real
    /// reachable state (a nested Function inside a root's Output Portal
    /// Reservation) whose *answer* happens to equal the non-Portal case,
    /// which is exactly what the bypass rule says should happen.
    ///
    #[test]
    fn function_is_unaffected_by_output_portal_placement() {
        let theme = okabe_ito();
        let plain = painted(super::SOURCE_PAINT_FACTS[1].1, Placement::Plain, &theme);
        let portal = painted(
            super::SOURCE_PAINT_FACTS[1].1,
            Placement::OutputPortal,
            &theme,
        );
        assert_eq!(plain, portal);
    }

    ///
    /// Region fallback only reaches a fact whose own background is
    /// transparent: `Number, Invalid`'s background is opaque
    /// (`source.number.background`), so its `Region` state equals its
    /// `plain` state exactly, while `Comment`'s transparent background lets
    /// the Region tint show once `region.background` is not itself
    /// transparent. Both are asserted here so the distinction — "does not
    /// count as a fact fill when evaluating Region fallback" applying only
    /// when there is no fact fill to begin with — is pinned as a property
    /// of the reused composition rather than assumed.
    ///
    #[test]
    fn region_fallback_only_reaches_a_transparent_role_background() {
        let theme = okabe_ito();
        let report = validate(&theme);

        let opaque_role_plain = find(&report, "Number, Invalid", "plain");
        let opaque_role_region = find(&report, "Number, Invalid", "Region");
        assert_eq!(
            opaque_role_plain.background, opaque_role_region.background,
            "an opaque role background must win over Region fallback"
        );

        let transparent_role_theme = Theme {
            region_background: Color32::from_rgba_unmultiplied(255, 255, 255, 255),
            ..theme
        };
        let transparent_report = validate(&transparent_role_theme);
        let transparent_plain = find(&transparent_report, "Comment", "plain");
        let transparent_region = find(&transparent_report, "Comment", "Region");
        assert_ne!(
            transparent_plain.background, transparent_region.background,
            "a transparent role background must let an opaque Region fill show"
        );
    }

    // === The shipped-Theme gate ===

    ///
    /// The two states Okabe–Ito's Sequence exception actually covers remain
    /// visible in the report — `.scratch/theming/issues/08`'s "A failing
    /// Theme is reported... Acceptance may annotate the report, but never
    /// hides the failure or converts it to a passing measurement." `Cursor`
    /// and `Region, Cursor's Cell` measure `#0072B2` against the *bare*
    /// Source background, `#000000` — precisely the pair the issue's own
    /// acceptance line names, ≈4.05:1, matching `syntax-highlighting/01`'s
    /// original figure.
    ///
    #[test]
    fn sequences_accepted_cursor_states_remain_visible() {
        let theme = okabe_ito();
        let report = validate(&theme);

        for state in ["Cursor", "Region, Cursor's Cell"] {
            let result = find(&report, "Sequence, Pending", state);
            assert!(
                !result.passes(),
                "Sequence, Pending / {state} unexpectedly passes at {:.2}:1",
                result.ratio
            );
            assert!(
                (result.ratio - 4.05).abs() < 0.01,
                "Sequence, Pending / {state} is {:.2}:1, expected 4.05:1",
                result.ratio
            );
            assert_eq!(
                result.background, theme.grid_background,
                "the accepted states must measure against the bare Grid background, \
                 not Sequence's own tint"
            );
        }
    }

    ///
    /// The below-floor states `.scratch/theming/issues/08` lists as pending
    /// explicit acceptance, confirmed through the real shipped composition
    /// rather than the hand-picked or historical colours the issue's table
    /// and comments originally recorded them from. `Number` and `Note`
    /// match the issue's table (4.446944:1, 4.053689:1); `Atom` is a further
    /// failure the expanded state coverage discovered, not in the issue's
    /// original table. Sequence's own `plain` and `Region` states are
    /// *not* the accepted exception: the issue's acceptance line names only
    /// `#0072B2` against the *bare* Grid background, `#000000`
    /// (`sequences_accepted_cursor_states_remain_visible`'s `Cursor`/
    /// `Region, Cursor's Cell` pair) — `plain` and `Region` measure a worse
    /// 3.67:1 against Sequence's own near-black background tint
    /// (`source.sequence.background`, not `base00`), a newly measured
    /// failing state the issue's own words say to record for review rather
    /// than fold silently into the one already-confirmed figure.
    ///
    #[test]
    fn pending_contrast_failures_are_confirmed_through_shipped_composition() {
        let report = validate(&okabe_ito());

        for (role, state, expected_ratio) in [
            ("Number, Invalid", "plain", 4.4469),
            ("Number, Invalid", "Region", 4.4469),
            ("Note, Invalid", "plain", 4.0537),
            ("Note, Invalid", "Region", 4.0537),
            ("Atom, Invalid", "plain", 3.9275),
            ("Atom, Invalid", "Region", 3.9275),
            ("Sequence, Pending", "plain", 3.6707),
            ("Sequence, Pending", "Region", 3.6707),
        ] {
            let result = find(&report, role, state);
            assert!(
                !result.passes(),
                "{role} / {state} unexpectedly passes at {:.4}:1 — if this was retuned or \
                 accepted, update .scratch/theming/issues/08 and accepted_failures together \
                 rather than leaving this assertion stale",
                result.ratio
            );
            assert!(
                (result.ratio - expected_ratio).abs() < 0.001,
                "{role} / {state} is {:.4}:1, expected {expected_ratio:.4}:1",
                result.ratio
            );
            assert!(
                !accepted_failures("okabe-ito")
                    .iter()
                    .any(|&(accepted_role, accepted_state)| accepted_role == role
                        && accepted_state == state),
                "{role} / {state} must stay out of accepted_failures while pending"
            );
        }
    }

    ///
    /// [`unaccepted_failures`] against a synthetic accepted set: a failure
    /// not listed there is reported.
    ///
    #[test]
    fn unaccepted_failures_lists_a_failure_the_accepted_set_does_not_record() {
        let theme = Theme {
            text: okabe_ito().panel_background,
            ..okabe_ito()
        };

        // Every one of Okabe-Ito's own accepted failures, but nothing about
        // the injected `text` failure — proving the gate function notices a
        // failure genuinely absent from the accepted set, not merely
        // failing on the fixture's Sequence baseline it inherited.
        let unaccepted = unaccepted_failures(&theme, accepted_failures("okabe-ito"));

        assert!(
            unaccepted
                .iter()
                .any(|result| result.role == "text" && result.state == "vs panel.background"),
            "the injected text failure must be reported as unaccepted"
        );
    }

    ///
    /// [`unaccepted_failures`] answers empty once every actual failure is
    /// named in the accepted set — the gate's passing case, not only its
    /// rejecting one.
    ///
    #[test]
    fn unaccepted_failures_is_empty_when_every_failure_is_recorded() {
        let theme = Theme {
            text: okabe_ito().panel_background,
            ..okabe_ito()
        };
        // Okabe-Ito's own real failures (Sequence's two accepted Cursor
        // states; Sequence's own two pending plain/Region states and the
        // three pending invalid-operand Diagnostic states, none accepted
        // but present regardless of this fixture) plus both states `text`'s
        // override touches — `panel.background` and `input.background` are
        // both near-black in Okabe-Ito, so overriding `text` to
        // `panel.background` fails against either.
        let mut accepted = accepted_failures("okabe-ito").to_vec();
        accepted.extend_from_slice(&[
            ("Sequence, Pending", "plain"),
            ("Sequence, Pending", "Region"),
            ("Number, Invalid", "plain"),
            ("Number, Invalid", "Region"),
            ("Note, Invalid", "plain"),
            ("Note, Invalid", "Region"),
            ("Atom, Invalid", "plain"),
            ("Atom, Invalid", "Region"),
            ("text", "vs panel.background"),
            ("text", "vs input.background"),
        ]);

        let unaccepted = unaccepted_failures(&theme, &accepted);

        assert!(
            unaccepted.is_empty(),
            "expected no unaccepted failures, got {unaccepted:?}"
        );
    }

    ///
    /// The shipped-Theme gate itself: every failing state of every shipped
    /// Theme must be in `accepted_failures`, or this test fails and lists
    /// them. It is `#[ignore]`d rather than green, because it is not
    /// green: `pending_contrast_failures_are_confirmed_through_shipped_
    /// composition` above confirms four below-floor states —
    /// `Number, Invalid`, `Note, Invalid` and `Atom, Invalid` (each in
    /// `plain` and `Region`), plus `Sequence, Pending`'s own `plain` and
    /// `Region` states, which measure against Sequence's own tint rather
    /// than the bare Grid background the issue's acceptance line names —
    /// that `.scratch/theming/issues/08` explicitly says await acceptance
    /// and must **not** be silently whitelisted into `accepted_failures` to
    /// make this test pass. Un-ignore this test only once a human has
    /// accepted those failures (adding them to `accepted_failures` with
    /// that acceptance recorded in the issue) or retuned the colours
    /// involved — never by widening `accepted_failures` without either.
    ///
    /// Only `okabe_ito()` ships today: `.scratch/theming/issues/06`'s
    /// further built-ins and `07`'s loader are not built yet, so `shipped`
    /// is a hand-kept array rather than an iterated built-in registry — a
    /// known weakness recorded in `.scratch/theming/issues/08`'s comments.
    ///
    #[test]
    #[ignore = "pending human acceptance of four contrast failures — .scratch/theming/issues/08's \
                Known dark failures table: invalid Number (4.4469:1), Note (4.0537:1) and Atom \
                (3.9275:1) operand Diagnostic states, each in plain and Region, plus Sequence, \
                Pending's own plain/Region states (3.6707:1, against its own background tint, \
                not the bare Grid background the accepted Cursor/Region-Cursor states measure \
                against). Do not remove this ignore by adding any of them to accepted_failures; \
                only by an explicit human decision to accept or retune, recorded in the issue."]
    fn shipped_theme_gate() {
        let shipped = [okabe_ito()];

        for theme in &shipped {
            let unaccepted = unaccepted_failures(theme, accepted_failures(&theme.identity));
            assert!(
                unaccepted.is_empty(),
                "{}'s unrecorded contrast failures: {unaccepted:?}",
                theme.identity
            );
        }
    }
}
