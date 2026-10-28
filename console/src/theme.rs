//!
//! The resolved Orcvs Theme: named colour, width and optional-colour
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
    /// [`grayscale_light`] declares this, and a custom document inherits it from
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
/// A chrome (panel/widget/selection/input-caret/Overview outline) stroke
/// width in display points: 0 to 2 inclusive (`schema.md`'s width catalogue).
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
    /// underline widths, and by the Source View's outline on the Overview.
    pub(crate) fn points(self) -> f32 {
        self.0
    }
}

// === Property keys ===

///
/// The `style` property catalogue, written once: the optional Cursor
/// colours, which [`crate::theme_document::OptionalColor`] carries and no key enum holds, then each
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
            OptionalColor {
                CursorBackground(cursor_background) => "cursor.background",
                CursorForeground(cursor_foreground) => "cursor.foreground",
                RegionCursorBackground(region_cursor_background) => "region.cursor.background",
                RegionCursorForeground(region_cursor_foreground) => "region.cursor.foreground",
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
                SourceOrdinaryBorder(source_ordinary_border) => "source.ordinary.border",
                SourceCommentBorder(source_comment_border) => "source.comment.border",
                SourceNumberBorder(source_number_border) => "source.number.border",
                SourceNoteBorder(source_note_border) => "source.note.border",
                SourceFunctionBorder(source_function_border) => "source.function.border",
                SourceBangBorder(source_bang_border) => "source.bang.border",
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
                PanelSeparator(panel_separator) => "panel.separator",
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
                OverviewSourceView(overview_source_view) => "overview.source_view",
                ButtonForeground(button_foreground) => "button.foreground",
                ButtonBackground(button_background) => "button.background",
                ButtonBorder(button_border) => "button.border",
                ButtonPressedForeground(button_pressed_foreground) => "button.pressed.foreground",
                ButtonPressedBackground(button_pressed_background) => "button.pressed.background",
                ButtonPressedBorder(button_pressed_border) => "button.pressed.border",
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
                SourceOrdinaryBorder(source_ordinary_border_width) => "source.ordinary.border.width",
                SourceCommentBorder(source_comment_border_width) => "source.comment.border.width",
                SourceNumberBorder(source_number_border_width) => "source.number.border.width",
                SourceNoteBorder(source_note_border_width) => "source.note.border.width",
                SourceFunctionBorder(source_function_border_width) => "source.function.border.width",
                SourceBangBorder(source_bang_border_width) => "source.bang.border.width",
                ButtonBorder(button_border_width) => "button.border.width",
            }
            ChromeWidthKey {
                PanelBorder(panel_border_width) => "panel.border.width",
                PanelSeparator(panel_separator_width) => "panel.separator.width",
                SelectionBorder(selection_border_width) => "selection.border.width",
                WidgetBorder(widget_border_width) => "widget.border.width",
                WidgetInactiveBorder(widget_inactive_border_width) => "widget.inactive.border.width",
                InputCursor(input_cursor_width) => "input.cursor.width",
                OverviewSourceView(overview_source_view_width) => "overview.source_view.width",
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
/// out explicitly (see [`grayscale_dark`]), so the compiler — not a runtime
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
    pub(crate) source_ordinary_border: Color32,
    pub(crate) source_comment_border: Color32,
    pub(crate) source_number_border: Color32,
    pub(crate) source_note_border: Color32,
    pub(crate) source_function_border: Color32,
    pub(crate) source_bang_border: Color32,

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
    /// The line along each bar's edge that faces the Source, and the
    /// Overview's edges that do.
    pub(crate) panel_separator: Color32,
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
    /// The Source View's outline on the Overview.
    pub(crate) overview_source_view: Color32,
    /// The Panel's Cell buttons, P and R: the glyph, fill and border of one
    /// at rest, and of one pressed, as P is while Playback plays.
    pub(crate) button_foreground: Color32,
    pub(crate) button_background: Color32,
    pub(crate) button_border: Color32,
    pub(crate) button_pressed_foreground: Color32,
    pub(crate) button_pressed_background: Color32,
    pub(crate) button_pressed_border: Color32,

    /// Omitted in a document: inherit the parent's value. `None` here: the
    /// optional colour is cleared, enabling its fallback. `Some`: an
    /// explicit colour, including transparent. See
    /// [`crate::theme_document::OptionalColor`].
    pub(crate) cursor_background: Option<Color32>,
    /// The glyph colour on the single-Cell Cursor; `None` leaves the glyph
    /// its role's colour.
    pub(crate) cursor_foreground: Option<Color32>,
    pub(crate) region_cursor_background: Option<Color32>,
    pub(crate) region_cursor_foreground: Option<Color32>,

    pub(crate) grid_border_width: GridWidth,
    pub(crate) sector_seam_width: GridWidth,
    pub(crate) cell_selection_border_width: GridWidth,
    pub(crate) cursor_border_width: GridWidth,
    pub(crate) region_border_width: GridWidth,
    pub(crate) diagnostic_border_width: GridWidth,
    pub(crate) output_portal_border_width: GridWidth,
    pub(crate) input_portal_border_width: GridWidth,
    pub(crate) locked_border_width: GridWidth,
    pub(crate) source_ordinary_border_width: GridWidth,
    pub(crate) source_comment_border_width: GridWidth,
    pub(crate) source_number_border_width: GridWidth,
    pub(crate) source_note_border_width: GridWidth,
    pub(crate) source_function_border_width: GridWidth,
    pub(crate) source_bang_border_width: GridWidth,
    pub(crate) button_border_width: GridWidth,

    pub(crate) panel_border_width: ChromeWidth,
    pub(crate) panel_separator_width: ChromeWidth,
    pub(crate) selection_border_width: ChromeWidth,
    pub(crate) widget_border_width: ChromeWidth,
    pub(crate) widget_inactive_border_width: ChromeWidth,
    pub(crate) input_cursor_width: ChromeWidth,
    pub(crate) overview_source_view_width: ChromeWidth,
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

    ///
    /// The glyph colour on the Cursor's Cell while a Region spans more than
    /// one Cell, falling back as [`Self::region_cursor_fill`] does:
    /// `region.cursor.foreground` when it holds a colour, transparent
    /// included, and `cursor.foreground` only where it is cleared. `None`
    /// leaves the glyph its role colour. Painting and the contrast check both
    /// read it here.
    ///
    pub(crate) fn region_cursor_glyph(&self) -> Option<Color32> {
        self.region_cursor_foreground.or(self.cursor_foreground)
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
    /// The identity a setting names, as written in `config.toml`. Unchecked:
    /// `SelectedThemes` reports one no Theme of its appearance answers to.
    ///
    pub(crate) fn configured(identity: String) -> Self {
        Self(Cow::Owned(identity))
    }

    ///
    /// The built-in each appearance starts with and falls back to when its
    /// selection names no available Theme of that appearance.
    ///
    pub(crate) const fn default_for(appearance: Appearance) -> Self {
        match appearance {
            Appearance::Dark => GRAYSCALE_DARK_IDENTITY,
            Appearance::Light => GRAYSCALE_LIGHT_IDENTITY,
        }
    }
}

// What only loading a Theme document asks of an identity; the web loads none.
#[cfg(any(not(target_arch = "wasm32"), test))]
impl ThemeIdentity {
    ///
    /// A custom Theme's identity: its file name's stem. `None` for an empty
    /// stem, which names nothing.
    ///
    pub(crate) fn from_stem(stem: &str) -> Option<Self> {
        (!stem.is_empty()).then(|| Self(Cow::Owned(stem.to_owned())))
    }

    pub(crate) fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for ThemeIdentity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// The reserved identity of the grayscale dark built-in, and the dark Theme
/// selection's default and fallback.
pub(crate) const GRAYSCALE_DARK_IDENTITY: ThemeIdentity = ThemeIdentity::reserved("grayscale-dark");

///
/// The Grayscale Dark built-in Theme: light grey text and Cells on black, at
/// the reserved identity `GRAYSCALE_DARK_IDENTITY`. Source roles are told
/// apart by grey level and by filled and bordered Number, Note, Function and
/// Bang Cells; hue marks only a green Bang and a red Diagnostic glyph. Both
/// Portals fill their Cells light grey under a dark glyph, and the Region
/// fills opaque. Every field is spelled out explicitly (see [`Theme`]).
///
/// This is the dark Theme selection's default and fallback
/// (`crate::theme_selection`). `console/src/theme.md` records its values.
///
pub fn grayscale_dark() -> Theme {
    Theme {
        identity: GRAYSCALE_DARK_IDENTITY,
        name: "Grayscale Dark".to_owned(),
        appearance: Appearance::Dark,

        window_background: straight_rgba(0x00_00_00_FF),
        panel_background: straight_rgba(0x00_00_00_FF),
        grid_background: straight_rgba(0x00_00_00_FF),
        cell_background: straight_rgba(0x00_00_00_00),

        source_ordinary: straight_rgba(0xC8_C8_C8_FF),
        source_comment: straight_rgba(0x55_55_55_FF),
        source_number: straight_rgba(0x89_89_89_FF),
        source_note: straight_rgba(0xCC_CC_CC_FF),
        source_function: straight_rgba(0xE6_E6_E6_FF),
        source_bang: straight_rgba(0x00_FF_00_FF),
        source_ordinary_background: straight_rgba(0x00_00_00_00),
        source_comment_background: straight_rgba(0x00_00_00_00),
        source_number_background: straight_rgba(0x22_22_22_FF),
        source_note_background: straight_rgba(0x22_22_22_FF),
        source_function_background: straight_rgba(0x44_44_44_FF),
        source_bang_background: straight_rgba(0x11_11_11_FF),
        source_ordinary_border: straight_rgba(0x00_00_00_00),
        source_comment_border: straight_rgba(0x00_00_00_00),
        source_number_border: straight_rgba(0x22_22_22_FF),
        source_note_border: straight_rgba(0x22_22_22_FF),
        source_function_border: straight_rgba(0x44_44_44_FF),
        source_bang_border: straight_rgba(0x00_FF_00_FF),

        diagnostic_foreground: straight_rgba(0xFF_3B_3B_FF),
        diagnostic_background: straight_rgba(0x00_00_00_00),
        diagnostic_border: straight_rgba(0xC8_C8_C8_FF),
        output_portal_foreground: straight_rgba(0x33_33_33_FF),
        output_portal_background: straight_rgba(0xCC_CC_CC_FF),
        output_portal_border: straight_rgba(0xFF_FF_FF_66),
        input_portal_foreground: straight_rgba(0x33_33_33_FF),
        input_portal_background: straight_rgba(0xCC_CC_CC_FF),
        input_portal_border: straight_rgba(0xFF_FF_FF_66),
        locked_foreground: straight_rgba(0x00_00_00_00),
        locked_background: straight_rgba(0x00_00_00_FF),
        locked_border: straight_rgba(0xCC_CC_CC_FF),

        grid_border: straight_rgba(0xFF_FF_FF_1A),
        sector_seam: straight_rgba(0xFF_FF_FF_33),
        cursor_border: straight_rgba(0xE6_E6_E6_FF),
        region_border: straight_rgba(0xDD_DD_DD_FF),
        region_background: straight_rgba(0x38_38_38_FF),

        panel_border: straight_rgba(0x2A_2A_2A_FF),
        panel_separator: straight_rgba(0x55_55_55_FF),
        selection_background: straight_rgba(0x33_33_33_FF),
        selection_border: straight_rgba(0xE6_E6_E6_FF),
        selection_border_rest: straight_rgba(0xA8_A8_A8_FF),
        widget_inactive_border: straight_rgba(0x2A_2A_2A_FF),

        text: straight_rgba(0xC8_C8_C8_FF),
        text_active: straight_rgba(0xE6_E6_E6_FF),
        text_muted: straight_rgba(0xC8_C8_C8_99),
        input_background: straight_rgba(0x00_00_00_FF),
        link: straight_rgba(0xE6_E6_E6_FF),
        code_background: straight_rgba(0x2A_2A_2A_FF),
        input_cursor: straight_rgba(0xE6_E6_E6_FF),
        error: straight_rgba(0xE6_E6_E6_FF),
        warning: straight_rgba(0xE6_E6_E6_FF),
        overview_source_view: straight_rgba(0xFF_FF_FF_FF),
        button_foreground: straight_rgba(0xE6_E6_E6_FF),
        button_background: straight_rgba(0x44_44_44_FF),
        button_border: straight_rgba(0x44_44_44_FF),
        button_pressed_foreground: straight_rgba(0x00_00_00_FF),
        button_pressed_background: straight_rgba(0x00_FF_00_FF),
        button_pressed_border: straight_rgba(0x00_FF_00_FF),

        cursor_background: None,
        cursor_foreground: Some(straight_rgba(0xE6_E6_E6_FF)),
        region_cursor_background: Some(straight_rgba(0x33_33_33_FF)),
        region_cursor_foreground: None,

        grid_border_width: GridWidth::from_points(0.5).expect("0.5 is within 0..=1"),
        sector_seam_width: GridWidth::from_points(0.75).expect("0.75 is within 0..=1"),
        cell_selection_border_width: GridWidth::from_points(1.0).expect("1.0 is within 0..=1"),
        cursor_border_width: GridWidth::from_points(1.0).expect("1.0 is within 0..=1"),
        region_border_width: GridWidth::from_points(1.0).expect("1.0 is within 0..=1"),
        diagnostic_border_width: GridWidth::from_points(0.0).expect("0.0 is within 0..=1"),
        output_portal_border_width: GridWidth::from_points(0.0).expect("0.0 is within 0..=1"),
        input_portal_border_width: GridWidth::from_points(0.5).expect("0.5 is within 0..=1"),
        locked_border_width: GridWidth::from_points(1.0).expect("1.0 is within 0..=1"),
        source_ordinary_border_width: GridWidth::from_points(0.5).expect("0.5 is within 0..=1"),
        source_comment_border_width: GridWidth::from_points(0.5).expect("0.5 is within 0..=1"),
        source_number_border_width: GridWidth::from_points(1.0).expect("1.0 is within 0..=1"),
        source_note_border_width: GridWidth::from_points(1.0).expect("1.0 is within 0..=1"),
        source_function_border_width: GridWidth::from_points(1.0).expect("1.0 is within 0..=1"),
        source_bang_border_width: GridWidth::from_points(0.5).expect("0.5 is within 0..=1"),
        button_border_width: GridWidth::from_points(1.0).expect("1.0 is within 0..=1"),

        panel_border_width: ChromeWidth::from_points(1.0).expect("1.0 is within 0..=2"),
        panel_separator_width: ChromeWidth::from_points(2.0).expect("2.0 is within 0..=2"),
        selection_border_width: ChromeWidth::from_points(1.0).expect("1.0 is within 0..=2"),
        widget_border_width: ChromeWidth::from_points(1.0).expect("1.0 is within 0..=2"),
        widget_inactive_border_width: ChromeWidth::from_points(0.0).expect("0.0 is within 0..=2"),
        input_cursor_width: ChromeWidth::from_points(2.0).expect("2.0 is within 0..=2"),
        overview_source_view_width: ChromeWidth::from_points(1.0).expect("1.0 is within 0..=2"),
    }
}

/// The reserved identity of the grayscale light built-in, and the light
/// Theme selection's default and fallback.
pub(crate) const GRAYSCALE_LIGHT_IDENTITY: ThemeIdentity =
    ThemeIdentity::reserved("grayscale-light");

///
/// The Grayscale Light built-in Theme: dark grey text and Cells on white, at
/// the reserved identity `GRAYSCALE_LIGHT_IDENTITY`, with [`grayscale_dark`]'s
/// filled and bordered Source roles. Hue marks only a magenta Bang and a red,
/// red-framed Diagnostic. Both Portals draw a light glyph on a mid-grey fill,
/// and the Region fills opaque. Every field is spelled out explicitly (see
/// [`Theme`]).
///
/// This is the light Theme selection's default and fallback
/// (`crate::theme_selection`). `console/src/theme.md` records its values.
///
pub fn grayscale_light() -> Theme {
    Theme {
        identity: GRAYSCALE_LIGHT_IDENTITY,
        name: "Grayscale Light".to_owned(),
        appearance: Appearance::Light,

        window_background: straight_rgba(0xFF_FF_FF_FF),
        panel_background: straight_rgba(0xFF_FF_FF_FF),
        grid_background: straight_rgba(0xFF_FF_FF_FF),
        cell_background: straight_rgba(0x00_00_00_00),

        source_ordinary: straight_rgba(0x37_37_37_FF),
        source_comment: straight_rgba(0xAD_AD_AD_FF),
        source_number: straight_rgba(0x61_61_61_FF),
        source_note: straight_rgba(0x33_33_33_FF),
        source_function: straight_rgba(0x19_19_19_FF),
        source_bang: straight_rgba(0xD1_00_D1_FF),
        source_ordinary_background: straight_rgba(0x00_00_00_00),
        source_comment_background: straight_rgba(0x00_00_00_00),
        source_number_background: straight_rgba(0xDD_DD_DD_FF),
        source_note_background: straight_rgba(0xDD_DD_DD_FF),
        source_function_background: straight_rgba(0xBB_BB_BB_FF),
        source_bang_background: straight_rgba(0xFF_FF_FF_FF),
        source_ordinary_border: straight_rgba(0x00_00_00_00),
        source_comment_border: straight_rgba(0x00_00_00_00),
        source_number_border: straight_rgba(0xDD_DD_DD_FF),
        source_note_border: straight_rgba(0xDD_DD_DD_FF),
        source_function_border: straight_rgba(0xBB_BB_BB_FF),
        source_bang_border: straight_rgba(0xFF_00_FF_FF),

        diagnostic_foreground: straight_rgba(0xC7_00_00_FF),
        diagnostic_background: straight_rgba(0xFF_FF_FF_FF),
        diagnostic_border: straight_rgba(0xC7_00_00_FF),
        output_portal_foreground: straight_rgba(0xE4_E4_E4_FF),
        output_portal_background: straight_rgba(0x66_66_66_FF),
        output_portal_border: straight_rgba(0x66_66_66_FF),
        input_portal_foreground: straight_rgba(0xE4_E4_E4_FF),
        input_portal_background: straight_rgba(0x66_66_66_FF),
        input_portal_border: straight_rgba(0x66_66_66_FF),
        locked_foreground: straight_rgba(0x00_00_00_00),
        locked_background: straight_rgba(0xFF_FF_FF_FF),
        locked_border: straight_rgba(0x33_33_33_FF),

        grid_border: straight_rgba(0x00_00_00_1A),
        sector_seam: straight_rgba(0x00_00_00_33),
        cursor_border: straight_rgba(0x19_19_19_FF),
        region_border: straight_rgba(0x19_19_19_FF),
        region_background: straight_rgba(0xE1_E1_E1_FF),

        panel_border: straight_rgba(0xD5_D5_D5_FF),
        panel_separator: straight_rgba(0xAD_AD_AD_FF),
        selection_background: straight_rgba(0xCC_CC_CC_FF),
        selection_border: straight_rgba(0x19_19_19_FF),
        selection_border_rest: straight_rgba(0x57_57_57_FF),
        widget_inactive_border: straight_rgba(0xD5_D5_D5_FF),

        text: straight_rgba(0x37_37_37_FF),
        text_active: straight_rgba(0x19_19_19_FF),
        text_muted: straight_rgba(0x37_37_37_AF),
        input_background: straight_rgba(0xFF_FF_FF_FF),
        link: straight_rgba(0x19_19_19_FF),
        code_background: straight_rgba(0xD5_D5_D5_FF),
        input_cursor: straight_rgba(0x19_19_19_FF),
        error: straight_rgba(0x19_19_19_FF),
        warning: straight_rgba(0x19_19_19_FF),
        overview_source_view: straight_rgba(0x00_00_00_FF),
        button_foreground: straight_rgba(0x19_19_19_FF),
        button_background: straight_rgba(0xBB_BB_BB_FF),
        button_border: straight_rgba(0xBB_BB_BB_FF),
        button_pressed_foreground: straight_rgba(0xFF_FF_FF_FF),
        button_pressed_background: straight_rgba(0xD1_00_D1_FF),
        button_pressed_border: straight_rgba(0xD1_00_D1_FF),

        cursor_background: None,
        cursor_foreground: Some(straight_rgba(0x19_19_19_FF)),
        region_cursor_background: Some(straight_rgba(0xCC_CC_CC_FF)),
        region_cursor_foreground: None,

        grid_border_width: GridWidth::from_points(0.5).expect("0.5 is within 0..=1"),
        sector_seam_width: GridWidth::from_points(0.75).expect("0.75 is within 0..=1"),
        cell_selection_border_width: GridWidth::from_points(0.5).expect("0.5 is within 0..=1"),
        cursor_border_width: GridWidth::from_points(1.0).expect("1.0 is within 0..=1"),
        region_border_width: GridWidth::from_points(1.0).expect("1.0 is within 0..=1"),
        diagnostic_border_width: GridWidth::from_points(1.0).expect("1.0 is within 0..=1"),
        output_portal_border_width: GridWidth::from_points(0.5).expect("0.5 is within 0..=1"),
        input_portal_border_width: GridWidth::from_points(0.5).expect("0.5 is within 0..=1"),
        locked_border_width: GridWidth::from_points(1.0).expect("1.0 is within 0..=1"),
        source_ordinary_border_width: GridWidth::from_points(0.5).expect("0.5 is within 0..=1"),
        source_comment_border_width: GridWidth::from_points(0.5).expect("0.5 is within 0..=1"),
        source_number_border_width: GridWidth::from_points(1.0).expect("1.0 is within 0..=1"),
        source_note_border_width: GridWidth::from_points(1.0).expect("1.0 is within 0..=1"),
        source_function_border_width: GridWidth::from_points(1.0).expect("1.0 is within 0..=1"),
        source_bang_border_width: GridWidth::from_points(1.0).expect("1.0 is within 0..=1"),
        button_border_width: GridWidth::from_points(1.0).expect("1.0 is within 0..=1"),

        panel_border_width: ChromeWidth::from_points(1.0).expect("1.0 is within 0..=2"),
        panel_separator_width: ChromeWidth::from_points(2.0).expect("2.0 is within 0..=2"),
        selection_border_width: ChromeWidth::from_points(1.0).expect("1.0 is within 0..=2"),
        widget_border_width: ChromeWidth::from_points(1.0).expect("1.0 is within 0..=2"),
        widget_inactive_border_width: ChromeWidth::from_points(0.0).expect("0.0 is within 0..=2"),
        input_cursor_width: ChromeWidth::from_points(2.0).expect("2.0 is within 0..=2"),
        overview_source_view_width: ChromeWidth::from_points(1.0).expect("1.0 is within 0..=2"),
    }
}

#[cfg(test)]
pub(crate) mod record {
    use egui::Color32;

    use super::{Theme, straight_rgba};

    ///
    /// The colour a `theme.md` table cell records: `#RRGGBB`, `#RRGGBBAA`,
    /// or `none` for a cleared optional colour.
    ///
    fn recorded_colour(cell: &str) -> Option<Color32> {
        if cell.trim() == "none" {
            return None;
        }
        let hex = cell.trim().trim_matches('`').trim_start_matches('#');
        let rgba = match hex.len() {
            6 => u32::from_str_radix(hex, 16).expect("hex") << 8 | 0xFF,
            8 => u32::from_str_radix(hex, 16).expect("hex"),
            _ => panic!("{cell:?} is not a recorded colour"),
        };
        Some(straight_rgba(rgba))
    }

    /// A value a `theme.md` record table holds for one property.
    #[derive(Debug, PartialEq)]
    enum Recorded {
        Colour(Option<Color32>),
        Width(f32),
    }

    /// Every property a Theme holds, by its `schema.md` name, as a record
    /// table states it: optional colours included where cleared.
    macro_rules! recorded_fields {
        (
            OptionalColor { $($optional:ident($optional_field:ident) => $optional_name:literal),* $(,)? }
            ColorKey { $($colour:ident($colour_field:ident) => $colour_name:literal),* $(,)? }
            GridWidthKey { $($grid:ident($grid_field:ident) => $grid_name:literal),* $(,)? }
            ChromeWidthKey { $($chrome:ident($chrome_field:ident) => $chrome_name:literal),* $(,)? }
        ) => {
            fn recorded_fields(theme: &Theme) -> Vec<(&'static str, Recorded)> {
                let mut fields =
                    vec![$(($optional_name, Recorded::Colour(theme.$optional_field))),*];
                fields.extend([$(($colour_name, Recorded::Colour(Some(theme.$colour_field)))),*]);
                fields.extend([$(($grid_name, Recorded::Width(theme.$grid_field.points()))),*]);
                fields.extend([$(($chrome_name, Recorded::Width(theme.$chrome_field.points()))),*]);
                fields
            }
        };
    }
    super::style_catalogue!(recorded_fields);

    ///
    /// Asserts `console/src/theme.md`'s table for `theme`, under the heading
    /// "The <name> built-in Theme", read from the page itself so the record
    /// and the code cannot drift: every property it names holds the recorded
    /// colour or width, and it names every property exactly once.
    ///
    pub(crate) fn assert_matches_its_table(theme: &Theme) {
        let name = &theme.name;
        let record = include_str!("theme.md");
        let section = record
            .split(&format!("### The {name} built-in Theme"))
            .nth(1)
            .and_then(|rest| rest.split("\n### ").next())
            .unwrap_or_else(|| panic!("theme.md records {name}"));
        let rows = section
            .split("\n\n")
            .find(|block| block.starts_with(&format!("| Property | {name} |")))
            .unwrap_or_else(|| panic!("the {name} property table"));

        let fields = recorded_fields(theme);
        let mut named: Vec<&str> = Vec::new();
        for row in rows.lines().skip(2) {
            let cells: Vec<&str> = row.trim_matches('|').split('|').collect();
            let [properties, cell] = cells[..] else {
                panic!("{row:?} is not a two-column row");
            };
            for property in properties.split('`').skip(1).step_by(2) {
                named.push(property);
                let value = fields
                    .iter()
                    .find(|(field, _)| *field == property)
                    .map(|(_, value)| value)
                    .unwrap_or_else(|| panic!("{property} is not a property"));
                let recorded = match value {
                    Recorded::Colour(_) => Recorded::Colour(recorded_colour(cell)),
                    Recorded::Width(_) => {
                        Recorded::Width(cell.trim().parse().expect("a recorded width"))
                    }
                };
                assert_eq!(*value, recorded, "{name}'s {property}");
            }
        }

        let mut expected: Vec<&str> = fields.iter().map(|&(property, _)| property).collect();
        expected.sort_unstable();
        named.sort_unstable();
        assert_eq!(
            named, expected,
            "theme.md names every property of {name} exactly once"
        );
    }
}

#[cfg(test)]
mod tests {
    use egui::Color32;

    use super::{
        Appearance, ChromeWidth, GridWidth, ThemeIdentity, grayscale_dark, grayscale_light,
    };
    use crate::contrast::colour_vision::seen_apart;
    use crate::theme_registry::ThemeRegistry;
    use crate::theme_registry::tests_support::built_ins;

    ///
    /// Each built-in draws a pressed button in its Bang ink, fill and border
    /// alike; `contrast::validate` holds its glyph to the floor over that.
    ///
    #[test]
    fn a_built_in_pressed_button_is_its_bang_ink() {
        for theme in [grayscale_dark(), grayscale_light()] {
            assert_eq!(
                theme.button_pressed_background, theme.source_bang,
                "{}",
                theme.identity
            );
            assert_eq!(
                theme.button_pressed_border, theme.source_bang,
                "{}",
                theme.identity
            );
        }
    }

    ///
    /// Each appearance's default is a built-in of that appearance.
    ///
    #[test]
    fn each_appearances_default_is_a_built_in_of_that_appearance() {
        let registry = ThemeRegistry::built_in();
        for appearance in [Appearance::Dark, Appearance::Light] {
            let theme = registry.fallback(appearance);
            assert_eq!(theme.appearance, appearance);
            assert_eq!(theme.identity, ThemeIdentity::default_for(appearance));
        }
    }

    ///
    /// Every built-in holds the values its table in `console/src/theme.md`
    /// records.
    ///
    #[test]
    fn every_built_in_defines_every_key_at_its_table() {
        let registry = ThemeRegistry::built_in();
        for theme in built_ins(&registry) {
            super::record::assert_matches_its_table(theme);
        }
    }

    ///
    /// Where two Source roles draw glyphs a viewer cannot tell apart, closer
    /// than the colour-vision floor on the Grid background, the Cell around
    /// them must still differ, in fill or in a border that draws, or the two
    /// meanings read as one. A border draws only while it is visible and
    /// wider than zero. The two Portals may draw alike: both mark a Cell the
    /// Tick reached, and `theme.md` records which built-ins draw them alike.
    ///
    #[test]
    fn roles_whose_glyphs_read_alike_differ_in_fill_or_border() {
        let registry = ThemeRegistry::built_in();
        for theme in built_ins(&registry) {
            let roles = [
                (
                    "Ordinary",
                    theme.source_ordinary,
                    theme.source_ordinary_background,
                    theme.source_ordinary_border,
                    theme.source_ordinary_border_width,
                ),
                (
                    "Comment",
                    theme.source_comment,
                    theme.source_comment_background,
                    theme.source_comment_border,
                    theme.source_comment_border_width,
                ),
                (
                    "Number",
                    theme.source_number,
                    theme.source_number_background,
                    theme.source_number_border,
                    theme.source_number_border_width,
                ),
                (
                    "Note",
                    theme.source_note,
                    theme.source_note_background,
                    theme.source_note_border,
                    theme.source_note_border_width,
                ),
                (
                    "Function",
                    theme.source_function,
                    theme.source_function_background,
                    theme.source_function_border,
                    theme.source_function_border_width,
                ),
                (
                    "Bang",
                    theme.source_bang,
                    theme.source_bang_background,
                    theme.source_bang_border,
                    theme.source_bang_border_width,
                ),
                (
                    "Diagnostic",
                    theme.diagnostic_foreground,
                    theme.diagnostic_background,
                    theme.diagnostic_border,
                    theme.diagnostic_border_width,
                ),
                (
                    "Output Portal",
                    theme.output_portal_foreground,
                    theme.output_portal_background,
                    theme.output_portal_border,
                    theme.output_portal_border_width,
                ),
                (
                    "Input Portal",
                    theme.input_portal_foreground,
                    theme.input_portal_background,
                    theme.input_portal_border,
                    theme.input_portal_border_width,
                ),
            ];
            let drawn = |border: Color32, width: GridWidth| {
                if border.a() != 0 && width.points() > 0.0 {
                    border
                } else {
                    Color32::TRANSPARENT
                }
            };
            let roles = roles.map(|(name, glyph, fill, border, width)| {
                (name, glyph, fill, drawn(border, width))
            });
            for (index, (first, glyph, fill, border)) in roles.iter().enumerate() {
                for (second, other_glyph, other_fill, other_border) in &roles[index + 1..] {
                    let portals = (*first, *second) == ("Output Portal", "Input Portal");
                    let ground = theme.grid_background;
                    if !seen_apart(ground.blend(*glyph), ground.blend(*other_glyph)) && !portals {
                        assert!(
                            (fill, border) != (other_fill, other_border),
                            "{}: {first} and {second} read alike in glyph, fill and border",
                            theme.identity
                        );
                    }
                }
            }
        }
    }

    ///
    /// A dark built-in sets its text and Cells on black and never in white,
    /// and a light one sets them on white.
    ///
    #[test]
    fn dark_built_ins_sit_on_black_and_light_ones_on_white() {
        let registry = ThemeRegistry::built_in();
        for theme in built_ins(&registry) {
            let ground = match theme.appearance {
                Appearance::Dark => Color32::BLACK,
                Appearance::Light => Color32::WHITE,
            };
            assert_eq!(theme.window_background, ground, "{}", theme.identity);
            assert_eq!(theme.grid_background, ground, "{}", theme.identity);
            if theme.appearance == Appearance::Dark {
                for (name, colour) in [
                    ("text", theme.text),
                    ("source.ordinary", theme.source_ordinary),
                    ("source.function", theme.source_function),
                    ("cursor.border", theme.cursor_border),
                ] {
                    assert_ne!(
                        colour,
                        Color32::WHITE,
                        "{}'s {name} is white",
                        theme.identity
                    );
                }
            }
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
                ..ThemeRegistry::built_in().fallback(Appearance::Dark).clone()
            };
            assert_eq!(
                theme.region_cursor_fill(),
                expected,
                "cursor.background {cursor_background:?}, \
                 region.cursor.background {region_cursor_background:?}"
            );
        }
    }

    ///
    /// The glyph on the Cursor inside a spanning Region draws in
    /// `region.cursor.foreground` whenever it holds a colour, transparent
    /// included, and in `cursor.foreground` only where the Region's glyph
    /// colour is cleared. Both cleared leave the glyph its role colour.
    ///
    #[test]
    fn the_region_cursor_glyph_falls_back_to_the_cursor_glyph_only_when_cleared() {
        let cursor = Color32::from_rgb(1, 2, 3);
        let region_cursor = Color32::from_rgb(4, 5, 6);
        for (cursor_foreground, region_cursor_foreground, expected) in [
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
                cursor_foreground,
                region_cursor_foreground,
                ..ThemeRegistry::built_in().fallback(Appearance::Dark).clone()
            };
            assert_eq!(
                theme.region_cursor_glyph(),
                expected,
                "cursor.foreground {cursor_foreground:?}, \
                 region.cursor.foreground {region_cursor_foreground:?}"
            );
        }
    }
}
