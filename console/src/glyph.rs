use egui::Color32;

use crate::Color;

const DEFAULT_GLYPH_BG_COLOR: Color32 = Color32::TRANSPARENT;
const DEFAULT_GLYPH_STROKE_COLOR: Color32 = Color32::TRANSPARENT;
const DEFAULT_GLYPH_FONT_COLOR: Color32 = Color::rgb(164, 166, 169).build();

const DEFAULT_GLYPH_SELECTED_BG_COLOR: Color32 = Color::rgb(0, 92, 128).build();
const DEFAULT_GLYPH_SELECTED_FONT_COLOR: Color32 = DEFAULT_GLYPH_FONT_COLOR;

const DEFAULT_VISUAL_SELECTED_STROKE_COLOR_BLINK: Color32 = Color::rgb(192, 222, 255).build();

#[cfg_attr(feature = "persistence", derive(Serialize, Deserialize))]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Glyph {
    Function,
    Number,
    Note,
    String,
    Terminator(Terminator),
}
pub type G = Glyph;

#[derive(Debug)]
pub struct GlyphStyle {
    pub bg_fill: Color32,
    pub stroke_color: Color32,
    pub font_color: Color32,
}

pub const CURSOR_VISUALS: GlyphStyle = GlyphStyle {
    bg_fill: Color32::TRANSPARENT,
    stroke_color: DEFAULT_VISUAL_SELECTED_STROKE_COLOR_BLINK,
    font_color: DEFAULT_GLYPH_FONT_COLOR,
};

#[cfg_attr(feature = "persistence", derive(Serialize, Deserialize))]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Terminator {
    Dot,
    Marker,
    Space,
}

impl Default for Glyph {
    fn default() -> Self {
        Glyph::Terminator(Terminator::default())
    }
}

impl Default for Terminator {
    fn default() -> Self {
        Terminator::Space
    }
}

impl From<Glyph> for String {
    fn from(g: Glyph) -> Self {
        match g {
            Glyph::Function => "F".to_string(),
            Glyph::Number => "h".to_string(),
            Glyph::Note => "n".to_string(),
            Glyph::String => "s".to_string(),
            Glyph::Terminator(t) => t.into(),
        }
    }
}

impl From<Terminator> for String {
    fn from(t: Terminator) -> Self {
        match t {
            Terminator::Dot => ".".to_string(),
            Terminator::Marker => "+".to_string(),
            Terminator::Space => " ".to_string(),
        }
    }
}

impl Glyph {
    #[inline]
    pub fn is_glyph(s: &str) -> bool {
        !Self::is_terminator(s) && !Self::is_marker(s)
    }

    #[inline]
    pub fn is_marker(s: &str) -> bool {
        s == "+"
    }

    #[inline]
    pub fn is_terminator(s: &str) -> bool {
        match s {
            "." => true,
            " " => true,
            _ => false,
        }
    }

    #[inline]
    pub fn is_terminator_bytes(b: u8) -> bool {
        match b {
            46 => true,
            32 => true,
            // 43 => true,
            _ => false,
        }
    }

    #[inline]
    pub fn is_character(s: &str) -> bool {
        !Self::is_terminator(s)
    }

    pub fn highlight() -> Self {
        Glyph::Terminator(Terminator::Dot)
    }
    pub fn marker() -> Self {
        Glyph::Terminator(Terminator::Marker)
    }

    pub fn style(&self, selected: bool) -> GlyphStyle {
        let default = GlyphStyle {
            bg_fill: DEFAULT_GLYPH_BG_COLOR,
            stroke_color: DEFAULT_GLYPH_STROKE_COLOR,
            font_color: DEFAULT_GLYPH_FONT_COLOR,
        };

        let default_selected = GlyphStyle {
            bg_fill: DEFAULT_GLYPH_SELECTED_BG_COLOR,
            stroke_color: DEFAULT_GLYPH_SELECTED_FONT_COLOR,
            font_color: DEFAULT_GLYPH_SELECTED_FONT_COLOR,
        };

        // Color::rgb(97, 0, 255).build()
        // Color::rgb(255, 0, 230).build(),

        match (self, selected) {
            // (Glyph::Function, true) => v,
            (Glyph::Function, true) => GlyphStyle {
                bg_fill: Color::rgb(200, 75, 255).build(),
                stroke_color: default_selected.stroke_color,
                font_color: default_selected.font_color,
            },
            (Glyph::Function, false) => GlyphStyle {
                bg_fill: Color::rgb(255, 0, 230).build(),
                stroke_color: Color::rgb(0, 0, 0).build(),
                font_color: Color::rgb(255, 255, 255).build(),
            },
            // (Glyph::Number, true) => GlyphVisuals {
            //     bg_fill: Color::rgb(97, 0, 255).build(),
            //     stroke_color: Color::rgb(97, 0, 255).with_alpha(128).build(),
            // },
            (Glyph::Number, true) => default_selected,
            (Glyph::Number, false) => default,
            // (Glyph::Number, false) => GlyphStyle {
            //     bg_fill: Color::rgb(20, 146, 135).build(),
            //     stroke_color: Color::rgb(156, 192, 189).build(),
            //     font_color: Color::rgb(156, 192, 189).build(),
            // },
            // (Glyph::Number, false) => GlyphStyle {
            //     bg_fill: Color::rgb(97, 0, 255).build(),
            //     stroke_color: Color::rgb(97, 0, 255).with_alpha(128).build(),
            // },
            // (Glyph::Note, true) => GlyphVisuals {
            //     bg_fill: Color::rgb(97, 0, 255).build(),
            //     stroke_color: Color::rgb(97, 0, 255).with_alpha(128).build(),
            // },
            (Glyph::Note, true) => default_selected,
            (Glyph::Note, false) => GlyphStyle {
                bg_fill: Color::rgb(25, 150, 135).build(),
                stroke_color: Color::rgb(33, 33, 33).build(),
                font_color: Color::rgb(200, 200, 200).build(),
            },
            // (Glyph::Note, false) => GlyphStyle {
            //     bg_fill: Color::rgb(20, 146, 135).build(),
            //     stroke_color: Color::rgb(156, 192, 189).build(),
            //     font_color: Color::rgb(156, 192, 189).build(),
            // },
            (Glyph::String, true) => default_selected,
            (Glyph::String, false) => GlyphStyle {
                bg_fill: Color::rgb(125, 225, 220).build(),
                stroke_color: Color::rgb(33, 33, 33).build(),
                font_color: Color::rgb(200, 200, 200).build(),
            },
            // (Glyph::String, false) => GlyphStyle {
            //     bg_fill: Color::rgb(97, 0, 255).build(),
            //     stroke_color: Color::rgb(97, 0, 255).with_alpha(128).build(),
            // },
            (Glyph::Terminator(_), true) => default_selected,
            (Glyph::Terminator(_), false) => default,
            // (Glyph::Terminator(_), true) => GlyphVisuals {
            //     bg_fill: Color::rgb(97, 0, 255).build(),
            //     stroke_color: Color::rgb(97, 0, 255).with_alpha(128).build(),
            // },
            // (Glyph::Terminator(_), false) => GlyphStyle {
            //     bg_fill: Color::rgb(97, 0, 255).build(),
            //     stroke_color: Color::rgb(97, 0, 255).with_alpha(128).build(),
            // },
        }
    }
}
