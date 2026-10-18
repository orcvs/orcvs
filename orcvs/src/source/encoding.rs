//! What an answered value becomes in Cells, before any destination is asked.
//!
//! CONTEXT.md's Spatial Output is "a Function output delivered through a
//! Portal as literal Source encoding", and this is that encoding as a value
//! rather than as a `String`. It is the counterpart to `lang::Token::decode`,
//! which reads Cells back as an Atom according to the receiving operand's
//! declared type.
//!
//! It lives beside [`super::portal`] rather than inside it because the two
//! answer different questions. Whether a value can be Cells at all is true of
//! the value wherever it lands and needs no Grid to decide; whether those Cells
//! fit is true of one destination and needs nothing else about the value than
//! how many there are. Do not hand the Portal a `&str` across that seam: the
//! Portal would then answer both, and [`super::portal::PortalError`] would
//! carry a content refusal that has nothing to do with a destination.
//!
//! Rendering is not display. Per ADR 0003 a Tick's spatial output *is* program
//! text — "a complete executable Orcvs program and the accumulated output of
//! preceding Ticks" — so these Cells are read back by a later Turn of the same
//! Tick through `Token::decode`, committed as Source, and parsed by the next
//! Tick. A Cell holds exactly one printable ASCII character and nothing else,
//! which is why a value has nowhere to keep its type once it is written and
//! why ADR 0034 has the receiving operand decide the reading.

use lang::{Atom, Spelling};

use super::CellContent;

///
/// A non-empty run of Source Cells.
///
/// Non-emptiness and printable content are properties of the type rather than
/// checks a caller repeats: every Atom spells two Cells, so an `Encoding` that
/// exists places at least one. [`super::portal::Portal`]
/// relies on that, and it is held here, where the invariant is established.
///
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Encoding(Vec<CellContent>);

impl Encoding {
    ///
    /// The Cells one answered value occupies, wherever it is delivered.
    ///
    /// Every Atom spells two printable ASCII Cells, which [`lang::Spelling`]
    /// guarantees, so a value always becomes Cells and only a destination can
    /// refuse them.
    ///
    pub(super) fn render(atom: Atom) -> Self {
        Self::spelled(atom.spelling())
    }

    ///
    /// The two Cells `spelling` occupies, for a Source effect that writes a
    /// Function's spelling rather than answering a value.
    ///
    pub(super) fn spelled(spelling: Spelling) -> Self {
        Self(CellContent::spelled(spelling).to_vec())
    }

    ///
    /// The two spaces a Clear writes at its Output Portal.
    ///
    pub(super) fn clear() -> Self {
        Self(vec![CellContent::SPACE; 2])
    }

    ///
    /// Source characters a test states directly, held to a Cell's definition:
    /// `None` when any byte is outside printable ASCII.
    ///
    /// Panics on empty input: an `Encoding` places at least one Cell, and a
    /// test that states none has not chosen a Cell count.
    ///
    #[cfg(test)]
    pub(super) fn literal(text: &str) -> Option<Self> {
        assert!(!text.is_empty(), "an Encoding places at least one Cell");
        text.bytes()
            .map(CellContent::new)
            .collect::<Option<Vec<_>>>()
            .map(Self)
    }

    /// How many Cells this encoding places.
    pub(super) fn len(&self) -> usize {
        self.0.len()
    }

    /// The validated content, for the Portal that admits it.
    pub(super) fn content(&self) -> Vec<CellContent> {
        self.0.clone()
    }
}

impl std::fmt::Display for Encoding {
    /// The characters this encoding places, for a producer's diagnostic.
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        for content in &self.0 {
            write!(f, "{}", content.as_char())?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::Encoding;
    use lang::{Atom, Function, Note, Token};

    #[test]
    fn every_value_the_language_holds_renders_to_two_cells_that_read_back() {
        // Every Atom a value Function can answer spells two Cells, and the
        // reader for its type, a path separate from the spelling, decodes
        // those Cells back to the same Atom.
        let mut atoms: Vec<Atom> = vec![Atom::Bang];
        atoms.extend((u8::MIN..=u8::MAX).map(Atom::Number));
        atoms.extend((0..=0x7F).map(|note| Atom::Note(Note::try_from(note).unwrap())));
        atoms.extend(Function::ALL.iter().copied().map(Atom::Function));

        for atom in atoms {
            let cells = Encoding::render(atom);
            assert_eq!(cells.len(), 2, "{atom:?}");
            let text = cells.to_string();
            let (read, expected) = match atom {
                Atom::Number(_) => (Token::Number.decode(&text), atom),
                Atom::Note(_) => (Token::Note.decode(&text), atom),
                Atom::Bang => (Token::Untyped.decode(&text), atom),
                // The Bang Function's `**` is the Bang a later reader finds,
                // not a Function spelling the Parser looks up.
                Atom::Function(Function::Bang) => (Token::Untyped.decode(&text), Atom::Bang),
                Atom::Function(_) => (Function::try_from(text.as_str()).map(Atom::Function), atom),
            };
            assert_eq!(read.ok(), Some(expected), "{atom:?} rendered as {text:?}");
        }
    }

    #[test]
    fn text_outside_printable_ascii_becomes_no_cells_at_all() {
        // Refused whole: a literal that fails leaves nothing to place half of.
        for refused in ["\0", "\u{1f}", "\u{7f}", "é", "0Aé", "A\0"] {
            assert_eq!(Encoding::literal(refused), None, "{refused:?}");
        }
    }

    #[test]
    fn a_clear_is_the_two_spaces_a_literal_states() {
        assert_eq!(Some(Encoding::clear()), Encoding::literal("  "));
    }

    #[test]
    #[should_panic(expected = "an Encoding places at least one Cell")]
    fn a_literal_of_no_cells_is_a_caller_defect() {
        // Every Atom spells two Cells, so an empty literal is a test that has
        // not chosen a Cell count rather than a value that plans no write.
        let _ = Encoding::literal("");
    }
}
