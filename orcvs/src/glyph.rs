use std::fmt;

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
    Comment,
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
            Token::Comment => G::Comment,
            Token::Function => G::Function,
            Token::Note => G::Note,
            Token::Number => G::Number,
            Token::Char => G::Char,
        }
    }
}

impl fmt::Display for GlyphString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = self.s.clone().unwrap_or_else(|| match self.t {
            Glyph::Bang => "*".to_string(),
            Glyph::Char => "c".to_string(),
            // A Comment declares no spelling, so it hints at none. The
            // fallbacks beside this one stand in for a Cell a signature says
            // something belongs in — `h` for an unfilled Number slot, `n` for
            // a Note. A Comment's claim is a Cell claim rather than an
            // operand list (ADR 0035) and its text is never decoded, so an
            // empty Cell it claims holds nothing and renders as nothing. Its
            // Glyph still names the Cell for paint, which is the whole of
            // what a Comment's classification is for.
            Glyph::Comment => " ".to_string(),
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
        assert_eq!(Glyph::from(lang::Token::Function), Glyph::Function);
    }

    #[test]
    fn a_lone_asterisk_remains_an_ordinary_character() {
        assert_eq!(Glyph::from(lang::Token::Char), Glyph::Char);
    }

    /// A Comment paints as itself rather than as ordinary text. ADR 0035 made
    /// it a Language Unit the Parser establishes, so its Cells arrive here
    /// with a Token of their own instead of falling through to the unclaimed
    /// Cell backfill the way Comment text used to.
    #[test]
    fn a_comment_has_its_own_paint_classification() {
        assert_eq!(Glyph::from(lang::Token::Comment), Glyph::Comment);
        assert_ne!(Glyph::from(lang::Token::Comment), Glyph::Char);
    }

    /// A Comment claims a whole row, empty Cells and all, so its Glyph reaches
    /// Cells with no content. It classifies them for paint and stands in for
    /// nothing: the placeholder an operand Glyph draws says what a signature
    /// declares belongs there, and a Comment declares nothing.
    #[test]
    fn a_comment_hints_at_no_spelling_in_a_cell_it_claims_and_leaves_empty() {
        assert_eq!(GlyphString::new(None, Glyph::Comment).to_string(), " ");
        assert_eq!(
            GlyphString::new(Some("x".to_string()), Glyph::Comment).to_string(),
            "x"
        );
        // The contrast that makes it a decision: an unfilled operand slot does
        // stand in for the spelling its signature names.
        assert_eq!(GlyphString::new(None, Glyph::Number).to_string(), "h");
    }

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
