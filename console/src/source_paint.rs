//!
//! Theme-owned Source Paint settings: the colour every Source Paint role — the
//! Source background and each Token's glyph — draws from, independently of the
//! fixed [`crate::style::PALETTE`].
//!
//! Mirrors [`crate::cursor_effects::CursorEffectSettings`], which
//! `syntax-highlighting/01` names as the shape to follow: one console-owned
//! settings value feeds both the Source Grid's rendering and the
//! `Theme → Source colours` UI, colour pickers use the opaque colour-edit
//! convention, and persistence has its own key with a default fallback.
//!

use egui::Color32;

///
/// The Okabe–Ito assignment `console/src/theme.md` records, as published in R
/// `grDevices`' `palette.colors("Okabe-Ito")` (Masataka Okabe & Kei Ito) —
/// except [`DEFAULT_ORDINARY`], a prototype pick rather than a named
/// Okabe–Ito swatch. Chosen in the Source Paint prototype
/// (`console/prototypes/syntax-highlighting/source-paint-prototype.html`,
/// `?variant=A&palette=okabe`).
///
pub(crate) const DEFAULT_SOURCE_BACKGROUND: Color32 = Color32::from_rgb(0, 0, 0); // #000000 black
pub(crate) const DEFAULT_ORDINARY: Color32 = Color32::from_rgb(255, 255, 255); // #FFFFFF
pub(crate) const DEFAULT_COMMENT: Color32 = Color32::from_rgb(153, 153, 153); // #999999 gray
pub(crate) const DEFAULT_FUNCTION: Color32 = Color32::from_rgb(0, 158, 115); // #009E73 bluish green
pub(crate) const DEFAULT_BANG: Color32 = Color32::from_rgb(204, 121, 167); // #CC79A7 reddish purple
pub(crate) const DEFAULT_NUMBER: Color32 = Color32::from_rgb(86, 180, 233); // #56B4E9 sky blue
pub(crate) const DEFAULT_NOTE: Color32 = Color32::from_rgb(240, 228, 66); // #F0E442 yellow
pub(crate) const DEFAULT_SEQUENCE: Color32 = Color32::from_rgb(0, 114, 178); // #0072B2 blue
pub(crate) const DEFAULT_DIAGNOSTIC: Color32 = Color32::from_rgb(213, 94, 0); // #D55E00 vermillion
pub(crate) const DEFAULT_RESULT: Color32 = Color32::from_rgb(230, 159, 0); // #E69F00 orange

///
/// `syntax-highlighting/02`'s default Fill tint strength: a Function or
/// Operand Cell's background, mixed toward its Token colour by this
/// percentage of the way from the Source background. Chosen in the Source
/// Paint prototype at the same seam ticket 01's colours were.
///
pub(crate) const DEFAULT_FILL_TINT: u8 = 16;

///
/// One opaque colour per Source Paint role: the Source background and every
/// Token's glyph.
///
/// Atom follows Ordinary, as Char already did — a Cell painting no glyph of
/// its own has nothing to colour differently. Sequence no longer shares
/// Ordinary's colour and carries its own field. Diagnostic and Result both
/// became settings before either had a painter, so the persisted shape was
/// complete once rather than gaining a field — and a migration — later.
/// `syntax-highlighting/04` gives Diagnostic its painter: an unbound
/// Function, Number, Note, Atom or Sequence entry draws its glyph in
/// Diagnostic instead of its Token colour. Result still has none until
/// `syntax-highlighting/06`.
///
/// `fill_tint` joins the ten colours as `syntax-highlighting/02`'s one
/// non-colour role: the percentage a Function or Operand Cell's background is
/// mixed toward its Token colour, away from `source_background`. It is a
/// percentage rather than a colour, so it is not one of the ten roles above,
/// but it lives on this settings value for the same reason they do — a
/// viewer's `Theme → Source colours` edit and persistence's own key.
///
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SourcePaintSettings {
    source_background: Color32,
    ordinary: Color32,
    comment: Color32,
    function: Color32,
    bang: Color32,
    number: Color32,
    note: Color32,
    sequence: Color32,
    diagnostic: Color32,
    result: Color32,
    fill_tint: u8,
}

impl Default for SourcePaintSettings {
    fn default() -> Self {
        Self {
            source_background: DEFAULT_SOURCE_BACKGROUND,
            ordinary: DEFAULT_ORDINARY,
            comment: DEFAULT_COMMENT,
            function: DEFAULT_FUNCTION,
            bang: DEFAULT_BANG,
            number: DEFAULT_NUMBER,
            note: DEFAULT_NOTE,
            sequence: DEFAULT_SEQUENCE,
            diagnostic: DEFAULT_DIAGNOSTIC,
            result: DEFAULT_RESULT,
            fill_tint: DEFAULT_FILL_TINT,
        }
    }
}

impl SourcePaintSettings {
    pub(crate) fn source_background(self) -> Color32 {
        self.source_background
    }
    pub(crate) fn ordinary(self) -> Color32 {
        self.ordinary
    }
    pub(crate) fn comment(self) -> Color32 {
        self.comment
    }
    pub(crate) fn function(self) -> Color32 {
        self.function
    }
    pub(crate) fn bang(self) -> Color32 {
        self.bang
    }
    pub(crate) fn number(self) -> Color32 {
        self.number
    }
    pub(crate) fn note(self) -> Color32 {
        self.note
    }
    pub(crate) fn sequence(self) -> Color32 {
        self.sequence
    }
    /// The glyph colour an unbound Function, Number, Note, Atom or Sequence
    /// entry draws with instead of its Token colour: `style::
    /// cell_visuals_with_cursor_colour` reads it wherever `LanguageMap::
    /// bound_at` answers `Some(false)` for one of those Tokens
    /// (syntax-highlighting/04).
    pub(crate) fn diagnostic(self) -> Color32 {
        self.diagnostic
    }
    // No shipped Token reaches this one yet — `syntax-highlighting/06` gives
    // Result the classification it colours, and `source_paint::tests` already
    // reads it to pin its default and contrast against the floor. `dead_code`
    // cannot see through `#[cfg(test)]` into a production build, so it is
    // silenced here rather than deleting an accessor the next ticket is about
    // to call from `paint.rs`.
    #[allow(dead_code)]
    pub(crate) fn result(self) -> Color32 {
        self.result
    }
    /// The Fill tint strength: what percentage of the way from the Source
    /// background to a Token colour a Function or Operand Cell's background
    /// is mixed. `0` paints no tint at all.
    pub(crate) fn fill_tint(self) -> u8 {
        self.fill_tint
    }

    pub(crate) fn source_background_mut(&mut self) -> &mut Color32 {
        &mut self.source_background
    }
    pub(crate) fn ordinary_mut(&mut self) -> &mut Color32 {
        &mut self.ordinary
    }
    pub(crate) fn comment_mut(&mut self) -> &mut Color32 {
        &mut self.comment
    }
    pub(crate) fn function_mut(&mut self) -> &mut Color32 {
        &mut self.function
    }
    pub(crate) fn bang_mut(&mut self) -> &mut Color32 {
        &mut self.bang
    }
    pub(crate) fn number_mut(&mut self) -> &mut Color32 {
        &mut self.number
    }
    pub(crate) fn note_mut(&mut self) -> &mut Color32 {
        &mut self.note
    }
    pub(crate) fn sequence_mut(&mut self) -> &mut Color32 {
        &mut self.sequence
    }
    pub(crate) fn diagnostic_mut(&mut self) -> &mut Color32 {
        &mut self.diagnostic
    }
    pub(crate) fn result_mut(&mut self) -> &mut Color32 {
        &mut self.result
    }
    pub(crate) fn fill_tint_mut(&mut self) -> &mut u8 {
        &mut self.fill_tint
    }

    ///
    /// Ten `r,g,b` groups and the Fill tint percentage, semicolon-joined in
    /// field order.
    ///
    /// Mirrors [`crate::cursor_effects::CursorEffectSettings::encode`]: a flat
    /// string an `eframe::Storage` value holds directly, so this settings
    /// value costs the workspace no serde dependency of its own.
    ///
    #[cfg(any(feature = "persistence", test))]
    pub(crate) fn encode(self) -> String {
        let channel = |colour: Color32| format!("{},{},{}", colour.r(), colour.g(), colour.b());
        let colours = [
            self.source_background,
            self.ordinary,
            self.comment,
            self.function,
            self.bang,
            self.number,
            self.note,
            self.sequence,
            self.diagnostic,
            self.result,
        ]
        .into_iter()
        .map(channel)
        .collect::<Vec<_>>()
        .join(";");
        format!("{colours};{}", self.fill_tint)
    }

    ///
    /// The inverse of [`Self::encode`], refusing anything that is not exactly
    /// ten well-formed `r,g,b` groups followed by a Fill tint percentage of
    /// `100` or less — a short, long, malformed, or out-of-range value is
    /// refused whole rather than partly decoded, so a caller falls back to
    /// [`Self::default`] instead of restoring some roles and defaulting
    /// others.
    ///
    #[cfg(any(feature = "persistence", test))]
    pub(crate) fn decode(value: &str) -> Option<Self> {
        let colour = |group: &str| -> Option<Color32> {
            let channels = group
                .split(',')
                .map(str::parse::<u8>)
                .collect::<Result<Vec<_>, _>>()
                .ok()?;
            (channels.len() == 3).then(|| Color32::from_rgb(channels[0], channels[1], channels[2]))
        };
        let mut groups = value.split(';');
        let settings = Self {
            source_background: colour(groups.next()?)?,
            ordinary: colour(groups.next()?)?,
            comment: colour(groups.next()?)?,
            function: colour(groups.next()?)?,
            bang: colour(groups.next()?)?,
            number: colour(groups.next()?)?,
            note: colour(groups.next()?)?,
            sequence: colour(groups.next()?)?,
            diagnostic: colour(groups.next()?)?,
            result: colour(groups.next()?)?,
            fill_tint: groups.next()?.parse().ok()?,
        };

        (groups.next().is_none() && settings.fill_tint <= 100).then_some(settings)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_the_okabe_ito_assignment() {
        let settings = SourcePaintSettings::default();

        assert_eq!(settings.source_background(), Color32::from_rgb(0, 0, 0));
        assert_eq!(settings.ordinary(), Color32::from_rgb(255, 255, 255));
        assert_eq!(settings.comment(), Color32::from_rgb(153, 153, 153));
        assert_eq!(settings.function(), Color32::from_rgb(0, 158, 115));
        assert_eq!(settings.bang(), Color32::from_rgb(204, 121, 167));
        assert_eq!(settings.number(), Color32::from_rgb(86, 180, 233));
        assert_eq!(settings.note(), Color32::from_rgb(240, 228, 66));
        assert_eq!(settings.sequence(), Color32::from_rgb(0, 114, 178));
        assert_eq!(settings.diagnostic(), Color32::from_rgb(213, 94, 0));
        assert_eq!(settings.result(), Color32::from_rgb(230, 159, 0));
        assert_eq!(settings.fill_tint(), 16);
    }

    ///
    /// Resetting is `= SourcePaintSettings::default()` at the call site
    /// (`console.rs`); what this pins is that the value a reset restores
    /// changes every one of the ten colours and the Fill tint percentage
    /// together rather than a subset of them, so an edit that only touched
    /// some fields' defaults would be caught here.
    ///
    #[test]
    fn every_role_differs_from_a_changed_settings_value_once_reset() {
        let mut changed = SourcePaintSettings::default();
        *changed.source_background_mut() = Color32::from_rgb(1, 1, 1);
        *changed.ordinary_mut() = Color32::from_rgb(2, 2, 2);
        *changed.comment_mut() = Color32::from_rgb(3, 3, 3);
        *changed.function_mut() = Color32::from_rgb(4, 4, 4);
        *changed.bang_mut() = Color32::from_rgb(5, 5, 5);
        *changed.number_mut() = Color32::from_rgb(6, 6, 6);
        *changed.note_mut() = Color32::from_rgb(7, 7, 7);
        *changed.sequence_mut() = Color32::from_rgb(8, 8, 8);
        *changed.diagnostic_mut() = Color32::from_rgb(9, 9, 9);
        *changed.result_mut() = Color32::from_rgb(10, 10, 10);
        *changed.fill_tint_mut() = 99;
        assert_ne!(changed, SourcePaintSettings::default());

        let reset = SourcePaintSettings::default();
        assert_eq!(reset, SourcePaintSettings::default());
        assert_ne!(reset, changed);
    }

    #[test]
    fn settings_round_trip_through_encode_and_decode() {
        let mut settings = SourcePaintSettings::default();
        *settings.source_background_mut() = Color32::from_rgb(1, 2, 3);
        *settings.ordinary_mut() = Color32::from_rgb(4, 5, 6);
        *settings.sequence_mut() = Color32::from_rgb(7, 8, 9);
        *settings.diagnostic_mut() = Color32::from_rgb(10, 11, 12);
        *settings.result_mut() = Color32::from_rgb(13, 14, 15);
        *settings.fill_tint_mut() = 42;

        assert_eq!(
            SourcePaintSettings::decode(&settings.encode()),
            Some(settings)
        );
    }

    #[test]
    fn malformed_or_incomplete_settings_are_refused_whole() {
        assert_eq!(SourcePaintSettings::decode("garbage"), None);
        assert_eq!(SourcePaintSettings::decode(""), None);

        // Ten groups: the ten colours alone, missing the Fill tint
        // percentage that makes eleven.
        let encoded = SourcePaintSettings::default().encode();
        let short = encoded.rsplit_once(';').expect("more than one group").0;
        assert_eq!(SourcePaintSettings::decode(short), None);

        // Twelve groups: one group too many.
        let long = format!("{encoded};1,2,3");
        assert_eq!(SourcePaintSettings::decode(&long), None);
    }

    ///
    /// The Fill tint percentage is a `u8`, so encoding never overflows it,
    /// but a stored value above the 0-100 range a viewer's Slider can reach
    /// is still refused rather than silently clamped — the same rule
    /// [`crate::cursor_effects::CursorEffectSettings::decode`] holds for its
    /// own `amount` and `frequency`.
    ///
    #[test]
    fn a_fill_tint_above_the_percentage_range_is_refused() {
        let mut settings = SourcePaintSettings::default();
        *settings.fill_tint_mut() = 100;
        let encoded = settings.encode();
        assert_eq!(SourcePaintSettings::decode(&encoded), Some(settings));

        let (prefix, _) = encoded.rsplit_once(';').expect("more than one group");
        let too_high = format!("{prefix};101");
        assert_eq!(SourcePaintSettings::decode(&too_high), None);
    }
}
