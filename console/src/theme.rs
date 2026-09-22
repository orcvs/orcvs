//!
//! The resolved Orcvs Theme: named colour, width and optional-fill
//! properties the whole console paints from, and the pure inheritance
//! resolver that builds a custom Theme's resolved values from a built-in and
//! a sparse document of overrides.
//!
//! ADR 0053 decides the model; `.scratch/theming/schema.md` is the
//! implementation contract this module follows — the property catalogue,
//! the dark built-in's exact values, and the composition/inheritance rules.
//! This module owns the resolved [`Theme`], the built-in Okabe–Ito
//! definition, the unresolved `ThemeDocument`, and the pure `resolve`
//! function. It has no file I/O and no YAML parser: `.scratch/theming/
//! issues/07` owns loading native/web documents into a `ThemeDocument` and
//! calling `resolve`.
//! Source painting, settings and persistence are unchanged by this slice;
//! `.scratch/theming/issues/06`'s later slices consume this module.
//!

use egui::Color32;

// === Appearance ===

///
/// A Theme's declared light/dark appearance. Only a Theme of matching
/// appearance can be a custom Theme's parent (ADR 0053: "Custom appearance
/// follows the parent").
///
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Appearance {
    Dark,
    /// No light built-in exists yet (`.scratch/theming/issues/04`), so
    /// nothing shipped constructs this variant outside a document's own
    /// declared `appearance` once `07`'s parser exists.
    #[cfg_attr(
        not(test),
        expect(dead_code, reason = "no light built-in exists yet: theming/04")
    )]
    Light,
}

// === Bounded widths ===

///
/// Why a width property failed validation, independently of which key was
/// being resolved — [`resolve`] attaches the offending property's name and
/// bound when it turns this into a [`ThemeError`].
///
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum WidthError {
    NonFinite,
    OutOfRange,
}

///
/// A Grid, Cell or Sector Seam stroke width in display points: 0 to 1
/// inclusive (`schema.md`'s width catalogue). Zero hides the stroke; the
/// value is fixed in display points and never scales with Grid zoom.
///
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct GridWidth(f32);

impl GridWidth {
    pub(crate) const MAX_POINTS: f32 = 1.0;

    pub(crate) fn from_points(points: f32) -> Result<Self, WidthError> {
        if !points.is_finite() {
            return Err(WidthError::NonFinite);
        }
        if !(0.0..=Self::MAX_POINTS).contains(&points) {
            return Err(WidthError::OutOfRange);
        }
        Ok(Self(points))
    }

    /// Consumed by `console.rs`'s Grid line/Sector Seam strokes and
    /// `cursor_effects.rs`'s Cursor/Region frame width
    /// (`.scratch/theming/issues/06` slice C's fixed display-point strokes).
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

    pub(crate) fn from_points(points: f32) -> Result<Self, WidthError> {
        if !points.is_finite() {
            return Err(WidthError::NonFinite);
        }
        if !(0.0..=Self::MAX_POINTS).contains(&points) {
            return Err(WidthError::OutOfRange);
        }
        Ok(Self(points))
    }

    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "consumed by theming/06 slice C's fixed stroke widths"
        )
    )]
    pub(crate) fn points(self) -> f32 {
        self.0
    }
}

// === Optional Cursor fills ===

///
/// An explicit value for one of the two optional Cursor fills
/// (`cursor.background`, `region.cursor.background`). A document that omits
/// the property entirely inherits the parent's resolved value instead of
/// holding one of these two states — see [`ThemeDocument::cursor_background`].
///
#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "consumed by theming/07's parser, which builds a ThemeDocument from a raw \
                   document; this slice's Theme is always a built-in, never a resolved document"
    )
)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum OptionalFill {
    /// The YAML string `"none"`: clears the optional fill and enables its
    /// existing fallback (the ordinary Cursor fill in a Region, or no fill
    /// at all for the ordinary Cursor).
    None,
    /// An explicit colour, including a fully transparent one — distinct
    /// from `None` above, since a supplied transparent colour is still a
    /// supplied value and does not trigger the fallback.
    Color(Color32),
}

// === Property keys ===

///
/// Every named colour property `schema.md`'s catalogue defines, less the
/// two optional Cursor fills, which [`OptionalFill`] carries instead of a
/// plain `Color32`. Case- and dot-sensitive schema spellings are not
/// reproduced here: `.scratch/theming/issues/07`'s parser is what maps raw
/// document text to these variants.
///
#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "consumed by Theme::color, itself unconsumed until theming/07"
    )
)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ColorKey {
    WindowBackground,
    PanelBackground,
    GridBackground,
    CellBackground,
    SourceOrdinary,
    SourceComment,
    SourceNumber,
    SourceNote,
    SourceFunction,
    SourceBang,
    SourceSequence,
    SourceOrdinaryBackground,
    SourceCommentBackground,
    SourceNumberBackground,
    SourceNoteBackground,
    SourceFunctionBackground,
    SourceBangBackground,
    SourceAtomBackground,
    SourceSequenceBackground,
    DiagnosticForeground,
    DiagnosticBackground,
    DiagnosticBorder,
    OutputPortalForeground,
    OutputPortalBackground,
    OutputPortalBorder,
    GridBorder,
    SectorSeam,
    CursorBorder,
    RegionBorder,
    CursorArea,
    RegionBackground,
    PanelBorder,
    SelectionBackground,
    SelectionBorder,
    SelectionBorderRest,
    WidgetInactiveBorder,
    Text,
    TextActive,
    TextMuted,
    InputBackground,
    Link,
    CodeBackground,
    InputCursor,
    Error,
    Warning,
}

///
/// The Grid/Cell/Sector Seam width properties, bounded 0 to 1 point by
/// [`GridWidth`].
///
#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "consumed by theming/06 slice C and theming/07's parser"
    )
)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum GridWidthKey {
    GridBorder,
    SectorSeam,
    CellSelectionBorder,
    CursorBorder,
    RegionBorder,
    DiagnosticBorder,
    OutputPortalBorder,
}

#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "consumed by theming/06 slice C and theming/07's parser"
    )
)]
impl GridWidthKey {
    pub(crate) fn name(self) -> &'static str {
        match self {
            Self::GridBorder => "grid.border.width",
            Self::SectorSeam => "sector.seam.width",
            Self::CellSelectionBorder => "cell.selection.border.width",
            Self::CursorBorder => "cursor.border.width",
            Self::RegionBorder => "region.border.width",
            Self::DiagnosticBorder => "diagnostic.border.width",
            Self::OutputPortalBorder => "output_portal.border.width",
        }
    }
}

///
/// The chrome width properties, bounded 0 to 2 points by [`ChromeWidth`].
///
#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "consumed by theming/06 slice C and theming/07's parser"
    )
)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ChromeWidthKey {
    PanelBorder,
    SelectionBorder,
    WidgetBorder,
    WidgetInactiveBorder,
    InputCursor,
}

#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "consumed by theming/06 slice C and theming/07's parser"
    )
)]
impl ChromeWidthKey {
    pub(crate) fn name(self) -> &'static str {
        match self {
            Self::PanelBorder => "panel.border.width",
            Self::SelectionBorder => "selection.border.width",
            Self::WidgetBorder => "widget.border.width",
            Self::WidgetInactiveBorder => "widget.inactive.border.width",
            Self::InputCursor => "input.cursor.width",
        }
    }
}

// === Resolved Theme ===

///
/// A fully resolved Theme: every named property `schema.md` catalogues,
/// with no unset value. Built-ins are constructed with every field spelled
/// out explicitly (see [`okabe_ito`]), so the compiler — not a runtime
/// check — is what proves a built-in defines every key: omit a field from
/// the struct literal and the crate fails to build. A custom Theme's
/// resolved value is a built-in's, copied and selectively replaced by the
/// pure `resolve` function.
///
/// Fields are `pub(crate)` rather than encapsulated behind accessors: unlike
/// the console's former `SourcePaintSettings` and
/// [`crate::cursor_effects::CursorEffectSettings`], a resolved `Theme` is
/// immutable once built — nothing edits one in place or persists it
/// field-by-field — so there is no invariant an accessor needs to protect.
/// `Theme::color`, `Theme::grid_width` and `Theme::chrome_width` exist
/// alongside the fields for callers that index by key.
///
#[derive(Clone, Debug, PartialEq)]
pub struct Theme {
    /// The identity a Theme is selected by: a built-in's reserved name, or
    /// a custom Theme's filename stem, assigned by
    /// `.scratch/theming/issues/07`'s loader outside this module.
    pub(crate) identity: String,
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
    pub(crate) source_sequence: Color32,
    pub(crate) source_ordinary_background: Color32,
    pub(crate) source_comment_background: Color32,
    pub(crate) source_number_background: Color32,
    pub(crate) source_note_background: Color32,
    pub(crate) source_function_background: Color32,
    pub(crate) source_bang_background: Color32,
    pub(crate) source_atom_background: Color32,
    pub(crate) source_sequence_background: Color32,

    pub(crate) diagnostic_foreground: Color32,
    pub(crate) diagnostic_background: Color32,
    pub(crate) diagnostic_border: Color32,
    pub(crate) output_portal_foreground: Color32,
    pub(crate) output_portal_background: Color32,
    pub(crate) output_portal_border: Color32,

    pub(crate) grid_border: Color32,
    pub(crate) sector_seam: Color32,
    pub(crate) cursor_border: Color32,
    pub(crate) region_border: Color32,
    pub(crate) cursor_area: Color32,
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
    /// explicit colour, including transparent. See [`OptionalFill`].
    pub(crate) cursor_background: Option<Color32>,
    pub(crate) region_cursor_background: Option<Color32>,

    pub(crate) grid_border_width: GridWidth,
    pub(crate) sector_seam_width: GridWidth,
    pub(crate) cell_selection_border_width: GridWidth,
    pub(crate) cursor_border_width: GridWidth,
    pub(crate) region_border_width: GridWidth,
    pub(crate) diagnostic_border_width: GridWidth,
    pub(crate) output_portal_border_width: GridWidth,

    pub(crate) panel_border_width: ChromeWidth,
    pub(crate) selection_border_width: ChromeWidth,
    pub(crate) widget_border_width: ChromeWidth,
    pub(crate) widget_inactive_border_width: ChromeWidth,
    pub(crate) input_cursor_width: ChromeWidth,
}

#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "these by-key accessors are for theming/07's parser and future by-key callers; \
                   slice B's painting reads Theme's fields directly, never through a key"
    )
)]
impl Theme {
    ///
    /// Reads a named colour property by key. Exhaustive over [`ColorKey`],
    /// so a catalogue addition that forgets a field here fails to compile
    /// rather than silently resolving to a stale value.
    ///
    pub(crate) fn color(&self, key: ColorKey) -> Color32 {
        *self.color_ref(key)
    }

    fn color_ref(&self, key: ColorKey) -> &Color32 {
        match key {
            ColorKey::WindowBackground => &self.window_background,
            ColorKey::PanelBackground => &self.panel_background,
            ColorKey::GridBackground => &self.grid_background,
            ColorKey::CellBackground => &self.cell_background,
            ColorKey::SourceOrdinary => &self.source_ordinary,
            ColorKey::SourceComment => &self.source_comment,
            ColorKey::SourceNumber => &self.source_number,
            ColorKey::SourceNote => &self.source_note,
            ColorKey::SourceFunction => &self.source_function,
            ColorKey::SourceBang => &self.source_bang,
            ColorKey::SourceSequence => &self.source_sequence,
            ColorKey::SourceOrdinaryBackground => &self.source_ordinary_background,
            ColorKey::SourceCommentBackground => &self.source_comment_background,
            ColorKey::SourceNumberBackground => &self.source_number_background,
            ColorKey::SourceNoteBackground => &self.source_note_background,
            ColorKey::SourceFunctionBackground => &self.source_function_background,
            ColorKey::SourceBangBackground => &self.source_bang_background,
            ColorKey::SourceAtomBackground => &self.source_atom_background,
            ColorKey::SourceSequenceBackground => &self.source_sequence_background,
            ColorKey::DiagnosticForeground => &self.diagnostic_foreground,
            ColorKey::DiagnosticBackground => &self.diagnostic_background,
            ColorKey::DiagnosticBorder => &self.diagnostic_border,
            ColorKey::OutputPortalForeground => &self.output_portal_foreground,
            ColorKey::OutputPortalBackground => &self.output_portal_background,
            ColorKey::OutputPortalBorder => &self.output_portal_border,
            ColorKey::GridBorder => &self.grid_border,
            ColorKey::SectorSeam => &self.sector_seam,
            ColorKey::CursorBorder => &self.cursor_border,
            ColorKey::RegionBorder => &self.region_border,
            ColorKey::CursorArea => &self.cursor_area,
            ColorKey::RegionBackground => &self.region_background,
            ColorKey::PanelBorder => &self.panel_border,
            ColorKey::SelectionBackground => &self.selection_background,
            ColorKey::SelectionBorder => &self.selection_border,
            ColorKey::SelectionBorderRest => &self.selection_border_rest,
            ColorKey::WidgetInactiveBorder => &self.widget_inactive_border,
            ColorKey::Text => &self.text,
            ColorKey::TextActive => &self.text_active,
            ColorKey::TextMuted => &self.text_muted,
            ColorKey::InputBackground => &self.input_background,
            ColorKey::Link => &self.link,
            ColorKey::CodeBackground => &self.code_background,
            ColorKey::InputCursor => &self.input_cursor,
            ColorKey::Error => &self.error,
            ColorKey::Warning => &self.warning,
        }
    }

    fn color_mut(&mut self, key: ColorKey) -> &mut Color32 {
        match key {
            ColorKey::WindowBackground => &mut self.window_background,
            ColorKey::PanelBackground => &mut self.panel_background,
            ColorKey::GridBackground => &mut self.grid_background,
            ColorKey::CellBackground => &mut self.cell_background,
            ColorKey::SourceOrdinary => &mut self.source_ordinary,
            ColorKey::SourceComment => &mut self.source_comment,
            ColorKey::SourceNumber => &mut self.source_number,
            ColorKey::SourceNote => &mut self.source_note,
            ColorKey::SourceFunction => &mut self.source_function,
            ColorKey::SourceBang => &mut self.source_bang,
            ColorKey::SourceSequence => &mut self.source_sequence,
            ColorKey::SourceOrdinaryBackground => &mut self.source_ordinary_background,
            ColorKey::SourceCommentBackground => &mut self.source_comment_background,
            ColorKey::SourceNumberBackground => &mut self.source_number_background,
            ColorKey::SourceNoteBackground => &mut self.source_note_background,
            ColorKey::SourceFunctionBackground => &mut self.source_function_background,
            ColorKey::SourceBangBackground => &mut self.source_bang_background,
            ColorKey::SourceAtomBackground => &mut self.source_atom_background,
            ColorKey::SourceSequenceBackground => &mut self.source_sequence_background,
            ColorKey::DiagnosticForeground => &mut self.diagnostic_foreground,
            ColorKey::DiagnosticBackground => &mut self.diagnostic_background,
            ColorKey::DiagnosticBorder => &mut self.diagnostic_border,
            ColorKey::OutputPortalForeground => &mut self.output_portal_foreground,
            ColorKey::OutputPortalBackground => &mut self.output_portal_background,
            ColorKey::OutputPortalBorder => &mut self.output_portal_border,
            ColorKey::GridBorder => &mut self.grid_border,
            ColorKey::SectorSeam => &mut self.sector_seam,
            ColorKey::CursorBorder => &mut self.cursor_border,
            ColorKey::RegionBorder => &mut self.region_border,
            ColorKey::CursorArea => &mut self.cursor_area,
            ColorKey::RegionBackground => &mut self.region_background,
            ColorKey::PanelBorder => &mut self.panel_border,
            ColorKey::SelectionBackground => &mut self.selection_background,
            ColorKey::SelectionBorder => &mut self.selection_border,
            ColorKey::SelectionBorderRest => &mut self.selection_border_rest,
            ColorKey::WidgetInactiveBorder => &mut self.widget_inactive_border,
            ColorKey::Text => &mut self.text,
            ColorKey::TextActive => &mut self.text_active,
            ColorKey::TextMuted => &mut self.text_muted,
            ColorKey::InputBackground => &mut self.input_background,
            ColorKey::Link => &mut self.link,
            ColorKey::CodeBackground => &mut self.code_background,
            ColorKey::InputCursor => &mut self.input_cursor,
            ColorKey::Error => &mut self.error,
            ColorKey::Warning => &mut self.warning,
        }
    }

    pub(crate) fn grid_width(&self, key: GridWidthKey) -> GridWidth {
        match key {
            GridWidthKey::GridBorder => self.grid_border_width,
            GridWidthKey::SectorSeam => self.sector_seam_width,
            GridWidthKey::CellSelectionBorder => self.cell_selection_border_width,
            GridWidthKey::CursorBorder => self.cursor_border_width,
            GridWidthKey::RegionBorder => self.region_border_width,
            GridWidthKey::DiagnosticBorder => self.diagnostic_border_width,
            GridWidthKey::OutputPortalBorder => self.output_portal_border_width,
        }
    }

    fn set_grid_width(&mut self, key: GridWidthKey, value: GridWidth) {
        match key {
            GridWidthKey::GridBorder => self.grid_border_width = value,
            GridWidthKey::SectorSeam => self.sector_seam_width = value,
            GridWidthKey::CellSelectionBorder => self.cell_selection_border_width = value,
            GridWidthKey::CursorBorder => self.cursor_border_width = value,
            GridWidthKey::RegionBorder => self.region_border_width = value,
            GridWidthKey::DiagnosticBorder => self.diagnostic_border_width = value,
            GridWidthKey::OutputPortalBorder => self.output_portal_border_width = value,
        }
    }

    pub(crate) fn chrome_width(&self, key: ChromeWidthKey) -> ChromeWidth {
        match key {
            ChromeWidthKey::PanelBorder => self.panel_border_width,
            ChromeWidthKey::SelectionBorder => self.selection_border_width,
            ChromeWidthKey::WidgetBorder => self.widget_border_width,
            ChromeWidthKey::WidgetInactiveBorder => self.widget_inactive_border_width,
            ChromeWidthKey::InputCursor => self.input_cursor_width,
        }
    }

    fn set_chrome_width(&mut self, key: ChromeWidthKey, value: ChromeWidth) {
        match key {
            ChromeWidthKey::PanelBorder => self.panel_border_width = value,
            ChromeWidthKey::SelectionBorder => self.selection_border_width = value,
            ChromeWidthKey::WidgetBorder => self.widget_border_width = value,
            ChromeWidthKey::WidgetInactiveBorder => self.widget_inactive_border_width = value,
            ChromeWidthKey::InputCursor => self.input_cursor_width = value,
        }
    }
}

///
/// Decodes a straight (non-premultiplied) `0xRRGGBBAA` literal — the way
/// `schema.md` writes hex colours — into the [`Color32`] every resolved
/// Theme value is stored as. `Color32`'s own byte layout is premultiplied
/// alpha (`ecolor::Color32`'s documentation), so every literal below goes
/// through this straight-to-premultiplied conversion via
/// [`Color32::from_rgba_unmultiplied_const`] rather than assuming the
/// schema's straight bytes are already what `Color32` stores. This matches
/// how [`crate::style::PALETTE`] and
/// [`crate::source_paint::SourcePaintSettings`] already build their
/// `Color32` constants, so a resolved Theme value composites with painting
/// code the same way those do.
///
const fn straight_rgba(rgba: u32) -> Color32 {
    let [r, g, b, a] = rgba.to_be_bytes();
    Color32::from_rgba_unmultiplied_const(r, g, b, a)
}

///
/// The reserved identity of the Okabe–Ito built-in, and the default both the
/// dark and light Theme name settings hold until a viewer picks another —
/// `console/src/persistence.rs` restores both from this same identity.
///
pub(crate) const OKABE_ITO_IDENTITY: &str = "okabe-ito";

///
/// The Okabe–Ito built-in dark Theme: `schema.md`'s complete dark
/// definition, at the reserved identity `OKABE_ITO_IDENTITY`. Every field
/// is spelled out explicitly, so the compiler enforces "built-ins define
/// every property" rather than a runtime completeness check — remove a field
/// from this literal and the crate fails to build.
///
/// `okabe_ito_defines_every_key_at_the_schema_values` in this module's tests
/// pins every value against `schema.md`, and
/// `okabe_ito_matches_todays_style_and_palette_constants` cross-checks the
/// values that still have a counterpart in `style::PALETTE` and
/// `style.rs`'s `DEFAULT_*` constants, so this built-in reproduces today's
/// shipped appearance and not merely the document
/// `examples/okabe-ito-copy.yaml` records.
///
pub fn okabe_ito() -> Theme {
    Theme {
        identity: OKABE_ITO_IDENTITY.to_owned(),
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
        source_sequence: straight_rgba(0x00_72_B2_FF),
        source_ordinary_background: straight_rgba(0x00_00_00_00),
        source_comment_background: straight_rgba(0x00_00_00_00),
        source_number_background: straight_rgba(0x0E_1D_25_FF),
        source_note_background: straight_rgba(0x26_24_0B_FF),
        source_function_background: straight_rgba(0x00_19_12_FF),
        source_bang_background: straight_rgba(0x00_00_00_00),
        source_atom_background: straight_rgba(0x25_26_25_FF),
        source_sequence_background: straight_rgba(0x00_12_1C_FF),

        diagnostic_foreground: straight_rgba(0xD5_5E_00_FF),
        diagnostic_background: straight_rgba(0x00_00_00_00),
        diagnostic_border: straight_rgba(0x00_00_00_00),
        output_portal_foreground: straight_rgba(0xE6_9F_00_FF),
        output_portal_background: straight_rgba(0x25_19_00_FF),
        output_portal_border: straight_rgba(0x00_00_00_00),

        grid_border: straight_rgba(0x1D_37_31_48),
        sector_seam: straight_rgba(0x37_65_56_6E),
        cursor_border: straight_rgba(0xEA_EB_E5_FF),
        region_border: straight_rgba(0xEA_EB_E5_FF),
        cursor_area: straight_rgba(0x4C_BE_9C_FF),
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

        panel_border_width: ChromeWidth::from_points(1.0).expect("1.0 is within 0..=2"),
        selection_border_width: ChromeWidth::from_points(1.0).expect("1.0 is within 0..=2"),
        widget_border_width: ChromeWidth::from_points(1.0).expect("1.0 is within 0..=2"),
        widget_inactive_border_width: ChromeWidth::from_points(0.0).expect("0.0 is within 0..=2"),
        input_cursor_width: ChromeWidth::from_points(2.0).expect("2.0 is within 0..=2"),
    }
}

// === Custom document model ===

///
/// The parsed-but-unresolved form of a custom Theme document:
/// `schema.md`'s `inherits`, `name`, `appearance` and `style` fields,
/// decoded into typed properties. Holds no file path, byte source or raw
/// text — `.scratch/theming/issues/07` owns turning a YAML document into
/// this shape; tests in this module construct it directly.
///
#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "constructed by theming/07's parser; this slice never loads a file"
    )
)]
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct ThemeDocument {
    /// The built-in identity named by `inherits`.
    pub(crate) parent: String,
    /// The declared display label (`name`).
    pub(crate) name: String,
    /// `appearance`, when the document declares one explicitly. `None`
    /// resolves from the parent (ADR 0053: "Custom appearance follows the
    /// parent").
    pub(crate) appearance: Option<Appearance>,
    pub(crate) colors: Vec<(ColorKey, Color32)>,
    pub(crate) grid_widths: Vec<(GridWidthKey, f32)>,
    pub(crate) chrome_widths: Vec<(ChromeWidthKey, f32)>,
    /// `None`: `cursor.background` is omitted and inherits the parent's
    /// resolved value. `Some`: the document supplied one of
    /// [`OptionalFill`]'s two explicit states.
    pub(crate) cursor_background: Option<OptionalFill>,
    pub(crate) region_cursor_background: Option<OptionalFill>,
}

///
/// Why [`resolve`] refused a document.
///
#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "returned by resolve, itself unconsumed until theming/07"
    )
)]
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum ThemeError {
    /// `inherits` names an identity absent from the built-in set passed to
    /// [`resolve`] — including a chain through another custom Theme, which
    /// is never a member of that set. `schema.md`: "Only built-ins can be
    /// parents. Parent chains, missing parents ... are errors."
    UnknownParent { parent: String },
    /// The identity being resolved into collides with a built-in's
    /// reserved identity. ADR 0053: "Built-in Theme identities are
    /// reserved: a file using one is refused."
    ReservedIdentity { identity: String },
    /// `appearance` was declared explicitly and disagrees with the
    /// parent's.
    AppearanceMismatch {
        parent: Appearance,
        declared: Appearance,
    },
    /// A width property's value is not finite (NaN or infinite).
    NonFiniteWidth { property: &'static str, points: f32 },
    /// A width property's value is finite but outside its inclusive bound.
    WidthOutOfRange {
        property: &'static str,
        points: f32,
        max: f32,
    },
}

impl std::fmt::Display for ThemeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownParent { parent } => {
                write!(f, "unknown parent Theme identity {parent:?}")
            }
            Self::ReservedIdentity { identity } => {
                write!(f, "{identity:?} is a reserved built-in Theme identity")
            }
            Self::AppearanceMismatch { parent, declared } => write!(
                f,
                "declared appearance {declared:?} does not match parent appearance {parent:?}"
            ),
            Self::NonFiniteWidth { property, points } => {
                write!(f, "{property} is not finite: {points}")
            }
            Self::WidthOutOfRange {
                property,
                points,
                max,
            } => write!(
                f,
                "{property} is {points}, outside the inclusive range 0..={max}"
            ),
        }
    }
}

#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "consumed by resolve, itself unconsumed until theming/07"
    )
)]
fn width_error(property: &'static str, points: f32, max: f32, error: WidthError) -> ThemeError {
    match error {
        WidthError::NonFinite => ThemeError::NonFiniteWidth { property, points },
        WidthError::OutOfRange => ThemeError::WidthOutOfRange {
            property,
            points,
            max,
        },
    }
}

///
/// The pure inheritance resolver: `schema.md`'s "Source composition and
/// inheritance" step 1 (copy the parent's resolved properties, replace only
/// what the document supplies) and its optional-fill rule. Copies the
/// built-in named by `document.parent`, replaces every explicit property
/// `document` supplies, and returns the result under `identity`.
///
/// No file I/O: `built_ins` is whatever built-in set the caller already
/// holds — `.scratch/theming/issues/07` reads native/web documents and
/// calls this; tests pass `&[okabe_ito()]` directly. `identity` is supplied
/// by the caller rather than read from `document`, because a custom
/// document's identity is its filename stem (ADR 0053), which this module
/// never reads a file to learn.
///
#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "consumed by theming/07's loader, which resolves a loaded document against the \
                   built-in set; this slice's Theme is always the okabe_ito() built-in itself"
    )
)]
pub(crate) fn resolve(
    built_ins: &[Theme],
    identity: &str,
    document: &ThemeDocument,
) -> Result<Theme, ThemeError> {
    if built_ins
        .iter()
        .any(|built_in| built_in.identity == identity)
    {
        return Err(ThemeError::ReservedIdentity {
            identity: identity.to_owned(),
        });
    }

    let parent = built_ins
        .iter()
        .find(|built_in| built_in.identity == document.parent)
        .ok_or_else(|| ThemeError::UnknownParent {
            parent: document.parent.clone(),
        })?;

    if let Some(declared) = document.appearance
        && declared != parent.appearance
    {
        return Err(ThemeError::AppearanceMismatch {
            parent: parent.appearance,
            declared,
        });
    }

    let mut resolved = parent.clone();
    resolved.identity = identity.to_owned();
    resolved.name = document.name.clone();

    for &(key, colour) in &document.colors {
        *resolved.color_mut(key) = colour;
    }

    for &(key, points) in &document.grid_widths {
        let width = GridWidth::from_points(points)
            .map_err(|error| width_error(key.name(), points, GridWidth::MAX_POINTS, error))?;
        resolved.set_grid_width(key, width);
    }

    for &(key, points) in &document.chrome_widths {
        let width = ChromeWidth::from_points(points)
            .map_err(|error| width_error(key.name(), points, ChromeWidth::MAX_POINTS, error))?;
        resolved.set_chrome_width(key, width);
    }

    // Omitted (`None`): inherit — `resolved` already carries the parent's
    // value from the `clone()` above, restated here so the inheritance rule
    // is visible rather than merely relied upon.
    resolved.cursor_background = match document.cursor_background {
        None => parent.cursor_background,
        Some(OptionalFill::None) => None,
        Some(OptionalFill::Color(colour)) => Some(colour),
    };
    resolved.region_cursor_background = match document.region_cursor_background {
        None => parent.region_cursor_background,
        Some(OptionalFill::None) => None,
        Some(OptionalFill::Color(colour)) => Some(colour),
    };

    Ok(resolved)
}

#[cfg(test)]
mod tests {
    use egui::Color32;

    use super::{
        Appearance, ChromeWidth, ChromeWidthKey, ColorKey, GridWidth, GridWidthKey, OptionalFill,
        ThemeDocument, ThemeError, okabe_ito, resolve, straight_rgba,
    };
    use crate::style::{DEFAULT_BANG, DEFAULT_ORDINARY, DEFAULT_SOURCE_BACKGROUND, PALETTE};

    fn child(parent: &str) -> ThemeDocument {
        ThemeDocument {
            parent: parent.to_owned(),
            name: "Custom".to_owned(),
            ..ThemeDocument::default()
        }
    }

    ///
    /// Restates `schema.md`'s dark property table as `Theme` field
    /// assertions — the same pinning-by-restatement `style.rs`'s
    /// `palette_tokens_match_the_decided_record` already uses for the fixed
    /// chrome palette. This test is intentionally close to `okabe_ito`'s own
    /// body: a later edit to either one without the other is exactly what
    /// it is meant to catch.
    ///
    #[test]
    fn okabe_ito_defines_every_key_at_the_schema_values() {
        let theme = okabe_ito();

        assert_eq!(theme.identity, "okabe-ito");
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
        assert_eq!(theme.source_sequence, straight_rgba(0x00_72_B2_FF));
        assert_eq!(
            theme.source_ordinary_background,
            straight_rgba(0x00_00_00_00)
        );
        assert_eq!(
            theme.source_comment_background,
            straight_rgba(0x00_00_00_00)
        );
        assert_eq!(theme.source_number_background, straight_rgba(0x0E_1D_25_FF));
        assert_eq!(theme.source_note_background, straight_rgba(0x26_24_0B_FF));
        assert_eq!(
            theme.source_function_background,
            straight_rgba(0x00_19_12_FF)
        );
        assert_eq!(theme.source_bang_background, straight_rgba(0x00_00_00_00));
        assert_eq!(theme.source_atom_background, straight_rgba(0x25_26_25_FF));
        assert_eq!(
            theme.source_sequence_background,
            straight_rgba(0x00_12_1C_FF)
        );

        assert_eq!(theme.diagnostic_foreground, straight_rgba(0xD5_5E_00_FF));
        assert_eq!(theme.diagnostic_background, straight_rgba(0x00_00_00_00));
        assert_eq!(theme.diagnostic_border, straight_rgba(0x00_00_00_00));
        assert_eq!(theme.output_portal_foreground, straight_rgba(0xE6_9F_00_FF));
        assert_eq!(theme.output_portal_background, straight_rgba(0x25_19_00_FF));
        assert_eq!(theme.output_portal_border, straight_rgba(0x00_00_00_00));

        assert_eq!(theme.grid_border, straight_rgba(0x1D_37_31_48));
        assert_eq!(theme.sector_seam, straight_rgba(0x37_65_56_6E));
        assert_eq!(theme.cursor_border, straight_rgba(0xEA_EB_E5_FF));
        assert_eq!(theme.region_border, straight_rgba(0xEA_EB_E5_FF));
        assert_eq!(theme.cursor_area, straight_rgba(0x4C_BE_9C_FF));
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
        assert_eq!(theme.panel_border_width.points(), 1.0);
        assert_eq!(theme.selection_border_width.points(), 1.0);
        assert_eq!(theme.widget_border_width.points(), 1.0);
        assert_eq!(theme.widget_inactive_border_width.points(), 0.0);
        assert_eq!(theme.input_cursor_width.points(), 2.0);
    }

    ///
    /// Cross-checks the built-in against the values it must preserve:
    /// `style::PALETTE`'s fixed chrome/grid constants and the three
    /// `style::DEFAULT_*` constants `style()`'s chrome baseline still opens
    /// with (`.scratch/theming/issues/03` derives that baseline from the
    /// resolved Theme instead; until then the two are independent and this
    /// is what keeps them agreeing). `SourcePaintSettings` and
    /// `CursorEffectSettings`' colours are gone from this slice — every
    /// value that once lived there is now pinned directly against
    /// `schema.md` by `okabe_ito_defines_every_key_at_the_schema_values`
    /// above, so this test's job narrows to the values Slice B did not
    /// replace: chrome/grid geometry colours and the `text.muted`/
    /// `panel.border` derivations that still come from egui/`PALETTE`
    /// computations, not from a hand-copied literal.
    ///
    #[test]
    fn okabe_ito_matches_todays_style_and_palette_constants() {
        let theme = okabe_ito();

        assert_eq!(theme.window_background, PALETTE.page);
        assert_eq!(theme.panel_background, PALETTE.page);
        assert_eq!(theme.grid_background, DEFAULT_SOURCE_BACKGROUND);
        assert_eq!(theme.source_ordinary, DEFAULT_ORDINARY);
        assert_eq!(theme.source_bang, DEFAULT_BANG);

        assert_eq!(theme.grid_border, PALETTE.grid_line);
        assert_eq!(theme.sector_seam, PALETTE.sector_line);

        // `style.rs::style()`'s chrome stroke: `PALETTE.grid_line.to_opaque()`,
        // the actual gamma-correct un-premultiply — not a straight-hex
        // approximation of `grid_line`'s own hue.
        assert_eq!(theme.panel_border, PALETTE.grid_line.to_opaque());
        assert_eq!(theme.widget_inactive_border, PALETTE.grid_line.to_opaque());

        assert_eq!(theme.selection_background, PALETTE.selection_fill);
        assert_eq!(theme.selection_border, PALETTE.selection_stroke);
        assert_eq!(theme.selection_border_rest, PALETTE.selection_stroke_rest);

        assert_eq!(theme.text, DEFAULT_ORDINARY);
        assert_eq!(theme.text_active, PALETTE.selection_stroke);
        // egui's own weak-text attenuation: `Visuals::weak_text_color`
        // multiplies the noninteractive text colour — `DEFAULT_ORDINARY`,
        // per `style.rs::style()` — by `weak_text_alpha` (0.6 at
        // `Visuals::dark()`'s default, which `style()` never overrides).
        assert_eq!(theme.text_muted, DEFAULT_ORDINARY.gamma_multiply(0.6));
        assert_eq!(theme.input_background, DEFAULT_SOURCE_BACKGROUND);
        assert_eq!(theme.error, DEFAULT_BANG);
        assert_eq!(theme.warning, DEFAULT_BANG);
    }

    #[test]
    fn omitted_properties_inherit_from_the_parent() {
        let built_ins = [okabe_ito()];
        let document = child("okabe-ito");

        let resolved = resolve(&built_ins, "my-dark", &document).expect("valid document");

        assert_eq!(resolved.identity, "my-dark");
        assert_eq!(resolved.name, "Custom");
        assert_eq!(resolved.appearance, Appearance::Dark);
        assert_eq!(resolved.grid_background, built_ins[0].grid_background);
        assert_eq!(resolved.source_function, built_ins[0].source_function);
        assert_eq!(resolved.grid_border_width, built_ins[0].grid_border_width);
    }

    #[test]
    fn explicit_color_overrides_the_parent() {
        let built_ins = [okabe_ito()];
        let overridden = Color32::from_rgb(1, 2, 3);
        let document = ThemeDocument {
            colors: vec![(ColorKey::GridBackground, overridden)],
            ..child("okabe-ito")
        };

        let resolved = resolve(&built_ins, "my-dark", &document).expect("valid document");

        assert_eq!(resolved.grid_background, overridden);
        // Every other property is untouched.
        assert_eq!(resolved.panel_background, built_ins[0].panel_background);
    }

    ///
    /// Changing a role's foreground does not recalculate its background —
    /// `schema.md`: "Changing a role foreground does not recalculate them."
    /// Overriding `source.function` alone must leave
    /// `source.function.background` exactly as the parent resolved it.
    ///
    #[test]
    fn foreground_change_leaves_inherited_background_unchanged() {
        let built_ins = [okabe_ito()];
        let retuned = Color32::from_rgb(9, 9, 9);
        let document = ThemeDocument {
            colors: vec![(ColorKey::SourceFunction, retuned)],
            ..child("okabe-ito")
        };

        let resolved = resolve(&built_ins, "my-dark", &document).expect("valid document");

        assert_eq!(resolved.source_function, retuned);
        assert_eq!(
            resolved.source_function_background,
            built_ins[0].source_function_background
        );
    }

    #[test]
    fn explicit_width_overrides_the_parent() {
        let built_ins = [okabe_ito()];
        let document = ThemeDocument {
            grid_widths: vec![(GridWidthKey::GridBorder, 0.25)],
            chrome_widths: vec![(ChromeWidthKey::PanelBorder, 1.5)],
            ..child("okabe-ito")
        };

        let resolved = resolve(&built_ins, "my-dark", &document).expect("valid document");

        assert_eq!(resolved.grid_border_width.points(), 0.25);
        assert_eq!(resolved.panel_border_width.points(), 1.5);
        // Untouched Grid/chrome widths still inherit.
        assert_eq!(resolved.sector_seam_width, built_ins[0].sector_seam_width);
        assert_eq!(
            resolved.selection_border_width,
            built_ins[0].selection_border_width
        );
    }

    #[test]
    fn unknown_parent_is_rejected() {
        let built_ins = [okabe_ito()];
        let document = child("not-a-built-in");

        let error = resolve(&built_ins, "my-dark", &document).unwrap_err();

        assert_eq!(
            error,
            ThemeError::UnknownParent {
                parent: "not-a-built-in".to_owned()
            }
        );
    }

    ///
    /// A custom Theme can never chain from another custom Theme: `resolve`
    /// only ever looks a parent up in the `built_ins` slice the caller
    /// passes, so a name that only exists among already-resolved custom
    /// Themes is indistinguishable from an unknown one here.
    /// `schema.md`: "Only built-ins can be parents. Parent chains ... are
    /// errors."
    ///
    #[test]
    fn a_custom_theme_cannot_chain_from_another_custom_theme() {
        let built_ins = [okabe_ito()];
        let grandparent = child("okabe-ito");
        let custom = resolve(&built_ins, "my-dark", &grandparent).expect("valid document");
        // `custom` is deliberately not added to `built_ins`.
        let chained = child("my-dark");

        let error = resolve(&built_ins, "my-darker", &chained).unwrap_err();

        assert_eq!(
            error,
            ThemeError::UnknownParent {
                parent: "my-dark".to_owned()
            }
        );
        assert_eq!(custom.identity, "my-dark");
    }

    #[test]
    fn reserved_identity_is_rejected() {
        let built_ins = [okabe_ito()];
        let document = child("okabe-ito");

        let error = resolve(&built_ins, "okabe-ito", &document).unwrap_err();

        assert_eq!(
            error,
            ThemeError::ReservedIdentity {
                identity: "okabe-ito".to_owned()
            }
        );
    }

    #[test]
    fn appearance_mismatch_is_rejected() {
        let built_ins = [okabe_ito()];
        let document = ThemeDocument {
            appearance: Some(Appearance::Light),
            ..child("okabe-ito")
        };

        let error = resolve(&built_ins, "my-dark", &document).unwrap_err();

        assert_eq!(
            error,
            ThemeError::AppearanceMismatch {
                parent: Appearance::Dark,
                declared: Appearance::Light,
            }
        );
    }

    #[test]
    fn matching_declared_appearance_is_accepted() {
        let built_ins = [okabe_ito()];
        let document = ThemeDocument {
            appearance: Some(Appearance::Dark),
            ..child("okabe-ito")
        };

        let resolved = resolve(&built_ins, "my-dark", &document).expect("matching appearance");

        assert_eq!(resolved.appearance, Appearance::Dark);
    }

    #[test]
    fn grid_width_accepts_zero_fractional_and_the_inclusive_upper_bound() {
        for points in [0.0, 0.001, 0.5, 0.999, 1.0] {
            assert_eq!(
                GridWidth::from_points(points).map(GridWidth::points),
                Ok(points)
            );
        }
    }

    #[test]
    fn chrome_width_accepts_zero_fractional_and_the_inclusive_upper_bound() {
        for points in [0.0, 0.5, 1.0, 1.999, 2.0] {
            assert_eq!(
                ChromeWidth::from_points(points).map(ChromeWidth::points),
                Ok(points)
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
                GridWidth::from_points(points).is_err(),
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
                ChromeWidth::from_points(points).is_err(),
                "{points} should have been rejected"
            );
        }
    }

    #[test]
    fn resolve_rejects_a_nonfinite_grid_width_with_the_offending_property_named() {
        let built_ins = [okabe_ito()];
        let document = ThemeDocument {
            grid_widths: vec![(GridWidthKey::SectorSeam, f32::NAN)],
            ..child("okabe-ito")
        };

        let error = resolve(&built_ins, "my-dark", &document).unwrap_err();

        assert!(matches!(
            error,
            ThemeError::NonFiniteWidth {
                property: "sector.seam.width",
                ..
            }
        ));
    }

    #[test]
    fn resolve_rejects_an_out_of_range_chrome_width_with_the_offending_property_named() {
        let built_ins = [okabe_ito()];
        let document = ThemeDocument {
            chrome_widths: vec![(ChromeWidthKey::InputCursor, 2.5)],
            ..child("okabe-ito")
        };

        let error = resolve(&built_ins, "my-dark", &document).unwrap_err();

        assert_eq!(
            error,
            ThemeError::WidthOutOfRange {
                property: "input.cursor.width",
                points: 2.5,
                max: ChromeWidth::MAX_POINTS,
            }
        );
    }

    #[test]
    fn omitted_optional_fills_inherit_the_parent() {
        let built_ins = [okabe_ito()];
        let document = child("okabe-ito");

        let resolved = resolve(&built_ins, "my-dark", &document).expect("valid document");

        assert_eq!(resolved.cursor_background, built_ins[0].cursor_background);
        assert_eq!(
            resolved.region_cursor_background,
            built_ins[0].region_cursor_background
        );
    }

    #[test]
    fn explicit_none_clears_the_optional_fill_even_over_a_parent_colour() {
        let mut with_fill = okabe_ito();
        with_fill.cursor_background = Some(Color32::from_rgb(1, 2, 3));
        with_fill.region_cursor_background = Some(Color32::from_rgb(4, 5, 6));
        let built_ins = [with_fill];
        let document = ThemeDocument {
            cursor_background: Some(OptionalFill::None),
            region_cursor_background: Some(OptionalFill::None),
            ..child("okabe-ito")
        };

        let resolved = resolve(&built_ins, "my-dark", &document).expect("valid document");

        assert_eq!(resolved.cursor_background, None);
        assert_eq!(resolved.region_cursor_background, None);
    }

    ///
    /// An explicit transparent colour is a supplied value, not the `none`
    /// clear state — `schema.md`: "Transparent supplied colour suppresses
    /// fallback; it remains distinct from absence."
    ///
    #[test]
    fn explicit_transparent_colour_is_a_supplied_value_not_a_clear() {
        let built_ins = [okabe_ito()];
        let document = ThemeDocument {
            cursor_background: Some(OptionalFill::Color(Color32::TRANSPARENT)),
            region_cursor_background: Some(OptionalFill::Color(Color32::TRANSPARENT)),
            ..child("okabe-ito")
        };

        let resolved = resolve(&built_ins, "my-dark", &document).expect("valid document");

        assert_eq!(resolved.cursor_background, Some(Color32::TRANSPARENT));
        assert_eq!(
            resolved.region_cursor_background,
            Some(Color32::TRANSPARENT)
        );
        assert_ne!(resolved.cursor_background, None);
    }

    #[test]
    fn explicit_optional_fill_colour_is_stored_including_opaque() {
        let built_ins = [okabe_ito()];
        let colour = Color32::from_rgb(10, 20, 30);
        let document = ThemeDocument {
            cursor_background: Some(OptionalFill::Color(colour)),
            ..child("okabe-ito")
        };

        let resolved = resolve(&built_ins, "my-dark", &document).expect("valid document");

        assert_eq!(resolved.cursor_background, Some(colour));
    }

    #[test]
    fn theme_color_reads_the_key_a_document_wrote() {
        let built_ins = [okabe_ito()];
        let overridden = Color32::from_rgb(7, 7, 7);
        let document = ThemeDocument {
            colors: vec![(ColorKey::Text, overridden)],
            ..child("okabe-ito")
        };

        let resolved = resolve(&built_ins, "my-dark", &document).expect("valid document");

        assert_eq!(resolved.color(ColorKey::Text), overridden);
    }

    ///
    /// Every [`ColorKey`] variant, applied as a distinct override and read
    /// back through [`Theme::color`], with every *other* key left at the
    /// parent's value. This is the exhaustive companion to
    /// `okabe_ito_defines_every_key_at_the_schema_values`: that test proves
    /// the built-in sets every key, this one proves every key can be
    /// independently addressed by `resolve`'s `color_mut` match without a
    /// key silently writing, or being read from, the wrong field.
    ///
    #[test]
    fn every_color_key_can_be_independently_overridden() {
        let built_ins = [okabe_ito()];
        let keys = [
            ColorKey::WindowBackground,
            ColorKey::PanelBackground,
            ColorKey::GridBackground,
            ColorKey::CellBackground,
            ColorKey::SourceOrdinary,
            ColorKey::SourceComment,
            ColorKey::SourceNumber,
            ColorKey::SourceNote,
            ColorKey::SourceFunction,
            ColorKey::SourceBang,
            ColorKey::SourceSequence,
            ColorKey::SourceOrdinaryBackground,
            ColorKey::SourceCommentBackground,
            ColorKey::SourceNumberBackground,
            ColorKey::SourceNoteBackground,
            ColorKey::SourceFunctionBackground,
            ColorKey::SourceBangBackground,
            ColorKey::SourceAtomBackground,
            ColorKey::SourceSequenceBackground,
            ColorKey::DiagnosticForeground,
            ColorKey::DiagnosticBackground,
            ColorKey::DiagnosticBorder,
            ColorKey::OutputPortalForeground,
            ColorKey::OutputPortalBackground,
            ColorKey::OutputPortalBorder,
            ColorKey::GridBorder,
            ColorKey::SectorSeam,
            ColorKey::CursorBorder,
            ColorKey::RegionBorder,
            ColorKey::CursorArea,
            ColorKey::RegionBackground,
            ColorKey::PanelBorder,
            ColorKey::SelectionBackground,
            ColorKey::SelectionBorder,
            ColorKey::SelectionBorderRest,
            ColorKey::WidgetInactiveBorder,
            ColorKey::Text,
            ColorKey::TextActive,
            ColorKey::TextMuted,
            ColorKey::InputBackground,
            ColorKey::Link,
            ColorKey::CodeBackground,
            ColorKey::InputCursor,
            ColorKey::Error,
            ColorKey::Warning,
        ];
        assert_eq!(
            keys.len(),
            45,
            "the catalogue has 45 non-optional colour keys"
        );

        for &key in &keys {
            let colour = Color32::from_rgb(1, 2, 3);
            let document = ThemeDocument {
                colors: vec![(key, colour)],
                ..child("okabe-ito")
            };

            let resolved = resolve(&built_ins, "my-dark", &document).expect("valid document");

            assert_eq!(
                resolved.color(key),
                colour,
                "{key:?} did not take its override"
            );
            for &other in &keys {
                if other != key {
                    assert_eq!(
                        resolved.color(other),
                        built_ins[0].color(other),
                        "{key:?}'s override changed {other:?}"
                    );
                }
            }
        }
    }

    ///
    /// The Grid/chrome-bounded width counterpart of
    /// `every_color_key_can_be_independently_overridden`, and what exercises
    /// [`Theme::grid_width`] and [`Theme::chrome_width`] — the by-key
    /// readers slice B's flat per-frame lookup can build from.
    ///
    #[test]
    fn every_grid_and_chrome_width_key_can_be_independently_overridden() {
        let built_ins = [okabe_ito()];

        let grid_keys = [
            GridWidthKey::GridBorder,
            GridWidthKey::SectorSeam,
            GridWidthKey::CellSelectionBorder,
            GridWidthKey::CursorBorder,
            GridWidthKey::RegionBorder,
            GridWidthKey::DiagnosticBorder,
            GridWidthKey::OutputPortalBorder,
        ];
        assert_eq!(
            grid_keys.len(),
            7,
            "the catalogue has 7 Grid-bounded width keys"
        );
        for &key in &grid_keys {
            let document = ThemeDocument {
                grid_widths: vec![(key, 0.1)],
                ..child("okabe-ito")
            };
            let resolved = resolve(&built_ins, "my-dark", &document).expect("valid document");
            assert_eq!(
                resolved.grid_width(key).points(),
                0.1,
                "{key:?} did not take its override"
            );
        }

        let chrome_keys = [
            ChromeWidthKey::PanelBorder,
            ChromeWidthKey::SelectionBorder,
            ChromeWidthKey::WidgetBorder,
            ChromeWidthKey::WidgetInactiveBorder,
            ChromeWidthKey::InputCursor,
        ];
        assert_eq!(
            chrome_keys.len(),
            5,
            "the catalogue has 5 chrome-bounded width keys"
        );
        for &key in &chrome_keys {
            let document = ThemeDocument {
                chrome_widths: vec![(key, 1.9)],
                ..child("okabe-ito")
            };
            let resolved = resolve(&built_ins, "my-dark", &document).expect("valid document");
            assert_eq!(
                resolved.chrome_width(key).points(),
                1.9,
                "{key:?} did not take its override"
            );
        }
    }

    #[test]
    fn grid_and_chrome_width_key_names_match_the_schema_catalogue() {
        assert_eq!(GridWidthKey::GridBorder.name(), "grid.border.width");
        assert_eq!(GridWidthKey::SectorSeam.name(), "sector.seam.width");
        assert_eq!(
            GridWidthKey::CellSelectionBorder.name(),
            "cell.selection.border.width"
        );
        assert_eq!(GridWidthKey::CursorBorder.name(), "cursor.border.width");
        assert_eq!(GridWidthKey::RegionBorder.name(), "region.border.width");
        assert_eq!(
            GridWidthKey::DiagnosticBorder.name(),
            "diagnostic.border.width"
        );
        assert_eq!(
            GridWidthKey::OutputPortalBorder.name(),
            "output_portal.border.width"
        );
        assert_eq!(ChromeWidthKey::PanelBorder.name(), "panel.border.width");
        assert_eq!(
            ChromeWidthKey::SelectionBorder.name(),
            "selection.border.width"
        );
        assert_eq!(ChromeWidthKey::WidgetBorder.name(), "widget.border.width");
        assert_eq!(
            ChromeWidthKey::WidgetInactiveBorder.name(),
            "widget.inactive.border.width"
        );
        assert_eq!(ChromeWidthKey::InputCursor.name(), "input.cursor.width");
    }
}
