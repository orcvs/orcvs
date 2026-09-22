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

// === Contrast validation ===

///
/// Measures a Theme's contrast: [`validate`] answers each [`ContrastCheck`]'s
/// WCAG ratio and whether it clears [`CONTRAST_FLOOR`], replacing
/// `style.rs`'s former per-colour assertions of Okabe–Ito's exact values
/// (`.scratch/theming/issues/08`) with a measurement any Theme can be put
/// through, built-in or loaded.
///
/// A single module rather than five individually annotated items: every
/// item here is unused outside this module's own tests until
/// `.scratch/theming/issues/07` wires a loader and shows `validate`'s report
/// when a viewer loads a scheme, so one inner attribute states that once for
/// the whole module instead of repeating the same reason at each item.
///
pub(crate) mod contrast {
    #![cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "unused outside this module's own tests until theming/07 wires a loader \
                       and shows validate's report when a viewer loads a scheme"
        )
    )]

    use egui::Color32;

    use super::Theme;

    ///
    /// The WCAG 2.1 Success Criterion 1.4.3 ("Contrast (Minimum)") ratio
    /// every [`ContrastCheck`] is measured against: 4.5:1, the floor
    /// normal-size text must clear. Stated once, here, with its source,
    /// rather than repeated as a literal at each call site —
    /// `.scratch/theming/issues/08`'s own acceptance line.
    ///
    pub(crate) const CONTRAST_FLOOR: f32 = 4.5;

    ///
    /// One named colour pair [`validate`] measures: a Token's glyph colour,
    /// or one of ADR 0053's named keys, against the background it is read
    /// on.
    ///
    /// The seven rows of ADR 0053's Token table are Source Paint colours and
    /// read against `base00` ([`Theme::grid_background`], the Source
    /// background) — as do the Diagnostic and Output Portal foreground keys
    /// the same table names. `text` and `text.muted` are chrome, not Source
    /// Paint, so they read against `panel.background`
    /// ([`Theme::panel_background`]) instead: the surface ADR 0053's chrome
    /// mapping actually draws them on.
    ///
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub(crate) enum ContrastCheck {
        Comment,
        /// Ordinary, Char and Atom share one glyph colour
        /// ([`Theme::source_ordinary`]) and so share one check — ADR 0053's
        /// Token table lists the three together for the same reason.
        Ordinary,
        Number,
        Note,
        Function,
        Bang,
        Sequence,
        DiagnosticForeground,
        OutputPortalForeground,
        Text,
        TextMuted,
    }

    impl ContrastCheck {
        /// Every check [`validate`] measures, in ADR 0053's Token-table
        /// order followed by its named-key table order.
        const ALL: [Self; 11] = [
            Self::Comment,
            Self::Ordinary,
            Self::Number,
            Self::Note,
            Self::Function,
            Self::Bang,
            Self::Sequence,
            Self::DiagnosticForeground,
            Self::OutputPortalForeground,
            Self::Text,
            Self::TextMuted,
        ];

        /// The label a [`ContrastResult`] names this check by: ADR 0053's
        /// Token name, or a named key at its exact spelling.
        pub(crate) fn name(self) -> &'static str {
            match self {
                Self::Comment => "Comment",
                Self::Ordinary => "Ordinary, Char, Atom",
                Self::Number => "Number",
                Self::Note => "Note",
                Self::Function => "Function",
                Self::Bang => "Bang",
                Self::Sequence => "Sequence",
                Self::DiagnosticForeground => "diagnostic.foreground",
                Self::OutputPortalForeground => "output_portal.foreground",
                Self::Text => "text",
                Self::TextMuted => "text.muted",
            }
        }

        /// This check's foreground and the background it is read against,
        /// both read from `theme`.
        fn colours(self, theme: &Theme) -> (Color32, Color32) {
            let base00 = theme.grid_background;
            let panel = theme.panel_background;
            match self {
                Self::Comment => (theme.source_comment, base00),
                Self::Ordinary => (theme.source_ordinary, base00),
                Self::Number => (theme.source_number, base00),
                Self::Note => (theme.source_note, base00),
                Self::Function => (theme.source_function, base00),
                Self::Bang => (theme.source_bang, base00),
                Self::Sequence => (theme.source_sequence, base00),
                Self::DiagnosticForeground => (theme.diagnostic_foreground, base00),
                Self::OutputPortalForeground => (theme.output_portal_foreground, base00),
                Self::Text => (theme.text, panel),
                Self::TextMuted => (theme.text_muted, panel),
            }
        }
    }

    ///
    /// One [`ContrastCheck`]'s measured result against a `Theme`: its WCAG
    /// ratio. [`Self::passes`] derives the pass/fail answer from `ratio`
    /// and [`CONTRAST_FLOOR`] rather than storing a second field that could
    /// disagree with it.
    ///
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub(crate) struct ContrastResult {
        pub(crate) check: ContrastCheck,
        pub(crate) ratio: f32,
    }

    impl ContrastResult {
        /// Whether [`Self::ratio`] clears [`CONTRAST_FLOOR`].
        pub(crate) fn passes(self) -> bool {
            self.ratio >= CONTRAST_FLOOR
        }
    }

    ///
    /// Measures every [`ContrastCheck`] against `theme` and reports each
    /// one's WCAG ratio.
    ///
    /// Never refuses `theme`: a scheme that fails a check is measured and
    /// reported exactly like one that passes, because the floor is a fact
    /// about the scheme worth seeing rather than a gate a Theme must clear
    /// before it loads. `.scratch/theming/issues/07` shows this report when
    /// a viewer loads a scheme; a failing one loads anyway, because the
    /// viewer chose it.
    ///
    /// # Deviation from the issue's stated signature
    ///
    /// `.scratch/theming/issues/08` asks for `validate(scheme, template)`
    /// over a `Scheme` of sixteen base16 slots and a separate template
    /// mapping Tokens onto them. `06`, this issue's blocker, is only partly
    /// landed on this branch's base: it shipped a resolved `Theme` with
    /// every Token's colour already a direct field, but never the
    /// `Scheme`/`TEMPLATE` split, and none of its further built-ins beyond
    /// `okabe_ito`. No `Scheme` or `TEMPLATE` type exists in this crate
    /// today. The closest faithful shape reachable from what exists takes
    /// one argument, the already-resolved Theme: the mapping `scheme` and
    /// `template` would have performed together has already happened by the
    /// time a `Theme` exists. This signature is worth revisiting once `06`
    /// introduces `Scheme` — `07`'s loader will otherwise build a `Theme`
    /// the same way [`okabe_ito`](super::okabe_ito) does, so a loaded scheme
    /// and a built-in reach `validate` identically either way. Recorded in
    /// `.scratch/theming/issues/08`'s comments as well.
    ///
    pub(crate) fn validate(theme: &Theme) -> Vec<ContrastResult> {
        ContrastCheck::ALL
            .into_iter()
            .map(|check| {
                let (foreground, background) = check.colours(theme);
                // `contrast` requires both arguments opaque (see its own
                // doc comment). `to_opaque` normalizes `background` first —
                // deliberately, rather than by a `debug_assert` on an
                // invariant this crate does not actually prove: `resolve`
                // already lets a custom Theme document override
                // `grid.background`/`panel.background` with any colour,
                // including a translucent one, and CLAUDE.md reserves
                // assertions for proven invariants. Normalizing composites
                // it onto an opaque version of itself
                // (`Color32::to_opaque`, the same un-premultiply
                // `theme::tests::okabe_ito_matches_todays_style_and_
                // palette_constants` already uses for `panel_border`), so a
                // translucent background is measured deliberately rather
                // than refused or silently misread. It is a no-op for an
                // already-opaque background, which every shipped Theme has
                // today.
                let background = background.to_opaque();
                // The pinned "self behind on_top" compositing every other
                // resolved-Theme channel uses (`style.rs`'s
                // `compose_cell_fill`, `ordinary_border`): a translucent
                // foreground such as `text.muted` is measured as the colour
                // it actually displays over `background`, not its stored
                // premultiplied bytes misread as straight sRGB. An opaque
                // foreground is unchanged by this — `background.blend
                // (foreground)` collapses to `foreground` outright once its
                // alpha is 255.
                let displayed = background.blend(foreground);
                let ratio = contrast(displayed, background);
                ContrastResult { check, ratio }
            })
            .collect()
    }

    ///
    /// The WCAG 2.1 relative-luminance contrast ratio between two colours —
    /// ported from `style.rs`'s own `contrast`, which
    /// `.scratch/theming/issues/08` removes now that this validator replaces
    /// the per-colour assertions it measured.
    ///
    /// Both arguments must already be opaque: `.r()`/`.g()`/`.b()` read
    /// [`Color32`]'s premultiplied bytes directly, which equal straight sRGB
    /// only at full alpha. [`validate`] is this function's one caller and
    /// guarantees this — `background` by `Color32::to_opaque`, `foreground`
    /// by compositing it over that already-opaque `background` first, which
    /// is always itself opaque (a blend onto a 255-alpha base always sums to
    /// 255 alpha).
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
            0.2126 * channel(colour.r())
                + 0.7152 * channel(colour.g())
                + 0.0722 * channel(colour.b())
        };
        let (first, second) = (luminance(foreground), luminance(background));
        (first.max(second) + 0.05) / (first.min(second) + 0.05)
    }

    #[cfg(test)]
    mod tests {
        use egui::Color32;

        use super::{ContrastCheck, ContrastResult, contrast, validate};
        use crate::theme::{Theme, okabe_ito};

        /// The repeated `report.iter().find(...)` every test below needs at
        /// least once.
        fn find(report: &[ContrastResult], check: ContrastCheck) -> ContrastResult {
            *report
                .iter()
                .find(|result| result.check == check)
                .unwrap_or_else(|| panic!("validate did not report {check:?}"))
        }

        ///
        /// The WCAG spec's own stated extreme: black text on a white
        /// background is the maximum contrast ratio, 21:1 ("Understanding
        /// Success Criterion 1.4.3") — a fact about the ratio's own defined
        /// range, not a figure this module's arithmetic could drift into by
        /// coincidence.
        ///
        #[test]
        fn contrast_of_black_and_white_is_the_wcag_maximum_21_to_1() {
            let ratio = contrast(Color32::WHITE, Color32::BLACK);
            assert!((ratio - 21.0).abs() < 0.01, "got {ratio:.2}:1");
        }

        ///
        /// The ratio's own stated minimum: two identical colours are 1:1,
        /// the numerator and denominator of the WCAG formula collapsing to
        /// the same value regardless of which colour is chosen.
        ///
        #[test]
        fn contrast_of_identical_colours_is_the_wcag_minimum_1_to_1() {
            let mid_tone = Color32::from_rgb(96, 140, 60);
            let ratio = contrast(mid_tone, mid_tone);
            assert!((ratio - 1.0).abs() < 0.001, "got {ratio:.2}:1");
        }

        ///
        /// A worked mid pair, computed by hand from the WCAG 2.1
        /// relative-luminance formula rather than by running this module's
        /// own code under test: pure red's relative luminance is
        /// `0.2126 * 1.0 + 0.7152 * 0.0 + 0.0722 * 0.0 = 0.2126` (its own
        /// channel fully saturated, the other two at zero); white's is
        /// `1.0` (the three coefficients sum to `1.0` exactly). Their ratio
        /// is `(1.0 + 0.05) / (0.2126 + 0.05) = 1.05 / 0.2626 ≈ 4.00:1`.
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

        ///
        /// Every check measured against the background ADR 0053 documents
        /// for it, proven by construction rather than by re-deriving
        /// Okabe–Ito's own figures: `grid_background` and
        /// `panel_background` are set far enough apart that a check reading
        /// the wrong one measures a detectably different ratio, not a
        /// coincidentally close one, so a `colours` mapping bug would fail
        /// this test rather than pass it quietly.
        ///
        #[test]
        fn every_check_measures_against_its_documented_background() {
            let grid_background = Color32::from_rgb(10, 10, 10);
            let panel_background = Color32::from_rgb(200, 200, 200);
            let shared_foreground = Color32::from_rgb(128, 128, 128);
            let theme = Theme {
                grid_background,
                panel_background,
                source_comment: shared_foreground,
                source_ordinary: shared_foreground,
                source_number: shared_foreground,
                source_note: shared_foreground,
                source_function: shared_foreground,
                source_bang: shared_foreground,
                source_sequence: shared_foreground,
                diagnostic_foreground: shared_foreground,
                output_portal_foreground: shared_foreground,
                text: shared_foreground,
                text_muted: shared_foreground,
                ..okabe_ito()
            };

            let against_grid = contrast(shared_foreground, grid_background);
            let against_panel = contrast(shared_foreground, panel_background);
            assert!(
                (against_grid - against_panel).abs() > 1.0,
                "the two backgrounds must read differently enough for this test to catch a swap"
            );

            let report = validate(&theme);
            for check in ContrastCheck::ALL {
                let expected = match check {
                    ContrastCheck::Text | ContrastCheck::TextMuted => against_panel,
                    _ => against_grid,
                };
                let result = find(&report, check);
                assert!(
                    (result.ratio - expected).abs() < 0.01,
                    "{} measured {:.2}:1, expected {:.2}:1 against its documented background",
                    check.name(),
                    result.ratio,
                    expected
                );
            }
        }

        ///
        /// A translucent foreground composited over an opaque background
        /// before measurement, the way `text.muted`'s alpha is in a real
        /// Theme. 50%-alpha white over black composites to mid-grey `(128,
        /// 128, 128)` — halfway, by construction of the blend — whose ratio
        /// against black is worked by hand rather than reproduced from this
        /// module's own code: relative luminance
        /// `((128 / 255 + 0.055) / 1.055) ^ 2.4 ≈ 0.216`, ratio
        /// `(0.216 + 0.05) / (0.0 + 0.05) ≈ 5.32:1`.
        ///
        #[test]
        fn validate_composites_a_translucent_foreground_before_measuring() {
            let translucent_white = Color32::from_rgba_unmultiplied(255, 255, 255, 128);
            let theme = Theme {
                grid_background: Color32::BLACK,
                source_comment: translucent_white,
                ..okabe_ito()
            };

            let report = validate(&theme);

            let comment = find(&report, ContrastCheck::Comment);
            assert!(
                (comment.ratio - 5.32).abs() < 0.01,
                "50%-white-on-black composites to mid-grey: got {:.2}:1, expected ~5.32:1",
                comment.ratio
            );
        }

        ///
        /// A translucent `background` is normalized to opaque before it is
        /// used at all — for the blend as well as the final measurement —
        /// rather than read as though its premultiplied bytes were already
        /// straight sRGB. Comparing against `Color32::to_opaque` called
        /// directly, the same public operation [`validate`] uses, proves
        /// the normalization actually happens rather than merely failing to
        /// panic on a translucent background by chance.
        ///
        #[test]
        fn validate_normalizes_a_translucent_background_before_measuring() {
            let translucent_panel = Color32::from_rgba_unmultiplied(11, 17, 18, 200);
            let opaque_text = Color32::from_rgb(234, 235, 229);
            let theme = Theme {
                panel_background: translucent_panel,
                text: opaque_text,
                ..okabe_ito()
            };

            let report = validate(&theme);
            let text = find(&report, ContrastCheck::Text);

            let opaque_panel = translucent_panel.to_opaque();
            let expected = contrast(opaque_panel.blend(opaque_text), opaque_panel);
            assert!(
                (text.ratio - expected).abs() < 0.001,
                "got {:.2}:1, expected {:.2}:1 against the opaque-normalized panel background",
                text.ratio,
                expected
            );

            // And proof this is not vacuous: measuring against the raw,
            // un-normalized translucent background gives a different answer,
            // so normalizing is doing real work rather than being a no-op
            // here.
            let unnormalized = contrast(opaque_text, translucent_panel);
            assert!(
                (expected - unnormalized).abs() > 0.01,
                "the opaque-normalized and raw-translucent ratios should differ for this \
                 background/foreground pair, or this test proves nothing"
            );
        }

        ///
        /// The validator against a Theme built to fail more than one check
        /// — proof `validate` discriminates a pass from a failure rather
        /// than reporting every check as passing regardless of the colours
        /// it is given. `text` is set equal to `panel_background`, and
        /// `source_number` equal to `grid_background` (identical colours
        /// measure 1:1, this module's own documented WCAG minimum), while
        /// every other property is left at Okabe–Ito's passing values.
        ///
        #[test]
        fn validate_reports_a_theme_built_to_fail_rather_than_a_vacuous_pass() {
            let base = okabe_ito();
            let theme = Theme {
                text: base.panel_background,
                source_number: base.grid_background,
                ..base
            };

            let report = validate(&theme);

            let text = find(&report, ContrastCheck::Text);
            assert!(
                !text.passes(),
                "text equal to panel_background should fail: {:.2}:1",
                text.ratio
            );
            assert!(
                (text.ratio - 1.0).abs() < 0.01,
                "identical colours measure 1:1, got {:.2}:1",
                text.ratio
            );

            let number = find(&report, ContrastCheck::Number);
            assert!(
                !number.passes(),
                "source_number equal to base00 should fail: {:.2}:1",
                number.ratio
            );

            // A check untouched by the failing override still passes, so
            // the failures above are specific to the retuned properties
            // rather than `validate` reporting every check as failing once
            // one does.
            let comment = find(&report, ContrastCheck::Comment);
            assert!(
                comment.passes(),
                "Comment was not retuned and should still clear the floor: {:.2}:1",
                comment.ratio
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use egui::Color32;

    use super::contrast::{ContrastCheck, validate};
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

    ///
    /// Every shipped Theme this crate ships today, checked against an
    /// accepted-failures record keyed by Theme identity —
    /// `.scratch/theming/issues/08`'s "every shipped scheme passes" line,
    /// resolved so a fixed Sequence and a newly failing check both force a
    /// deliberate edit to `accepted_failures` below, rather than the test
    /// excusing a whole check globally (as an earlier version of this test
    /// did) or silently starting to fail once a colour moves.
    ///
    /// Only `okabe_ito()` ships today: `.scratch/theming/issues/06`'s
    /// further built-ins and `07`'s loader are not built yet, so `shipped`
    /// is a hand-kept array rather than an iterated built-in registry — a
    /// known weakness recorded in `.scratch/theming/issues/08`'s comments,
    /// to close once `06` gives this crate a list to iterate instead.
    ///
    #[test]
    fn every_shipped_theme_matches_its_accepted_failure_record() {
        let shipped = [okabe_ito()];

        // Keyed by Theme identity. Okabe–Ito's one entry is Sequence's
        // recorded floor shortfall (`theme.md`), pending the
        // retune-vs-exception decision `.scratch/theming/issues/08`'s
        // comments leave for a human; an identity absent from this list is
        // expected to have no failing checks.
        let accepted_failures: &[(&str, &[ContrastCheck])] =
            &[("okabe-ito", &[ContrastCheck::Sequence])];

        for theme in &shipped {
            let mut expected: Vec<&str> = accepted_failures
                .iter()
                .find(|(identity, _)| *identity == theme.identity)
                .map(|(_, checks)| checks.iter().map(|check| check.name()).collect())
                .unwrap_or_default();
            expected.sort_unstable();

            let mut actual: Vec<&str> = validate(theme)
                .into_iter()
                .filter(|result| !result.passes())
                .map(|result| result.check.name())
                .collect();
            actual.sort_unstable();

            assert_eq!(
                actual, expected,
                "{}'s failing checks changed — update accepted_failures deliberately, in the \
                 same commit as the fix or the newly discovered failure",
                theme.identity
            );
        }
    }
}
