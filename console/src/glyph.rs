use std::{fmt, iter};

use lang::Token;

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
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Glyph {
    Bang,
    Char,
    Function,
    Highlight,
    Marker,
    Number,
    Note,
    Space,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum SemanticGlyph {
    Bang,
    Char,
    Function,
    Highlight,
    Marker,
    Number,
    Note,
    Space,
}
pub type G = Glyph;

impl From<Token> for Glyph {
    fn from(t: Token) -> Self {
        match t {
            Token::Bang => G::Bang,
            Token::Activation => G::Char,
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
        let s = self.s.clone().unwrap_or_else(|| match self.t {
            Glyph::Bang => "*".to_string(),
            Glyph::Char => "c".to_string(),
            Glyph::Function => "F".to_string(),
            Glyph::Highlight => ".".to_string(),
            Glyph::Marker => "+".to_string(),
            Glyph::Note => "n".to_string(),
            Glyph::Number => "h".to_string(),
            Glyph::Space => " ".to_string(),
        });

        write!(f, "{}", s)
    }
}

impl Glyph {
    pub(crate) fn semantic(self) -> SemanticGlyph {
        match self {
            Glyph::Bang => SemanticGlyph::Bang,
            Glyph::Char => SemanticGlyph::Char,
            Glyph::Function => SemanticGlyph::Function,
            Glyph::Highlight => SemanticGlyph::Highlight,
            Glyph::Marker => SemanticGlyph::Marker,
            Glyph::Number => SemanticGlyph::Number,
            Glyph::Note => SemanticGlyph::Note,
            Glyph::Space => SemanticGlyph::Space,
        }
    }

    pub fn to_glyphs(tokens: Vec<Token>) -> Vec<Glyph> {
        tokens
            .into_iter()
            .flat_map(|t| iter::repeat_n(Glyph::from(t), t.len()))
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

    #[test]
    fn occupied_glyphs_display_their_source_content() {
        assert_eq!(
            GlyphString::new(Some("+".to_string()), Glyph::Function).to_string(),
            "+"
        );
        assert_eq!(
            GlyphString::new(Some("A".to_string()), Glyph::Number).to_string(),
            "A"
        );
        assert_eq!(
            GlyphString::new(Some("*".to_string()), Glyph::Char).to_string(),
            "*"
        );
    }

    #[test]
    fn a_complete_bang_has_its_own_semantic_paint_classification() {
        assert_eq!(Glyph::from(lang::Token::Bang), Glyph::Bang);
        assert_eq!(Glyph::Bang.semantic(), super::SemanticGlyph::Bang);
        assert_eq!(Glyph::Function.semantic(), super::SemanticGlyph::Function);
    }

    #[test]
    fn a_lone_asterisk_remains_an_ordinary_character() {
        assert_eq!(Glyph::Char.semantic(), super::SemanticGlyph::Char);
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
        assert!(!t);

        let b = "!".as_bytes();
        let t = Glyph::is_terminator_bytes(b[0]);
        assert!(!t);
    }
}
