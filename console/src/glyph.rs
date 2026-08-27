use std::{fmt, iter};

use egui::Color32;
use lang::Token;

use crate::Color;

const DEFAULT_GLYPH_BG_COLOR: Color32 = Color32::TRANSPARENT;
const DEFAULT_GLYPH_STROKE_COLOR: Color32 = Color32::TRANSPARENT;
const DEFAULT_GLYPH_FONT_COLOR: Color32 = Color::rgb(164, 166, 169).build();

const DEFAULT_GLYPH_SELECTED_BG_COLOR: Color32 = Color::rgb(0, 92, 128).build();
const DEFAULT_GLYPH_SELECTED_FONT_COLOR: Color32 = DEFAULT_GLYPH_FONT_COLOR;

const DEFAULT_VISUAL_SELECTED_STROKE_COLOR_BLINK: Color32 = Color::rgb(192, 222, 255).build();

#[derive(Clone, Debug, PartialEq)]
pub struct GlyphString {
    s: Option<String>,
    t: Glyph,
}

impl GlyphString {
    pub fn new(s: Option<String>, t: Glyph) -> Self {
        Self { s, t }
    }

    pub fn marker() -> GlyphString {
        Self {
            s: None,
            t: Glyph::Marker,
        }
    }

    pub fn highlight() -> GlyphString {
        Self {
            s: None,
            t: Glyph::Highlight,
        }
    }

    pub fn space() -> GlyphString {
        Self {
            s: None,
            t: Glyph::Space,
        }
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

        match (self.t, selected) {
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
            (Glyph::Char, true) => default_selected,
            (Glyph::Char, false) => GlyphStyle {
                bg_fill: Color::rgb(125, 225, 220).build(),
                stroke_color: Color::rgb(33, 33, 33).build(),
                font_color: Color::rgb(200, 200, 200).build(),
            },
            // (Glyph::String, false) => GlyphStyle {
            //     bg_fill: Color::rgb(97, 0, 255).build(),
            //     stroke_color: Color::rgb(97, 0, 255).with_alpha(128).build(),
            // },
            (Glyph::Space | Glyph::Marker | Glyph::Highlight, true) => default_selected,
            (Glyph::Space | Glyph::Marker | Glyph::Highlight, false) => default,
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

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Glyph {
    Char,
    Function,
    Highlight,
    Marker,
    Number,
    Note,
    Space,
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

impl From<Token> for Glyph {
    fn from(t: Token) -> Self {
        match t {
            Token::Function => G::Function,
            Token::Note => G::Note,
            Token::Number => G::Number,
            Token::NumberN(_) => G::Number,
            Token::Char => G::Char,
        }
    }
}

impl fmt::Display for GlyphString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self.t {
            Glyph::Char => "c".to_string(),
            Glyph::Function => "F".to_string(),
            Glyph::Highlight => ".".to_string(),
            Glyph::Marker => "+".to_string(),
            Glyph::Note => "n".to_string(),
            Glyph::Number => "h".to_string(),
            Glyph::Space => " ".to_string(),
        };

        write!(f, "{}", s)
    }
}

impl Glyph {
    pub fn to_glyphs(tokens: Vec<Token>) -> Vec<Glyph> {
        tokens
            .into_iter()
            .flat_map(|t| iter::repeat(Glyph::from(t)).take(t.len()))
            .collect()
    }

    // #[inline]
    // pub fn is_terminator(s: &str) -> bool {
    //     match s {
    //         "." => true,
    //         " " => true,
    //         _ => false,
    //     }
    // }

    #[inline]
    pub fn is_terminator_bytes(b: u8) -> bool {
        match b {
            46 => true,
            32 => true,
            // 43 => true,
            _ => false,
        }
    }
}

#[cfg(test)]
mod test {
    use crate::glyph::{Glyph, GlyphString};

    #[test]
    fn background_glyphs_remain_distinct() {
        let marker = GlyphString::marker();
        let highlight = GlyphString::highlight();
        let space = GlyphString::space();

        assert_eq!(marker, GlyphString::new(None, Glyph::Marker));
        assert_eq!(highlight, GlyphString::new(None, Glyph::Highlight));
        assert_eq!(space, GlyphString::new(None, Glyph::Space));
        assert_ne!(marker, highlight);
        assert_ne!(marker, space);
        assert_ne!(highlight, space);
        assert_eq!(marker.to_string(), "+");
        assert_eq!(highlight.to_string(), ".");
        assert_eq!(space.to_string(), " ");
    }

    // #[test]
    // fn test_is_terminator() {
    //     let t = Glyph::is_terminator(".");
    //     assert!(t);

    //     let t = Glyph::is_terminator(" ");
    //     assert!(t);

    //     let t = Glyph::is_terminator("+");
    //     assert!(t == false);

    //     let t = Glyph::is_terminator("..");
    //     assert!(t == false);

    //     let t = Glyph::is_terminator("!");
    //     assert!(t == false);
    // }

    #[test]
    fn test_is_terminator_bytes() {
        let b = ".".as_bytes();
        let t = Glyph::is_terminator_bytes(b[0]);
        assert!(t);

        let b = " ".as_bytes();
        let t = Glyph::is_terminator_bytes(b[0]);
        assert!(t);

        let b = "+".as_bytes();
        let t = Glyph::is_terminator_bytes(b[0]);
        assert!(t == false);

        let b = "!".as_bytes();
        let t = Glyph::is_terminator_bytes(b[0]);
        assert!(t == false);
    }
}
