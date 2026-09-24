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
//! function. It has no file I/O and no document decoder:
//! the crate-private `theme_document` module decodes a document's bytes into a
//! `ThemeDocument`, and the crate-private `theme_registry` module's native
//! discovery (`.scratch/theming/issues/07`) reads those bytes and calls
//! `resolve`.
//! Source painting, settings and persistence are unchanged by this slice;
//! `.scratch/theming/issues/06`'s later slices consume this module.
//!

use std::borrow::Cow;

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
    /// [`orcvs_light`] declares this, and a custom document inherits it from
    /// a light parent. Declaring `appearance = "light"` only restates the
    /// parent's appearance: [`resolve`] refuses one that differs.
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

    /// Consumed by `style.rs::style`'s chrome borders, input caret and IME
    /// underline widths (`.scratch/theming/issues/03`).
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
    all(target_arch = "wasm32", not(test)),
    expect(
        dead_code,
        reason = "only native discovery loads a Theme document; the web has the built-ins alone"
    )
)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum OptionalFill {
    /// The document string `"none"`: clears the optional fill and enables its
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
/// Writes each key enum's `schema.md` spelling once and derives both
/// directions from that one table: `name`, an exhaustive `match` over the
/// enum, so a new variant without a spelling fails to compile; and
/// `from_name`, its inverse, generated from the same literals, so the two
/// cannot disagree. A spelling repeated within one enum is an unreachable
/// `from_name` arm, which the workspace's `-D warnings` gate refuses.
///
macro_rules! property_names {
    ($key:ident { $($variant:ident => $name:literal),* $(,)? }) => {
        #[cfg_attr(
            all(target_arch = "wasm32", not(test)),
            expect(dead_code, reason = "only native discovery loads a Theme document; the web has the built-ins alone")
        )]
        impl $key {
            /// The property's literal, case-sensitive `schema.md` name.
            pub(crate) fn name(self) -> &'static str {
                match self {
                    $(Self::$variant => $name,)*
                }
            }

            /// The key `schema.md` spells `name`, matched exactly — dots
            /// are literal characters, and case matters.
            pub(crate) fn from_name(name: &str) -> Option<Self> {
                match name {
                    $($name => Some(Self::$variant),)*
                    _ => None,
                }
            }
        }
    };
}

///
/// Every named colour property `schema.md`'s catalogue defines, less the
/// two optional Cursor fills, which [`OptionalFill`] carries instead of a
/// plain `Color32`. Each variant's case- and dot-sensitive schema spelling
/// is written once, in the `property_names!` table below it, which the
/// Theme document decoder reads to map raw document text to a variant.
/// `Ord` is declaration order, which is the catalogue's: the decoder sorts a
/// document's properties by it so the same document compares equal whatever
/// order its format's map yields.
///
#[cfg_attr(
    all(target_arch = "wasm32", not(test)),
    expect(
        dead_code,
        reason = "only native discovery loads a Theme document; the web has the built-ins alone"
    )
)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
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

property_names! {
    ColorKey {
        WindowBackground => "window.background",
        PanelBackground => "panel.background",
        GridBackground => "grid.background",
        CellBackground => "cell.background",
        SourceOrdinary => "source.ordinary",
        SourceComment => "source.comment",
        SourceNumber => "source.number",
        SourceNote => "source.note",
        SourceFunction => "source.function",
        SourceBang => "source.bang",
        SourceSequence => "source.sequence",
        SourceOrdinaryBackground => "source.ordinary.background",
        SourceCommentBackground => "source.comment.background",
        SourceNumberBackground => "source.number.background",
        SourceNoteBackground => "source.note.background",
        SourceFunctionBackground => "source.function.background",
        SourceBangBackground => "source.bang.background",
        SourceAtomBackground => "source.atom.background",
        SourceSequenceBackground => "source.sequence.background",
        DiagnosticForeground => "diagnostic.foreground",
        DiagnosticBackground => "diagnostic.background",
        DiagnosticBorder => "diagnostic.border",
        OutputPortalForeground => "output_portal.foreground",
        OutputPortalBackground => "output_portal.background",
        OutputPortalBorder => "output_portal.border",
        GridBorder => "grid.border",
        SectorSeam => "sector.seam",
        CursorBorder => "cursor.border",
        RegionBorder => "region.border",
        CursorArea => "cursor.area",
        RegionBackground => "region.background",
        PanelBorder => "panel.border",
        SelectionBackground => "selection.background",
        SelectionBorder => "selection.border",
        SelectionBorderRest => "selection.border.rest",
        WidgetInactiveBorder => "widget.inactive.border",
        Text => "text",
        TextActive => "text.active",
        TextMuted => "text.muted",
        InputBackground => "input.background",
        Link => "link",
        CodeBackground => "code.background",
        InputCursor => "input.cursor",
        Error => "error",
        Warning => "warning",
    }
}

///
/// The Grid/Cell/Sector Seam width properties, bounded 0 to 1 point by
/// [`GridWidth`].
///
#[cfg_attr(
    all(target_arch = "wasm32", not(test)),
    expect(
        dead_code,
        reason = "only native discovery loads a Theme document; the web has the built-ins alone"
    )
)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum GridWidthKey {
    GridBorder,
    SectorSeam,
    CellSelectionBorder,
    CursorBorder,
    RegionBorder,
    DiagnosticBorder,
    OutputPortalBorder,
}

property_names! {
    GridWidthKey {
        GridBorder => "grid.border.width",
        SectorSeam => "sector.seam.width",
        CellSelectionBorder => "cell.selection.border.width",
        CursorBorder => "cursor.border.width",
        RegionBorder => "region.border.width",
        DiagnosticBorder => "diagnostic.border.width",
        OutputPortalBorder => "output_portal.border.width",
    }
}

///
/// The chrome width properties, bounded 0 to 2 points by [`ChromeWidth`].
///
#[cfg_attr(
    all(target_arch = "wasm32", not(test)),
    expect(
        dead_code,
        reason = "only native discovery loads a Theme document; the web has the built-ins alone"
    )
)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum ChromeWidthKey {
    PanelBorder,
    SelectionBorder,
    WidgetBorder,
    WidgetInactiveBorder,
    InputCursor,
}

property_names! {
    ChromeWidthKey {
        PanelBorder => "panel.border.width",
        SelectionBorder => "selection.border.width",
        WidgetBorder => "widget.border.width",
        WidgetInactiveBorder => "widget.inactive.border.width",
        InputCursor => "input.cursor.width",
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
        reason = "these by-key accessors are for resolve and future by-key callers; \
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
/// its file name (ADR 0053), which the Theme registry supplies. A saved
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
    #[cfg_attr(
        all(target_arch = "wasm32", not(test)),
        expect(
            dead_code,
            reason = "only native discovery loads a Theme document; the web has the built-ins alone"
        )
    )]
    pub(crate) fn from_stem(stem: &str) -> Option<Self> {
        (!stem.is_empty()).then(|| Self(Cow::Owned(stem.to_owned())))
    }

    ///
    /// The identity a setting names, as written in `~/.orcvs/config.toml`.
    /// It is not checked here: `SelectedThemes` reports one no Theme of its
    /// appearance answers to, and presents that appearance's built-in.
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

    /// Whether this is a built-in's reserved identity.
    #[cfg_attr(
        all(target_arch = "wasm32", not(test)),
        expect(
            dead_code,
            reason = "only native discovery loads a Theme document; the web has the built-ins alone"
        )
    )]
    pub(crate) fn is_reserved(&self) -> bool {
        *self == OKABE_ITO_IDENTITY || *self == ORCVS_LIGHT_IDENTITY
    }

    #[cfg_attr(
        all(target_arch = "wasm32", not(test)),
        expect(
            dead_code,
            reason = "only native discovery loads a Theme document; the web has the built-ins alone"
        )
    )]
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
/// is spelled out explicitly, so the compiler enforces "built-ins define
/// every property" rather than a runtime completeness check — remove a field
/// from this literal and the crate fails to build.
///
/// `okabe_ito_defines_every_key_at_the_schema_values` in this module's tests
/// pins every field against `schema.md`'s own dark table, and
/// `style::tests::okabe_ito_chrome_matches_the_decided_record` cross-checks
/// the chrome keys against `theme.md`'s decided record read through
/// [`crate::style::style`], so this built-in reproduces today's shipped
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
        source_sequence: straight_rgba(0x00_72_B2_FF),
        source_ordinary_background: straight_rgba(0x00_00_00_00),
        source_comment_background: straight_rgba(0x00_00_00_00),
        source_number_background: straight_rgba(0x56_B4_E9_1A),
        source_note_background: straight_rgba(0xF0_E4_42_1A),
        source_function_background: straight_rgba(0x00_9E_73_1A),
        source_bang_background: straight_rgba(0x00_00_00_00),
        source_atom_background: straight_rgba(0xEA_EB_E5_1A),
        source_sequence_background: straight_rgba(0x00_72_B2_1A),

        diagnostic_foreground: straight_rgba(0xD5_5E_00_FF),
        diagnostic_background: straight_rgba(0x00_00_00_00),
        diagnostic_border: straight_rgba(0x00_00_00_00),
        output_portal_foreground: straight_rgba(0xE6_9F_00_FF),
        output_portal_background: straight_rgba(0xE6_9F_00_1A),
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

/// The reserved identity of the light built-in, named by `schema.md`
/// ("`orcvs-light` the reserved light identity") and recorded in
/// `console/src/theme.md`'s Shipped Themes table.
pub(crate) const ORCVS_LIGHT_IDENTITY: ThemeIdentity = ThemeIdentity::reserved("orcvs-light");

///
/// The Orcvs Light built-in Theme: the complete light definition
/// `.scratch/theming/issues/04` prepares for review, at the reserved
/// identity `ORCVS_LIGHT_IDENTITY`. Every field is spelled out explicitly
/// for the same reason [`okabe_ito`]'s is — including the properties whose
/// value it shares with the dark built-in, which `04` requires be recorded
/// rather than left to a toolkit default.
///
/// The chrome colours start from the `feat/egui-theming` branch's hand-tuned
/// `LIGHT_PALETTE`, mapped onto the named keys as
/// `.scratch/theming/issues/04`'s 2026-09-21 comment directs (page →
/// `window.background`/`panel.background`, source → `grid.background`, grid
/// line → `grid.border`, sector line → `sector.seam`, selection fill →
/// `selection.background`, and so on).
///
/// The **Source glyph hues do not**. A review found that palette's Function
/// green and Bang red at near-identical relative luminance, and a
/// colour-vision measurement of the whole definition
/// (`contrast::distinguish`) then found worse: its Diagnostic and
/// Output Portal measured 0.70 apart under protanopia and its Note and
/// Number 2.16 under deuteranopia, against a floor of 5.0 — pairs a
/// red–green colour-blind reader reads as one colour, and pairs
/// [`okabe_ito`] keeps apart by construction. The user's 2026-09-23 decision
/// re-picks every glyph hue from the same Okabe–Ito palette and the same
/// role-to-hue assignment the dark built-in already uses (Number sky blue,
/// Note yellow, Function bluish green, Bang reddish purple, Sequence blue,
/// Diagnostic vermillion, Output Portal orange), keeping each hue exactly —
/// the OKLCh hue angle is held to within 0.7° — and moving lightness alone,
/// as far as a near-white ground requires and, for Diagnostic and Output
/// Portal, as far as `contrast::CONFUSION_FLOOR` requires on top of that.
/// Sequence is `#0072B2` unchanged: no reachable state draws a glyph in it,
/// so only its tint reaches the screen. `console/src/theme.md` records each
/// value, its hue and lightness, the measured ratios and the colour-vision
/// separations. The user accepted that definition on 2026-09-23. It is the
/// light Theme selection's default and fallback (`crate::theme_selection`),
/// which `crate::style::install` registers for `egui::Theme::Light`.
///
/// `orcvs_light_defines_every_key_at_the_recorded_values` in this module's
/// tests pins every field against that record, and
/// `style::tests::orcvs_light_chrome_matches_the_decided_record` cross-checks
/// the chrome keys through [`crate::style::style`].
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
        source_sequence: straight_rgba(0x00_72_B2_FF),
        source_ordinary_background: straight_rgba(0x00_00_00_00),
        source_comment_background: straight_rgba(0x00_00_00_00),
        source_number_background: straight_rgba(0x00_6D_9B_1A),
        source_note_background: straight_rgba(0x70_69_00_1A),
        source_function_background: straight_rgba(0x00_75_55_1A),
        source_bang_background: straight_rgba(0x00_00_00_00),
        source_atom_background: straight_rgba(0x30_3F_3B_1A),
        source_sequence_background: straight_rgba(0x00_72_B2_1A),

        diagnostic_foreground: straight_rgba(0x65_28_00_FF),
        diagnostic_background: straight_rgba(0x00_00_00_00),
        diagnostic_border: straight_rgba(0x00_00_00_00),
        output_portal_foreground: straight_rgba(0x6F_4A_00_FF),
        output_portal_background: straight_rgba(0x6F_4A_00_1A),
        output_portal_border: straight_rgba(0x00_00_00_00),

        grid_border: straight_rgba(0x34_5B_50_40),
        sector_seam: straight_rgba(0x26_68_54_70),
        cursor_border: straight_rgba(0x30_3F_3B_FF),
        region_border: straight_rgba(0x30_3F_3B_FF),
        cursor_area: straight_rgba(0x14_82_60_FF),
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
/// text — [`crate::theme_document::decode`] builds one from a TOML, JSON
/// or YAML document's bytes; tests in this module construct it directly.
///
#[cfg_attr(
    all(target_arch = "wasm32", not(test)),
    expect(
        dead_code,
        reason = "only native discovery loads a Theme document; the web has the built-ins alone"
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
    all(target_arch = "wasm32", not(test)),
    expect(
        dead_code,
        reason = "only native discovery loads a Theme document; the web has the built-ins alone"
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
    ReservedIdentity { identity: ThemeIdentity },
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
    /// `window.background`'s resolved alpha is not 255. `schema.md`: "The
    /// opaque root is `window.background`; alpha other than 255 on this
    /// property is an error."
    NonOpaqueWindowBackground { alpha: u8 },
}

impl std::fmt::Display for ThemeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownParent { parent } => {
                write!(f, "unknown parent Theme identity {parent:?}")
            }
            Self::ReservedIdentity { identity } => {
                write!(f, "\"{identity}\" is a reserved built-in Theme identity")
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
            Self::NonOpaqueWindowBackground { alpha } => write!(
                f,
                "window.background has alpha {alpha}, not 255: the application window must stay opaque"
            ),
        }
    }
}

#[cfg_attr(
    all(target_arch = "wasm32", not(test)),
    expect(
        dead_code,
        reason = "only native discovery loads a Theme document; the web has the built-ins alone"
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
/// holds — the crate-private `theme_registry` module reads native and web
/// documents and calls this; tests pass `&[okabe_ito()]` directly. `identity` is supplied
/// by the caller rather than read from `document`, because a custom
/// document's identity is its filename stem (ADR 0053), which this module
/// never reads a file to learn.
///
#[cfg_attr(
    all(target_arch = "wasm32", not(test)),
    expect(
        dead_code,
        reason = "only native discovery loads a Theme document; the web has the built-ins alone"
    )
)]
pub(crate) fn resolve(
    built_ins: &[Theme],
    identity: &ThemeIdentity,
    document: &ThemeDocument,
) -> Result<Theme, ThemeError> {
    if identity.is_reserved() {
        return Err(ThemeError::ReservedIdentity {
            identity: identity.clone(),
        });
    }

    let parent = built_ins
        .iter()
        .find(|built_in| built_in.identity.as_str() == document.parent)
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
    resolved.identity = identity.clone();
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

    // The opaque root check runs on the *resolved* value, not merely a
    // supplied override: an inherited `window.background` is already opaque
    // (every built-in sets it that way), so this only ever refuses a
    // document that explicitly overrides it away from 255.
    if resolved.window_background.a() != 255 {
        return Err(ThemeError::NonOpaqueWindowBackground {
            alpha: resolved.window_background.a(),
        });
    }

    Ok(resolved)
}

#[cfg(test)]
mod tests {
    use egui::Color32;

    use super::{
        Appearance, ChromeWidth, ChromeWidthKey, ColorKey, GridWidth, GridWidthKey, OptionalFill,
        ThemeDocument, ThemeError, ThemeIdentity, okabe_ito, orcvs_light, resolve, straight_rgba,
    };

    /// A custom Theme's identity, as a file named `stem` supplies it.
    fn identity(stem: &str) -> ThemeIdentity {
        ThemeIdentity::from_stem(stem).expect("a non-empty stem")
    }

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
        assert_eq!(theme.source_sequence, straight_rgba(0x00_72_B2_FF));
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
        assert_eq!(theme.source_atom_background, straight_rgba(0xEA_EB_E5_1A));
        assert_eq!(
            theme.source_sequence_background,
            straight_rgba(0x00_72_B2_1A)
        );

        assert_eq!(theme.diagnostic_foreground, straight_rgba(0xD5_5E_00_FF));
        assert_eq!(theme.diagnostic_background, straight_rgba(0x00_00_00_00));
        assert_eq!(theme.diagnostic_border, straight_rgba(0x00_00_00_00));
        assert_eq!(theme.output_portal_foreground, straight_rgba(0xE6_9F_00_FF));
        assert_eq!(theme.output_portal_background, straight_rgba(0xE6_9F_00_1A));
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
    /// `.scratch/theming/issues/04`'s 2026-09-23 decision, checked rather
    /// than recorded: every Source glyph hue Orcvs Light draws is
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
            ("Sequence", dark.source_sequence, light.source_sequence),
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
    /// The light counterpart of the test above: `console/src/theme.md`'s
    /// Orcvs Light table, restated as `Theme` field assertions. Every one of
    /// `schema.md`'s named properties appears here at an exact value —
    /// including the ones Orcvs Light shares with Okabe–Ito, which
    /// `.scratch/theming/issues/04` requires be recorded explicitly rather
    /// than left to a toolkit default, and including the border-width keys
    /// ADR 0053 added.
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
        assert_eq!(theme.source_sequence, straight_rgba(0x00_72_B2_FF));
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
        assert_eq!(theme.source_atom_background, straight_rgba(0x30_3F_3B_1A));
        assert_eq!(
            theme.source_sequence_background,
            straight_rgba(0x00_72_B2_1A)
        );

        assert_eq!(theme.diagnostic_foreground, straight_rgba(0x65_28_00_FF));
        assert_eq!(theme.diagnostic_background, straight_rgba(0x00_00_00_00));
        assert_eq!(theme.diagnostic_border, straight_rgba(0x00_00_00_00));
        assert_eq!(theme.output_portal_foreground, straight_rgba(0x6F_4A_00_FF));
        assert_eq!(theme.output_portal_background, straight_rgba(0x6F_4A_00_1A));
        assert_eq!(theme.output_portal_border, straight_rgba(0x00_00_00_00));

        assert_eq!(theme.grid_border, straight_rgba(0x34_5B_50_40));
        assert_eq!(theme.sector_seam, straight_rgba(0x26_68_54_70));
        assert_eq!(theme.cursor_border, straight_rgba(0x30_3F_3B_FF));
        assert_eq!(theme.region_border, straight_rgba(0x30_3F_3B_FF));
        assert_eq!(theme.cursor_area, straight_rgba(0x14_82_60_FF));
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

        // Every width is shared with Okabe–Ito and recorded anyway — ADR
        // 0053's border-width keys, at the same display-point defaults.
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
    /// The two built-ins are genuinely different Themes, not one palette
    /// under two identities: they disagree on the page, the Source ground
    /// and every glyph colour a near-white ground forced them to disagree
    /// on, and they declare opposite appearances.
    ///
    /// `source.sequence` is deliberately absent from the list below, and is
    /// asserted *equal* instead. The user's 2026-09-23 decision re-picks
    /// Orcvs Light's glyph hues from the same Okabe–Ito palette, darkening
    /// each only as far as the near-white ground requires; no reachable state
    /// draws a glyph in Okabe–Ito's blue `#0072B2`, so no floor asked for any
    /// darkening and the two built-ins share it exactly. That
    /// shared value is the rule working, not the two Themes collapsing into
    /// one — every other pair below still differs.
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

        assert_eq!(
            dark.source_sequence, light.source_sequence,
            "source.sequence is the one glyph colour the near-white ground did not force to \
             move; if this ever differs, theme.md's Orcvs Light table has drifted from its \
             own 'unchanged' provenance"
        );
    }

    ///
    /// A custom document inheriting the light built-in resolves to light —
    /// the appearance rule ADR 0053 states ("A light custom Theme must start
    /// from a light built-in"), now exercised against a real light parent
    /// rather than only the synthetic mismatch below.
    ///
    #[test]
    fn a_document_inheriting_the_light_built_in_resolves_light() {
        let built_ins = [okabe_ito(), orcvs_light()];
        let mut document = child("orcvs-light");
        document.appearance = Some(Appearance::Light);

        let resolved = resolve(&built_ins, &identity("mine"), &document)
            .expect("light parent, light document");

        assert_eq!(resolved.appearance, Appearance::Light);
        assert_eq!(resolved.panel_background, orcvs_light().panel_background);

        let mut mismatched = child("orcvs-light");
        mismatched.appearance = Some(Appearance::Dark);
        assert_eq!(
            resolve(&built_ins, &identity("mine"), &mismatched),
            Err(ThemeError::AppearanceMismatch {
                parent: Appearance::Light,
                declared: Appearance::Dark,
            })
        );
    }

    ///
    /// Both built-in identities are reserved, not only Okabe–Ito's.
    ///
    #[test]
    fn the_light_built_in_identity_is_reserved() {
        let built_ins = [okabe_ito(), orcvs_light()];
        assert_eq!(
            resolve(&built_ins, &identity("orcvs-light"), &child("okabe-ito")),
            Err(ThemeError::ReservedIdentity {
                identity: identity("orcvs-light"),
            })
        );
    }

    #[test]
    fn omitted_properties_inherit_from_the_parent() {
        let built_ins = [okabe_ito()];
        let document = child("okabe-ito");

        let resolved =
            resolve(&built_ins, &identity("my-dark"), &document).expect("valid document");

        assert_eq!(resolved.identity.as_str(), "my-dark");
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

        let resolved =
            resolve(&built_ins, &identity("my-dark"), &document).expect("valid document");

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

        let resolved =
            resolve(&built_ins, &identity("my-dark"), &document).expect("valid document");

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

        let resolved =
            resolve(&built_ins, &identity("my-dark"), &document).expect("valid document");

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

        let error = resolve(&built_ins, &identity("my-dark"), &document).unwrap_err();

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
        let custom =
            resolve(&built_ins, &identity("my-dark"), &grandparent).expect("valid document");
        // `custom` is deliberately not added to `built_ins`.
        let chained = child("my-dark");

        let error = resolve(&built_ins, &identity("my-darker"), &chained).unwrap_err();

        assert_eq!(
            error,
            ThemeError::UnknownParent {
                parent: "my-dark".to_owned()
            }
        );
        assert_eq!(custom.identity.as_str(), "my-dark");
    }

    #[test]
    fn reserved_identity_is_rejected() {
        let built_ins = [okabe_ito()];
        let document = child("okabe-ito");

        let error = resolve(&built_ins, &identity("okabe-ito"), &document).unwrap_err();

        assert_eq!(
            error,
            ThemeError::ReservedIdentity {
                identity: identity("okabe-ito")
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

        let error = resolve(&built_ins, &identity("my-dark"), &document).unwrap_err();

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

        let resolved =
            resolve(&built_ins, &identity("my-dark"), &document).expect("matching appearance");

        assert_eq!(resolved.appearance, Appearance::Dark);
    }

    ///
    /// `.scratch/theming/schema.md`: "The opaque root is `window.background`;
    /// alpha other than 255 on this property is an error." Nothing composites
    /// over the OS window itself, so a translucent `window.background` would
    /// leak the desktop through the console rather than through a Panel or
    /// Grid surface, which `.scratch/theming/issues/03`'s own chrome mapping
    /// keeps distinct (`window.background` for the opaque backdrop,
    /// `panel.background`/`grid.background` for the surfaces that may carry
    /// alpha). Both a partial alpha and a fully transparent one (`0`) are
    /// refused the same way — the rule names one exact value, not merely
    /// "some minimum visibility."
    ///
    #[test]
    fn resolve_rejects_a_nonopaque_window_background() {
        let built_ins = [okabe_ito()];

        for (window_background, alpha) in [
            (Color32::from_rgba_unmultiplied(10, 20, 30, 200), 200),
            (Color32::TRANSPARENT, 0),
        ] {
            let document = ThemeDocument {
                colors: vec![(ColorKey::WindowBackground, window_background)],
                ..child("okabe-ito")
            };

            let error = resolve(&built_ins, &identity("my-dark"), &document).unwrap_err();

            assert_eq!(error, ThemeError::NonOpaqueWindowBackground { alpha });
        }
    }

    ///
    /// An opaque override (alpha 255) is accepted like any other colour —
    /// the check only ever refuses, never silently forces opacity.
    ///
    #[test]
    fn resolve_accepts_an_opaque_window_background() {
        let built_ins = [okabe_ito()];
        let opaque = Color32::from_rgb(10, 20, 30);
        let document = ThemeDocument {
            colors: vec![(ColorKey::WindowBackground, opaque)],
            ..child("okabe-ito")
        };

        let resolved =
            resolve(&built_ins, &identity("my-dark"), &document).expect("opaque window.background");

        assert_eq!(resolved.window_background, opaque);
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

        let error = resolve(&built_ins, &identity("my-dark"), &document).unwrap_err();

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

        let error = resolve(&built_ins, &identity("my-dark"), &document).unwrap_err();

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

        let resolved =
            resolve(&built_ins, &identity("my-dark"), &document).expect("valid document");

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

        let resolved =
            resolve(&built_ins, &identity("my-dark"), &document).expect("valid document");

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

        let resolved =
            resolve(&built_ins, &identity("my-dark"), &document).expect("valid document");

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

        let resolved =
            resolve(&built_ins, &identity("my-dark"), &document).expect("valid document");

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

        let resolved =
            resolve(&built_ins, &identity("my-dark"), &document).expect("valid document");

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

            let resolved =
                resolve(&built_ins, &identity("my-dark"), &document).expect("valid document");

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
            let resolved =
                resolve(&built_ins, &identity("my-dark"), &document).expect("valid document");
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
            let resolved =
                resolve(&built_ins, &identity("my-dark"), &document).expect("valid document");
            assert_eq!(
                resolved.chrome_width(key).points(),
                1.9,
                "{key:?} did not take its override"
            );
        }
    }

    ///
    /// `schema.md`'s colour table, restated. The decoder maps document keys
    /// through `ColorKey::from_name`, so a spelling wrong here is a
    /// property no document can set — or, swapped with a neighbour, one that
    /// sets the wrong field. Each spelling also has to come back as its key.
    ///
    #[test]
    fn color_key_names_match_the_schema_catalogue() {
        let catalogue = [
            (ColorKey::WindowBackground, "window.background"),
            (ColorKey::PanelBackground, "panel.background"),
            (ColorKey::GridBackground, "grid.background"),
            (ColorKey::CellBackground, "cell.background"),
            (ColorKey::SourceOrdinary, "source.ordinary"),
            (ColorKey::SourceComment, "source.comment"),
            (ColorKey::SourceNumber, "source.number"),
            (ColorKey::SourceNote, "source.note"),
            (ColorKey::SourceFunction, "source.function"),
            (ColorKey::SourceBang, "source.bang"),
            (ColorKey::SourceSequence, "source.sequence"),
            (
                ColorKey::SourceOrdinaryBackground,
                "source.ordinary.background",
            ),
            (
                ColorKey::SourceCommentBackground,
                "source.comment.background",
            ),
            (ColorKey::SourceNumberBackground, "source.number.background"),
            (ColorKey::SourceNoteBackground, "source.note.background"),
            (
                ColorKey::SourceFunctionBackground,
                "source.function.background",
            ),
            (ColorKey::SourceBangBackground, "source.bang.background"),
            (ColorKey::SourceAtomBackground, "source.atom.background"),
            (
                ColorKey::SourceSequenceBackground,
                "source.sequence.background",
            ),
            (ColorKey::DiagnosticForeground, "diagnostic.foreground"),
            (ColorKey::DiagnosticBackground, "diagnostic.background"),
            (ColorKey::DiagnosticBorder, "diagnostic.border"),
            (ColorKey::OutputPortalForeground, "output_portal.foreground"),
            (ColorKey::OutputPortalBackground, "output_portal.background"),
            (ColorKey::OutputPortalBorder, "output_portal.border"),
            (ColorKey::GridBorder, "grid.border"),
            (ColorKey::SectorSeam, "sector.seam"),
            (ColorKey::CursorBorder, "cursor.border"),
            (ColorKey::RegionBorder, "region.border"),
            (ColorKey::CursorArea, "cursor.area"),
            (ColorKey::RegionBackground, "region.background"),
            (ColorKey::PanelBorder, "panel.border"),
            (ColorKey::SelectionBackground, "selection.background"),
            (ColorKey::SelectionBorder, "selection.border"),
            (ColorKey::SelectionBorderRest, "selection.border.rest"),
            (ColorKey::WidgetInactiveBorder, "widget.inactive.border"),
            (ColorKey::Text, "text"),
            (ColorKey::TextActive, "text.active"),
            (ColorKey::TextMuted, "text.muted"),
            (ColorKey::InputBackground, "input.background"),
            (ColorKey::Link, "link"),
            (ColorKey::CodeBackground, "code.background"),
            (ColorKey::InputCursor, "input.cursor"),
            (ColorKey::Error, "error"),
            (ColorKey::Warning, "warning"),
        ];
        assert_eq!(catalogue.len(), 45);
        for (key, name) in catalogue {
            assert_eq!(key.name(), name);
            assert_eq!(ColorKey::from_name(name), Some(key));
        }
        assert_eq!(ColorKey::from_name("Text"), None);
        assert_eq!(ColorKey::from_name("cursor.background"), None);
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
