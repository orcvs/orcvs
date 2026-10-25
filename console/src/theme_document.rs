//!
//! A custom Theme document, from its bytes to a resolved [`Theme`]:
//! [`decode`] reads one into a [`ThemeDocument`] (`schema.md`'s "Decoding and
//! limits"), and [`resolve`] applies it over a built-in. No file I/O. The web
//! reads no Theme document, so nothing here is reached there.
//!
//! A Theme document is TOML and nothing else, decoded by `toml` straight into
//! a derived Serde structure. TOML is typed, its root is always a table, and
//! its parser refuses a repeated key, so what is left to Orcvs is the
//! structure's shape and its values:
//!
//! - the root and `style` deny unknown fields, and field names are
//!   case-sensitive;
//! - `style` holds one optional field per property, derived from
//!   [`crate::theme::style_catalogue`], the declaration the property keys'
//!   spellings come from;
//! - `style` must be a table: a derived struct also takes an array of its
//!   fields in declaration order, so the parsed value is checked first;
//! - each value deserializes into a newtype that validates it — the format
//!   marker, the version, labels, the appearance, colours, widths and the
//!   optional Cursor colours — so `toml` attaches its line and column, and the
//!   key it was reading, to every refusal.
//!
//! A width's bound is checked here, as read, and nowhere else on the way to a
//! resolved Theme: the document holds the typed width. [`resolve`] checks
//! parent/appearance agreement with the identity the caller took from
//! the file name.
//!

use std::fmt;

use egui::Color32;
use serde::Deserialize;

use crate::theme::{
    Appearance, ChromeWidth, GRAYSCALE_DARK_IDENTITY, GRAYSCALE_LIGHT_IDENTITY, GridWidth,
    MY_GRAYSCALE_DARK_IDENTITY, MY_GRAYSCALE_LIGHT_IDENTITY, OKABE_ITO_IDENTITY,
    ORCVS_LIGHT_IDENTITY, Theme, ThemeIdentity, style_catalogue,
};

/// `schema.md`'s per-document byte limit, checked before any decoding.
pub(crate) const MAX_DOCUMENT_BYTES: usize = 1024 * 1024;

/// The longest decoder message a [`DocumentError`] keeps, in UTF-8 bytes.
const MAX_MESSAGE_BYTES: usize = 1024;

/// The longest `name` or `inherits` value, in UTF-8 bytes.
const MAX_LABEL_BYTES: usize = 256;

const FORMAT_MARKER: &str = "orcvs-theme";
const VERSION: i64 = 1;

// === Property keys ===

///
/// Derives each key's `name`: an exhaustive `match` over the enum, so a new
/// variant without a spelling in [`style_catalogue`] fails to compile. The
/// optional colours have no key enum and so no `name`. Only a colour key's name
/// reaches a refusal, so the width keys' names are compiled for tests alone.
///
macro_rules! property_names {
    (
        OptionalColor { $($optional:tt)* }
        ColorKey { $($color:ident($color_field:ident) => $color_name:literal),* $(,)? }
        $($key:ident { $($variant:ident($field:ident) => $name:literal),* $(,)? })*
    ) => {
        impl ColorKey {
            /// The property's literal, case-sensitive `schema.md` name.
            pub(crate) fn name(self) -> &'static str {
                match self {
                    $(Self::$color => $color_name,)*
                }
            }
        }
        $(
            #[cfg(test)]
            impl $key {
                /// The property's literal, case-sensitive `schema.md` name.
                pub(crate) fn name(self) -> &'static str {
                    match self {
                        $(Self::$variant => $name,)*
                    }
                }
            }
        )*
    };
}

style_catalogue!(property_names);

///
/// Derives the three key enums from [`style_catalogue`], in its order, and the
/// Theme's routing by key: each variant names the Theme field it reads and
/// writes, so adding a property is one catalogue line. [`resolve`] writes a
/// document's explicit properties through the setters; only tests read by
/// key, to check each key reaches its own field, since painting reads the
/// fields directly.
///
macro_rules! property_keys {
    (
        OptionalColor { $($optional:tt)* }
        ColorKey { $($color:ident($color_field:ident) => $color_name:literal),* $(,)? }
        GridWidthKey { $($grid:ident($grid_field:ident) => $grid_name:literal),* $(,)? }
        ChromeWidthKey { $($chrome:ident($chrome_field:ident) => $chrome_name:literal),* $(,)? }
    ) => {
        ///
        /// Every named colour property `schema.md`'s catalogue defines, less
        /// the optional Cursor colours, which [`OptionalColor`] carries
        /// instead of a plain `Color32`. Declaration order is the
        /// catalogue's, and the order a decoded document lists its
        /// properties in.
        ///
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        pub(crate) enum ColorKey {
            $($color,)*
        }

        ///
        /// The Grid/Cell/Sector Seam width properties, bounded 0 to 1 point
        /// by [`GridWidth`].
        ///
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        pub(crate) enum GridWidthKey {
            $($grid,)*
        }

        ///
        /// The chrome width properties, bounded 0 to 2 points by
        /// [`ChromeWidth`].
        ///
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        pub(crate) enum ChromeWidthKey {
            $($chrome,)*
        }

        impl Theme {
            fn color_mut(&mut self, key: ColorKey) -> &mut Color32 {
                match key {
                    $(ColorKey::$color => &mut self.$color_field,)*
                }
            }

            fn set_grid_width(&mut self, key: GridWidthKey, value: GridWidth) {
                match key {
                    $(GridWidthKey::$grid => self.$grid_field = value,)*
                }
            }

            fn set_chrome_width(&mut self, key: ChromeWidthKey, value: ChromeWidth) {
                match key {
                    $(ChromeWidthKey::$chrome => self.$chrome_field = value,)*
                }
            }
        }

        #[cfg(test)]
        impl Theme {
            fn color(&self, key: ColorKey) -> Color32 {
                match key {
                    $(ColorKey::$color => self.$color_field,)*
                }
            }

            fn grid_width(&self, key: GridWidthKey) -> GridWidth {
                match key {
                    $(GridWidthKey::$grid => self.$grid_field,)*
                }
            }

            fn chrome_width(&self, key: ChromeWidthKey) -> ChromeWidth {
                match key {
                    $(ChromeWidthKey::$chrome => self.$chrome_field,)*
                }
            }
        }
    };
}

style_catalogue!(property_keys);

// === Optional Cursor colours ===

///
/// An explicit value for one of the optional Cursor colours
/// (`cursor.background`, `cursor.foreground`, `region.cursor.background`,
/// `region.cursor.foreground`). A document that omits the property entirely
/// inherits the parent's resolved value instead of holding one of these two
/// states — see [`ThemeDocument::cursor_background`].
///
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum OptionalColor {
    /// The document string `"none"`: clears the optional colour and enables its
    /// fallback (the ordinary Cursor fill or glyph colour in a Region, no fill
    /// at all for the ordinary Cursor, and the role's own colour for the
    /// Cursor's glyph).
    None,
    /// An explicit colour, including a fully transparent one — distinct
    /// from `None` above, since a supplied transparent colour is still a
    /// supplied value and does not trigger the fallback.
    Color(Color32),
}

// === Identity ===

impl ThemeIdentity {
    /// Whether this is a built-in's reserved identity.
    pub(crate) fn is_reserved(&self) -> bool {
        [
            GRAYSCALE_DARK_IDENTITY,
            GRAYSCALE_LIGHT_IDENTITY,
            MY_GRAYSCALE_DARK_IDENTITY,
            MY_GRAYSCALE_LIGHT_IDENTITY,
            OKABE_ITO_IDENTITY,
            ORCVS_LIGHT_IDENTITY,
        ]
        .contains(self)
    }
}

// === Custom document model ===

///
/// The parsed-but-unresolved form of a custom Theme document:
/// `schema.md`'s `inherits`, `name`, `appearance` and `style` fields,
/// decoded into typed properties. Holds no file path, byte source or raw
/// text — [`crate::theme_document::decode`] builds one from a TOML
/// document's bytes; tests in this module construct it directly.
///
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct ThemeDocument {
    /// The built-in identity named by `inherits`.
    pub(crate) parent: String,
    /// The declared display label (`name`).
    pub(crate) name: String,
    /// `appearance`, when the document declares one explicitly. `None`
    /// resolves from the parent.
    pub(crate) appearance: Option<Appearance>,
    pub(crate) colors: Vec<(ColorKey, Color32)>,
    pub(crate) grid_widths: Vec<(GridWidthKey, GridWidth)>,
    pub(crate) chrome_widths: Vec<(ChromeWidthKey, ChromeWidth)>,
    /// `None`: `cursor.background` is omitted and inherits the parent's
    /// resolved value. `Some`: the document supplied one of
    /// [`OptionalColor`]'s two explicit states.
    pub(crate) cursor_background: Option<OptionalColor>,
    pub(crate) cursor_foreground: Option<OptionalColor>,
    pub(crate) region_cursor_background: Option<OptionalColor>,
    pub(crate) region_cursor_foreground: Option<OptionalColor>,
}

///
/// Why [`resolve`] refused a document.
///
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum ThemeError {
    /// `inherits` names an identity absent from the built-in set passed to
    /// [`resolve`] — including a chain through another custom Theme, which
    /// is never a member of that set. `schema.md`: "Only built-ins can be
    /// parents. Parent chains, missing parents ... are errors."
    UnknownParent { parent: String },
    /// The identity being resolved into collides with a built-in's
    /// reserved identity: a file using one is refused.
    ReservedIdentity { identity: ThemeIdentity },
    /// `appearance` was declared explicitly and disagrees with the
    /// parent's.
    AppearanceMismatch {
        parent: Appearance,
        declared: Appearance,
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
            Self::NonOpaqueWindowBackground { alpha } => write!(
                f,
                "{} has alpha {alpha}, not 255: the application window must stay opaque",
                ColorKey::WindowBackground.name()
            ),
        }
    }
}

///
/// The pure inheritance resolver: `schema.md`'s "Source composition and
/// inheritance" step 1 (copy the parent's resolved properties, replace only
/// what the document supplies) and its optional-colour rule. Copies the
/// built-in named by `document.parent`, replaces every explicit property
/// `document` supplies, and returns the result under `identity`.
///
/// No file I/O: `built_ins` is whatever built-in set the caller already
/// holds — the crate-private `theme_registry` module reads native documents
/// and calls this; tests pass `&[okabe_ito()]` directly. `identity` is supplied
/// by the caller rather than read from `document`, because a custom
/// document's identity is its filename stem, which this module
/// never reads a file to learn.
///
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

    for &(key, width) in &document.grid_widths {
        resolved.set_grid_width(key, width);
    }

    for &(key, width) in &document.chrome_widths {
        resolved.set_chrome_width(key, width);
    }

    // Omitted (`None`): inherit — `resolved` already carries the parent's
    // value from the `clone()` above, restated here so the inheritance rule
    // is visible rather than merely relied upon.
    let optional = |supplied: Option<OptionalColor>, inherited: Option<Color32>| match supplied {
        None => inherited,
        Some(OptionalColor::None) => None,
        Some(OptionalColor::Color(colour)) => Some(colour),
    };
    resolved.cursor_background = optional(document.cursor_background, parent.cursor_background);
    resolved.cursor_foreground = optional(document.cursor_foreground, parent.cursor_foreground);
    resolved.region_cursor_background = optional(
        document.region_cursor_background,
        parent.region_cursor_background,
    );
    resolved.region_cursor_foreground = optional(
        document.region_cursor_foreground,
        parent.region_cursor_foreground,
    );

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

// === Errors ===

///
/// Why [`decode`] refused a document, and which file it was.
///
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DocumentError {
    pub(crate) file_name: String,
    pub(crate) reason: DocumentErrorReason,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum DocumentErrorReason {
    /// The document is larger than [`MAX_DOCUMENT_BYTES`].
    TooLarge { bytes: usize },
    /// The bytes are not UTF-8; `valid_up_to` is the first bad byte's offset.
    NotUtf8 { valid_up_to: usize },
    /// `toml` refused the document, or a newtype inside the document type
    /// did. `message` is the decoder's own, restated with its line and
    /// column and the key it was reading, and without the echoed source
    /// line. It is capped at [`MAX_MESSAGE_BYTES`].
    Invalid { message: String },
}

impl fmt::Display for DocumentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let file_name = &self.file_name;
        match &self.reason {
            DocumentErrorReason::TooLarge { bytes } => write!(
                f,
                "{file_name}: Theme document is {bytes} bytes, over the {MAX_DOCUMENT_BYTES}-byte \
                 limit"
            ),
            DocumentErrorReason::NotUtf8 { valid_up_to } => write!(
                f,
                "{file_name}: Theme document is not UTF-8 (invalid byte at offset {valid_up_to})"
            ),
            DocumentErrorReason::Invalid { message } => {
                write!(f, "{file_name}: invalid TOML Theme document: {message}")
            }
        }
    }
}

// === Entry point ===

///
/// Decodes one Theme document. `file_name` only labels the error: whether a
/// name is a Theme file's is `theme_registry`'s decision, made once, where it
/// takes the identity the caller passes with the result to [`resolve`].
/// Performs no I/O.
///
/// Refuses, in order: more than [`MAX_DOCUMENT_BYTES`], and non-UTF-8
/// bytes. One leading U+FEFF is then stripped, and the text is decoded
/// whole — a document is accepted entirely or not at all.
///
pub(crate) fn decode(file_name: &str, bytes: &[u8]) -> Result<ThemeDocument, DocumentError> {
    let error = |reason| DocumentError {
        file_name: file_name.to_owned(),
        reason,
    };

    if bytes.len() > MAX_DOCUMENT_BYTES {
        return Err(error(DocumentErrorReason::TooLarge { bytes: bytes.len() }));
    }
    let text = std::str::from_utf8(bytes).map_err(|utf8| {
        error(DocumentErrorReason::NotUtf8 {
            valid_up_to: utf8.valid_up_to(),
        })
    })?;
    let text = text.strip_prefix('\u{FEFF}').unwrap_or(text);

    let invalid = |message: String| {
        error(DocumentErrorReason::Invalid {
            message: bounded(message),
        })
    };
    let root = toml::de::DeTable::parse(text).map_err(|e| invalid(toml_message(e, text)))?;
    // A derived struct also takes an array of its fields in declaration
    // order, and TOML can spell one for `style`, so `style` is checked to
    // be a table before the document type sees it. The root always is one.
    if let Some(style) = root.get_ref().get("style")
        && !matches!(style.get_ref(), toml::de::DeValue::Table(_))
    {
        return Err(invalid(format!(
            "{}, in `style`: `style` is a table of properties",
            position(text, style.span().start)
        )));
    }
    let raw = RawDocument::deserialize(toml::de::Deserializer::from(root))
        .map_err(|e| invalid(toml_message(e, text)))?;
    Ok(raw.into_document())
}

///
/// `toml`'s `Display` renders a snippet that echoes the whole offending
/// source line, which in a one-line document is the document. This keeps
/// the decoder's message, states its position — 1-based line, and column
/// counted in characters — and names the key it was reading. `toml` keeps
/// that key path private and shows it only in its `Display` without a
/// snippet, as a last `in `…`` line, so it is read from there.
///
fn toml_message(mut error: toml::de::Error, text: &str) -> String {
    let message = error.message().to_owned();
    error.set_input(None);
    let shown = error.to_string();
    let key = shown
        .strip_prefix(message.as_str())
        .map(str::trim)
        .and_then(|rest| rest.strip_prefix("in `")?.strip_suffix('`'))
        .map_or_else(String::new, |key| format!(", in `{key}`"));
    match error.span() {
        Some(span) => format!("{}{key}: {message}", position(text, span.start)),
        None => format!("{message}{key}"),
    }
}

/// `offset`'s 1-based line, and column counted in characters.
fn position(text: &str, offset: usize) -> String {
    let before = text.get(..offset).unwrap_or(text);
    let line = before.matches('\n').count() + 1;
    let line_start = before.rfind('\n').map_or(0, |newline| newline + 1);
    let column = before[line_start..].chars().count() + 1;
    format!("line {line}, column {column}")
}

///
/// Caps a decoder message at [`MAX_MESSAGE_BYTES`], cut on a character
/// boundary and marked with `…`. A message can quote the offending value,
/// and a value can be most of a mebibyte.
///
fn bounded(mut message: String) -> String {
    if message.len() > MAX_MESSAGE_BYTES {
        let mut end = MAX_MESSAGE_BYTES - '…'.len_utf8();
        while !message.is_char_boundary(end) {
            end -= 1;
        }
        message.truncate(end);
        message.push('…');
    }
    message
}

// === Document type ===

///
/// The document's root table. Field names are `schema.md`'s, case-sensitive;
/// `format` and `version` hold nothing once checked.
///
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawDocument {
    #[serde(rename = "format")]
    _format: FormatMarker,
    #[serde(rename = "version")]
    _version: Version,
    name: Label,
    inherits: Label,
    appearance: Option<AppearanceValue>,
    #[serde(default)]
    style: Style,
}

impl RawDocument {
    fn into_document(self) -> ThemeDocument {
        let mut document = self.style.into_document();
        document.parent = self.inherits.0;
        document.name = self.name.0;
        document.appearance = self
            .appearance
            .map(|AppearanceValue(appearance)| appearance);
        document
    }
}

// === Validating values ===

/// `format`: the string `orcvs-theme`.
#[derive(Deserialize)]
#[serde(try_from = "String")]
struct FormatMarker;

impl TryFrom<String> for FormatMarker {
    type Error = String;

    fn try_from(value: String) -> Result<Self, String> {
        if value == FORMAT_MARKER {
            Ok(Self)
        } else {
            Err(format!(
                "{value:?} is not an Orcvs Theme; a Theme document declares \
                 format = \"{FORMAT_MARKER}\""
            ))
        }
    }
}

/// `version`: the integer 1.
#[derive(Deserialize)]
#[serde(try_from = "i64")]
struct Version;

impl TryFrom<i64> for Version {
    type Error = String;

    fn try_from(value: i64) -> Result<Self, String> {
        if value == VERSION {
            Ok(Self)
        } else {
            Err(format!(
                "version {value} is unsupported; this build reads Theme format version \
                 {VERSION} only"
            ))
        }
    }
}

/// `name` and `inherits`: nonempty, no control characters, at most
/// [`MAX_LABEL_BYTES`] UTF-8 bytes. Too long is an error, never a truncation.
#[derive(Deserialize)]
#[serde(try_from = "String")]
struct Label(String);

impl TryFrom<String> for Label {
    type Error = String;

    fn try_from(value: String) -> Result<Self, String> {
        if value.is_empty() {
            return Err("the label is empty".to_owned());
        }
        if value.chars().any(char::is_control) {
            return Err("the label contains a control character".to_owned());
        }
        if value.len() > MAX_LABEL_BYTES {
            return Err(format!(
                "the label is {} UTF-8 bytes, over the {MAX_LABEL_BYTES}-byte limit",
                value.len()
            ));
        }
        Ok(Self(value))
    }
}

///
/// `appearance`: the string `dark` or `light`. A string validated like the
/// others rather than a derived enum, because `toml` also hands an enum a
/// table as its variant, and `{ dark = {} }` is not an appearance.
///
#[derive(Deserialize)]
#[serde(try_from = "String")]
struct AppearanceValue(Appearance);

impl TryFrom<String> for AppearanceValue {
    type Error = String;

    fn try_from(value: String) -> Result<Self, String> {
        match value.as_str() {
            "dark" => Ok(Self(Appearance::Dark)),
            "light" => Ok(Self(Appearance::Light)),
            _ => Err(format!(
                "{value:?} is not an appearance; expected `dark` or `light`"
            )),
        }
    }
}

/// A colour property: `#` and exactly 6 or 8 hexadecimal digits, straight
/// RGB(A); six mean opaque.
#[derive(Deserialize)]
#[serde(try_from = "String")]
struct ColorValue(egui::Color32);

impl TryFrom<String> for ColorValue {
    type Error = String;

    fn try_from(value: String) -> Result<Self, String> {
        if value == "none" {
            return Err(NONE_IS_NOT_A_COLOUR.to_owned());
        }
        colour(&value).map(Self)
    }
}

fn colour(value: &str) -> Result<egui::Color32, String> {
    let malformed =
        || format!("{value:?} is not a colour; a colour is \"#\" and 6 or 8 hexadecimal digits");
    // `from_hex` also takes CSS's 3- and 4-digit forms, which the schema
    // does not, and `from_str_radix` beneath it would take a leading `+`.
    let digits = value.strip_prefix('#').ok_or_else(malformed)?;
    if !matches!(digits.len(), 6 | 8) || !digits.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(malformed());
    }
    // Straight RGB(A) to premultiplied, as `theme::straight_rgba` does for
    // the built-ins: both go through `from_rgba_unmultiplied_const`.
    egui::Color32::from_hex(value).map_err(|_| malformed())
}

/// One of the optional Cursor colours: a colour, or the string `none`.
#[derive(Deserialize)]
#[serde(try_from = "String")]
struct OptionalColorValue(OptionalColor);

impl TryFrom<String> for OptionalColorValue {
    type Error = String;

    fn try_from(value: String) -> Result<Self, String> {
        match value.as_str() {
            "none" => Ok(Self(OptionalColor::None)),
            value => colour(value).map(|colour| Self(OptionalColor::Color(colour))),
        }
    }
}

///
/// A width in points, refused unless finite and within `0..=max`. Checked as
/// read, before narrowing to the `f32` a typed width holds: narrowing rounds a
/// value just past a bound onto it, and one beyond `f32`'s range to
/// infinity, and neither may be clamped. An `f64` within the bound narrows to
/// an `f32` within it, so the typed width's constructor accepts the answer.
/// `toml` hands an integer to an `f64` too, so both spellings are widths, and
/// nothing else is.
///
fn points(points: f64, max: f32) -> Result<f32, String> {
    if !points.is_finite() {
        return Err(format!(
            "{points} is not a width; a width is a finite number of points"
        ));
    }
    if !(0.0..=f64::from(max)).contains(&points) {
        return Err(format!(
            "{points:?} is not a width; a width is 0 to {max} points"
        ));
    }
    Ok(points as f32)
}

const NARROWED: &str = "a width within its bound narrows to an f32 within it";

/// A Grid width: 0 to [`GridWidth::MAX_POINTS`].
#[derive(Deserialize)]
#[serde(try_from = "f64")]
struct GridPoints(GridWidth);

impl TryFrom<f64> for GridPoints {
    type Error = String;

    fn try_from(value: f64) -> Result<Self, String> {
        let points = points(value, GridWidth::MAX_POINTS)?;
        Ok(Self(GridWidth::from_points(points).expect(NARROWED)))
    }
}

/// A chrome width: 0 to [`ChromeWidth::MAX_POINTS`].
#[derive(Deserialize)]
#[serde(try_from = "f64")]
struct ChromePoints(ChromeWidth);

impl TryFrom<f64> for ChromePoints {
    type Error = String;

    fn try_from(value: f64) -> Result<Self, String> {
        let points = points(value, ChromeWidth::MAX_POINTS)?;
        Ok(Self(ChromeWidth::from_points(points).expect(NARROWED)))
    }
}

// === Style ===

///
/// Joins property spellings, each in backticks, as an English list:
/// `` `a`, `b` and `c` ``.
///
macro_rules! spelling_list {
    ($only:literal) => {
        concat!("`", $only, "`")
    };
    ($penultimate:literal, $last:literal) => {
        concat!("`", $penultimate, "` and `", $last, "`")
    };
    ($first:literal, $($rest:literal),+) => {
        concat!("`", $first, "`, ", spelling_list!($($rest),+))
    };
}

///
/// Derives the `style` table from [`style_catalogue`]: one optional field per
/// property, renamed to its literal dotted spelling, the optional Cursor
/// colours among them, and [`NONE_IS_NOT_A_COLOUR`], the refusal naming
/// their spellings. Its `into_document` lists the supplied properties in
/// catalogue order.
///
macro_rules! document_style {
    (
        OptionalColor { $($optional:ident($optional_field:ident) => $optional_name:literal),* $(,)? }
        ColorKey { $($color:ident($color_field:ident) => $color_name:literal),* $(,)? }
        GridWidthKey { $($grid:ident($grid_field:ident) => $grid_name:literal),* $(,)? }
        ChromeWidthKey { $($chrome:ident($chrome_field:ident) => $chrome_name:literal),* $(,)? }
    ) => {
        /// The refusal of `"none"` on any property but the optional colours,
        /// which it names.
        const NONE_IS_NOT_A_COLOUR: &str = concat!(
            "\"none\" is not a colour; only ",
            spelling_list!($($optional_name),*),
            " can be \"none\""
        );

        /// The decoded `style` table: `None` for every omitted property.
        #[derive(Default, Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Style {
            $(#[serde(rename = $color_name)] $color_field: Option<ColorValue>,)*
            $(#[serde(rename = $grid_name)] $grid_field: Option<GridPoints>,)*
            $(#[serde(rename = $chrome_name)] $chrome_field: Option<ChromePoints>,)*
            $(#[serde(rename = $optional_name)] $optional_field: Option<OptionalColorValue>,)*
        }

        impl Style {
            fn into_document(self) -> ThemeDocument {
                let mut document = ThemeDocument::default();
                $(if let Some(ColorValue(colour)) = self.$color_field {
                    document.colors.push((ColorKey::$color, colour));
                })*
                $(if let Some(GridPoints(width)) = self.$grid_field {
                    document.grid_widths.push((GridWidthKey::$grid, width));
                })*
                $(if let Some(ChromePoints(width)) = self.$chrome_field {
                    document.chrome_widths.push((ChromeWidthKey::$chrome, width));
                })*
                $(document.$optional_field = self.$optional_field.map(|OptionalColorValue(optional)| optional);)*
                document
            }
        }
    };
}

style_catalogue!(document_style);

#[cfg(test)]
mod tests {
    use egui::Color32;

    use super::{
        ChromeWidthKey, ColorKey, DocumentErrorReason, GridWidthKey, MAX_DOCUMENT_BYTES,
        MAX_MESSAGE_BYTES, OptionalColor, ThemeDocument, ThemeError, decode, resolve,
    };
    use crate::theme::{Appearance, ChromeWidth, GridWidth, Theme, ThemeIdentity, straight_rgba};
    use crate::theme_registry::tests_support::default_theme;

    /// A Theme field, read by name.
    type Field<T> = fn(&Theme) -> T;

    /// A custom Theme's identity, as a file named `stem` supplies it.
    fn identity(stem: &str) -> ThemeIdentity {
        ThemeIdentity::from_stem(stem).expect("a non-empty stem")
    }

    /// The dark default's identity, as a document's `inherits` names it.
    fn dark_parent() -> String {
        ThemeIdentity::default_for(Appearance::Dark).to_string()
    }

    /// The light default's identity, as a document's `inherits` names it.
    fn light_parent() -> String {
        ThemeIdentity::default_for(Appearance::Light).to_string()
    }

    /// The four required root fields, inheriting the dark default.
    fn root_fields() -> String {
        format!(
            "format = \"orcvs-theme\"\nversion = 1\nname = \"T\"\ninherits = \"{}\"\n",
            dark_parent()
        )
    }

    const MY_DARK_TOML: &str = include_str!("../../.scratch/theming/examples/my-dark.toml");
    const OKABE_ITO_COPY_TOML: &str =
        include_str!("../../.scratch/theming/examples/okabe-ito-copy.toml");

    ///
    /// Every property name `style_catalogue` spells: the four optional colours,
    /// then each key enum's, in the catalogue's order.
    ///
    macro_rules! catalogue_names {
        ($($group:ident { $($variant:ident($field:ident) => $name:literal),* $(,)? })*) => {
            &[$($($name,)*)*]
        };
    }

    const CATALOGUE_NAMES: &[&str] = crate::theme::style_catalogue!(catalogue_names);

    /// How many properties each part of `style_catalogue` holds.
    struct CatalogueCounts {
        fills: usize,
        colors: usize,
        grid_widths: usize,
        chrome_widths: usize,
    }

    macro_rules! catalogue_counts {
        (
            OptionalColor { $($fill:ident($fill_field:ident) => $fill_name:literal),* $(,)? }
            ColorKey { $($color:ident($color_field:ident) => $color_name:literal),* $(,)? }
            GridWidthKey { $($grid:ident($grid_field:ident) => $grid_name:literal),* $(,)? }
            ChromeWidthKey { $($chrome:ident($chrome_field:ident) => $chrome_name:literal),* $(,)? }
        ) => {
            CatalogueCounts {
                fills: [$($fill_name),*].len(),
                colors: [$($color_name),*].len(),
                grid_widths: [$($grid_name),*].len(),
                chrome_widths: [$($chrome_name),*].len(),
            }
        };
    }

    const CATALOGUE_COUNTS: CatalogueCounts = crate::theme::style_catalogue!(catalogue_counts);

    /// The `"name" = value` lines of an example document, as name and value.
    fn example_entries(toml: &str) -> Vec<(&str, &str)> {
        toml.lines()
            .filter_map(|line| line.strip_prefix('"')?.split_once("\" = "))
            .collect()
    }

    ///
    /// Asserts `entries` names every catalogue property once and nothing
    /// else, in any order. A document inherits what it omits from its parent,
    /// so an example that resolves to its parent proves nothing about the
    /// properties it leaves out.
    ///
    fn assert_spells_the_catalogue(entries: &[(&str, &str)]) {
        let mut names = entries.iter().map(|&(name, _)| name).collect::<Vec<_>>();
        names.sort_unstable();
        let mut catalogue = CATALOGUE_NAMES.to_vec();
        catalogue.sort_unstable();
        assert_eq!(names, catalogue);
    }

    fn my_dark() -> ThemeDocument {
        ThemeDocument {
            parent: dark_parent(),
            name: "My Dark".to_owned(),
            appearance: None,
            colors: vec![
                (ColorKey::GridBackground, straight_rgba(0x10_18_20_FF)),
                (
                    ColorKey::SourceFunctionBackground,
                    straight_rgba(0x00_19_12_FF),
                ),
                (ColorKey::GridBorder, straight_rgba(0x35_50_60_80)),
                (ColorKey::Text, straight_rgba(0xE0_E0_E0_FF)),
            ],
            grid_widths: vec![(GridWidthKey::GridBorder, grid(0.5))],
            chrome_widths: Vec::new(),
            cursor_background: Some(OptionalColor::None),
            cursor_foreground: None,
            region_cursor_background: None,
            region_cursor_foreground: None,
        }
    }

    // --- Documents ---

    const FILE: &str = "theme.toml";

    ///
    /// A document with the four required root fields, `extra_root` after
    /// them, and a `[style]` table holding `entries` — each a property name,
    /// written as a quoted key, and a value already in TOML syntax.
    ///
    fn document(extra_root: &[(&str, &str)], entries: &[(&str, &str)]) -> String {
        let mut text = root_fields();
        for (key, value) in extra_root {
            text.push_str(&format!("{key} = {value}\n"));
        }
        text.push_str("\n[style]\n");
        for (key, value) in entries {
            text.push_str(&format!("\"{key}\" = {value}\n"));
        }
        text
    }

    fn grid(points: f32) -> GridWidth {
        GridWidth::from_points(points).expect("a Grid width in range")
    }

    fn chrome(points: f32) -> ChromeWidth {
        ChromeWidth::from_points(points).expect("a chrome width in range")
    }

    fn style(entries: &[(&str, &str)]) -> String {
        document(&[], entries)
    }

    /// `value` as a TOML basic string.
    fn quoted(value: &str) -> String {
        format!("{value:?}")
    }

    fn decodes(text: &str) -> ThemeDocument {
        decode(FILE, text.as_bytes())
            .unwrap_or_else(|error| panic!("should decode:\n{text}\n{error}"))
    }

    ///
    /// Asserts `text` is refused by `toml` or the document type, with every
    /// one of `needles` in the message, and returns the message.
    ///
    fn refuses(text: &str, needles: &[&str]) -> String {
        let error =
            decode(FILE, text.as_bytes()).expect_err(&format!("should be refused:\n{text}"));
        let DocumentErrorReason::Invalid { message } = &error.reason else {
            panic!("expected a decode error, got {error:?}");
        };
        for needle in needles {
            assert!(
                message.contains(needle),
                "{needle:?} missing from the error:\n{message}\nfor:\n{text}"
            );
        }
        message.clone()
    }

    fn with_bom(text: &str) -> Vec<u8> {
        let mut bytes = "\u{FEFF}".as_bytes().to_vec();
        bytes.extend_from_slice(text.as_bytes());
        bytes
    }

    // --- The examples ---

    #[test]
    fn the_my_dark_example_decodes_to_its_properties() {
        assert_eq!(decodes(MY_DARK_TOML), my_dark());
    }

    ///
    /// `okabe-ito-copy.toml` spells out every catalogue property at the
    /// built-in's value, so resolving it must give back the built-in under
    /// the copy's identity and display label.
    ///
    #[test]
    fn the_okabe_ito_copy_example_resolves_to_the_built_in() {
        assert_spells_the_catalogue(&example_entries(OKABE_ITO_COPY_TOML));
        let document = decodes(OKABE_ITO_COPY_TOML);
        assert_eq!(document.cursor_background, Some(OptionalColor::None));
        assert_eq!(document.cursor_foreground, Some(OptionalColor::None));
        assert_eq!(document.region_cursor_background, Some(OptionalColor::None));
        assert_eq!(document.region_cursor_foreground, Some(OptionalColor::None));
        assert_eq!(document.appearance, Some(Appearance::Dark));

        let resolved = resolve(
            &[crate::theme::okabe_ito()],
            &identity("okabe-ito-copy"),
            &document,
        )
        .expect("resolves");

        let mut expected = crate::theme::okabe_ito();
        expected.identity = identity("okabe-ito-copy");
        expected.name = "Okabe-Ito copy".to_owned();
        assert_eq!(resolved, expected);
    }

    ///
    /// Each catalogue property name, alone in a document at the example's
    /// value, sets exactly one property, whose key spells that name — and no
    /// two reach the same property, so every catalogue entry is reachable by
    /// its own spelling and by no other.
    ///
    #[test]
    fn every_catalogue_property_name_selects_its_own_property() {
        let entries = example_entries(OKABE_ITO_COPY_TOML);
        assert_spells_the_catalogue(&entries);
        let mut reached = Vec::new();
        for (name, value) in entries {
            let document = decodes(&style(&[(name, value)]));
            let mut set = document
                .colors
                .iter()
                .map(|&(key, _)| key.name())
                .chain(document.grid_widths.iter().map(|&(key, _)| key.name()))
                .chain(document.chrome_widths.iter().map(|&(key, _)| key.name()))
                .chain(document.cursor_background.map(|_| "cursor.background"))
                .chain(document.cursor_foreground.map(|_| "cursor.foreground"))
                .chain(
                    document
                        .region_cursor_background
                        .map(|_| "region.cursor.background"),
                )
                .chain(
                    document
                        .region_cursor_foreground
                        .map(|_| "region.cursor.foreground"),
                );
            assert_eq!(set.next(), Some(name), "{name}");
            assert_eq!(set.next(), None, "{name} set a second property");
            assert!(!reached.contains(&name), "{name} reached twice");
            reached.push(name);
        }
    }

    // --- Empty and omitted style ---

    #[test]
    fn an_omitted_or_empty_style_overrides_nothing() {
        let bare = ThemeDocument {
            parent: dark_parent(),
            name: "T".to_owned(),
            ..ThemeDocument::default()
        };
        assert_eq!(decodes(&style(&[])), bare, "empty style");
        assert_eq!(decodes(&root_fields()), bare, "omitted style");
    }

    ///
    /// `style` is a table. A derived struct would also take an array of its
    /// fields in declaration order, so a complete array is refused too.
    ///
    #[test]
    fn a_style_that_is_not_a_table_is_refused() {
        let in_order = [
            vec!["\"#FFFFFF\""; CATALOGUE_COUNTS.colors],
            vec!["0.5"; CATALOGUE_COUNTS.grid_widths],
            vec!["1"; CATALOGUE_COUNTS.chrome_widths],
            vec!["\"none\""; CATALOGUE_COUNTS.fills],
        ]
        .concat()
        .join(", ");
        let in_order = format!("[{in_order}]");
        for value in ["\"x\"", "1", "[]", in_order.as_str()] {
            refuses(
                &format!("{}style = {value}\n", root_fields()),
                &["line 5, column 9, in `style`: `style` is a table of properties"],
            );
        }
    }

    // --- Optional colours ---

    #[test]
    fn optional_colours_are_omitted_cleared_or_a_supplied_transparent_colour() {
        let omitted = decodes(&style(&[]));
        assert_eq!(omitted.cursor_background, None);
        assert_eq!(omitted.cursor_foreground, None);
        assert_eq!(omitted.region_cursor_background, None);
        assert_eq!(omitted.region_cursor_foreground, None);

        let none = quoted("none");
        let cleared = decodes(&style(&[
            ("cursor.background", &none),
            ("cursor.foreground", &none),
            ("region.cursor.background", &none),
            ("region.cursor.foreground", &none),
        ]));
        assert_eq!(cleared.cursor_background, Some(OptionalColor::None));
        assert_eq!(cleared.cursor_foreground, Some(OptionalColor::None));
        assert_eq!(cleared.region_cursor_background, Some(OptionalColor::None));
        assert_eq!(cleared.region_cursor_foreground, Some(OptionalColor::None));

        let transparent = quoted("#12345600");
        let supplied = decodes(&style(&[
            ("cursor.background", &transparent),
            ("cursor.foreground", &transparent),
            ("region.cursor.background", &transparent),
            ("region.cursor.foreground", &transparent),
        ]));
        let colour = OptionalColor::Color(straight_rgba(0x12_34_56_00));
        assert_eq!(supplied.cursor_background, Some(colour));
        assert_eq!(supplied.cursor_foreground, Some(colour));
        assert_eq!(supplied.region_cursor_background, Some(colour));
        assert_eq!(supplied.region_cursor_foreground, Some(colour));
        assert!(supplied.colors.is_empty());
    }

    #[test]
    fn an_optional_colour_refuses_a_malformed_colour() {
        refuses(
            &style(&[("cursor.background", &quoted("None"))]),
            &["in `style.cursor.background`", "6 or 8 hexadecimal digits"],
        );
    }

    #[test]
    fn none_is_refused_for_every_other_colour() {
        refuses(
            &style(&[("text", &quoted("none"))]),
            &[
                "in `style.text`",
                "\"none\" is not a colour; only `cursor.background`, `cursor.foreground`, \
                 `region.cursor.background` and `region.cursor.foreground` can be \"none\"",
            ],
        );
    }

    // --- Colours ---

    #[test]
    fn six_digit_colours_are_opaque_and_eight_digit_colours_carry_alpha() {
        let document = decodes(&style(&[
            ("text", &quoted("#a1B2c3")),
            ("link", &quoted("#A1B2C340")),
        ]));
        assert_eq!(
            document.colors,
            vec![
                (ColorKey::Text, Color32::from_rgb(0xA1, 0xB2, 0xC3)),
                (ColorKey::Link, straight_rgba(0xA1_B2_C3_40)),
            ]
        );
    }

    #[test]
    fn a_colour_is_a_hash_and_six_or_eight_hex_digits() {
        for bad in [
            "#FFF",
            "#FFFFF",
            "#FFFFFFF",
            "#FFFFFFFFF",
            "FFFFFF",
            "#GGGGGG",
            "#+FFFFF",
            " #FFFFFF",
            "",
        ] {
            refuses(
                &style(&[("grid.border", &quoted(bad))]),
                &["in `style.grid.border`", "6 or 8 hexadecimal digits"],
            );
        }
    }

    /// `schema.md`: booleans, arrays and tables are never a valid value, and
    /// neither is a number where a string belongs.
    #[test]
    fn a_colour_refuses_every_other_kind_of_value() {
        for value in [
            "5",
            "0.5",
            "true",
            "[\"#FFFFFF\"]",
            "{ a = \"#FFFFFF\" }",
            "1979-05-27",
        ] {
            refuses(
                &style(&[("text", value)]),
                &["invalid type", "in `style.text`"],
            );
        }
    }

    // --- Widths ---

    #[test]
    fn widths_accept_integers_and_floats_within_their_bounds() {
        let document = decodes(&style(&[
            ("grid.border.width", "0"),
            ("sector.seam.width", "0.75"),
            ("panel.border.width", "2"),
            ("input.cursor.width", "1.5"),
        ]));
        assert_eq!(
            document.grid_widths,
            vec![
                (GridWidthKey::GridBorder, grid(0.0)),
                (GridWidthKey::SectorSeam, grid(0.75))
            ]
        );
        assert_eq!(
            document.chrome_widths,
            vec![
                (ChromeWidthKey::PanelBorder, chrome(2.0)),
                (ChromeWidthKey::InputCursor, chrome(1.5))
            ]
        );
    }

    #[test]
    fn a_width_refuses_a_string_or_any_other_kind() {
        for value in ["\"0.5\"", "true", "[0.5]", "{ a = 0.5 }"] {
            refuses(
                &style(&[("grid.border.width", value)]),
                &["invalid type", "in `style.grid.border.width`"],
            );
        }
    }

    #[test]
    fn a_width_must_be_finite() {
        for property in [
            "grid.border.width",
            "sector.seam.width",
            "input.cursor.width",
        ] {
            for value in ["nan", "inf", "-inf", "+inf"] {
                refuses(
                    &style(&[(property, value)]),
                    &[&format!("in `style.{property}`"), "finite number of points"],
                );
            }
        }
    }

    ///
    /// `schema.md`: a width outside its inclusive bound is refused, never
    /// clamped — including one the `f32` a ThemeDocument stores would round
    /// onto the bound, and one too large for an `f32` at all.
    ///
    #[test]
    fn a_width_outside_its_bound_is_refused_before_narrowing() {
        let cases = [
            ("grid.border.width", "1.00000005", "0 to 1 points"),
            ("grid.border.width", "-1e-50", "0 to 1 points"),
            ("panel.border.width", "2.0000001", "0 to 2 points"),
            ("input.cursor.width", "7.5", "0 to 2 points"),
            ("panel.border.width", "1e300", "0 to 2 points"),
            ("overview.source_view.width", "2.5", "0 to 2 points"),
        ];
        for (property, value, range) in cases {
            refuses(
                &style(&[(property, value)]),
                &[&format!("in `style.{property}`"), range],
            );
        }
    }

    // --- Root fields ---

    #[test]
    fn an_unknown_or_wrong_case_root_field_is_refused() {
        for field in ["colour", "Name", "FORMAT", "Style"] {
            refuses(
                &document(&[(field, "\"x\"")], &[]),
                &[&format!("unknown field `{field}`")],
            );
        }
    }

    #[test]
    fn a_missing_required_field_is_refused() {
        let parent = dark_parent();
        let complete = [
            ("format", "orcvs-theme"),
            ("name", "T"),
            ("inherits", parent.as_str()),
        ];
        for missing in ["format", "version", "name", "inherits"] {
            let toml: String = complete
                .iter()
                .filter(|(key, _)| *key != missing)
                .map(|(key, value)| format!("{key} = {value:?}\n"))
                .chain((missing != "version").then(|| "version = 1\n".to_owned()))
                .collect();
            refuses(&toml, &["missing field", missing]);
        }
    }

    #[test]
    fn a_repeated_root_field_is_refused() {
        refuses(
            &root_fields().replace("name = \"T\"\n", "name = \"T\"\nname = \"U\"\n"),
            &["line 4", "duplicate key"],
        );
    }

    #[test]
    fn the_format_marker_must_be_orcvs_theme() {
        refuses(
            &style(&[]).replacen("orcvs-theme", "orcvs-palette", 1),
            &[
                "in `format`",
                "\"orcvs-palette\" is not an Orcvs Theme",
                "orcvs-theme",
            ],
        );
        refuses(
            &style(&[]).replacen("\"orcvs-theme\"", "1", 1),
            &["invalid type", "in `format`"],
        );
    }

    #[test]
    fn the_version_must_be_the_integer_one() {
        let with_version = |version: &str| style(&[]).replacen("1", version, 1);
        refuses(
            &with_version("2"),
            &["in `version`", "version 2 is unsupported", "version 1 only"],
        );
        refuses(&with_version("0"), &["version 0 is unsupported"]);
        refuses(&with_version("-1"), &["version -1 is unsupported"]);
        refuses(
            &with_version("1.0"),
            &["invalid type: floating point", "in `version`"],
        );
        refuses(
            &with_version("\"1\""),
            &["invalid type: string", "in `version`"],
        );
    }

    #[test]
    fn appearance_is_dark_or_light() {
        for (value, appearance) in [("dark", Appearance::Dark), ("light", Appearance::Light)] {
            let document = decodes(&document(&[("appearance", &quoted(value))], &[]));
            assert_eq!(document.appearance, Some(appearance));
        }
        for value in ["Dark", "dim", ""] {
            refuses(
                &document(&[("appearance", &quoted(value))], &[]),
                &["in `appearance`", "`dark` or `light`"],
            );
        }
        refuses(
            &document(&[("appearance", "true")], &[]),
            &["invalid type: boolean `true`", "in `appearance`"],
        );
    }

    ///
    /// `toml` hands a table to an enum as its variant, so `appearance` is a
    /// string it validates, never a derived enum: a table naming a valid
    /// appearance is a wrong type like any other.
    ///
    #[test]
    fn an_appearance_table_is_refused() {
        for value in ["{ dark = {} }", "{ light = [] }", "{ dark = 1 }"] {
            refuses(
                &document(&[("appearance", value)], &[]),
                &["invalid type: map", "in `appearance`"],
            );
        }
    }

    ///
    /// No other field is an enum, and none takes a table in place of its
    /// scalar either.
    ///
    #[test]
    fn a_table_is_refused_for_every_scalar_field() {
        let root = root_fields();
        for field in ["format", "version", "name", "inherits"] {
            let text = root
                .lines()
                .map(|line| match line.split_once(" = ") {
                    Some((key, _)) if key == field => format!("{key} = {{ dark = {{}} }}\n"),
                    _ => format!("{line}\n"),
                })
                .collect::<String>();
            refuses(&text, &["invalid type: map", &format!("in `{field}`")]);
        }
        for property in [
            "text",
            "cursor.background",
            "grid.border.width",
            "panel.border.width",
        ] {
            refuses(
                &style(&[(property, "{ none = {} }")]),
                &["invalid type: map", &format!("in `style.{property}`")],
            );
        }
    }

    // --- Labels ---

    fn document_with_labels(name: &str, inherits: &str) -> String {
        format!("format = \"orcvs-theme\"\nversion = 1\nname = {name}\ninherits = {inherits}\n")
    }

    #[test]
    fn name_and_inherits_must_be_nonempty_and_free_of_control_characters() {
        let (empty, parent, name) = (quoted(""), quoted(&dark_parent()), quoted("T"));
        refuses(
            &document_with_labels(&empty, &parent),
            &["in `name`", "the label is empty"],
        );
        refuses(
            &document_with_labels(&name, &empty),
            &["in `inherits`", "the label is empty"],
        );
        refuses(
            &document_with_labels("\"A\\tB\"", &parent),
            &["in `name`", "the label contains a control character"],
        );
        refuses(
            &document_with_labels(&name, "\"a\\u0007b\""),
            &["in `inherits`", "the label contains a control character"],
        );
        refuses(
            &document_with_labels("\"\"\"A\nB\"\"\"", &parent),
            &["in `name`", "the label contains a control character"],
        );
        refuses(
            &document_with_labels("1", &parent),
            &["invalid type: integer", "in `name`"],
        );
    }

    ///
    /// 256 UTF-8 bytes is the limit, not 256 characters: 128 two-byte `é`
    /// fit, 129 do not, and a 257-byte name is refused rather than cut.
    ///
    #[test]
    fn name_and_inherits_stay_within_256_utf8_bytes() {
        for field in ["name", "inherits"] {
            let with = |value: &str| match field {
                "name" => document_with_labels(&quoted(value), &quoted(&dark_parent())),
                _ => document_with_labels(&quoted("T"), &quoted(value)),
            };
            let at_limit = decodes(&with(&"a".repeat(256)));
            let held = if field == "name" {
                at_limit.name
            } else {
                at_limit.parent
            };
            assert_eq!(held.len(), 256, "{field}");
            decodes(&with(&"é".repeat(128)));

            refuses(
                &with(&"a".repeat(257)),
                &[
                    &format!("in `{field}`"),
                    "the label is 257 UTF-8 bytes",
                    "256-byte limit",
                ],
            );
            refuses(&with(&"é".repeat(129)), &["the label is 258 UTF-8 bytes"]);
        }
    }

    // --- Style keys ---

    #[test]
    fn an_unknown_or_wrong_case_style_property_is_refused() {
        let colour = quoted("#FFFFFF");
        for property in [
            "Text",
            "grid.Background",
            "source.char",
            "grid_background",
            "text.",
            "style",
            "cursor.area",
        ] {
            refuses(
                &style(&[(property, &colour)]),
                &[&format!("unknown field `{property}`"), "in `style`"],
            );
        }
    }

    ///
    /// The error explains the valid key. `serde`'s refusal lists
    /// the valid property names, colours first, as far as the message cap
    /// allows; a near miss is not singled out.
    ///
    #[test]
    fn an_unknown_style_property_lists_the_valid_names() {
        let message = refuses(
            &style(&[("grid_background", &quoted("#FFFFFF"))]),
            &[
                "unknown field `grid_background`, expected one of",
                "`window.background`",
                "`grid.background`",
            ],
        );
        assert!(
            message.len() <= MAX_MESSAGE_BYTES,
            "{} bytes",
            message.len()
        );
    }

    ///
    /// The list of 63 names, colours first, is longer than the message cap,
    /// which cuts it and marks the cut: a mistyped property is refused with
    /// a message that names it but may not list the width or fill it meant.
    ///
    #[test]
    fn an_unknown_width_property_is_refused_with_a_capped_list() {
        let message = refuses(
            &style(&[("grid.border.widht", "0.5")]),
            &["unknown field `grid.border.widht`, expected one of"],
        );
        assert!(
            message.len() <= MAX_MESSAGE_BYTES,
            "{} bytes",
            message.len()
        );
        assert!(message.ends_with('…'), "{message}");
        for cut in ["`panel.border.width`", "`cursor.background`"] {
            assert!(!message.contains(cut), "{cut} listed: {message}");
        }
    }

    ///
    /// A dotted property name is one literal key, never a nested path: the
    /// TOML dotted key `grid.background = ...` and a nested `grid` table
    /// are both refused.
    ///
    #[test]
    fn a_style_property_is_never_a_nested_path() {
        let root = root_fields();
        refuses(
            &format!("{root}[style]\ngrid.background = \"#FFFFFF\"\n"),
            &["unknown field `grid`"],
        );
        refuses(
            &format!("{root}[style.grid]\nbackground = \"#FFFFFF\"\n"),
            &["unknown field `grid`"],
        );
    }

    #[test]
    fn a_repeated_style_property_is_refused() {
        refuses(
            &style(&[("text", "\"#FFFFFF\""), ("text", "\"#000000\"")]),
            &["duplicate key", "line 8"],
        );
        refuses(
            &style(&[
                ("cursor.background", "\"none\""),
                ("cursor.background", "\"none\""),
            ]),
            &["duplicate key", "line 8"],
        );
        refuses(
            &style(&[("grid.border.width", "0.5"), ("grid.border.width", "0.5")]),
            &["duplicate key", "line 8"],
        );
    }

    // --- Error text ---

    #[test]
    fn toml_errors_carry_line_and_column_without_echoing_the_line() {
        let message = refuses(
            &style(&[("text", "5")]),
            &["line 7, column 10, in `style.text`: invalid type: integer `5`"],
        );
        assert!(!message.contains('\n'), "{message}");
        let message = refuses(
            "format = \"orcvs-theme\"\nversion = \n",
            &["line 2, column"],
        );
        assert!(!message.contains('\n'), "{message}");
    }

    ///
    /// `toml`'s own `Display` echoes the offending source line, so a
    /// one-line document near the byte limit would make a message near a
    /// mebibyte. Every stored message is bounded, whatever produced it.
    ///
    #[test]
    fn a_long_single_line_document_yields_a_bounded_message() {
        let comment = "x".repeat(MAX_DOCUMENT_BYTES / 2);
        let message = refuses(
            &style(&[("text", &format!("5 # {comment}"))]),
            &["line 7, column 10"],
        );
        assert!(
            message.len() <= MAX_MESSAGE_BYTES,
            "{} bytes",
            message.len()
        );

        // A message that quotes a huge value is cut on a character boundary
        // and marked as cut.
        let value = quoted(&format!("#{}", "é".repeat(MAX_DOCUMENT_BYTES / 4)));
        let message = refuses(&style(&[("text", &value)]), &["in `style.text`"]);
        assert!(
            message.len() <= MAX_MESSAGE_BYTES,
            "{} bytes",
            message.len()
        );
        assert!(message.ends_with('…'));
    }

    #[test]
    fn the_error_names_the_file_the_property_and_the_position() {
        let shown = decode("bad.toml", style(&[("grid.border", "\"#12\"")]).as_bytes())
            .expect_err("a malformed colour")
            .to_string();
        assert_eq!(
            shown,
            "bad.toml: invalid TOML Theme document: line 7, column 17, in \
             `style.grid.border`: \"#12\" is not a colour; a colour is \"#\" and 6 or 8 \
             hexadecimal digits"
        );
    }

    // --- Bytes before decoding ---

    #[test]
    fn a_leading_byte_order_mark_is_stripped() {
        let document = decode("my-dark.toml", &with_bom(MY_DARK_TOML))
            .unwrap_or_else(|error| panic!("{error}"));
        assert_eq!(document, my_dark());
        // The mark is not counted in a reported column.
        let error = decode(FILE, &with_bom("format = 1\n")).expect_err("a wrong type");
        assert!(error.to_string().contains("line 1, column 10"), "{error}");
    }

    ///
    /// `schema.md`'s 1 MiB limit: a valid document padded with a trailing
    /// comment to exactly the limit decodes, and one byte more is refused
    /// before any decoder runs.
    ///
    #[test]
    fn the_byte_limit_is_one_mebibyte_inclusive() {
        let padded = |total: usize| {
            let mut text = MY_DARK_TOML.to_owned();
            text.push('#');
            let fill = total - text.len() - 1;
            text.push_str(&"x".repeat(fill));
            text.push('\n');
            assert_eq!(text.len(), total);
            text
        };

        assert_eq!(decodes(&padded(MAX_DOCUMENT_BYTES)), my_dark());

        let error = decode("my-dark.toml", padded(MAX_DOCUMENT_BYTES + 1).as_bytes())
            .expect_err("over the limit");
        assert_eq!(
            error.reason,
            DocumentErrorReason::TooLarge {
                bytes: MAX_DOCUMENT_BYTES + 1
            }
        );
        assert_eq!(MAX_DOCUMENT_BYTES, 1_048_576);
    }

    #[test]
    fn the_byte_limit_applies_before_the_utf8_check() {
        let error = decode("big.toml", &vec![0xFF; MAX_DOCUMENT_BYTES + 1]).expect_err("too large");
        assert!(matches!(error.reason, DocumentErrorReason::TooLarge { .. }));
    }

    #[test]
    fn non_utf8_bytes_are_refused() {
        let mut bytes = MY_DARK_TOML.as_bytes().to_vec();
        bytes.insert(10, 0xFF);
        let error = decode("my-dark.toml", &bytes).expect_err("not UTF-8");
        assert_eq!(
            error.reason,
            DocumentErrorReason::NotUtf8 { valid_up_to: 10 }
        );
    }

    // --- Resolving ---

    fn child(parent: &str) -> ThemeDocument {
        ThemeDocument {
            parent: parent.to_owned(),
            name: "Custom".to_owned(),
            ..ThemeDocument::default()
        }
    }

    ///
    /// A custom document inheriting the light built-in resolves to light —
    /// a light custom Theme starts from a light built-in — exercised against
    /// a real light parent as well as the synthetic mismatch below.
    ///
    #[test]
    fn a_document_inheriting_the_light_built_in_resolves_light() {
        let built_ins = [
            default_theme(Appearance::Dark),
            default_theme(Appearance::Light),
        ];
        let mut document = child(&light_parent());
        document.appearance = Some(Appearance::Light);

        let resolved = resolve(&built_ins, &identity("mine"), &document)
            .expect("light parent, light document");

        assert_eq!(resolved.appearance, Appearance::Light);
        assert_eq!(
            resolved.panel_background,
            default_theme(Appearance::Light).panel_background
        );

        let mut mismatched = child(&light_parent());
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
    /// The light default's identity is reserved as well as the dark one's.
    ///
    #[test]
    fn the_light_built_in_identity_is_reserved() {
        let built_ins = [
            default_theme(Appearance::Dark),
            default_theme(Appearance::Light),
        ];
        assert_eq!(
            resolve(
                &built_ins,
                &ThemeIdentity::default_for(Appearance::Light),
                &child(&dark_parent())
            ),
            Err(ThemeError::ReservedIdentity {
                identity: ThemeIdentity::default_for(Appearance::Light),
            })
        );
    }

    #[test]
    fn omitted_properties_inherit_from_the_parent() {
        let built_ins = [default_theme(Appearance::Dark)];
        let document = child(&dark_parent());

        let resolved =
            resolve(&built_ins, &identity("my-dark"), &document).expect("valid document");

        assert_eq!(resolved.identity.as_str(), "my-dark");
        assert_eq!(resolved.name, "Custom");
        assert_eq!(resolved.appearance, Appearance::Dark);
        assert_eq!(resolved.grid_background, built_ins[0].grid_background);
        assert_eq!(resolved.source_function, built_ins[0].source_function);
        assert_eq!(resolved.grid_border_width, built_ins[0].grid_border_width);
        assert_eq!(
            resolved.overview_source_view,
            built_ins[0].overview_source_view
        );
        assert_eq!(
            resolved.overview_source_view_width,
            built_ins[0].overview_source_view_width
        );
    }

    #[test]
    fn explicit_color_overrides_the_parent() {
        let built_ins = [default_theme(Appearance::Dark)];
        let overridden = Color32::from_rgb(1, 2, 3);
        let document = ThemeDocument {
            colors: vec![(ColorKey::GridBackground, overridden)],
            ..child(&dark_parent())
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
        let built_ins = [default_theme(Appearance::Dark)];
        let retuned = Color32::from_rgb(9, 9, 9);
        let document = ThemeDocument {
            colors: vec![(ColorKey::SourceFunction, retuned)],
            ..child(&dark_parent())
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
        let built_ins = [default_theme(Appearance::Dark)];
        let document = ThemeDocument {
            grid_widths: vec![(
                GridWidthKey::GridBorder,
                GridWidth::from_points(0.25).expect("0.25 is within 0..=1"),
            )],
            chrome_widths: vec![(
                ChromeWidthKey::PanelBorder,
                ChromeWidth::from_points(1.5).expect("1.5 is within 0..=2"),
            )],
            ..child(&dark_parent())
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
        let built_ins = [default_theme(Appearance::Dark)];
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
        let built_ins = [default_theme(Appearance::Dark)];
        let grandparent = child(&dark_parent());
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
        let built_ins = [default_theme(Appearance::Dark)];
        let document = child(&dark_parent());

        let error = resolve(
            &built_ins,
            &ThemeIdentity::default_for(Appearance::Dark),
            &document,
        )
        .unwrap_err();

        assert_eq!(
            error,
            ThemeError::ReservedIdentity {
                identity: ThemeIdentity::default_for(Appearance::Dark)
            }
        );
    }

    #[test]
    fn appearance_mismatch_is_rejected() {
        let built_ins = [default_theme(Appearance::Dark)];
        let document = ThemeDocument {
            appearance: Some(Appearance::Light),
            ..child(&dark_parent())
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
        let built_ins = [default_theme(Appearance::Dark)];
        let document = ThemeDocument {
            appearance: Some(Appearance::Dark),
            ..child(&dark_parent())
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
    /// Grid surface, which the chrome mapping keeps distinct
    /// (`window.background` for the opaque backdrop,
    /// `panel.background`/`grid.background` for the surfaces that may carry
    /// alpha). Both a partial alpha and a fully transparent one (`0`) are
    /// refused the same way — the rule names one exact value, not merely
    /// "some minimum visibility."
    ///
    #[test]
    fn resolve_rejects_a_nonopaque_window_background() {
        let built_ins = [default_theme(Appearance::Dark)];

        for (window_background, alpha) in [
            (Color32::from_rgba_unmultiplied(10, 20, 30, 200), 200),
            (Color32::TRANSPARENT, 0),
        ] {
            let document = ThemeDocument {
                colors: vec![(ColorKey::WindowBackground, window_background)],
                ..child(&dark_parent())
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
        let built_ins = [default_theme(Appearance::Dark)];
        let opaque = Color32::from_rgb(10, 20, 30);
        let document = ThemeDocument {
            colors: vec![(ColorKey::WindowBackground, opaque)],
            ..child(&dark_parent())
        };

        let resolved =
            resolve(&built_ins, &identity("my-dark"), &document).expect("opaque window.background");

        assert_eq!(resolved.window_background, opaque);
    }

    #[test]
    fn omitted_optional_colours_inherit_the_parent() {
        // One parent leaves every optional colour cleared and the other
        // sets each, so omission is seen to inherit both states.
        let cleared = Theme {
            cursor_background: None,
            cursor_foreground: None,
            region_cursor_background: None,
            region_cursor_foreground: None,
            ..default_theme(Appearance::Dark)
        };
        let coloured = Theme {
            cursor_background: Some(Color32::from_rgb(1, 2, 3)),
            cursor_foreground: Some(Color32::from_rgb(7, 8, 9)),
            region_cursor_background: Some(Color32::from_rgb(4, 5, 6)),
            region_cursor_foreground: Some(Color32::from_rgb(9, 8, 7)),
            ..default_theme(Appearance::Dark)
        };
        for parent in [cleared, coloured] {
            let built_ins = [parent];
            let document = child(&dark_parent());

            let resolved =
                resolve(&built_ins, &identity("my-dark"), &document).expect("valid document");

            assert_eq!(resolved.cursor_background, built_ins[0].cursor_background);
            assert_eq!(resolved.cursor_foreground, built_ins[0].cursor_foreground);
            assert_eq!(
                resolved.region_cursor_background,
                built_ins[0].region_cursor_background
            );
            assert_eq!(
                resolved.region_cursor_foreground,
                built_ins[0].region_cursor_foreground
            );
        }
    }

    #[test]
    fn explicit_none_clears_the_optional_colour_even_over_a_parent_colour() {
        let mut with_fill = default_theme(Appearance::Dark);
        with_fill.cursor_background = Some(Color32::from_rgb(1, 2, 3));
        with_fill.cursor_foreground = Some(Color32::from_rgb(7, 8, 9));
        with_fill.region_cursor_background = Some(Color32::from_rgb(4, 5, 6));
        with_fill.region_cursor_foreground = Some(Color32::from_rgb(9, 8, 7));
        let built_ins = [with_fill];
        let document = ThemeDocument {
            cursor_background: Some(OptionalColor::None),
            cursor_foreground: Some(OptionalColor::None),
            region_cursor_background: Some(OptionalColor::None),
            region_cursor_foreground: Some(OptionalColor::None),
            ..child(&dark_parent())
        };

        let resolved =
            resolve(&built_ins, &identity("my-dark"), &document).expect("valid document");

        assert_eq!(resolved.cursor_background, None);
        assert_eq!(resolved.cursor_foreground, None);
        assert_eq!(resolved.region_cursor_background, None);
        assert_eq!(resolved.region_cursor_foreground, None);
    }

    ///
    /// An explicit transparent colour is a supplied value, not the `none`
    /// clear state — `schema.md`: "Transparent supplied colour suppresses
    /// fallback; it remains distinct from absence."
    ///
    #[test]
    fn explicit_transparent_colour_is_a_supplied_value_not_a_clear() {
        let built_ins = [default_theme(Appearance::Dark)];
        let document = ThemeDocument {
            cursor_background: Some(OptionalColor::Color(Color32::TRANSPARENT)),
            cursor_foreground: Some(OptionalColor::Color(Color32::TRANSPARENT)),
            region_cursor_background: Some(OptionalColor::Color(Color32::TRANSPARENT)),
            region_cursor_foreground: Some(OptionalColor::Color(Color32::TRANSPARENT)),
            ..child(&dark_parent())
        };

        let resolved =
            resolve(&built_ins, &identity("my-dark"), &document).expect("valid document");

        assert_eq!(resolved.cursor_background, Some(Color32::TRANSPARENT));
        assert_eq!(resolved.cursor_foreground, Some(Color32::TRANSPARENT));
        assert_eq!(
            resolved.region_cursor_background,
            Some(Color32::TRANSPARENT)
        );
        assert_eq!(
            resolved.region_cursor_foreground,
            Some(Color32::TRANSPARENT)
        );
        assert_ne!(resolved.cursor_background, None);
    }

    #[test]
    fn explicit_optional_colour_is_stored_including_opaque() {
        let built_ins = [default_theme(Appearance::Dark)];
        let colour = Color32::from_rgb(10, 20, 30);
        let document = ThemeDocument {
            cursor_background: Some(OptionalColor::Color(colour)),
            cursor_foreground: Some(OptionalColor::Color(colour)),
            region_cursor_foreground: Some(OptionalColor::Color(colour)),
            ..child(&dark_parent())
        };

        let resolved =
            resolve(&built_ins, &identity("my-dark"), &document).expect("valid document");

        assert_eq!(resolved.cursor_background, Some(colour));
        assert_eq!(resolved.cursor_foreground, Some(colour));
        assert_eq!(resolved.region_cursor_foreground, Some(colour));
    }

    #[test]
    fn theme_color_reads_the_key_a_document_wrote() {
        let built_ins = [default_theme(Appearance::Dark)];
        let overridden = Color32::from_rgb(7, 7, 7);
        let document = ThemeDocument {
            colors: vec![(ColorKey::Text, overridden)],
            ..child(&dark_parent())
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
        let built_ins = [default_theme(Appearance::Dark)];
        // Each key beside the field a reader addresses by name, written here
        // rather than taken from `style_catalogue!`, so a key routed to the
        // wrong field fails even though the macro's reader and writer agree.
        // The catalogue's count is what says the list is complete.
        let fields: [(ColorKey, Field<Color32>); 54] = [
            (ColorKey::WindowBackground, |theme| theme.window_background),
            (ColorKey::PanelBackground, |theme| theme.panel_background),
            (ColorKey::GridBackground, |theme| theme.grid_background),
            (ColorKey::CellBackground, |theme| theme.cell_background),
            (ColorKey::SourceOrdinary, |theme| theme.source_ordinary),
            (ColorKey::SourceComment, |theme| theme.source_comment),
            (ColorKey::SourceNumber, |theme| theme.source_number),
            (ColorKey::SourceNote, |theme| theme.source_note),
            (ColorKey::SourceFunction, |theme| theme.source_function),
            (ColorKey::SourceBang, |theme| theme.source_bang),
            (ColorKey::SourceOrdinaryBackground, |theme| {
                theme.source_ordinary_background
            }),
            (ColorKey::SourceCommentBackground, |theme| {
                theme.source_comment_background
            }),
            (ColorKey::SourceNumberBackground, |theme| {
                theme.source_number_background
            }),
            (ColorKey::SourceNoteBackground, |theme| {
                theme.source_note_background
            }),
            (ColorKey::SourceFunctionBackground, |theme| {
                theme.source_function_background
            }),
            (ColorKey::SourceBangBackground, |theme| {
                theme.source_bang_background
            }),
            (ColorKey::SourceOrdinaryBorder, |theme| {
                theme.source_ordinary_border
            }),
            (ColorKey::SourceCommentBorder, |theme| {
                theme.source_comment_border
            }),
            (ColorKey::SourceNumberBorder, |theme| {
                theme.source_number_border
            }),
            (ColorKey::SourceNoteBorder, |theme| theme.source_note_border),
            (ColorKey::SourceFunctionBorder, |theme| {
                theme.source_function_border
            }),
            (ColorKey::SourceBangBorder, |theme| theme.source_bang_border),
            (ColorKey::DiagnosticForeground, |theme| {
                theme.diagnostic_foreground
            }),
            (ColorKey::DiagnosticBackground, |theme| {
                theme.diagnostic_background
            }),
            (ColorKey::DiagnosticBorder, |theme| theme.diagnostic_border),
            (ColorKey::OutputPortalForeground, |theme| {
                theme.output_portal_foreground
            }),
            (ColorKey::OutputPortalBackground, |theme| {
                theme.output_portal_background
            }),
            (ColorKey::OutputPortalBorder, |theme| {
                theme.output_portal_border
            }),
            (ColorKey::InputPortalForeground, |theme| {
                theme.input_portal_foreground
            }),
            (ColorKey::InputPortalBackground, |theme| {
                theme.input_portal_background
            }),
            (ColorKey::InputPortalBorder, |theme| {
                theme.input_portal_border
            }),
            (ColorKey::LockedForeground, |theme| theme.locked_foreground),
            (ColorKey::LockedBackground, |theme| theme.locked_background),
            (ColorKey::LockedBorder, |theme| theme.locked_border),
            (ColorKey::GridBorder, |theme| theme.grid_border),
            (ColorKey::SectorSeam, |theme| theme.sector_seam),
            (ColorKey::CursorBorder, |theme| theme.cursor_border),
            (ColorKey::RegionBorder, |theme| theme.region_border),
            (ColorKey::RegionBackground, |theme| theme.region_background),
            (ColorKey::PanelBorder, |theme| theme.panel_border),
            (ColorKey::SelectionBackground, |theme| {
                theme.selection_background
            }),
            (ColorKey::SelectionBorder, |theme| theme.selection_border),
            (ColorKey::SelectionBorderRest, |theme| {
                theme.selection_border_rest
            }),
            (ColorKey::WidgetInactiveBorder, |theme| {
                theme.widget_inactive_border
            }),
            (ColorKey::Text, |theme| theme.text),
            (ColorKey::TextActive, |theme| theme.text_active),
            (ColorKey::TextMuted, |theme| theme.text_muted),
            (ColorKey::InputBackground, |theme| theme.input_background),
            (ColorKey::Link, |theme| theme.link),
            (ColorKey::CodeBackground, |theme| theme.code_background),
            (ColorKey::InputCursor, |theme| theme.input_cursor),
            (ColorKey::Error, |theme| theme.error),
            (ColorKey::Warning, |theme| theme.warning),
            (ColorKey::OverviewSourceView, |theme| {
                theme.overview_source_view
            }),
        ];
        assert_eq!(fields.len(), CATALOGUE_COUNTS.colors);

        for &(key, field) in &fields {
            let colour = Color32::from_rgb(1, 2, 3);
            let document = ThemeDocument {
                colors: vec![(key, colour)],
                ..child(&dark_parent())
            };

            let resolved =
                resolve(&built_ins, &identity("my-dark"), &document).expect("valid document");

            assert_eq!(
                field(&resolved),
                colour,
                "{key:?} did not take its override"
            );
            assert_eq!(resolved.color(key), colour, "{key:?} reads another field");
            for &(other, other_field) in &fields {
                if other != key {
                    assert_eq!(
                        other_field(&resolved),
                        other_field(&built_ins[0]),
                        "{key:?}'s override changed {other:?}"
                    );
                }
            }
        }
    }

    ///
    /// The Grid/chrome-bounded width counterpart of
    /// `every_color_key_can_be_independently_overridden`, and what exercises
    /// [`Theme::grid_width`] and [`Theme::chrome_width`], the by-key
    /// readers.
    ///
    #[test]
    fn every_grid_and_chrome_width_key_can_be_independently_overridden() {
        let built_ins = [default_theme(Appearance::Dark)];
        // Each key beside the field a reader addresses by name, as in
        // `every_color_key_can_be_independently_overridden`.
        let grid_fields: [(GridWidthKey, Field<f32>); 15] = [
            (GridWidthKey::GridBorder, |theme| {
                theme.grid_border_width.points()
            }),
            (GridWidthKey::SectorSeam, |theme| {
                theme.sector_seam_width.points()
            }),
            (GridWidthKey::CellSelectionBorder, |theme| {
                theme.cell_selection_border_width.points()
            }),
            (GridWidthKey::CursorBorder, |theme| {
                theme.cursor_border_width.points()
            }),
            (GridWidthKey::RegionBorder, |theme| {
                theme.region_border_width.points()
            }),
            (GridWidthKey::DiagnosticBorder, |theme| {
                theme.diagnostic_border_width.points()
            }),
            (GridWidthKey::OutputPortalBorder, |theme| {
                theme.output_portal_border_width.points()
            }),
            (GridWidthKey::InputPortalBorder, |theme| {
                theme.input_portal_border_width.points()
            }),
            (GridWidthKey::LockedBorder, |theme| {
                theme.locked_border_width.points()
            }),
            (GridWidthKey::SourceOrdinaryBorder, |theme| {
                theme.source_ordinary_border_width.points()
            }),
            (GridWidthKey::SourceCommentBorder, |theme| {
                theme.source_comment_border_width.points()
            }),
            (GridWidthKey::SourceNumberBorder, |theme| {
                theme.source_number_border_width.points()
            }),
            (GridWidthKey::SourceNoteBorder, |theme| {
                theme.source_note_border_width.points()
            }),
            (GridWidthKey::SourceFunctionBorder, |theme| {
                theme.source_function_border_width.points()
            }),
            (GridWidthKey::SourceBangBorder, |theme| {
                theme.source_bang_border_width.points()
            }),
        ];
        let chrome_fields: [(ChromeWidthKey, Field<f32>); 6] = [
            (ChromeWidthKey::PanelBorder, |theme| {
                theme.panel_border_width.points()
            }),
            (ChromeWidthKey::SelectionBorder, |theme| {
                theme.selection_border_width.points()
            }),
            (ChromeWidthKey::WidgetBorder, |theme| {
                theme.widget_border_width.points()
            }),
            (ChromeWidthKey::WidgetInactiveBorder, |theme| {
                theme.widget_inactive_border_width.points()
            }),
            (ChromeWidthKey::InputCursor, |theme| {
                theme.input_cursor_width.points()
            }),
            (ChromeWidthKey::OverviewSourceView, |theme| {
                theme.overview_source_view_width.points()
            }),
        ];
        assert_eq!(grid_fields.len(), CATALOGUE_COUNTS.grid_widths);
        assert_eq!(chrome_fields.len(), CATALOGUE_COUNTS.chrome_widths);
        let every_field: Vec<Field<f32>> = grid_fields
            .iter()
            .map(|&(_, field)| field)
            .chain(chrome_fields.iter().map(|&(_, field)| field))
            .collect();
        // Every width but the one at `overridden` in `every_field` is the
        // parent's.
        let unchanged_but = |resolved: &Theme, overridden: usize, key: &dyn std::fmt::Debug| {
            for (index, field) in every_field.iter().enumerate() {
                if index != overridden {
                    assert_eq!(
                        field(resolved),
                        field(&built_ins[0]),
                        "{key:?}'s override changed another width"
                    );
                }
            }
        };

        for (index, &(key, field)) in grid_fields.iter().enumerate() {
            let document = ThemeDocument {
                grid_widths: vec![(
                    key,
                    GridWidth::from_points(0.1).expect("0.1 is within 0..=1"),
                )],
                ..child(&dark_parent())
            };
            let resolved =
                resolve(&built_ins, &identity("my-dark"), &document).expect("valid document");
            assert_eq!(field(&resolved), 0.1, "{key:?} did not take its override");
            assert_eq!(
                resolved.grid_width(key).points(),
                0.1,
                "{key:?} reads another field"
            );
            unchanged_but(&resolved, index, &key);
        }

        for (index, &(key, field)) in chrome_fields.iter().enumerate() {
            let document = ThemeDocument {
                chrome_widths: vec![(
                    key,
                    ChromeWidth::from_points(1.9).expect("1.9 is within 0..=2"),
                )],
                ..child(&dark_parent())
            };
            let resolved =
                resolve(&built_ins, &identity("my-dark"), &document).expect("valid document");
            assert_eq!(field(&resolved), 1.9, "{key:?} did not take its override");
            assert_eq!(
                resolved.chrome_width(key).points(),
                1.9,
                "{key:?} reads another field"
            );
            unchanged_but(&resolved, grid_fields.len() + index, &key);
        }
    }

    ///
    /// `schema.md`'s colour table, restated. The decoder's style table is
    /// derived from the same spellings, so a spelling wrong here is a
    /// property no document can set — or, swapped with a neighbour, one that
    /// sets the wrong field.
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
            (ColorKey::DiagnosticForeground, "diagnostic.foreground"),
            (ColorKey::DiagnosticBackground, "diagnostic.background"),
            (ColorKey::DiagnosticBorder, "diagnostic.border"),
            (ColorKey::OutputPortalForeground, "output_portal.foreground"),
            (ColorKey::OutputPortalBackground, "output_portal.background"),
            (ColorKey::OutputPortalBorder, "output_portal.border"),
            (ColorKey::InputPortalForeground, "input_portal.foreground"),
            (ColorKey::InputPortalBackground, "input_portal.background"),
            (ColorKey::InputPortalBorder, "input_portal.border"),
            (ColorKey::LockedForeground, "locked.foreground"),
            (ColorKey::LockedBackground, "locked.background"),
            (ColorKey::LockedBorder, "locked.border"),
            (ColorKey::GridBorder, "grid.border"),
            (ColorKey::SectorSeam, "sector.seam"),
            (ColorKey::CursorBorder, "cursor.border"),
            (ColorKey::RegionBorder, "region.border"),
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
            (ColorKey::OverviewSourceView, "overview.source_view"),
        ];
        assert_eq!(catalogue.len(), 48);
        for (key, name) in catalogue {
            assert_eq!(key.name(), name);
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
        assert_eq!(
            GridWidthKey::InputPortalBorder.name(),
            "input_portal.border.width"
        );
        assert_eq!(GridWidthKey::LockedBorder.name(), "locked.border.width");
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
        assert_eq!(
            ChromeWidthKey::OverviewSourceView.name(),
            "overview.source_view.width"
        );
    }
}
