//!
//! Decodes one Orcvs Theme document's bytes into a [`ThemeDocument`]:
//! `schema.md`'s "Decoding and limits", with no file I/O and no inheritance.
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
//!   optional Cursor fills — so `toml` attaches its line and column, and the
//!   key it was reading, to every refusal.
//!
//! Width bounds are checked here as read and again by
//! [`crate::theme::resolve`], which also checks parent/appearance agreement
//! with the identity the caller took from the file name.
//!

use std::fmt;

use serde::Deserialize;

use crate::theme::{
    Appearance, ChromeWidth, ChromeWidthKey, ColorKey, GridWidth, GridWidthKey, OptionalFill,
    ThemeDocument, style_catalogue,
};

/// `schema.md`'s per-document byte limit, checked before any decoding.
pub(crate) const MAX_DOCUMENT_BYTES: usize = 1024 * 1024;

/// The longest decoder message a [`DocumentError`] keeps, in UTF-8 bytes.
const MAX_MESSAGE_BYTES: usize = 1024;

/// The longest `name` or `inherits` value, in UTF-8 bytes.
const MAX_LABEL_BYTES: usize = 256;

const FORMAT_MARKER: &str = "orcvs-theme";
const VERSION: i64 = 1;

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
/// takes the identity the caller passes with the result to
/// [`crate::theme::resolve`]. Performs no I/O.
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
            let fills = OPTIONAL_FILLS.map(|fill| format!("`{fill}`")).join(" and ");
            return Err(format!(
                "\"none\" is not a colour; only {fills} can be \"none\""
            ));
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

/// One of the two optional Cursor fills: a colour, or the string `none`.
#[derive(Deserialize)]
#[serde(try_from = "String")]
struct Fill(OptionalFill);

impl TryFrom<String> for Fill {
    type Error = String;

    fn try_from(value: String) -> Result<Self, String> {
        match value.as_str() {
            "none" => Ok(Self(OptionalFill::None)),
            value => colour(value).map(|colour| Self(OptionalFill::Color(colour))),
        }
    }
}

///
/// A width in points, refused unless finite and within `0..=max`: the
/// inclusive bound [`crate::theme::resolve`] applies too. Checked as read,
/// before narrowing to the `f32` `ThemeDocument` stores: narrowing rounds a
/// value just past a bound onto it, and one beyond `f32`'s range to
/// infinity, and neither may be clamped. `toml` hands an integer to an
/// `f64` too, so both spellings are widths, and nothing else is.
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

/// A Grid width: 0 to [`GridWidth::MAX_POINTS`].
#[derive(Deserialize)]
#[serde(try_from = "f64")]
struct GridPoints(f32);

impl TryFrom<f64> for GridPoints {
    type Error = String;

    fn try_from(value: f64) -> Result<Self, String> {
        points(value, GridWidth::MAX_POINTS).map(Self)
    }
}

/// A chrome width: 0 to [`ChromeWidth::MAX_POINTS`].
#[derive(Deserialize)]
#[serde(try_from = "f64")]
struct ChromePoints(f32);

impl TryFrom<f64> for ChromePoints {
    type Error = String;

    fn try_from(value: f64) -> Result<Self, String> {
        points(value, ChromeWidth::MAX_POINTS).map(Self)
    }
}

// === Style ===

///
/// Derives the `style` table from [`style_catalogue`]: one optional field per
/// property, renamed to its literal dotted spelling, the two optional Cursor
/// fills among them, and [`OPTIONAL_FILLS`], the fills' spellings. Its
/// `into_document` lists the supplied properties in catalogue order.
///
macro_rules! document_style {
    (
        OptionalFill { $($fill:ident($fill_field:ident) => $fill_name:literal),* $(,)? }
        ColorKey { $($color:ident($color_field:ident) => $color_name:literal),* $(,)? }
        GridWidthKey { $($grid:ident($grid_field:ident) => $grid_name:literal),* $(,)? }
        ChromeWidthKey { $($chrome:ident($chrome_field:ident) => $chrome_name:literal),* $(,)? }
    ) => {
        /// The only properties that take `"none"`.
        const OPTIONAL_FILLS: [&str; 2] = [$($fill_name),*];

        /// The decoded `style` table: `None` for every omitted property.
        #[derive(Default, Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Style {
            $(#[serde(rename = $color_name)] $color_field: Option<ColorValue>,)*
            $(#[serde(rename = $grid_name)] $grid_field: Option<GridPoints>,)*
            $(#[serde(rename = $chrome_name)] $chrome_field: Option<ChromePoints>,)*
            $(#[serde(rename = $fill_name)] $fill_field: Option<Fill>,)*
        }

        impl Style {
            fn into_document(self) -> ThemeDocument {
                let mut document = ThemeDocument::default();
                $(if let Some(ColorValue(colour)) = self.$color_field {
                    document.colors.push((ColorKey::$color, colour));
                })*
                $(if let Some(GridPoints(points)) = self.$grid_field {
                    document.grid_widths.push((GridWidthKey::$grid, points));
                })*
                $(if let Some(ChromePoints(points)) = self.$chrome_field {
                    document.chrome_widths.push((ChromeWidthKey::$chrome, points));
                })*
                $(document.$fill_field = self.$fill_field.map(|Fill(fill)| fill);)*
                document
            }
        }
    };
}

style_catalogue!(document_style);

#[cfg(test)]
mod tests {
    use egui::Color32;

    use super::{DocumentErrorReason, MAX_DOCUMENT_BYTES, MAX_MESSAGE_BYTES, decode};
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

    // --- Documents ---

    const FILE: &str = "theme.toml";

    ///
    /// A document with the four required root fields, `extra_root` after
    /// them, and a `[style]` table holding `entries` — each a property name,
    /// written as a quoted key, and a value already in TOML syntax.
    ///
    fn document(extra_root: &[(&str, &str)], entries: &[(&str, &str)]) -> String {
        let mut text = String::from(
            "format = \"orcvs-theme\"\nversion = 1\nname = \"T\"\ninherits = \"okabe-ito\"\n",
        );
        for (key, value) in extra_root {
            text.push_str(&format!("{key} = {value}\n"));
        }
        text.push_str("\n[style]\n");
        for (key, value) in entries {
            text.push_str(&format!("\"{key}\" = {value}\n"));
        }
        text
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
        let document = decodes(OKABE_ITO_COPY_TOML);
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

    ///
    /// Each of the example's 59 property names, alone in a document, sets
    /// exactly one property, whose key spells that name — and the 59 reach
    /// 59 different properties, so every catalogue entry is reachable by
    /// its own spelling and by no other.
    ///
    #[test]
    fn every_catalogue_property_name_selects_its_own_property() {
        let entries = OKABE_ITO_COPY_TOML
            .lines()
            .filter_map(|line| line.strip_prefix('"')?.split_once("\" = "))
            .collect::<Vec<_>>();
        assert_eq!(entries.len(), 59, "45 colours, 2 optional fills, 12 widths");
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
                .chain(
                    document
                        .region_cursor_background
                        .map(|_| "region.cursor.background"),
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
            parent: "okabe-ito".to_owned(),
            name: "T".to_owned(),
            ..ThemeDocument::default()
        };
        assert_eq!(decodes(&style(&[])), bare, "empty style");
        assert_eq!(
            decodes(
                "format = \"orcvs-theme\"\nversion = 1\nname = \"T\"\ninherits = \"okabe-ito\"\n"
            ),
            bare,
            "omitted style"
        );
    }

    ///
    /// `style` is a table. A derived struct would also take an array of its
    /// fields in declaration order, so a complete array is refused too.
    ///
    #[test]
    fn a_style_that_is_not_a_table_is_refused() {
        let in_order = [
            ["\"#FFFFFF\""; 45].as_slice(),
            &["0.5"; 7],
            &["1"; 5],
            &["\"none\""; 2],
        ]
        .concat()
        .join(", ");
        let in_order = format!("[{in_order}]");
        for value in ["\"x\"", "1", "[]", in_order.as_str()] {
            refuses(
                &format!(
                    "format = \"orcvs-theme\"\nversion = 1\nname = \"T\"\n\
                     inherits = \"okabe-ito\"\nstyle = {value}\n"
                ),
                &["line 5, column 9, in `style`: `style` is a table of properties"],
            );
        }
    }

    // --- Optional fills ---

    #[test]
    fn optional_fills_are_omitted_cleared_or_a_supplied_transparent_colour() {
        let omitted = decodes(&style(&[]));
        assert_eq!(omitted.cursor_background, None);
        assert_eq!(omitted.region_cursor_background, None);

        let none = quoted("none");
        let cleared = decodes(&style(&[
            ("cursor.background", &none),
            ("region.cursor.background", &none),
        ]));
        assert_eq!(cleared.cursor_background, Some(OptionalFill::None));
        assert_eq!(cleared.region_cursor_background, Some(OptionalFill::None));

        let transparent = quoted("#12345600");
        let supplied = decodes(&style(&[
            ("cursor.background", &transparent),
            ("region.cursor.background", &transparent),
        ]));
        let colour = OptionalFill::Color(straight_rgba(0x12_34_56_00));
        assert_eq!(supplied.cursor_background, Some(colour));
        assert_eq!(supplied.region_cursor_background, Some(colour));
        assert!(supplied.colors.is_empty());
    }

    #[test]
    fn an_optional_fill_refuses_a_malformed_colour() {
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
                "\"none\" is not a colour",
                "`cursor.background`",
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
                (GridWidthKey::GridBorder, 0.0),
                (GridWidthKey::SectorSeam, 0.75)
            ]
        );
        assert_eq!(
            document.chrome_widths,
            vec![
                (ChromeWidthKey::PanelBorder, 2.0),
                (ChromeWidthKey::InputCursor, 1.5)
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
        for value in ["nan", "inf", "-inf", "+inf"] {
            refuses(
                &style(&[("grid.border.width", value)]),
                &["in `style.grid.border.width`", "finite number of points"],
            );
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
            refuses(&toml, &["missing field", missing]);
        }
    }

    #[test]
    fn a_repeated_root_field_is_refused() {
        refuses(
            "format = \"orcvs-theme\"\nversion = 1\nname = \"T\"\nname = \"U\"\ninherits = \"okabe-ito\"\n",
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
        let root =
            "format = \"orcvs-theme\"\nversion = 1\nname = \"T\"\ninherits = \"okabe-ito\"\n";
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
        let (empty, parent, name) = (quoted(""), quoted("okabe-ito"), quoted("T"));
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
            &document_with_labels(&name, "\"okabe\\u0007ito\""),
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
                "name" => document_with_labels(&quoted(value), &quoted("okabe-ito")),
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
        ] {
            refuses(
                &style(&[(property, &colour)]),
                &[&format!("unknown field `{property}`"), "in `style`"],
            );
        }
    }

    ///
    /// Issue 07: the error explains the valid key. `serde`'s refusal lists
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
    /// The list of 59 names, colours first, is longer than the message cap,
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
        let root =
            "format = \"orcvs-theme\"\nversion = 1\nname = \"T\"\ninherits = \"okabe-ito\"\n";
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
}
