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
        assert!(!t);

        let b = "!".as_bytes();
        let t = Glyph::is_terminator_bytes(b[0]);
        assert!(!t);
    }
}
