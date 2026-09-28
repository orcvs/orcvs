//!
//! Colour-vision separation of the Source glyph channels: a test-only check
//! on the built-in Themes. [`distinguish`] simulates dichromatic vision over
//! the same composited colours `validate` measures and reports how far
//! apart each pair of Source glyph channels stays. It answers a question
//! `validate` structurally cannot: two channels that each clear the
//! contrast floor against their own background can still read as one colour
//! to a dichromat.
//!
//! Its boundaries are stated at [`CONFUSION_FLOOR`] and [`ColourVision`]: the
//! floor is gated for the two red–green dichromacies and
//! measured-but-ungated for tritanopia, and it measures only the glyphs
//! painted on the background tints, never the tints against each other.
//!
//! Only `shipped_theme_colour_vision_gate` and its neighbours call it. It is
//! not applied to a loaded Theme document: the tritanopia judgement is made
//! for the built-ins' palette, not a policy a user Theme must satisfy.

use std::fmt;

use egui::Color32;

use super::{CursorPlacement, Role, painted};
use crate::theme::Theme;

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
const CONFUSION_FLOOR: f32 = 5.0;

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
enum ColourVision {
    Protanopia,
    Deuteranopia,
    Tritanopia,
}

impl ColourVision {
    const ALL: [Self; 3] = [Self::Protanopia, Self::Deuteranopia, Self::Tritanopia];

    /// Whether [`CONFUSION_FLOOR`] is a gate for this dichromacy or only a
    /// reference line beside a reported measurement — see this enum's own
    /// documentation for why tritanopia is the latter.
    fn gated(self) -> bool {
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
enum GlyphChannel {
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
/// paints on, the same pair `measure` takes a contrast ratio between — so
/// a translucent channel, a role tint, the Region wash and the doubled
/// Portal-over-role tint all count, exactly as they do on screen.
///
#[derive(Clone, Debug, PartialEq)]
struct ConfusionResult {
    vision: ColourVision,
    first: GlyphChannel,
    second: GlyphChannel,
    /// Where the two were closest. Both colours are read at this same
    /// placement: two Cells a reader compares are on one screen under one
    /// Cursor and Region state, so comparing a `plain` Cell's glyph with a
    /// `Region` Cell's would measure a difference the reader can already see
    /// from the wash.
    placement: CursorPlacement,
    first_colour: Color32,
    second_colour: Color32,
    /// CIEDE2000, between the two colours *after* simulation.
    distance: f32,
}

impl ConfusionResult {
    /// Whether [`Self::distance`] clears [`CONFUSION_FLOOR`]. Answered for
    /// every dichromacy, including the ungated one — whether a failure is a
    /// gate failure is [`ColourVision::gated`]'s question, not this one's.
    fn passes(&self) -> bool {
        self.distance >= CONFUSION_FLOOR
    }
}

///
/// [`distinguish`]'s complete answer for one Theme, shaped like
/// `ContrastReport`: the floor and scope carried in the data, then one
/// result per `(dichromacy, channel pair)`.
///
struct ConfusionReport {
    floor: f32,
    scope: &'static str,
    results: Vec<ConfusionResult>,
}

/// [`ConfusionReport::scope`]'s exact text.
const CONFUSION_SCOPE: &str = "Pairwise separation of the Source glyph channels under \
simulated dichromacy, measured on the composited colours each one actually displays. The \
floor gates protanopia and deuteranopia; tritanopia is measured and reported, not gated. Not \
background tints against each other, not text contrast, not border/focus visibility, and not \
anomalous trichromacy, which is a continuum this reports the endpoint of.";

///
/// Measures, for every pair of distinct [`GlyphChannel`]s and every
/// [`ColourVision`], the smallest CIEDE2000 separation the two keep once
/// both are simulated — over the same reachable painted states `validate`
/// measures contrast for, and over the colours those states actually
/// display.
///
/// Never refuses `theme`, for `validate`'s reason: a scheme that confuses
/// two channels is measured and reported exactly like one that does not.
///
fn distinguish(theme: &Theme) -> ConfusionReport {
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
/// window backdrop first, as `contrast`'s callers do, because
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

impl Role {
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
    /// showing, which is how a Theme carries a fact on the background
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
}

mod tests {
    use egui::Color32;

    use super::{
        CONFUSION_FLOOR, ColourVision, ConfusionReport, ConfusionResult, GlyphChannel, difference,
        distinguish, lab, simulate,
    };
    use crate::contrast::tests::unaccepted;
    use crate::contrast::{contrast, validate};
    use crate::theme::{ORCVS_LIGHT_IDENTITY, Theme, okabe_ito, orcvs_light};

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
    /// which is why this test measures the collapse as a ratio rather than
    /// asserting the remainder falls under [`CONFUSION_FLOOR`].
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
    /// A light glyph definition that clears contrast and fails colour
    /// vision: every painted state [`validate`] measures clears
    /// [`crate::contrast::CONTRAST_FLOOR`], and it still fails here, which is
    /// the whole reason this second measurement exists.
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
    /// Why this gate measures a simulated colour difference rather than
    /// relative luminance. The rejected definition's Function `#077055` and
    /// Bang `#AD2A3B` are within 1.09:1 of each other in relative luminance
    /// — on a greyscale display they are one tone — and a dichromat still
    /// separates them easily, because every dichromacy keeps lightness *and*
    /// one chromatic axis. Equal luminance is therefore a fact about those two colours,
    /// not a verdict on them: the pair measures 14.60 under deuteranopia,
    /// nearly three times [`CONFUSION_FLOOR`].
    ///
    /// The corollary: since lightness survives every dichromacy, lightness
    /// is what separates two colours a dichromacy would otherwise merge —
    /// which is why Orcvs Light's Diagnostic and Output Portal are darkened
    /// past what contrast alone asks for.
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
    /// A light glyph definition that clears contrast and fails colour
    /// vision. Only the glyph channels differ from the shipped light Theme:
    /// holding the chrome keys fixed is what isolates the glyphs.
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
    /// darkens that assignment's hues.
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
