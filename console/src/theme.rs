//!
//! The resolved Orcvs Theme: named colour, width and optional-fill
//! properties the whole console paints from, and the pure inheritance
//! resolver that builds a custom Theme's resolved values from a built-in and
//! a sparse document of overrides.
//!
//! `.scratch/theming/schema.md` is the implementation contract this module
//! follows — the property catalogue, the dark built-in's exact values, and
//! the composition/inheritance rules.
//! This module owns the resolved [`Theme`], the built-ins and the property
//! catalogue: what the console paints from on every target. It has no file
//! I/O and no document model: the crate-private `theme_document` module
//! decodes a document's bytes into a `ThemeDocument` and resolves it, and the
//! crate-private `theme_registry` module's native discovery reads those bytes.
//!

use std::borrow::Cow;

use egui::Color32;

// === Appearance ===

///
/// A Theme's declared light/dark appearance. Only a Theme of matching
/// appearance can be a custom Theme's parent.
///
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Appearance {
    Dark,
    /// [`orcvs_light`] declares this, and a custom document inherits it from
    /// a light parent. Declaring `appearance = "light"` only restates the
    /// parent's appearance: [`crate::theme_document::resolve`] refuses one that
    /// differs.
    Light,
}

///
/// egui's own dark/light appearance is the one a Theme declares: the console
/// registers each appearance's Theme in the egui slot of the same name
/// (`crate::style::install`), and presents the Source from the Theme of the
/// appearance egui is presenting.
///
impl Appearance {
    /// The appearance as a viewer reads it in a sentence.
    pub(crate) const fn word(self) -> &'static str {
        match self {
            Self::Dark => "dark",
            Self::Light => "light",
        }
    }
}

impl From<egui::Theme> for Appearance {
    fn from(theme: egui::Theme) -> Self {
        match theme {
            egui::Theme::Dark => Self::Dark,
            egui::Theme::Light => Self::Light,
        }
    }
}

// === Bounded widths ===

///
/// A Grid, Cell or Sector Seam stroke width in display points: 0 to 1
/// inclusive (`schema.md`'s width catalogue). Zero hides the stroke; the
/// value is fixed in display points and never scales with the Cell size.
///
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct GridWidth(f32);

impl GridWidth {
    pub(crate) const MAX_POINTS: f32 = 1.0;

    /// `None` unless `points` is finite and within `0..=MAX_POINTS`.
    pub(crate) fn from_points(points: f32) -> Option<Self> {
        (points.is_finite() && (0.0..=Self::MAX_POINTS).contains(&points)).then_some(Self(points))
    }

    /// Consumed by `console::shapes`'s Grid line, Sector Seam and selection
    /// outline widths.
    pub(crate) fn points(self) -> f32 {
        self.0
    }
}

///
/// A chrome (panel/widget/selection/input-caret) stroke width in display
/// points: 0 to 2 inclusive (`schema.md`'s width catalogue).
///
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct ChromeWidth(f32);

impl ChromeWidth {
    pub(crate) const MAX_POINTS: f32 = 2.0;

    /// `None` unless `points` is finite and within `0..=MAX_POINTS`.
    pub(crate) fn from_points(points: f32) -> Option<Self> {
        (points.is_finite() && (0.0..=Self::MAX_POINTS).contains(&points)).then_some(Self(points))
    }

    /// Consumed by `style.rs::style`'s chrome borders, input caret and IME
    /// underline widths.
    pub(crate) fn points(self) -> f32 {
        self.0
    }
}

// === Property keys ===

///
/// The `style` property catalogue, written once: the two optional Cursor
/// fills, which [`crate::theme_document::OptionalFill`] carries and no key enum holds, then each
/// key enum's variants, each with its field — the Theme's, and the Theme
/// document decoder's style table's — and its literal, case- and
/// dot-sensitive `schema.md` spelling.
/// `style_catalogue!(then)` hands the whole table to the macro `then`: in
/// `theme_document`, `property_names`, which derives each key's `name`,
/// `property_keys`, which derives the key enums and the Theme's routing by
/// key, and the derived style table, so a property's spelling, its key, its
/// Theme field and its document field cannot disagree. A field repeated in the table
/// fails to compile, and a spelling repeated is an unreachable arm in the
/// derived table's key match, which the workspace's `-D warnings` gate
/// refuses.
///
#[cfg(any(not(target_arch = "wasm32"), test))]
macro_rules! style_catalogue {
    ($then:ident) => {
        $then! {
            OptionalFill {
                CursorBackground(cursor_background) => "cursor.background",
                RegionCursorBackground(region_cursor_background) => "region.cursor.background",
            }
            ColorKey {
                WindowBackground(window_background) => "window.background",
                PanelBackground(panel_background) => "panel.background",
                GridBackground(grid_background) => "grid.background",
                CellBackground(cell_background) => "cell.background",
                SourceOrdinary(source_ordinary) => "source.ordinary",
                SourceComment(source_comment) => "source.comment",
                SourceNumber(source_number) => "source.number",
                SourceNote(source_note) => "source.note",
                SourceFunction(source_function) => "source.function",
                SourceBang(source_bang) => "source.bang",
                SourceOrdinaryBackground(source_ordinary_background) => "source.ordinary.background",
                SourceCommentBackground(source_comment_background) => "source.comment.background",
                SourceNumberBackground(source_number_background) => "source.number.background",
                SourceNoteBackground(source_note_background) => "source.note.background",
                SourceFunctionBackground(source_function_background) => "source.function.background",
                SourceBangBackground(source_bang_background) => "source.bang.background",
                DiagnosticForeground(diagnostic_foreground) => "diagnostic.foreground",
                DiagnosticBackground(diagnostic_background) => "diagnostic.background",
                DiagnosticBorder(diagnostic_border) => "diagnostic.border",
                OutputPortalForeground(output_portal_foreground) => "output_portal.foreground",
                OutputPortalBackground(output_portal_background) => "output_portal.background",
                OutputPortalBorder(output_portal_border) => "output_portal.border",
                InputPortalForeground(input_portal_foreground) => "input_portal.foreground",
                InputPortalBackground(input_portal_background) => "input_portal.background",
                InputPortalBorder(input_portal_border) => "input_portal.border",
                LockedForeground(locked_foreground) => "locked.foreground",
                LockedBackground(locked_background) => "locked.background",
                LockedBorder(locked_border) => "locked.border",
                GridBorder(grid_border) => "grid.border",
                SectorSeam(sector_seam) => "sector.seam",
                CursorBorder(cursor_border) => "cursor.border",
                RegionBorder(region_border) => "region.border",
                RegionBackground(region_background) => "region.background",
                PanelBorder(panel_border) => "panel.border",
                SelectionBackground(selection_background) => "selection.background",
                SelectionBorder(selection_border) => "selection.border",
                SelectionBorderRest(selection_border_rest) => "selection.border.rest",
                WidgetInactiveBorder(widget_inactive_border) => "widget.inactive.border",
                Text(text) => "text",
                TextActive(text_active) => "text.active",
                TextMuted(text_muted) => "text.muted",
                InputBackground(input_background) => "input.background",
                Link(link) => "link",
                CodeBackground(code_background) => "code.background",
                InputCursor(input_cursor) => "input.cursor",
                Error(error) => "error",
                Warning(warning) => "warning",
            }
            GridWidthKey {
                GridBorder(grid_border_width) => "grid.border.width",
                SectorSeam(sector_seam_width) => "sector.seam.width",
                CellSelectionBorder(cell_selection_border_width) => "cell.selection.border.width",
                CursorBorder(cursor_border_width) => "cursor.border.width",
                RegionBorder(region_border_width) => "region.border.width",
                DiagnosticBorder(diagnostic_border_width) => "diagnostic.border.width",
                OutputPortalBorder(output_portal_border_width) => "output_portal.border.width",
                InputPortalBorder(input_portal_border_width) => "input_portal.border.width",
                LockedBorder(locked_border_width) => "locked.border.width",
            }
            ChromeWidthKey {
                PanelBorder(panel_border_width) => "panel.border.width",
                SelectionBorder(selection_border_width) => "selection.border.width",
                WidgetBorder(widget_border_width) => "widget.border.width",
                WidgetInactiveBorder(widget_inactive_border_width) => "widget.inactive.border.width",
                InputCursor(input_cursor_width) => "input.cursor.width",
            }
        }
    };
}
#[cfg(any(not(target_arch = "wasm32"), test))]
pub(crate) use style_catalogue;

// === Resolved Theme ===

///
/// A fully resolved Theme: every named property `schema.md` catalogues,
/// with no unset value. Built-ins are constructed with every field spelled
/// out explicitly (see [`okabe_ito`]), so the compiler — not a runtime
/// check — is what proves a built-in defines every key: omit a field from
/// the struct literal and the crate fails to build. A custom Theme's
/// resolved value is a built-in's, copied and selectively replaced by the
/// pure `theme_document::resolve`.
///
/// Fields are `pub(crate)` rather than encapsulated behind accessors: a
/// resolved `Theme` is immutable once built — nothing edits one in place or
/// persists it field-by-field — so there is no invariant an accessor needs to
/// protect. Only `theme_document` addresses a field by key.
///
#[derive(Clone, Debug, PartialEq)]
pub struct Theme {
    /// The identity a Theme is selected by: a built-in's reserved name, or
    /// a custom Theme's filename stem, assigned by the crate-private
    /// `theme_registry` module outside this one.
    pub(crate) identity: ThemeIdentity,
    /// The declared display label (`name` in `schema.md`'s document).
    /// Never used to select a Theme.
    pub(crate) name: String,
    pub(crate) appearance: Appearance,

    pub(crate) window_background: Color32,
    pub(crate) panel_background: Color32,
    pub(crate) grid_background: Color32,
    pub(crate) cell_background: Color32,

    pub(crate) source_ordinary: Color32,
    pub(crate) source_comment: Color32,
    pub(crate) source_number: Color32,
    pub(crate) source_note: Color32,
    pub(crate) source_function: Color32,
    pub(crate) source_bang: Color32,
    pub(crate) source_ordinary_background: Color32,
    pub(crate) source_comment_background: Color32,
    pub(crate) source_number_background: Color32,
    pub(crate) source_note_background: Color32,
    pub(crate) source_function_background: Color32,
    pub(crate) source_bang_background: Color32,

    pub(crate) diagnostic_foreground: Color32,
    pub(crate) diagnostic_background: Color32,
    pub(crate) diagnostic_border: Color32,
    pub(crate) output_portal_foreground: Color32,
    pub(crate) output_portal_background: Color32,
    pub(crate) output_portal_border: Color32,
    pub(crate) input_portal_foreground: Color32,
    pub(crate) input_portal_background: Color32,
    pub(crate) input_portal_border: Color32,
    pub(crate) locked_foreground: Color32,
    pub(crate) locked_background: Color32,
    pub(crate) locked_border: Color32,

    pub(crate) grid_border: Color32,
    pub(crate) sector_seam: Color32,
    pub(crate) cursor_border: Color32,
    pub(crate) region_border: Color32,
    pub(crate) region_background: Color32,

    pub(crate) panel_border: Color32,
    pub(crate) selection_background: Color32,
    pub(crate) selection_border: Color32,
    pub(crate) selection_border_rest: Color32,
    pub(crate) widget_inactive_border: Color32,

    pub(crate) text: Color32,
    pub(crate) text_active: Color32,
    pub(crate) text_muted: Color32,
    pub(crate) input_background: Color32,
    pub(crate) link: Color32,
    pub(crate) code_background: Color32,
    pub(crate) input_cursor: Color32,
    pub(crate) error: Color32,
    pub(crate) warning: Color32,

    /// Omitted in a document: inherit the parent's value. `None` here: the
    /// optional fill is cleared, enabling its existing fallback. `Some`: an
    /// explicit colour, including transparent. See
    /// [`crate::theme_document::OptionalFill`].
    pub(crate) cursor_background: Option<Color32>,
    pub(crate) region_cursor_background: Option<Color32>,

    pub(crate) grid_border_width: GridWidth,
    pub(crate) sector_seam_width: GridWidth,
    pub(crate) cell_selection_border_width: GridWidth,
    pub(crate) cursor_border_width: GridWidth,
    pub(crate) region_border_width: GridWidth,
    pub(crate) diagnostic_border_width: GridWidth,
    pub(crate) output_portal_border_width: GridWidth,
    pub(crate) input_portal_border_width: GridWidth,
    pub(crate) locked_border_width: GridWidth,

    pub(crate) panel_border_width: ChromeWidth,
    pub(crate) selection_border_width: ChromeWidth,
    pub(crate) widget_border_width: ChromeWidth,
    pub(crate) widget_inactive_border_width: ChromeWidth,
    pub(crate) input_cursor_width: ChromeWidth,
}

impl Theme {
    ///
    /// The fill of the Cursor's Cell while a Region spans more than one Cell:
    /// `region.cursor.background` when it holds a colour, transparent
    /// included, and `cursor.background` only where it is cleared. Painting
    /// and the contrast check both read it here, so the fallback has one
    /// statement.
    ///
    pub(crate) fn region_cursor_fill(&self) -> Option<Color32> {
        self.region_cursor_background.or(self.cursor_background)
    }
}

///
/// Decodes a straight (non-premultiplied) `0xRRGGBBAA` literal — the way
/// `schema.md` writes hex colours — into the [`Color32`] every resolved
/// Theme value is stored as. `Color32`'s own byte layout is premultiplied
/// alpha (`ecolor::Color32`'s documentation), so every literal below goes
/// through this straight-to-premultiplied conversion via
/// [`Color32::from_rgba_unmultiplied_const`] rather than assuming the
/// schema's straight bytes are already what `Color32` stores, so a resolved
/// Theme value composites with painting code the same way every other
/// `Color32` constant in the console does.
///
pub(crate) const fn straight_rgba(rgba: u32) -> Color32 {
    let [r, g, b, a] = rgba.to_be_bytes();
    Color32::from_rgba_unmultiplied_const(r, g, b, a)
}

// === Identity ===

///
/// The name a Theme is selected by, never its display label.
///
/// A built-in's is reserved and fixed here; a custom Theme's is the stem of
/// its file name, which the Theme registry supplies. A saved
/// selection is restored as one too, though it may name no Theme at all:
/// the fallback keeps it rather than rewriting it.
///
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct ThemeIdentity(Cow<'static, str>);

impl ThemeIdentity {
    const fn reserved(identity: &'static str) -> Self {
        Self(Cow::Borrowed(identity))
    }

    ///
    /// A custom Theme's identity: its file name's stem. `None` for an empty
    /// stem, which names nothing.
    ///
    #[cfg(any(not(target_arch = "wasm32"), test))]
    pub(crate) fn from_stem(stem: &str) -> Option<Self> {
        (!stem.is_empty()).then(|| Self(Cow::Owned(stem.to_owned())))
    }

    ///
    /// The identity a setting names, as written in `~/.orcvs/config.toml`.
    /// Unchecked: `SelectedThemes` reports one no Theme of its appearance
    /// answers to.
    ///
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) fn configured(identity: String) -> Self {
        Self(Cow::Owned(identity))
    }

    ///
    /// The built-in each appearance starts with and falls back to when its
    /// selection names no available Theme of that appearance.
    ///
    pub(crate) const fn default_for(appearance: Appearance) -> Self {
        match appearance {
            Appearance::Dark => OKABE_ITO_IDENTITY,
            Appearance::Light => ORCVS_LIGHT_IDENTITY,
        }
    }

    #[cfg(any(not(target_arch = "wasm32"), test))]
    pub(crate) fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for ThemeIdentity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// The reserved identity of the Okabe–Ito built-in, and the dark Theme
/// selection's default and fallback.
///
pub(crate) const OKABE_ITO_IDENTITY: ThemeIdentity = ThemeIdentity::reserved("okabe-ito");

///
/// The Okabe–Ito built-in dark Theme: `schema.md`'s complete dark
/// definition, at the reserved identity `OKABE_ITO_IDENTITY`. Every field
/// is spelled out explicitly (see [`Theme`]).
///
/// `okabe_ito_defines_every_key_at_the_schema_values` in this module's tests
/// pins every field against `schema.md`'s own dark table, and
/// `style::tests::okabe_ito_chrome_matches_the_decided_record` cross-checks
/// the chrome keys against `theme.md`'s decided record read through
/// [`crate::style::style`], so this built-in reproduces the shipped
/// appearance and not merely the document `examples/okabe-ito-copy.toml`
/// records.
///
pub fn okabe_ito() -> Theme {
    Theme {
        identity: OKABE_ITO_IDENTITY,
        name: "Okabe–Ito".to_owned(),
        appearance: Appearance::Dark,

        window_background: straight_rgba(0x0B_11_12_FF),
        panel_background: straight_rgba(0x0B_11_12_FF),
        grid_background: straight_rgba(0x00_00_00_FF),
        cell_background: straight_rgba(0x00_00_00_00),

        source_ordinary: straight_rgba(0xEA_EB_E5_FF),
        source_comment: straight_rgba(0x99_99_99_FF),
        source_number: straight_rgba(0x56_B4_E9_FF),
        source_note: straight_rgba(0xF0_E4_42_FF),
        source_function: straight_rgba(0x00_9E_73_FF),
        source_bang: straight_rgba(0xCC_79_A7_FF),
        source_ordinary_background: straight_rgba(0x00_00_00_00),
        source_comment_background: straight_rgba(0x00_00_00_00),
        source_number_background: straight_rgba(0x56_B4_E9_1A),
        source_note_background: straight_rgba(0xF0_E4_42_1A),
        source_function_background: straight_rgba(0x00_9E_73_1A),
        source_bang_background: straight_rgba(0x00_00_00_00),

        diagnostic_foreground: straight_rgba(0xD5_5E_00_FF),
        diagnostic_background: straight_rgba(0x00_00_00_00),
        diagnostic_border: straight_rgba(0x00_00_00_00),
        output_portal_foreground: straight_rgba(0xE6_9F_00_FF),
        output_portal_background: straight_rgba(0xE6_9F_00_1A),
        output_portal_border: straight_rgba(0x00_00_00_00),
        input_portal_foreground: straight_rgba(0xAC_77_FA_FF),
        input_portal_background: straight_rgba(0xAC_77_FA_1A),
        input_portal_border: straight_rgba(0x00_00_00_00),
        locked_foreground: straight_rgba(0x00_00_00_00),
        locked_background: straight_rgba(0x00_00_00_CC),
        locked_border: straight_rgba(0xEA_EB_E5_80),

        grid_border: straight_rgba(0x1D_37_31_48),
        sector_seam: straight_rgba(0x37_65_56_6E),
        cursor_border: straight_rgba(0xEA_EB_E5_FF),
        region_border: straight_rgba(0xEA_EB_E5_FF),
        region_background: straight_rgba(0xFF_FF_FF_2B),

        panel_border: straight_rgba(0x1C_39_32_FF),
        selection_background: straight_rgba(0x0A_2A_22_FF),
        selection_border: straight_rgba(0x65_E6_BE_FF),
        selection_border_rest: straight_rgba(0x52_C3_A3_FF),
        widget_inactive_border: straight_rgba(0x1C_39_32_FF),

        text: straight_rgba(0xEA_EB_E5_FF),
        text_active: straight_rgba(0x65_E6_BE_FF),
        text_muted: straight_rgba(0xE9_EB_E4_99),
        input_background: straight_rgba(0x00_00_00_FF),
        link: straight_rgba(0x5A_AA_FF_FF),
        code_background: straight_rgba(0x40_40_40_FF),
        input_cursor: straight_rgba(0xC0_DE_FF_FF),
        error: straight_rgba(0xCC_79_A7_FF),
        warning: straight_rgba(0xCC_79_A7_FF),

        cursor_background: None,
        region_cursor_background: None,

        grid_border_width: GridWidth::from_points(0.5).expect("0.5 is within 0..=1"),
        sector_seam_width: GridWidth::from_points(0.75).expect("0.75 is within 0..=1"),
        cell_selection_border_width: GridWidth::from_points(0.5).expect("0.5 is within 0..=1"),
        cursor_border_width: GridWidth::from_points(1.0).expect("1.0 is within 0..=1"),
        region_border_width: GridWidth::from_points(1.0).expect("1.0 is within 0..=1"),
        diagnostic_border_width: GridWidth::from_points(0.5).expect("0.5 is within 0..=1"),
        output_portal_border_width: GridWidth::from_points(0.5).expect("0.5 is within 0..=1"),
        input_portal_border_width: GridWidth::from_points(0.5).expect("0.5 is within 0..=1"),
        locked_border_width: GridWidth::from_points(0.5).expect("0.5 is within 0..=1"),

        panel_border_width: ChromeWidth::from_points(1.0).expect("1.0 is within 0..=2"),
        selection_border_width: ChromeWidth::from_points(1.0).expect("1.0 is within 0..=2"),
        widget_border_width: ChromeWidth::from_points(1.0).expect("1.0 is within 0..=2"),
        widget_inactive_border_width: ChromeWidth::from_points(0.0).expect("0.0 is within 0..=2"),
        input_cursor_width: ChromeWidth::from_points(2.0).expect("2.0 is within 0..=2"),
    }
}

/// The reserved identity of the light built-in, named by `schema.md`
/// ("`orcvs-light` the reserved light identity") and recorded in
/// `console/src/theme.md`'s Shipped Themes table.
pub(crate) const ORCVS_LIGHT_IDENTITY: ThemeIdentity = ThemeIdentity::reserved("orcvs-light");

///
/// The Orcvs Light built-in Theme: the complete light definition, at the
/// reserved identity `ORCVS_LIGHT_IDENTITY`. Every field is spelled out
/// explicitly for the same reason [`okabe_ito`]'s is, including the
/// properties whose value it shares with the dark built-in, so that a shared
/// value is recorded rather than left to a toolkit default.
///
/// The chrome colours map a hand-tuned light palette onto the named keys:
/// page → `window.background`/`panel.background`, source →
/// `grid.background`, grid line → `grid.border`, sector line →
/// `sector.seam`, selection fill → `selection.background`, and so on.
///
/// The Source glyph hues are the Okabe–Ito palette under the role-to-hue
/// assignment the dark built-in uses (Number sky blue, Note yellow, Function
/// bluish green, Bang reddish purple, Diagnostic vermillion, Output Portal
/// orange). Each keeps its hue exactly — the OKLCh hue angle is
/// held to within 0.7° — and moves lightness alone: as far as a near-white
/// ground requires and, for Diagnostic and Output Portal, further still, so
/// that a red–green colour-blind reader tells every pair of glyph roles
/// apart. Do not pick a glyph hue outside that assignment: [`okabe_ito`]
/// keeps its roles apart under protanopia and deuteranopia by construction,
/// and a hand-picked hue can land two roles on what those readers see as one
/// colour. Input Portal is the one glyph role that assignment leaves without
/// a hue, and it is a violet, the dark built-in's hue at a lower lightness;
/// `contrast::colour_vision`'s gate holds it apart from every other role in
/// both built-ins. `console/src/theme.md` records
/// each value, its hue and lightness, the measured ratios and the
/// colour-vision separations.
///
/// This is the light Theme selection's default and fallback
/// (`crate::theme_selection`), which `crate::style::install` registers for
/// `egui::Theme::Light`.
///
/// `orcvs_light_defines_every_key_at_the_recorded_values` in this module's
/// tests pins every field against that record,
/// `style::tests::orcvs_light_chrome_matches_the_decided_record` cross-checks
/// the chrome keys through [`crate::style::style`], and the contrast module's
/// `shipped_theme_colour_vision_gate` holds every glyph pair apart under
/// simulated protanopia and deuteranopia.
///
pub fn orcvs_light() -> Theme {
    Theme {
        identity: ORCVS_LIGHT_IDENTITY,
        name: "Orcvs Light".to_owned(),
        appearance: Appearance::Light,

        window_background: straight_rgba(0xEF_F4_F2_FF),
        panel_background: straight_rgba(0xEF_F4_F2_FF),
        grid_background: straight_rgba(0xFA_FC_FB_FF),
        cell_background: straight_rgba(0x00_00_00_00),

        source_ordinary: straight_rgba(0x30_3F_3B_FF),
        source_comment: straight_rgba(0x4E_5A_56_FF),
        source_number: straight_rgba(0x00_6D_9B_FF),
        source_note: straight_rgba(0x70_69_00_FF),
        source_function: straight_rgba(0x00_75_55_FF),
        source_bang: straight_rgba(0x90_42_6F_FF),
        source_ordinary_background: straight_rgba(0x00_00_00_00),
        source_comment_background: straight_rgba(0x00_00_00_00),
        source_number_background: straight_rgba(0x00_6D_9B_1A),
        source_note_background: straight_rgba(0x70_69_00_1A),
        source_function_background: straight_rgba(0x00_75_55_1A),
        source_bang_background: straight_rgba(0x00_00_00_00),

        diagnostic_foreground: straight_rgba(0x65_28_00_FF),
        diagnostic_background: straight_rgba(0x00_00_00_00),
        diagnostic_border: straight_rgba(0x00_00_00_00),
        output_portal_foreground: straight_rgba(0x6F_4A_00_FF),
        output_portal_background: straight_rgba(0x6F_4A_00_1A),
        output_portal_border: straight_rgba(0x00_00_00_00),
        input_portal_foreground: straight_rgba(0x64_07_AF_FF),
        input_portal_background: straight_rgba(0x64_07_AF_1A),
        input_portal_border: straight_rgba(0x00_00_00_00),
        locked_foreground: straight_rgba(0x00_00_00_00),
        locked_background: straight_rgba(0xFA_FC_FB_CC),
        locked_border: straight_rgba(0x30_3F_3B_80),

        grid_border: straight_rgba(0x34_5B_50_40),
        sector_seam: straight_rgba(0x26_68_54_70),
        cursor_border: straight_rgba(0x30_3F_3B_FF),
        region_border: straight_rgba(0x30_3F_3B_FF),
        region_background: straight_rgba(0x30_3F_3B_2B),

        panel_border: straight_rgba(0xC0_CE_C9_FF),
        selection_background: straight_rgba(0xCC_EB_E2_FF),
        selection_border: straight_rgba(0x07_62_47_FF),
        selection_border_rest: straight_rgba(0x18_7E_60_FF),
        widget_inactive_border: straight_rgba(0xC0_CE_C9_FF),

        text: straight_rgba(0x30_3F_3B_FF),
        text_active: straight_rgba(0x07_62_47_FF),
        text_muted: straight_rgba(0x30_3F_3B_BF),
        input_background: straight_rgba(0xFA_FC_FB_FF),
        link: straight_rgba(0x0B_62_B8_FF),
        code_background: straight_rgba(0xE6_E6_E6_FF),
        input_cursor: straight_rgba(0x00_53_7D_FF),
        error: straight_rgba(0x90_42_6F_FF),
        warning: straight_rgba(0x90_42_6F_FF),

        cursor_background: None,
        region_cursor_background: None,

        grid_border_width: GridWidth::from_points(0.5).expect("0.5 is within 0..=1"),
        sector_seam_width: GridWidth::from_points(0.75).expect("0.75 is within 0..=1"),
        cell_selection_border_width: GridWidth::from_points(0.5).expect("0.5 is within 0..=1"),
        cursor_border_width: GridWidth::from_points(1.0).expect("1.0 is within 0..=1"),
        region_border_width: GridWidth::from_points(1.0).expect("1.0 is within 0..=1"),
        diagnostic_border_width: GridWidth::from_points(0.5).expect("0.5 is within 0..=1"),
        output_portal_border_width: GridWidth::from_points(0.5).expect("0.5 is within 0..=1"),
        input_portal_border_width: GridWidth::from_points(0.5).expect("0.5 is within 0..=1"),
        locked_border_width: GridWidth::from_points(0.5).expect("0.5 is within 0..=1"),

        panel_border_width: ChromeWidth::from_points(1.0).expect("1.0 is within 0..=2"),
        selection_border_width: ChromeWidth::from_points(1.0).expect("1.0 is within 0..=2"),
        widget_border_width: ChromeWidth::from_points(1.0).expect("1.0 is within 0..=2"),
        widget_inactive_border_width: ChromeWidth::from_points(0.0).expect("0.0 is within 0..=2"),
        input_cursor_width: ChromeWidth::from_points(2.0).expect("2.0 is within 0..=2"),
    }
}

#[cfg(test)]
mod tests {
    use egui::Color32;

    use super::{Appearance, ChromeWidth, GridWidth, okabe_ito, orcvs_light, straight_rgba};

    ///
    /// Restates `schema.md`'s dark property table as `Theme` field
    /// assertions — the same pinning-by-restatement `style.rs`'s
    /// `okabe_ito_chrome_matches_the_decided_record` uses for the chrome
    /// keys. This test is intentionally close to `okabe_ito`'s own body: a
    /// later edit to either one without the other is exactly what it is
    /// meant to catch.
    ///
    #[test]
    fn okabe_ito_defines_every_key_at_the_schema_values() {
        let theme = okabe_ito();

        assert_eq!(theme.identity.as_str(), "okabe-ito");
        assert_eq!(theme.appearance, Appearance::Dark);

        assert_eq!(theme.window_background, straight_rgba(0x0B_11_12_FF));
        assert_eq!(theme.panel_background, straight_rgba(0x0B_11_12_FF));
        assert_eq!(theme.grid_background, straight_rgba(0x00_00_00_FF));
        assert_eq!(theme.cell_background, straight_rgba(0x00_00_00_00));

        assert_eq!(theme.source_ordinary, straight_rgba(0xEA_EB_E5_FF));
        assert_eq!(theme.source_comment, straight_rgba(0x99_99_99_FF));
        assert_eq!(theme.source_number, straight_rgba(0x56_B4_E9_FF));
        assert_eq!(theme.source_note, straight_rgba(0xF0_E4_42_FF));
        assert_eq!(theme.source_function, straight_rgba(0x00_9E_73_FF));
        assert_eq!(theme.source_bang, straight_rgba(0xCC_79_A7_FF));
        assert_eq!(
            theme.source_ordinary_background,
            straight_rgba(0x00_00_00_00)
        );
        assert_eq!(
            theme.source_comment_background,
            straight_rgba(0x00_00_00_00)
        );
        assert_eq!(theme.source_number_background, straight_rgba(0x56_B4_E9_1A));
        assert_eq!(theme.source_note_background, straight_rgba(0xF0_E4_42_1A));
        assert_eq!(
            theme.source_function_background,
            straight_rgba(0x00_9E_73_1A)
        );
        assert_eq!(theme.source_bang_background, straight_rgba(0x00_00_00_00));

        assert_eq!(theme.diagnostic_foreground, straight_rgba(0xD5_5E_00_FF));
        assert_eq!(theme.diagnostic_background, straight_rgba(0x00_00_00_00));
        assert_eq!(theme.diagnostic_border, straight_rgba(0x00_00_00_00));
        assert_eq!(theme.output_portal_foreground, straight_rgba(0xE6_9F_00_FF));
        assert_eq!(theme.output_portal_background, straight_rgba(0xE6_9F_00_1A));
        assert_eq!(theme.output_portal_border, straight_rgba(0x00_00_00_00));
        assert_eq!(theme.input_portal_foreground, straight_rgba(0xAC_77_FA_FF));
        assert_eq!(theme.input_portal_background, straight_rgba(0xAC_77_FA_1A));
        assert_eq!(theme.input_portal_border, straight_rgba(0x00_00_00_00));
        assert_eq!(theme.locked_foreground, straight_rgba(0x00_00_00_00));
        assert_eq!(theme.locked_background, straight_rgba(0x00_00_00_CC));
        assert_eq!(theme.locked_border, straight_rgba(0xEA_EB_E5_80));

        assert_eq!(theme.grid_border, straight_rgba(0x1D_37_31_48));
        assert_eq!(theme.sector_seam, straight_rgba(0x37_65_56_6E));
        assert_eq!(theme.cursor_border, straight_rgba(0xEA_EB_E5_FF));
        assert_eq!(theme.region_border, straight_rgba(0xEA_EB_E5_FF));
        assert_eq!(theme.region_background, straight_rgba(0xFF_FF_FF_2B));

        assert_eq!(theme.panel_border, straight_rgba(0x1C_39_32_FF));
        assert_eq!(theme.selection_background, straight_rgba(0x0A_2A_22_FF));
        assert_eq!(theme.selection_border, straight_rgba(0x65_E6_BE_FF));
        assert_eq!(theme.selection_border_rest, straight_rgba(0x52_C3_A3_FF));
        assert_eq!(theme.widget_inactive_border, straight_rgba(0x1C_39_32_FF));

        assert_eq!(theme.text, straight_rgba(0xEA_EB_E5_FF));
        assert_eq!(theme.text_active, straight_rgba(0x65_E6_BE_FF));
        assert_eq!(theme.text_muted, straight_rgba(0xE9_EB_E4_99));
        assert_eq!(theme.input_background, straight_rgba(0x00_00_00_FF));
        assert_eq!(theme.link, straight_rgba(0x5A_AA_FF_FF));
        assert_eq!(theme.code_background, straight_rgba(0x40_40_40_FF));
        assert_eq!(theme.input_cursor, straight_rgba(0xC0_DE_FF_FF));
        assert_eq!(theme.error, straight_rgba(0xCC_79_A7_FF));
        assert_eq!(theme.warning, straight_rgba(0xCC_79_A7_FF));

        assert_eq!(theme.cursor_background, None);
        assert_eq!(theme.region_cursor_background, None);

        assert_eq!(theme.grid_border_width.points(), 0.5);
        assert_eq!(theme.sector_seam_width.points(), 0.75);
        assert_eq!(theme.cell_selection_border_width.points(), 0.5);
        assert_eq!(theme.cursor_border_width.points(), 1.0);
        assert_eq!(theme.region_border_width.points(), 1.0);
        assert_eq!(theme.diagnostic_border_width.points(), 0.5);
        assert_eq!(theme.output_portal_border_width.points(), 0.5);
        assert_eq!(theme.input_portal_border_width.points(), 0.5);
        assert_eq!(theme.locked_border_width.points(), 0.5);
        assert_eq!(theme.panel_border_width.points(), 1.0);
        assert_eq!(theme.selection_border_width.points(), 1.0);
        assert_eq!(theme.widget_border_width.points(), 1.0);
        assert_eq!(theme.widget_inactive_border_width.points(), 0.0);
        assert_eq!(theme.input_cursor_width.points(), 2.0);
    }

    ///
    /// `colour`'s OKLCh hue angle in degrees, by Björn Ottosson's published
    /// OKLab matrices (2020) over `egui`'s own sRGB linearization. Only the
    /// hue test below needs it: nothing shipped reasons in OKLCh.
    ///
    fn oklch_hue(colour: Color32) -> f32 {
        let linear = |value: u8| egui::ecolor::linear_f32_from_gamma_u8(value);
        let (red, green, blue) = (linear(colour.r()), linear(colour.g()), linear(colour.b()));
        let long = (0.412_221_46 * red + 0.536_332_55 * green + 0.051_445_99 * blue).cbrt();
        let medium = (0.211_903_5 * red + 0.680_699_5 * green + 0.107_396_96 * blue).cbrt();
        let short = (0.088_302_46 * red + 0.281_718_85 * green + 0.629_978_7 * blue).cbrt();
        let a = 1.977_998_5 * long - 2.428_592_2 * medium + 0.450_593_7 * short;
        let b = 0.025_904_037 * long + 0.782_771_77 * medium - 0.808_675_77 * short;
        b.atan2(a).to_degrees().rem_euclid(360.0)
    }

    ///
    /// Checked rather than recorded: every Source glyph hue Orcvs Light draws is
    /// Okabe–Ito's, in the same role, with the OKLCh hue angle held to
    /// within 0.7° — rounding into 8-bit sRGB — so only lightness moved.
    /// `console/src/theme.md`'s "What moved, and what held" table states
    /// each angle.
    ///
    #[test]
    fn orcvs_light_keeps_every_okabe_ito_glyph_hue() {
        let (dark, light) = (okabe_ito(), orcvs_light());
        for (role, from, to) in [
            ("Number", dark.source_number, light.source_number),
            ("Note", dark.source_note, light.source_note),
            ("Function", dark.source_function, light.source_function),
            ("Bang", dark.source_bang, light.source_bang),
            (
                "Diagnostic",
                dark.diagnostic_foreground,
                light.diagnostic_foreground,
            ),
            (
                "Output Portal",
                dark.output_portal_foreground,
                light.output_portal_foreground,
            ),
        ] {
            let (from_hue, to_hue) = (oklch_hue(from), oklch_hue(to));
            let drift = (from_hue - to_hue)
                .abs()
                .min(360.0 - (from_hue - to_hue).abs());
            assert!(
                drift <= 0.7,
                "{role}: Okabe–Ito {from_hue:.2}°, Orcvs Light {to_hue:.2}°, {drift:.2}° apart"
            );
        }
    }

    ///
    /// Input Portal is the one glyph role Okabe–Ito assigns no hue: both
    /// built-ins draw it in the same OKLCh hue, a violet at 300°, within the
    /// same 0.7° rounding, and differ in lightness alone.
    ///
    #[test]
    fn both_built_ins_draw_input_portal_in_one_hue() {
        let (dark, light) = (okabe_ito(), orcvs_light());
        for hue in [
            oklch_hue(dark.input_portal_foreground),
            oklch_hue(light.input_portal_foreground),
        ] {
            assert!((hue - 300.0).abs() <= 0.7, "Input Portal at {hue:.2}°");
        }
    }

    ///
    /// The light counterpart of the test above: `console/src/theme.md`'s
    /// Orcvs Light table, restated as `Theme` field assertions. Every one of
    /// `schema.md`'s named properties appears here at an exact value —
    /// including the ones Orcvs Light shares with Okabe–Ito, which are
    /// recorded explicitly rather than left to a toolkit default, and
    /// including the border-width keys.
    ///
    #[test]
    fn orcvs_light_defines_every_key_at_the_recorded_values() {
        let theme = orcvs_light();

        assert_eq!(theme.identity.as_str(), "orcvs-light");
        assert_eq!(theme.name, "Orcvs Light");
        assert_eq!(theme.appearance, Appearance::Light);

        assert_eq!(theme.window_background, straight_rgba(0xEF_F4_F2_FF));
        assert_eq!(theme.panel_background, straight_rgba(0xEF_F4_F2_FF));
        assert_eq!(theme.grid_background, straight_rgba(0xFA_FC_FB_FF));
        // Shared with Okabe–Ito, written out rather than inherited.
        assert_eq!(theme.cell_background, straight_rgba(0x00_00_00_00));

        assert_eq!(theme.source_ordinary, straight_rgba(0x30_3F_3B_FF));
        assert_eq!(theme.source_comment, straight_rgba(0x4E_5A_56_FF));
        assert_eq!(theme.source_number, straight_rgba(0x00_6D_9B_FF));
        assert_eq!(theme.source_note, straight_rgba(0x70_69_00_FF));
        assert_eq!(theme.source_function, straight_rgba(0x00_75_55_FF));
        assert_eq!(theme.source_bang, straight_rgba(0x90_42_6F_FF));
        assert_eq!(
            theme.source_ordinary_background,
            straight_rgba(0x00_00_00_00)
        );
        assert_eq!(
            theme.source_comment_background,
            straight_rgba(0x00_00_00_00)
        );
        assert_eq!(theme.source_number_background, straight_rgba(0x00_6D_9B_1A));
        assert_eq!(theme.source_note_background, straight_rgba(0x70_69_00_1A));
        assert_eq!(
            theme.source_function_background,
            straight_rgba(0x00_75_55_1A)
        );
        assert_eq!(theme.source_bang_background, straight_rgba(0x00_00_00_00));

        assert_eq!(theme.diagnostic_foreground, straight_rgba(0x65_28_00_FF));
        assert_eq!(theme.diagnostic_background, straight_rgba(0x00_00_00_00));
        assert_eq!(theme.diagnostic_border, straight_rgba(0x00_00_00_00));
        assert_eq!(theme.output_portal_foreground, straight_rgba(0x6F_4A_00_FF));
        assert_eq!(theme.output_portal_background, straight_rgba(0x6F_4A_00_1A));
        assert_eq!(theme.output_portal_border, straight_rgba(0x00_00_00_00));
        assert_eq!(theme.input_portal_foreground, straight_rgba(0x64_07_AF_FF));
        assert_eq!(theme.input_portal_background, straight_rgba(0x64_07_AF_1A));
        assert_eq!(theme.input_portal_border, straight_rgba(0x00_00_00_00));
        assert_eq!(theme.locked_foreground, straight_rgba(0x00_00_00_00));
        assert_eq!(theme.locked_background, straight_rgba(0xFA_FC_FB_CC));
        assert_eq!(theme.locked_border, straight_rgba(0x30_3F_3B_80));

        assert_eq!(theme.grid_border, straight_rgba(0x34_5B_50_40));
        assert_eq!(theme.sector_seam, straight_rgba(0x26_68_54_70));
        assert_eq!(theme.cursor_border, straight_rgba(0x30_3F_3B_FF));
        assert_eq!(theme.region_border, straight_rgba(0x30_3F_3B_FF));
        assert_eq!(theme.region_background, straight_rgba(0x30_3F_3B_2B));

        assert_eq!(theme.panel_border, straight_rgba(0xC0_CE_C9_FF));
        assert_eq!(theme.selection_background, straight_rgba(0xCC_EB_E2_FF));
        assert_eq!(theme.selection_border, straight_rgba(0x07_62_47_FF));
        assert_eq!(theme.selection_border_rest, straight_rgba(0x18_7E_60_FF));
        assert_eq!(theme.widget_inactive_border, straight_rgba(0xC0_CE_C9_FF));

        assert_eq!(theme.text, straight_rgba(0x30_3F_3B_FF));
        assert_eq!(theme.text_active, straight_rgba(0x07_62_47_FF));
        assert_eq!(theme.text_muted, straight_rgba(0x30_3F_3B_BF));
        assert_eq!(theme.input_background, straight_rgba(0xFA_FC_FB_FF));
        assert_eq!(theme.link, straight_rgba(0x0B_62_B8_FF));
        assert_eq!(theme.code_background, straight_rgba(0xE6_E6_E6_FF));
        assert_eq!(theme.input_cursor, straight_rgba(0x00_53_7D_FF));
        assert_eq!(theme.error, straight_rgba(0x90_42_6F_FF));
        assert_eq!(theme.warning, straight_rgba(0x90_42_6F_FF));

        // Shared with Okabe–Ito: both optional Cursor fills are cleared.
        assert_eq!(theme.cursor_background, None);
        assert_eq!(theme.region_cursor_background, None);

        // Every width is shared with Okabe–Ito and recorded anyway, at the
        // same display-point defaults.
        assert_eq!(theme.grid_border_width.points(), 0.5);
        assert_eq!(theme.sector_seam_width.points(), 0.75);
        assert_eq!(theme.cell_selection_border_width.points(), 0.5);
        assert_eq!(theme.cursor_border_width.points(), 1.0);
        assert_eq!(theme.region_border_width.points(), 1.0);
        assert_eq!(theme.diagnostic_border_width.points(), 0.5);
        assert_eq!(theme.output_portal_border_width.points(), 0.5);
        assert_eq!(theme.input_portal_border_width.points(), 0.5);
        assert_eq!(theme.locked_border_width.points(), 0.5);
        assert_eq!(theme.panel_border_width.points(), 1.0);
        assert_eq!(theme.selection_border_width.points(), 1.0);
        assert_eq!(theme.widget_border_width.points(), 1.0);
        assert_eq!(theme.widget_inactive_border_width.points(), 0.0);
        assert_eq!(theme.input_cursor_width.points(), 2.0);
    }

    ///
    /// The two built-ins are genuinely different Themes, not one palette
    /// under two identities: they disagree on the page, the Source ground
    /// and every glyph colour a near-white ground forced them to disagree
    /// on, and they declare opposite appearances.
    ///
    #[test]
    fn the_two_built_ins_are_distinct_themes_of_opposite_appearance() {
        let dark = okabe_ito();
        let light = orcvs_light();

        assert_ne!(dark.identity, light.identity);
        assert_ne!(dark.appearance, light.appearance);
        for (name, dark_value, light_value) in [
            (
                "window.background",
                dark.window_background,
                light.window_background,
            ),
            (
                "grid.background",
                dark.grid_background,
                light.grid_background,
            ),
            (
                "source.ordinary",
                dark.source_ordinary,
                light.source_ordinary,
            ),
            ("source.comment", dark.source_comment, light.source_comment),
            ("source.number", dark.source_number, light.source_number),
            ("source.note", dark.source_note, light.source_note),
            (
                "source.function",
                dark.source_function,
                light.source_function,
            ),
            ("source.bang", dark.source_bang, light.source_bang),
            ("text", dark.text, light.text),
            (
                "selection.background",
                dark.selection_background,
                light.selection_background,
            ),
        ] {
            assert_ne!(
                dark_value, light_value,
                "{name} must differ between the built-ins"
            );
        }
    }

    #[test]
    fn grid_width_accepts_zero_fractional_and_the_inclusive_upper_bound() {
        for points in [0.0, 0.001, 0.5, 0.999, 1.0] {
            assert_eq!(
                GridWidth::from_points(points).map(GridWidth::points),
                Some(points)
            );
        }
    }

    #[test]
    fn chrome_width_accepts_zero_fractional_and_the_inclusive_upper_bound() {
        for points in [0.0, 0.5, 1.0, 1.999, 2.0] {
            assert_eq!(
                ChromeWidth::from_points(points).map(ChromeWidth::points),
                Some(points)
            );
        }
    }

    #[test]
    fn grid_width_rejects_out_of_range_and_nonfinite_values() {
        for points in [
            -0.01,
            1.000001,
            2.0,
            f32::NAN,
            f32::INFINITY,
            f32::NEG_INFINITY,
        ] {
            assert!(
                GridWidth::from_points(points).is_none(),
                "{points} should have been rejected"
            );
        }
    }

    #[test]
    fn chrome_width_rejects_out_of_range_and_nonfinite_values() {
        for points in [
            -0.01,
            2.000001,
            3.0,
            f32::NAN,
            f32::INFINITY,
            f32::NEG_INFINITY,
        ] {
            assert!(
                ChromeWidth::from_points(points).is_none(),
                "{points} should have been rejected"
            );
        }
    }

    ///
    /// The Cursor inside a spanning Region is filled with
    /// `region.cursor.background` whenever it holds a colour, transparent
    /// included, and with `cursor.background` only where the Region's fill is
    /// cleared. Both cleared leave the Cell unfilled.
    ///
    #[test]
    fn the_region_cursor_fill_falls_back_to_the_cursor_fill_only_when_cleared() {
        let cursor = Color32::from_rgb(1, 2, 3);
        let region_cursor = Color32::from_rgb(4, 5, 6);
        for (cursor_background, region_cursor_background, expected) in [
            (Some(cursor), Some(region_cursor), Some(region_cursor)),
            (
                Some(cursor),
                Some(Color32::TRANSPARENT),
                Some(Color32::TRANSPARENT),
            ),
            (Some(cursor), None, Some(cursor)),
            (None, Some(region_cursor), Some(region_cursor)),
            (None, None, None),
        ] {
            let theme = super::Theme {
                cursor_background,
                region_cursor_background,
                ..okabe_ito()
            };
            assert_eq!(
                theme.region_cursor_fill(),
                expected,
                "cursor.background {cursor_background:?}, \
                 region.cursor.background {region_cursor_background:?}"
            );
        }
    }
}
