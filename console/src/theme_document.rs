//!
//! Decodes one Orcvs Theme document's bytes into a [`ThemeDocument`]:
//! `schema.md`'s "Decoding and limits", with no file I/O and no inheritance.
//!
//! The file name's extension selects exactly one decoder — `toml` for
//! `.toml`, `serde_json` for `.json`, `serde-saphyr` for `.yaml`/`.yml` —
//! and each decodes straight into the one strict document type below. That
//! type, not the decoder, decides what is strict, so the three formats
//! behave alike:
//!
//! - the root denies unknown fields, and a repeated root field is an error;
//! - every scalar is read through `deserialize_any` by a visitor that
//!   accepts only its own kind, because `serde-saphyr` 1.3.0's typed numeric
//!   paths parse a quoted scalar as a number and `deserialize_any` respects
//!   the quoting;
//! - `style` is read by a map visitor whose literal, case-sensitive dotted
//!   keys select each value's kind from the catalogue, and which refuses an
//!   unknown or repeated property itself, because `serde_json` otherwise
//!   keeps the last duplicate.
//!
//! Semantic checks that need only the value — the format marker, the
//! version, label limits, colour syntax, `none`'s two properties, finite
//! widths — run inside those visitors, so every decoder attaches its own line
//! and column to the error. Width bounds and parent/appearance agreement are
//! [`crate::theme::resolve`]'s, which the caller runs with the identity it
//! took from the file name.
//!

use std::fmt;
use std::path::Path;

use serde::Deserialize;
use serde::de::{self, DeserializeSeed, Deserializer, Error as _, MapAccess, Visitor};

use crate::theme::{
    Appearance, ChromeWidth, ChromeWidthKey, ColorKey, GridWidth, GridWidthKey, OptionalFill,
    ThemeDocument,
};

/// `schema.md`'s per-document byte limit, checked before any decoding.
pub(crate) const MAX_DOCUMENT_BYTES: usize = 1024 * 1024;

/// The longest decoder message a [`DocumentError`] keeps, in UTF-8 bytes.
const MAX_MESSAGE_BYTES: usize = 1024;

/// The longest `name` or `inherits` value, in UTF-8 bytes.
const MAX_LABEL_BYTES: usize = 256;

const FORMAT_MARKER: &str = "orcvs-theme";
const VERSION: u64 = 1;
const CURSOR_BACKGROUND: &str = "cursor.background";
const REGION_CURSOR_BACKGROUND: &str = "region.cursor.background";

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
    /// The extension is not `.toml`, `.json`, `.yaml` or `.yml`, in any case.
    UnsupportedExtension,
    /// The document is larger than [`MAX_DOCUMENT_BYTES`].
    TooLarge { bytes: usize },
    /// The bytes are not UTF-8; `valid_up_to` is the first bad byte's offset.
    NotUtf8 { valid_up_to: usize },
    /// The format's decoder refused the document, or a check inside the
    /// document type did. `message` is the decoder's own, with its line and
    /// column — for TOML, restated without the echoed source line — and
    /// names the offending property where one is known. It is capped at
    /// [`MAX_MESSAGE_BYTES`].
    Invalid {
        format: &'static str,
        message: String,
    },
}

impl fmt::Display for DocumentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let file_name = &self.file_name;
        match &self.reason {
            DocumentErrorReason::UnsupportedExtension => write!(
                f,
                "{file_name}: unsupported Theme file extension; expected .toml, .json, .yaml or \
                 .yml"
            ),
            DocumentErrorReason::TooLarge { bytes } => write!(
                f,
                "{file_name}: Theme document is {bytes} bytes, over the {MAX_DOCUMENT_BYTES}-byte \
                 limit"
            ),
            DocumentErrorReason::NotUtf8 { valid_up_to } => write!(
                f,
                "{file_name}: Theme document is not UTF-8 (invalid byte at offset {valid_up_to})"
            ),
            DocumentErrorReason::Invalid { format, message } => {
                write!(f, "{file_name}: invalid {format} Theme document: {message}")
            }
        }
    }
}

// === Entry point ===

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Format {
    Toml,
    Json,
    Yaml,
}

impl Format {
    fn of(file_name: &str) -> Option<Self> {
        let extension = Path::new(file_name).extension()?.to_str()?;
        [
            ("toml", Self::Toml),
            ("json", Self::Json),
            ("yaml", Self::Yaml),
            ("yml", Self::Yaml),
        ]
        .into_iter()
        .find(|(spelling, _)| extension.eq_ignore_ascii_case(spelling))
        .map(|(_, format)| format)
    }

    fn label(self) -> &'static str {
        match self {
            Self::Toml => "TOML",
            Self::Json => "JSON",
            Self::Yaml => "YAML",
        }
    }
}

///
/// Decodes one Theme document. `file_name` is used only for its extension,
/// which selects the decoder case-insensitively, and to label the error; the
/// caller derives the identity from it separately and passes the result to
/// [`crate::theme::resolve`]. Performs no I/O.
///
/// Refuses, in order: an unsupported extension, more than
/// [`MAX_DOCUMENT_BYTES`], and non-UTF-8 bytes. One leading U+FEFF is then
/// stripped, since `serde_json` would reject it, and the text is decoded
/// whole — a document is accepted entirely or not at all.
///
pub(crate) fn decode(file_name: &str, bytes: &[u8]) -> Result<ThemeDocument, DocumentError> {
    let error = |reason| DocumentError {
        file_name: file_name.to_owned(),
        reason,
    };

    let format =
        Format::of(file_name).ok_or_else(|| error(DocumentErrorReason::UnsupportedExtension))?;
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
            format: format.label(),
            message: bounded(message),
        })
    };
    let Root(raw) = match format {
        Format::Toml => toml::from_str(text).map_err(|e| invalid(toml_message(&e, text)))?,
        Format::Json => serde_json::from_str(text).map_err(|e| invalid(e.to_string()))?,
        Format::Yaml => serde_saphyr::from_str_with_options(text, yaml_options())
            .map_err(|e| invalid(e.to_string()))?,
    };
    Ok(raw.into_document())
}

///
/// `toml`'s `Display` renders a snippet that echoes the whole offending
/// source line, which in a one-line document is the document. This keeps
/// the decoder's message and states its position the way the other two
/// decoders do: 1-based line, and column counted in characters.
///
fn toml_message(error: &toml::de::Error, text: &str) -> String {
    let message = error.message();
    let Some(before) = error.span().and_then(|span| text.get(..span.start)) else {
        return message.to_owned();
    };
    let line = before.matches('\n').count() + 1;
    let line_start = before.rfind('\n').map_or(0, |newline| newline + 1);
    let column = before[line_start..].chars().count() + 1;
    format!("line {line}, column {column}: {message}")
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

///
/// `schema.md`'s YAML options: duplicate keys, merge keys and unsupported
/// tags are errors, and YAML 1.1's `yes`/`no`/`on`/`off` are not booleans.
/// Everything else —
/// the parse budget, the alias limits, rejecting non-finite typeless floats
/// — is the crate's default, kept deliberately.
///
fn yaml_options() -> serde_saphyr::Options {
    let mut options = serde_saphyr::Options::default();
    options.duplicate_keys = serde_saphyr::DuplicateKeyPolicy::Error;
    options.merge_keys = serde_saphyr::MergeKeyPolicy::Error;
    options.strict_booleans = true;
    options.reject_unsupported_tags = true;
    // A plain `line L column C: message`, without the rendered source
    // snippet, which quotes the document back.
    options.with_snippet = false;
    options
}

// === Document type ===

///
/// The document's root, which must be a mapping. A derived struct also
/// takes a sequence of its fields in declaration order, and `serde_json`
/// offers one, so the root asks for a map and hands [`RawDocument`] only
/// that map's entries.
///
struct Root(RawDocument);

impl<'de> Deserialize<'de> for Root {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_map(RootVisitor).map(Self)
    }
}

struct RootVisitor;

impl<'de> Visitor<'de> for RootVisitor {
    type Value = RawDocument;

    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("a table of Theme document fields")
    }

    fn visit_map<A: MapAccess<'de>>(self, map: A) -> Result<RawDocument, A::Error> {
        RawDocument::deserialize(de::value::MapAccessDeserializer::new(map))
    }
}

///
/// The one strict document type every format decodes into. Field names are
/// `schema.md`'s, case-sensitive; `format` and `version` hold nothing once
/// checked.
///
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawDocument {
    #[serde(rename = "format")]
    _format: FormatMarker,
    #[serde(rename = "version")]
    _version: Version,
    #[serde(deserialize_with = "name")]
    name: String,
    #[serde(deserialize_with = "inherits")]
    inherits: String,
    #[serde(default, deserialize_with = "appearance")]
    appearance: Option<Appearance>,
    #[serde(default)]
    style: Style,
}

impl RawDocument {
    ///
    /// Moves the checked fields into a [`ThemeDocument`], with each property
    /// list in catalogue order. `toml` yields a table's keys sorted, while
    /// `serde_json` and `serde-saphyr` yield them in document order, and a
    /// document is the same document whichever format spelled it.
    ///
    fn into_document(self) -> ThemeDocument {
        let Style {
            mut colors,
            mut grid_widths,
            mut chrome_widths,
            cursor_background,
            region_cursor_background,
        } = self.style;
        colors.sort_unstable_by_key(|&(key, _)| key);
        grid_widths.sort_unstable_by_key(|&(key, _)| key);
        chrome_widths.sort_unstable_by_key(|&(key, _)| key);
        ThemeDocument {
            parent: self.inherits,
            name: self.name,
            appearance: self.appearance,
            colors,
            grid_widths,
            chrome_widths,
            cursor_background,
            region_cursor_background,
        }
    }
}

// === Scalars ===

///
/// Reads one string through `deserialize_any`, refusing every other kind,
/// then hands it to `check` — which names the property in its error.
///
struct StrictString<F> {
    expecting: &'static str,
    check: F,
}

impl<'de, T, F: FnOnce(&str) -> Result<T, String>> Visitor<'de> for StrictString<F> {
    type Value = T;

    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.expecting)
    }

    fn visit_str<E: de::Error>(self, value: &str) -> Result<T, E> {
        (self.check)(value).map_err(E::custom)
    }
}

fn strict_string<'de, D, T>(
    deserializer: D,
    expecting: &'static str,
    check: impl FnOnce(&str) -> Result<T, String>,
) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
{
    deserializer.deserialize_any(StrictString { expecting, check })
}

struct FormatMarker;

impl<'de> Deserialize<'de> for FormatMarker {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        strict_string(
            deserializer,
            "the string \"orcvs-theme\" for `format`",
            |value| {
                if value == FORMAT_MARKER {
                    Ok(Self)
                } else {
                    Err(format!(
                        "`format` is {value:?}; an Orcvs Theme document declares \
                     format = \"{FORMAT_MARKER}\""
                    ))
                }
            },
        )
    }
}

struct Version;

fn unsupported_version<E: de::Error>(value: impl fmt::Display) -> E {
    E::custom(format!(
        "`version` is {value}; this build reads Theme format version {VERSION} only"
    ))
}

impl<'de> Deserialize<'de> for Version {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct VersionVisitor;

        impl Visitor<'_> for VersionVisitor {
            type Value = Version;

            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "the integer {VERSION} for `version`")
            }

            fn visit_i64<E: de::Error>(self, value: i64) -> Result<Version, E> {
                match u64::try_from(value) {
                    Ok(value) => self.visit_u64(value),
                    Err(_) => Err(unsupported_version(value)),
                }
            }

            fn visit_u64<E: de::Error>(self, value: u64) -> Result<Version, E> {
                if value == VERSION {
                    Ok(Version)
                } else {
                    Err(unsupported_version(value))
                }
            }
        }

        deserializer.deserialize_any(VersionVisitor)
    }
}

/// `name` and `inherits`: nonempty, no control characters, at most
/// [`MAX_LABEL_BYTES`] UTF-8 bytes. Too long is an error, never a truncation.
fn label(property: &'static str, value: &str) -> Result<String, String> {
    if value.is_empty() {
        return Err(format!("`{property}` is empty"));
    }
    if value.chars().any(char::is_control) {
        return Err(format!("`{property}` contains a control character"));
    }
    if value.len() > MAX_LABEL_BYTES {
        return Err(format!(
            "`{property}` is {} UTF-8 bytes, over the {MAX_LABEL_BYTES}-byte limit",
            value.len()
        ));
    }
    Ok(value.to_owned())
}

fn name<'de, D: Deserializer<'de>>(deserializer: D) -> Result<String, D::Error> {
    strict_string(deserializer, "a string for `name`", |value| {
        label("name", value)
    })
}

fn inherits<'de, D: Deserializer<'de>>(deserializer: D) -> Result<String, D::Error> {
    strict_string(deserializer, "a string for `inherits`", |value| {
        label("inherits", value)
    })
}

fn appearance<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Option<Appearance>, D::Error> {
    strict_string(
        deserializer,
        "the string \"dark\" or \"light\" for `appearance`",
        |value| match value {
            "dark" => Ok(Some(Appearance::Dark)),
            "light" => Ok(Some(Appearance::Light)),
            other => Err(format!(
                "`appearance` is {other:?}; expected \"dark\" or \"light\""
            )),
        },
    )
}

/// `#` and exactly 6 or 8 hexadecimal digits, straight RGB(A); six mean
/// opaque.
fn colour(property: &str, value: &str) -> Result<egui::Color32, String> {
    let malformed =
        || format!("`{property}` is {value:?}; a colour is \"#\" and 6 or 8 hexadecimal digits");
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

///
/// A width, read through `deserialize_any` as an integer or a float and
/// nothing else, and refused unless finite and within `max`: the inclusive
/// bound [`crate::theme::resolve`] applies too.
///
struct Width {
    property: &'static str,
    max: f32,
}

impl Width {
    fn check<E: de::Error>(&self, points: f64) -> Result<f32, E> {
        if !points.is_finite() {
            return Err(E::custom(format!(
                "`{}` is {points}; a width must be a finite number of points",
                self.property
            )));
        }
        // Checked as read, before narrowing to the `f32` `ThemeDocument`
        // stores: narrowing rounds a value just past a bound onto it, and
        // one beyond `f32`'s range to infinity, and neither may be clamped.
        if !(0.0..=f64::from(self.max)).contains(&points) {
            return Err(E::custom(format!(
                "`{}` is {points:?}; a width must be 0 to {} points",
                self.property, self.max
            )));
        }
        Ok(points as f32)
    }
}

impl<'de> DeserializeSeed<'de> for Width {
    type Value = f32;

    fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> Result<f32, D::Error> {
        deserializer.deserialize_any(self)
    }
}

impl Visitor<'_> for Width {
    type Value = f32;

    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "a number of points for `{}`", self.property)
    }

    // A width is a few points: an integer large enough to lose precision is
    // refused by the bound whatever it rounds to.
    fn visit_i64<E: de::Error>(self, value: i64) -> Result<f32, E> {
        self.check(value as f64)
    }

    fn visit_u64<E: de::Error>(self, value: u64) -> Result<f32, E> {
        self.check(value as f64)
    }

    fn visit_f64<E: de::Error>(self, value: f64) -> Result<f32, E> {
        self.check(value)
    }
}

// === Style ===

///
/// One `style` property: the key's spelling selects its value kind.
///
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Property {
    Color(ColorKey),
    GridWidth(GridWidthKey),
    ChromeWidth(ChromeWidthKey),
    CursorBackground,
    RegionCursorBackground,
}

impl Property {
    fn from_name(name: &str) -> Option<Self> {
        match name {
            CURSOR_BACKGROUND => Some(Self::CursorBackground),
            REGION_CURSOR_BACKGROUND => Some(Self::RegionCursorBackground),
            _ => ColorKey::from_name(name)
                .map(Self::Color)
                .or_else(|| GridWidthKey::from_name(name).map(Self::GridWidth))
                .or_else(|| ChromeWidthKey::from_name(name).map(Self::ChromeWidth)),
        }
    }

    ///
    /// The most separators — `.` or `_` — any property name holds:
    /// `output_portal.border.width` has three.
    ///
    const MOST_SEPARATORS: usize = 3;

    ///
    /// The property `name` spells in another case, or with any of `.`, `_`
    /// or `-` for each of its separators: the suggestion an unknown name's
    /// error makes. Each way of reading the separators as `.` or `_` is
    /// looked up, so `output-portal.border` finds `output_portal.border`; a
    /// name with more separators than any property holds suggests nothing,
    /// which bounds the lookups at eight.
    ///
    fn spelled(name: &str) -> Option<Self> {
        let lower = name.to_lowercase();
        let separators = lower.matches(['.', '_', '-']).count();
        if separators > Self::MOST_SEPARATORS {
            return None;
        }
        (0..1_u32 << separators).find_map(|underscores| {
            let mut separator = 0;
            let candidate = lower
                .chars()
                .map(|c| match c {
                    '.' | '_' | '-' => {
                        let underscore = (underscores >> separator) & 1 == 1;
                        separator += 1;
                        if underscore { '_' } else { '.' }
                    }
                    c => c,
                })
                .collect::<String>();
            Self::from_name(&candidate)
        })
    }

    fn name(self) -> &'static str {
        match self {
            Self::Color(key) => key.name(),
            Self::GridWidth(key) => key.name(),
            Self::ChromeWidth(key) => key.name(),
            Self::CursorBackground => CURSOR_BACKGROUND,
            Self::RegionCursorBackground => REGION_CURSOR_BACKGROUND,
        }
    }
}

impl<'de> Deserialize<'de> for Property {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        strict_string(deserializer, "a style property name", |name| {
            Self::from_name(name).ok_or_else(|| {
                let form = "a property name is one lower-case, dot-separated key, such as \
                            \"grid.background\", and is case-sensitive";
                match Self::spelled(name) {
                    Some(property) => format!(
                        "unknown style property {name:?}; did you mean {:?}? {form}",
                        property.name()
                    ),
                    None => format!("unknown style property {name:?}; {form}"),
                }
            })
        })
    }
}

/// A colour property's value.
struct ColourValue {
    property: &'static str,
}

impl<'de> DeserializeSeed<'de> for ColourValue {
    type Value = egui::Color32;

    fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> Result<Self::Value, D::Error> {
        let property = self.property;
        strict_string(
            deserializer,
            "a \"#RRGGBB\" or \"#RRGGBBAA\" colour string",
            |value| match value {
                "none" => Err(format!(
                    "`{property}` is \"none\"; only `{CURSOR_BACKGROUND}` and \
                     `{REGION_CURSOR_BACKGROUND}` can be \"none\""
                )),
                value => colour(property, value),
            },
        )
    }
}

/// One of the two optional Cursor fills: a colour, or the string `none`.
struct OptionalFillValue {
    property: &'static str,
}

impl<'de> DeserializeSeed<'de> for OptionalFillValue {
    type Value = OptionalFill;

    fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> Result<Self::Value, D::Error> {
        let property = self.property;
        strict_string(
            deserializer,
            "a \"#RRGGBB\" or \"#RRGGBBAA\" colour string, or \"none\"",
            |value| match value {
                "none" => Ok(OptionalFill::None),
                value => colour(property, value).map(OptionalFill::Color),
            },
        )
    }
}

/// The decoded `style` table, in the shape [`ThemeDocument`] stores it.
#[derive(Default)]
struct Style {
    colors: Vec<(ColorKey, egui::Color32)>,
    grid_widths: Vec<(GridWidthKey, f32)>,
    chrome_widths: Vec<(ChromeWidthKey, f32)>,
    cursor_background: Option<OptionalFill>,
    region_cursor_background: Option<OptionalFill>,
}

impl Style {
    fn contains(&self, property: Property) -> bool {
        match property {
            Property::Color(key) => self.colors.iter().any(|&(held, _)| held == key),
            Property::GridWidth(key) => self.grid_widths.iter().any(|&(held, _)| held == key),
            Property::ChromeWidth(key) => self.chrome_widths.iter().any(|&(held, _)| held == key),
            Property::CursorBackground => self.cursor_background.is_some(),
            Property::RegionCursorBackground => self.region_cursor_background.is_some(),
        }
    }
}

impl<'de> Deserialize<'de> for Style {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_any(StyleVisitor)
    }
}

struct StyleVisitor;

impl<'de> Visitor<'de> for StyleVisitor {
    type Value = Style;

    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("a table of style properties for `style`")
    }

    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Style, A::Error> {
        let mut style = Style::default();
        while let Some(property) = map.next_key::<Property>()? {
            let name = property.name();
            if style.contains(property) {
                return Err(A::Error::custom(format!(
                    "style property `{name}` is repeated"
                )));
            }
            match property {
                Property::Color(key) => {
                    let colour = map.next_value_seed(ColourValue { property: name })?;
                    style.colors.push((key, colour));
                }
                Property::GridWidth(key) => {
                    let points = map.next_value_seed(Width {
                        property: name,
                        max: GridWidth::MAX_POINTS,
                    })?;
                    style.grid_widths.push((key, points));
                }
                Property::ChromeWidth(key) => {
                    let points = map.next_value_seed(Width {
                        property: name,
                        max: ChromeWidth::MAX_POINTS,
                    })?;
                    style.chrome_widths.push((key, points));
                }
                Property::CursorBackground => {
                    style.cursor_background =
                        Some(map.next_value_seed(OptionalFillValue { property: name })?);
                }
                Property::RegionCursorBackground => {
                    style.region_cursor_background =
                        Some(map.next_value_seed(OptionalFillValue { property: name })?);
                }
            }
        }
        Ok(style)
    }
}

#[cfg(test)]
mod tests {
    use egui::Color32;

    use super::{
        DocumentError, DocumentErrorReason, MAX_DOCUMENT_BYTES, MAX_MESSAGE_BYTES, Property, decode,
    };
    use crate::theme::{
        Appearance, ChromeWidthKey, ColorKey, GridWidthKey, OptionalFill, ThemeDocument,
        ThemeIdentity, okabe_ito, resolve, straight_rgba,
    };

    /// A custom Theme's identity, as a file named `stem` supplies it.
    fn identity(stem: &str) -> ThemeIdentity {
        ThemeIdentity::from_stem(stem).expect("a non-empty stem")
    }

    const MY_DARK_TOML: &str = include_str!("../../.scratch/theming/examples/my-dark.toml");
    const OKABE_ITO_COPY_TOML: &str =
        include_str!("../../.scratch/theming/examples/okabe-ito-copy.toml");

    /// `my-dark.toml` restated as JSON, in the same order.
    const MY_DARK_JSON: &str = r##"{
  "format": "orcvs-theme",
  "version": 1,
  "name": "My Dark",
  "inherits": "okabe-ito",
  "style": {
    "text": "#E0E0E0",
    "grid.background": "#101820",
    "grid.border": "#35506080",
    "grid.border.width": 0.5,
    "cursor.background": "none",
    "source.function.background": "#001912FF"
  }
}
"##;

    /// `my-dark.toml` restated as YAML. Colours are quoted: an unquoted `#`
    /// starts a YAML comment.
    const MY_DARK_YAML: &str = r##"format: orcvs-theme
version: 1
name: My Dark
inherits: okabe-ito
style:
  text: "#E0E0E0"
  grid.background: "#101820"
  grid.border: "#35506080"
  grid.border.width: 0.5
  cursor.background: none
  source.function.background: "#001912FF"
"##;

    fn my_dark() -> ThemeDocument {
        ThemeDocument {
            parent: "okabe-ito".to_owned(),
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
            grid_widths: vec![(GridWidthKey::GridBorder, 0.5)],
            chrome_widths: Vec::new(),
            cursor_background: Some(OptionalFill::None),
            region_cursor_background: None,
        }
    }

    // --- Documents, one builder per format ---

    const FORMATS: [&str; 3] = ["theme.toml", "theme.json", "theme.yaml"];

    ///
    /// A document with the four required root fields, `extra_root` spliced
    /// in after them, and `style` holding `entries` — each a property name
    /// and a value already written in that format's own syntax.
    ///
    fn document(file: &str, extra_root: &[(&str, &str)], entries: &[(&str, &str)]) -> String {
        let root = [
            ("format", quoted(file, "orcvs-theme")),
            ("version", "1".to_owned()),
            ("name", quoted(file, "T")),
            ("inherits", quoted(file, "okabe-ito")),
        ];
        let root = root
            .iter()
            .map(|(key, value)| (*key, value.as_str()))
            .chain(extra_root.iter().copied());
        if file.ends_with(".toml") {
            let mut text: String = root
                .map(|(key, value)| format!("{key} = {value}\n"))
                .collect();
            text.push_str("\n[style]\n");
            for (key, value) in entries {
                text.push_str(&format!("\"{key}\" = {value}\n"));
            }
            text
        } else if file.ends_with(".json") {
            let root: Vec<String> = root
                .map(|(key, value)| format!("\"{key}\": {value}"))
                .collect();
            let style: Vec<String> = entries
                .iter()
                .map(|(key, value)| format!("\"{key}\": {value}"))
                .collect();
            format!(
                "{{{}, \"style\": {{{}}}}}",
                root.join(", "),
                style.join(", ")
            )
        } else {
            let mut text: String = root
                .map(|(key, value)| format!("{key}: {value}\n"))
                .collect();
            if entries.is_empty() {
                text.push_str("style: {}\n");
            } else {
                text.push_str("style:\n");
                for (key, value) in entries {
                    text.push_str(&format!("  {key}: {value}\n"));
                }
            }
            text
        }
    }

    /// `value` as a quoted string in `file`'s format.
    fn quoted(file: &str, value: &str) -> String {
        if file.ends_with(".toml") || file.ends_with(".json") {
            format!("{value:?}")
        } else {
            format!("\"{value}\"")
        }
    }

    fn style(file: &str, entries: &[(&str, &str)]) -> String {
        document(file, &[], entries)
    }

    fn decodes(file: &str, text: &str) -> ThemeDocument {
        decode(file, text.as_bytes())
            .unwrap_or_else(|error| panic!("{file} should decode:\n{text}\n{error}"))
    }

    ///
    /// Asserts `text` is refused by `file`'s decoder or the document type,
    /// with every one of `needles` in the message, and returns the message.
    ///
    fn refuses(file: &str, text: &str, needles: &[&str]) -> String {
        let error =
            decode(file, text.as_bytes()).expect_err(&format!("{file} should be refused:\n{text}"));
        let DocumentErrorReason::Invalid { message, .. } = &error.reason else {
            panic!("{file}: expected a decode error, got {error:?}");
        };
        for needle in needles {
            assert!(
                message.contains(needle),
                "{file}: {needle:?} missing from the error:\n{message}\nfor:\n{text}"
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
        assert_eq!(decodes("my-dark.toml", MY_DARK_TOML), my_dark());
    }

    #[test]
    fn my_dark_decodes_identically_from_json_and_yaml() {
        assert_eq!(decodes("my-dark.json", MY_DARK_JSON), my_dark());
        assert_eq!(decodes("my-dark.yaml", MY_DARK_YAML), my_dark());
        assert_eq!(decodes("my-dark.yml", MY_DARK_YAML), my_dark());
    }

    ///
    /// `okabe-ito-copy.toml` spells out every catalogue property at the
    /// built-in's value, so resolving it must give back the built-in under
    /// the copy's identity and display label — which also pins every
    /// property name the decoder maps to the field `resolve` writes.
    ///
    #[test]
    fn the_okabe_ito_copy_example_resolves_to_the_built_in() {
        let document = decodes("okabe-ito-copy.toml", OKABE_ITO_COPY_TOML);
        assert_eq!(document.colors.len(), 45);
        assert_eq!(document.grid_widths.len(), 7);
        assert_eq!(document.chrome_widths.len(), 5);
        assert_eq!(document.cursor_background, Some(OptionalFill::None));
        assert_eq!(document.region_cursor_background, Some(OptionalFill::None));
        assert_eq!(document.appearance, Some(Appearance::Dark));

        let resolved =
            resolve(&[okabe_ito()], &identity("okabe-ito-copy"), &document).expect("resolves");

        let mut expected = okabe_ito();
        expected.identity = identity("okabe-ito-copy");
        expected.name = "Okabe-Ito copy".to_owned();
        assert_eq!(resolved, expected);
    }

    #[test]
    fn every_catalogue_property_name_selects_its_own_property() {
        let names = OKABE_ITO_COPY_TOML
            .lines()
            .filter_map(|line| line.strip_prefix('"')?.split_once('"'))
            .map(|(name, _)| name)
            .collect::<Vec<_>>();
        assert_eq!(names.len(), 59, "45 colours, 2 optional fills, 12 widths");
        let mut seen = Vec::new();
        for name in names {
            let property = Property::from_name(name).unwrap_or_else(|| panic!("{name}"));
            assert_eq!(property.name(), name);
            assert_eq!(Property::spelled(name), Some(property), "{name}");
            assert!(
                name.matches(['.', '_']).count() <= Property::MOST_SEPARATORS,
                "{name} has more separators than a suggestion searches"
            );
            assert!(!seen.contains(&property), "{name} maps to a property twice");
            seen.push(property);
        }
    }

    // --- Empty and omitted style ---

    #[test]
    fn an_omitted_or_empty_style_overrides_nothing() {
        let bare = ThemeDocument {
            parent: "okabe-ito".to_owned(),
            name: "T".to_owned(),
            ..ThemeDocument::default()
        };
        for file in FORMATS {
            assert_eq!(
                decodes(file, &style(file, &[])),
                bare,
                "{file}: empty style"
            );
        }
        let omitted = [
            (
                "theme.toml",
                "format = \"orcvs-theme\"\nversion = 1\nname = \"T\"\ninherits = \"okabe-ito\"\n",
            ),
            (
                "theme.json",
                r#"{"format": "orcvs-theme", "version": 1, "name": "T", "inherits": "okabe-ito"}"#,
            ),
            (
                "theme.yaml",
                "format: orcvs-theme\nversion: 1\nname: T\ninherits: okabe-ito\n",
            ),
        ];
        for (file, text) in omitted {
            assert_eq!(decodes(file, text), bare, "{file}: omitted style");
        }
    }

    ///
    /// YAML's `style:` with nothing after it is null, not an empty mapping,
    /// and null is never a valid value; `style: {}` is the empty spelling.
    ///
    #[test]
    fn a_null_style_is_refused() {
        refuses(
            "theme.yaml",
            "format: orcvs-theme\nversion: 1\nname: T\ninherits: okabe-ito\nstyle:\n",
            &["style", "line 5"],
        );
        refuses(
            "theme.json",
            r#"{"format": "orcvs-theme", "version": 1, "name": "T", "inherits": "okabe-ito", "style": null}"#,
            &["style"],
        );
    }

    // --- Optional fills ---

    #[test]
    fn optional_fills_are_omitted_cleared_or_a_supplied_transparent_colour() {
        for file in FORMATS {
            let omitted = decodes(file, &style(file, &[]));
            assert_eq!(omitted.cursor_background, None, "{file}");
            assert_eq!(omitted.region_cursor_background, None, "{file}");

            let none = quoted(file, "none");
            let cleared = decodes(
                file,
                &style(
                    file,
                    &[
                        ("cursor.background", &none),
                        ("region.cursor.background", &none),
                    ],
                ),
            );
            assert_eq!(
                cleared.cursor_background,
                Some(OptionalFill::None),
                "{file}"
            );
            assert_eq!(
                cleared.region_cursor_background,
                Some(OptionalFill::None),
                "{file}"
            );

            let transparent = quoted(file, "#12345600");
            let supplied = decodes(
                file,
                &style(
                    file,
                    &[
                        ("cursor.background", &transparent),
                        ("region.cursor.background", &transparent),
                    ],
                ),
            );
            let colour = OptionalFill::Color(straight_rgba(0x12_34_56_00));
            assert_eq!(supplied.cursor_background, Some(colour), "{file}");
            assert_eq!(supplied.region_cursor_background, Some(colour), "{file}");
            assert!(supplied.colors.is_empty(), "{file}");
        }
    }

    #[test]
    fn none_is_refused_for_every_other_colour() {
        for file in FORMATS {
            refuses(
                file,
                &style(file, &[("text", &quoted(file, "none"))]),
                &["`text` is \"none\"", "cursor.background"],
            );
        }
    }

    // --- Colours ---

    #[test]
    fn six_digit_colours_are_opaque_and_eight_digit_colours_carry_alpha() {
        for file in FORMATS {
            let document = decodes(
                file,
                &style(
                    file,
                    &[
                        ("text", &quoted(file, "#a1B2c3")),
                        ("link", &quoted(file, "#A1B2C340")),
                    ],
                ),
            );
            assert_eq!(
                document.colors,
                vec![
                    (ColorKey::Text, Color32::from_rgb(0xA1, 0xB2, 0xC3)),
                    (ColorKey::Link, straight_rgba(0xA1_B2_C3_40)),
                ],
                "{file}"
            );
        }
    }

    #[test]
    fn a_colour_is_a_hash_and_six_or_eight_hex_digits() {
        for file in FORMATS {
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
                    file,
                    &style(file, &[("grid.border", &quoted(file, bad))]),
                    &["`grid.border`", "6 or 8 hexadecimal digits"],
                );
            }
        }
    }

    ///
    /// `schema.md`: null, booleans, sequences and nested mappings are never
    /// a valid value, and neither is a number where a string belongs.
    ///
    #[test]
    fn a_colour_refuses_every_other_kind_of_value() {
        let per_format: [(&str, &[&str]); 3] = [
            (
                "theme.toml",
                &["5", "0.5", "true", "[\"#FFFFFF\"]", "{ a = \"#FFFFFF\" }"],
            ),
            (
                "theme.json",
                &[
                    "5",
                    "0.5",
                    "true",
                    "null",
                    "[\"#FFFFFF\"]",
                    "{\"a\": \"#FFFFFF\"}",
                ],
            ),
            (
                "theme.yaml",
                &[
                    "5",
                    "0.5",
                    "true",
                    "null",
                    "~",
                    "[\"#FFFFFF\"]",
                    "{a: \"#FFFFFF\"}",
                ],
            ),
        ];
        for (file, values) in per_format {
            for value in values {
                refuses(
                    file,
                    &style(file, &[("text", value)]),
                    &["invalid type", "colour string"],
                );
            }
        }
    }

    #[test]
    fn an_unquoted_yaml_colour_is_a_comment_and_so_refused() {
        refuses(
            "theme.yaml",
            "format: orcvs-theme\nversion: 1\nname: T\ninherits: okabe-ito\nstyle:\n  text: #E0E0E0\n",
            &["line 6", "unit value", "colour string"],
        );
    }

    // --- Widths ---

    #[test]
    fn widths_accept_integers_and_floats_within_their_bounds() {
        for file in FORMATS {
            let document = decodes(
                file,
                &style(
                    file,
                    &[
                        ("grid.border.width", "0"),
                        ("sector.seam.width", "0.75"),
                        ("panel.border.width", "2"),
                        ("input.cursor.width", "1.5"),
                    ],
                ),
            );
            assert_eq!(
                document.grid_widths,
                vec![
                    (GridWidthKey::GridBorder, 0.0),
                    (GridWidthKey::SectorSeam, 0.75)
                ],
                "{file}"
            );
            assert_eq!(
                document.chrome_widths,
                vec![
                    (ChromeWidthKey::PanelBorder, 2.0),
                    (ChromeWidthKey::InputCursor, 1.5)
                ],
                "{file}"
            );
        }
    }

    #[test]
    fn a_width_refuses_a_string_or_any_other_kind() {
        let per_format: [(&str, &[&str]); 3] = [
            ("theme.toml", &["\"0.5\"", "true", "[0.5]", "{ a = 0.5 }"]),
            (
                "theme.json",
                &["\"0.5\"", "true", "null", "[0.5]", "{\"a\": 0.5}"],
            ),
            (
                "theme.yaml",
                &["\"0.5\"", "'0.5'", "true", "null", "[0.5]", "{a: 0.5}"],
            ),
        ];
        for (file, values) in per_format {
            for value in values {
                refuses(
                    file,
                    &style(file, &[("grid.border.width", value)]),
                    &["invalid type", "number of points for `grid.border.width`"],
                );
            }
        }
    }

    #[test]
    fn a_width_must_be_finite() {
        for value in ["nan", "inf", "-inf", "+inf"] {
            refuses(
                "theme.toml",
                &style("theme.toml", &[("grid.border.width", value)]),
                &["`grid.border.width`", "finite"],
            );
        }
        for value in [".nan", ".inf", "-.inf", "1e999"] {
            refuses(
                "theme.yaml",
                &style("theme.yaml", &[("grid.border.width", value)]),
                &["line 6"],
            );
        }
        refuses(
            "theme.json",
            &style("theme.json", &[("grid.border.width", "1e999")]),
            &["line 1"],
        );
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
        ];
        // Only the property and bound are asserted: serde_json's default
        // float parser does not round-trip every value it echoes.
        for file in FORMATS {
            for (property, value, range) in cases {
                refuses(
                    file,
                    &style(file, &[(property, value)]),
                    &[&format!("`{property}` is "), range],
                );
            }
        }
    }

    // --- Root fields ---

    #[test]
    fn an_unknown_or_wrong_case_root_field_is_refused() {
        for file in FORMATS {
            let value = quoted(file, "x");
            for field in ["colour", "Name", "FORMAT", "Style"] {
                refuses(
                    file,
                    &document(file, &[(field, &value)], &[]),
                    &["unknown field", field],
                );
            }
        }
    }

    #[test]
    fn a_missing_required_field_is_refused() {
        let complete = [
            ("format", "orcvs-theme"),
            ("name", "T"),
            ("inherits", "okabe-ito"),
        ];
        for missing in ["format", "version", "name", "inherits"] {
            let toml: String = complete
                .iter()
                .filter(|(key, _)| *key != missing)
                .map(|(key, value)| format!("{key} = {value:?}\n"))
                .chain((missing != "version").then(|| "version = 1\n".to_owned()))
                .collect();
            refuses("theme.toml", &toml, &["missing field", missing]);
        }
    }

    ///
    /// The root is a mapping of named fields. A derived struct would also
    /// take a sequence of its fields in declaration order, which `serde_json`
    /// offers; TOML's root is always a table.
    ///
    #[test]
    fn a_root_sequence_is_refused() {
        refuses(
            "theme.json",
            r#"["orcvs-theme", 1, "Mine", "okabe-ito", "dark", {}]"#,
            &["invalid type: sequence", "line 1"],
        );
        refuses(
            "theme.yaml",
            "- orcvs-theme\n- 1\n- Mine\n- okabe-ito\n- dark\n- {}\n",
            &["expected mapping", "line 1"],
        );
    }

    #[test]
    fn a_repeated_root_field_is_refused() {
        refuses(
            "theme.toml",
            "format = \"orcvs-theme\"\nversion = 1\nname = \"T\"\nname = \"U\"\ninherits = \"okabe-ito\"\n",
            &["line 4"],
        );
        refuses(
            "theme.json",
            r#"{"format": "orcvs-theme", "version": 1, "name": "T", "name": "U", "inherits": "okabe-ito"}"#,
            &["duplicate field `name`", "line 1"],
        );
        refuses(
            "theme.yaml",
            "format: orcvs-theme\nversion: 1\nname: T\nname: U\ninherits: okabe-ito\n",
            &["duplicate", "line 4"],
        );
    }

    #[test]
    fn the_format_marker_must_be_orcvs_theme() {
        for file in FORMATS {
            let text = style(file, &[]).replacen("orcvs-theme", "orcvs-palette", 1);
            refuses(
                file,
                &text,
                &["`format` is \"orcvs-palette\"", "orcvs-theme"],
            );
        }
        refuses(
            "theme.json",
            r#"{"format": 1, "version": 1, "name": "T", "inherits": "okabe-ito"}"#,
            &["invalid type", "`format`"],
        );
    }

    #[test]
    fn the_version_must_be_the_integer_one() {
        for file in FORMATS {
            let with_version = |version: &str| style(file, &[]).replacen("1", version, 1);
            refuses(
                file,
                &with_version("2"),
                &["`version` is 2", "version 1 only"],
            );
            refuses(file, &with_version("0"), &["`version` is 0"]);
            refuses(file, &with_version("-1"), &["`version` is -1"]);
            refuses(
                file,
                &with_version("1.0"),
                &["floating point", "the integer 1"],
            );
            refuses(
                file,
                &with_version(&quoted(file, "1")),
                &["invalid type: string", "the integer 1"],
            );
        }
        refuses(
            "theme.yaml",
            "format: orcvs-theme\nversion: '1'\nname: T\ninherits: okabe-ito\n",
            &["invalid type: string", "line 2"],
        );
    }

    #[test]
    fn appearance_is_dark_or_light() {
        for file in FORMATS {
            for (value, appearance) in [("dark", Appearance::Dark), ("light", Appearance::Light)] {
                let document = decodes(
                    file,
                    &document(file, &[("appearance", &quoted(file, value))], &[]),
                );
                assert_eq!(document.appearance, Some(appearance), "{file}");
            }
            for value in ["Dark", "dim", ""] {
                refuses(
                    file,
                    &document(file, &[("appearance", &quoted(file, value))], &[]),
                    &["`appearance`", "\"dark\" or \"light\""],
                );
            }
            refuses(
                file,
                &document(file, &[("appearance", "true")], &[]),
                &["invalid type"],
            );
        }
    }

    // --- Labels ---

    #[test]
    fn name_and_inherits_must_be_nonempty_and_free_of_control_characters() {
        for file in FORMATS {
            let (empty, parent) = (quoted(file, ""), quoted(file, "okabe-ito"));
            refuses(
                file,
                &document_with_labels(file, &empty, &parent),
                &["`name` is empty"],
            );
            let name = quoted(file, "T");
            refuses(
                file,
                &document_with_labels(file, &name, &empty),
                &["`inherits` is empty"],
            );
        }
        refuses(
            "theme.json",
            r#"{"format": "orcvs-theme", "version": 1, "name": "A\tB", "inherits": "okabe-ito"}"#,
            &["`name` contains a control character"],
        );
        refuses(
            "theme.toml",
            "format = \"orcvs-theme\"\nversion = 1\nname = \"T\"\ninherits = \"okabe\\u0007ito\"\n",
            &["`inherits` contains a control character"],
        );
        refuses(
            "theme.yaml",
            "format: orcvs-theme\nversion: 1\nname: \"A\\nB\"\ninherits: okabe-ito\n",
            &["`name` contains a control character"],
        );
    }

    ///
    /// 256 UTF-8 bytes is the limit, not 256 characters: 128 two-byte `é`
    /// fit, 129 do not, and a 257-byte name is refused rather than cut.
    ///
    #[test]
    fn name_and_inherits_stay_within_256_utf8_bytes() {
        for file in FORMATS {
            for field in ["name", "inherits"] {
                let with = |value: &str| {
                    let quoted_value = quoted(file, value);
                    match field {
                        "name" => {
                            document_with_labels(file, &quoted_value, &quoted(file, "okabe-ito"))
                        }
                        _ => document_with_labels(file, &quoted(file, "T"), &quoted_value),
                    }
                };
                let at_limit = decodes(file, &with(&"a".repeat(256)));
                let held = if field == "name" {
                    at_limit.name
                } else {
                    at_limit.parent
                };
                assert_eq!(held.len(), 256, "{file} {field}");
                decodes(file, &with(&"é".repeat(128)));

                refuses(
                    file,
                    &with(&"a".repeat(257)),
                    &[&format!("`{field}` is 257 UTF-8 bytes"), "256-byte limit"],
                );
                refuses(
                    file,
                    &with(&"é".repeat(129)),
                    &[&format!("`{field}` is 258 UTF-8 bytes")],
                );
            }
        }
    }

    fn document_with_labels(file: &str, name: &str, inherits: &str) -> String {
        let format = quoted(file, "orcvs-theme");
        if file.ends_with(".toml") {
            format!("format = {format}\nversion = 1\nname = {name}\ninherits = {inherits}\n")
        } else if file.ends_with(".json") {
            format!(
                r#"{{"format": {format}, "version": 1, "name": {name}, "inherits": {inherits}}}"#
            )
        } else {
            format!("format: {format}\nversion: 1\nname: {name}\ninherits: {inherits}\n")
        }
    }

    // --- Style keys ---

    #[test]
    fn an_unknown_or_wrong_case_style_property_is_refused() {
        for file in FORMATS {
            let colour = quoted(file, "#FFFFFF");
            for property in [
                "Text",
                "grid.Background",
                "source.char",
                "grid_background",
                "text.",
                "style",
            ] {
                refuses(
                    file,
                    &style(file, &[(property, &colour)]),
                    &[&format!("unknown style property {property:?}")],
                );
            }
        }
    }

    ///
    /// Issue 07: the error explains the valid key. A near-miss in case or
    /// separator names the property it spells; any unknown name shows the
    /// form a property name takes.
    ///
    #[test]
    fn an_unknown_style_property_explains_the_valid_key() {
        for file in FORMATS {
            let colour = &quoted(file, "#FFFFFF");
            for (property, suggestion) in [
                ("grid_background", "did you mean \"grid.background\""),
                ("Grid.Background", "did you mean \"grid.background\""),
                ("panel-border-width", "did you mean \"panel.border.width\""),
                ("Text", "did you mean \"text\""),
            ] {
                refuses(file, &style(file, &[(property, colour)]), &[suggestion]);
            }
            let message = refuses(
                file,
                &style(file, &[("style", colour)]),
                &["\"grid.background\""],
            );
            assert!(!message.contains("did you mean"), "{file}: {message}");
        }
    }

    ///
    /// The Output Portal properties spell `output_portal` with an underscore,
    /// so a misspelling of them is suggested too, whichever separator or case
    /// it uses in place of that underscore.
    ///
    #[test]
    fn a_misspelled_output_portal_property_is_suggested() {
        for file in FORMATS {
            let colour = quoted(file, "#FFFFFF");
            let colour = colour.as_str();
            let width = "2";
            for (property, value, suggestion) in [
                ("Output_Portal.Border", colour, "\"output_portal.border\""),
                ("output-portal.border", colour, "\"output_portal.border\""),
                (
                    "output.portal.foreground",
                    colour,
                    "\"output_portal.foreground\"",
                ),
                (
                    "OUTPUT_PORTAL_BACKGROUND",
                    colour,
                    "\"output_portal.background\"",
                ),
                (
                    "output-portal-border-width",
                    width,
                    "\"output_portal.border.width\"",
                ),
            ] {
                refuses(
                    file,
                    &style(file, &[(property, value)]),
                    &[&format!("did you mean {suggestion}")],
                );
            }
        }
    }

    ///
    /// A dotted property name is one literal key, never a nested path: the
    /// TOML dotted key `grid.background = ...` and a nested `grid` mapping
    /// are both refused.
    ///
    #[test]
    fn a_style_property_is_never_a_nested_path() {
        refuses(
            "theme.toml",
            "format = \"orcvs-theme\"\nversion = 1\nname = \"T\"\ninherits = \"okabe-ito\"\n[style]\ngrid.background = \"#FFFFFF\"\n",
            &["unknown style property \"grid\""],
        );
        refuses(
            "theme.json",
            r##"{"format": "orcvs-theme", "version": 1, "name": "T", "inherits": "okabe-ito", "style": {"grid": {"background": "#FFFFFF"}}}"##,
            &["unknown style property \"grid\""],
        );
        refuses(
            "theme.yaml",
            "format: orcvs-theme\nversion: 1\nname: T\ninherits: okabe-ito\nstyle:\n  grid:\n    background: \"#FFFFFF\"\n",
            &["unknown style property \"grid\""],
        );
    }

    #[test]
    fn a_repeated_style_property_is_refused() {
        refuses(
            "theme.json",
            &style(
                "theme.json",
                &[("text", "\"#FFFFFF\""), ("text", "\"#000000\"")],
            ),
            &["style property `text` is repeated", "line 1"],
        );
        refuses(
            "theme.json",
            &style(
                "theme.json",
                &[
                    ("cursor.background", "\"none\""),
                    ("cursor.background", "\"none\""),
                ],
            ),
            &["style property `cursor.background` is repeated"],
        );
        refuses(
            "theme.json",
            &style(
                "theme.json",
                &[("grid.border.width", "0.5"), ("grid.border.width", "0.5")],
            ),
            &["style property `grid.border.width` is repeated"],
        );
        refuses(
            "theme.toml",
            &style(
                "theme.toml",
                &[("text", "\"#FFFFFF\""), ("text", "\"#000000\"")],
            ),
            &["duplicate key", "line 8"],
        );
        refuses(
            "theme.yaml",
            &style(
                "theme.yaml",
                &[("text", "\"#FFFFFF\""), ("text", "\"#000000\"")],
            ),
            &["duplicate", "line 7"],
        );
    }

    // --- YAML-only syntax ---

    #[test]
    fn yaml_merge_keys_are_refused() {
        refuses(
            "theme.yaml",
            "format: orcvs-theme\nversion: 1\nname: T\ninherits: okabe-ito\nstyle:\n  <<: {text: \"#FFFFFF\"}\n",
            &["merge keys", "line 6"],
        );
    }

    #[test]
    fn yaml_unknown_tags_are_refused() {
        refuses(
            "theme.yaml",
            "format: orcvs-theme\nversion: 1\nname: T\ninherits: okabe-ito\nstyle:\n  text: !colour \"#FFFFFF\"\n",
            &["unsupported tag", "line 6"],
        );
    }

    #[test]
    fn yaml_multiple_documents_are_refused() {
        refuses(
            "theme.yaml",
            "format: orcvs-theme\nversion: 1\nname: T\ninherits: okabe-ito\n---\nformat: orcvs-theme\nversion: 1\nname: U\ninherits: okabe-ito\n",
            &["multiple YAML documents"],
        );
    }

    #[test]
    fn yaml_tab_indentation_is_refused() {
        refuses(
            "theme.yaml",
            "format: orcvs-theme\nversion: 1\nname: T\ninherits: okabe-ito\nstyle:\n\ttext: \"#FFFFFF\"\n",
            &["tabs", "line 6"],
        );
    }

    #[test]
    fn yaml_only_true_and_false_are_booleans() {
        let document = decodes(
            "theme.yaml",
            "format: orcvs-theme\nversion: 1\nname: yes\ninherits: okabe-ito\n",
        );
        assert_eq!(document.name, "yes");
        refuses(
            "theme.yaml",
            "format: orcvs-theme\nversion: 1\nname: true\ninherits: okabe-ito\n",
            &["invalid type: boolean", "`name`"],
        );
    }

    #[test]
    fn yaml_scalar_aliases_are_accepted() {
        let document = decodes(
            "theme.yaml",
            "format: orcvs-theme\nversion: 1\nname: T\ninherits: okabe-ito\nstyle:\n  text: &ink \"#FFFFFF\"\n  link: *ink\n",
        );
        assert_eq!(
            document.colors,
            vec![
                (ColorKey::Text, Color32::WHITE),
                (ColorKey::Link, Color32::WHITE)
            ]
        );
    }

    ///
    /// A billion-laughs document: ten levels of ten-fold aliases. Nothing in
    /// the document type accepts a sequence, so it is refused at its first
    /// node, before any alias is replayed, and the alias limits stay as
    /// a second line.
    ///
    #[test]
    fn a_yaml_alias_bomb_is_refused() {
        let mut text = String::from(
            "format: orcvs-theme\nversion: 1\nname: T\ninherits: okabe-ito\nstyle:\n  text: &l0 [x, x, x, x, x, x, x, x, x, x]\n",
        );
        let levels = [
            "link",
            "error",
            "warning",
            "code.background",
            "input.cursor",
            "input.background",
            "text.muted",
            "text.active",
            "cell.background",
        ];
        for (level, property) in levels.iter().enumerate() {
            let previous = format!("*l{level}");
            let items = [previous.as_str(); 10].join(", ");
            text.push_str(&format!("  {property}: &l{} [{items}]\n", level + 1));
        }
        refuses("theme.yaml", &text, &["invalid type: sequence", "line 6"]);
    }

    ///
    /// `strict_booleans` stops YAML 1.1's `yes`/`no`/`on`/`off` being
    /// booleans, but the YAML 1.2 core schema still types `True`, `TRUE`,
    /// `Null`, `NULL` and `~`, so a string field holding one must quote it.
    ///
    #[test]
    fn yaml_core_booleans_and_nulls_are_not_strings() {
        for word in ["True", "TRUE", "False", "Null", "NULL", "~"] {
            refuses(
                "theme.yaml",
                &format!("format: orcvs-theme\nversion: 1\nname: {word}\ninherits: okabe-ito\n"),
                &["invalid type", "`name`", "line 3"],
            );
        }
        for word in ["True", "NULL"] {
            let document = decodes(
                "theme.yaml",
                &format!(
                    "format: orcvs-theme\nversion: 1\nname: \"{word}\"\ninherits: okabe-ito\n"
                ),
            );
            assert_eq!(document.name, word);
        }
    }

    ///
    /// An explicit YAML core tag decides its node's type, as YAML intends:
    /// `!!int "1"` is the integer 1, and `!!binary` is decoded to its text.
    /// The quoted-version refusal applies to untagged scalars.
    ///
    #[test]
    fn yaml_core_tags_decide_the_node_type() {
        let document = decodes(
            "theme.yaml",
            "format: orcvs-theme\nversion: !!int \"1\"\nname: T\ninherits: okabe-ito\n",
        );
        assert_eq!(document.name, "T");
        let document = decodes(
            "theme.yaml",
            "format: orcvs-theme\nversion: 1\nname: !!binary TXkgRGFyaw==\ninherits: okabe-ito\n",
        );
        assert_eq!(document.name, "My Dark");
        refuses(
            "theme.yaml",
            "format: orcvs-theme\nversion: !!str 1\nname: T\ninherits: okabe-ito\n",
            &["invalid type: string", "line 2"],
        );
    }

    #[test]
    fn yaml_errors_carry_line_and_column_without_a_snippet() {
        let message = refuses(
            "theme.yaml",
            &style("theme.yaml", &[("grid.border", "\"#12\"")]),
            &["at line 6, column 16", "`grid.border` is \"#12\""],
        );
        assert!(!message.contains("<input>"), "{message}");
        assert!(!message.contains('\n'), "{message}");
    }

    #[test]
    fn toml_errors_carry_line_and_column_without_echoing_the_line() {
        let message = refuses(
            "theme.toml",
            &style("theme.toml", &[("text", "5")]),
            &["line 7, column 10: invalid type: integer `5`"],
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
            "theme.toml",
            &style("theme.toml", &[("text", &format!("5 # {comment}"))]),
            &["line 7, column 10"],
        );
        assert!(
            message.len() <= MAX_MESSAGE_BYTES,
            "{} bytes",
            message.len()
        );

        // A decoder message that quotes a huge value is cut on a character
        // boundary and marked as cut.
        for file in FORMATS {
            let value = quoted(file, &format!("#{}", "é".repeat(MAX_DOCUMENT_BYTES / 4)));
            let message = refuses(file, &style(file, &[("text", &value)]), &["`text` is"]);
            assert!(
                message.len() <= MAX_MESSAGE_BYTES,
                "{file}: {} bytes",
                message.len()
            );
            assert!(message.ends_with('…'), "{file}");
        }
    }

    // --- Bytes before decoding ---

    #[test]
    fn a_leading_byte_order_mark_is_stripped_in_every_format() {
        for (file, text) in [
            ("my-dark.toml", MY_DARK_TOML),
            ("my-dark.json", MY_DARK_JSON),
            ("my-dark.yaml", MY_DARK_YAML),
        ] {
            let document = decode(file, &with_bom(text)).unwrap_or_else(|error| panic!("{error}"));
            assert_eq!(document, my_dark(), "{file}");
        }
    }

    #[test]
    fn only_one_byte_order_mark_is_stripped() {
        let twice = with_bom(&String::from_utf8(with_bom(MY_DARK_JSON)).expect("UTF-8"));
        let error = decode("my-dark.json", &twice).expect_err("second mark is content");
        assert!(matches!(
            error.reason,
            DocumentErrorReason::Invalid { format: "JSON", .. }
        ));
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

        assert_eq!(
            decodes("my-dark.toml", &padded(MAX_DOCUMENT_BYTES)),
            my_dark()
        );

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
        let error = decode("big.yaml", &vec![0xFF; MAX_DOCUMENT_BYTES + 1]).expect_err("too large");
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

    // --- Extensions ---

    #[test]
    fn the_extension_selects_the_decoder_case_insensitively() {
        for file in [
            "my-dark.toml",
            "my-dark.TOML",
            "My-Dark.Toml",
            "dir.d/my-dark.tOmL",
        ] {
            assert_eq!(decodes(file, MY_DARK_TOML), my_dark(), "{file}");
        }
        for file in ["my-dark.json", "my-dark.JSON", "my-dark.Json"] {
            assert_eq!(decodes(file, MY_DARK_JSON), my_dark(), "{file}");
        }
        for file in ["my-dark.yaml", "my-dark.YAML", "my-dark.yml", "my-dark.YmL"] {
            assert_eq!(decodes(file, MY_DARK_YAML), my_dark(), "{file}");
        }
    }

    ///
    /// Exactly one decoder per extension: JSON bytes in a `.toml` file are
    /// read as TOML, and refused, rather than sniffed.
    ///
    #[test]
    fn the_extension_not_the_content_chooses_the_decoder() {
        let error = decode("my-dark.toml", MY_DARK_JSON.as_bytes()).expect_err("JSON is not TOML");
        assert!(matches!(
            error.reason,
            DocumentErrorReason::Invalid { format: "TOML", .. }
        ));
    }

    #[test]
    fn an_unsupported_extension_is_refused() {
        for file in [
            "my-dark.txt",
            "my-dark",
            "my-dark.toml.bak",
            "my-dark.jsonc",
            "my-dark.",
            ".toml",
        ] {
            let error = decode(file, MY_DARK_TOML.as_bytes()).expect_err(file);
            assert_eq!(
                error,
                DocumentError {
                    file_name: file.to_owned(),
                    reason: DocumentErrorReason::UnsupportedExtension,
                }
            );
            assert!(
                error.to_string().contains(".toml, .json, .yaml or .yml"),
                "{error}"
            );
        }
    }

    // --- Error text ---

    #[test]
    fn the_error_names_the_file_the_property_and_the_decoders_position() {
        for (file, text, position) in [
            (
                "bad.toml",
                style("bad.toml", &[("grid.border", "\"#12\"")]),
                "line 7",
            ),
            (
                "bad.json",
                style("bad.json", &[("grid.border", "\"#12\"")]),
                "line 1 column",
            ),
            (
                "bad.yaml",
                style("bad.yaml", &[("grid.border", "\"#12\"")]),
                "at line 6, column 16",
            ),
        ] {
            let shown = decode(file, text.as_bytes()).expect_err(file).to_string();
            assert!(shown.starts_with(&format!("{file}: invalid ")), "{shown}");
            assert!(shown.contains("`grid.border` is \"#12\""), "{shown}");
            assert!(shown.contains(position), "{file}: {shown}");
        }
    }
}
