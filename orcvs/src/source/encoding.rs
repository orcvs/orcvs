//! What an answered value becomes in Cells, before any destination is asked.
//!
//! CONTEXT.md's Spatial Output is "a Function output delivered through a
//! Portal as literal Source encoding", and this is that encoding as a value
//! rather than as a `String`. It is the counterpart to `lang::Token::decode`,
//! which reads Cells back as an Atom according to the receiving operand's
//! declared type: one direction has been a named seam since the Parser was
//! written, and this is the other.
//!
//! It lives beside [`super::portal`] rather than inside it because the two
//! answer different questions. Whether a value can be Cells at all is true of
//! the value wherever it lands and needs no Grid to decide; whether those
//! Cells fit is true of one destination and needs nothing else about the
//! value than how many there are. Handing a `&str` across that seam made the
//! Portal answer both, which is why [`super::portal::PortalError`] carried a
//! content refusal that has nothing to do with a destination.
//!
//! Rendering is not display. Per ADR 0003 a Tick's spatial output *is* program
//! text — "a complete executable Orcvs program and the accumulated output of
//! preceding Ticks" — so these Cells are read back by a later Turn of the same
//! Tick through `Token::decode`, committed as Source, and parsed by the next
//! Tick. A Cell holds exactly one printable ASCII character and nothing else,
//! which is why a value has nowhere to keep its type once it is written and
//! why ADR 0034 has the receiving operand decide the reading.

use lang::{Atom, Value};

use super::CellContent;

///
/// A non-empty run of Source Cells.
///
/// Non-emptiness and printable content are properties of the type rather than
/// checks a caller repeats: [`Encoding::render`] answers [`Rendered::Nothing`]
/// for the two values that plan no write at all, so an `Encoding` that exists
/// places at least one Cell. That is the assertion [`super::portal::Portal`]
/// used to make about a string it was handed, held one step earlier where the
/// invariant is established.
///
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Encoding(Vec<CellContent>);

///
/// What one answered value becomes at a Portal.
///
/// The two values that plan no Cell write are answered here rather than by a
/// caller matching on them, because they are the same rule stated twice in
/// CONTEXT.md: the Absence Marker "is not a language value ... an Expression
/// answering it plans no Cell write", and per ADR 0007 the empty Sequence "is
/// a value holding no Atoms rather than the absence of a value" and plans none
/// either. They differ in kind and agree on effect, which is exactly what one
/// variant with two arms says.
///
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum Rendered {
    /// The value plans no Cell write and reaches no Portal.
    Nothing,
    /// The Cells this value occupies, wherever it is delivered.
    Cells(Encoding),
}

///
/// Why text could not become Source Cells at all.
///
/// One cause, and it is the one a Cell's definition allows: a byte outside
/// printable ASCII. [`Encoding::literal`] reaches it from any text it is
/// handed. [`Encoding::render`] reaches it from no value, because every Atom
/// spells itself in printable ASCII by construction: a Number is two
/// hexadecimal digits, a Bang is `**`, a Function spelling is asserted to be
/// two ASCII Cells at compile time, and a Note's rendering can only fail above
/// `0x7F`, which `Note`'s own `TryFrom<u8>` refuses. `render` still answers the
/// refusal rather than asserting it, because it runs inside Tick planning,
/// where ADR 0028 rules out a panic.
///
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum RenderError {
    /// The rendering contains bytes outside printable ASCII. Carries what was
    /// rendered, so the producer's diagnostic can name it.
    Unrepresentable(String),
}

impl Encoding {
    ///
    /// The Cells one answered value occupies, or the reason it occupies none.
    ///
    /// The whole rendering is decided before an `Encoding` exists, so a value
    /// that cannot be Cells yields nothing to write half of — the same shape
    /// ADR 0004's whole-destination rule gives [`super::portal::Portal`] one
    /// step later.
    ///
    /// A Sequence renders through the same arm a scalar does. Per ADR 0007 it
    /// encodes horizontally as ordinary Atoms "without a privileged
    /// literal-Sequence interpretation", so the only difference reaching this
    /// module is how many Cells come back.
    ///
    pub(super) fn render(value: &Value) -> Result<Rendered, RenderError> {
        let rendering = match value {
            Value::Atom(Atom::Empty) => return Ok(Rendered::Nothing),
            Value::Sequence(sequence) if sequence.is_empty() => return Ok(Rendered::Nothing),
            Value::Atom(atom) => atom.to_string(),
            Value::Sequence(sequence) => sequence.to_string(),
        };
        Self::literal(&rendering).map(Rendered::Cells)
    }

    ///
    /// Source characters no value produced.
    ///
    /// Clearing a Bang's display writes two spaces, which is Source content
    /// rather than an answer, and a test states Source text directly for the
    /// same reason. Both are still held to a Cell's definition, so this is a
    /// second constructor rather than a second rule.
    ///
    /// Panics on empty input: [`Encoding::render`] proves non-emptiness by
    /// answering [`Rendered::Nothing`] instead, and no caller here has a Cell
    /// count it has not already chosen.
    ///
    pub(super) fn literal(text: &str) -> Result<Self, RenderError> {
        assert!(!text.is_empty(), "an Encoding places at least one Cell");
        text.bytes()
            .map(CellContent::new)
            .collect::<Option<Vec<_>>>()
            .map(Self)
            .ok_or_else(|| RenderError::Unrepresentable(text.to_string()))
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
    use super::{Encoding, RenderError, Rendered};
    use lang::{Atom, Function, Note, Sequence, Value};

    fn sequence(atoms: impl IntoIterator<Item = Atom>) -> Value {
        Value::Sequence(Sequence::new(atoms).expect("stated members"))
    }

    #[test]
    fn the_two_values_that_plan_no_write_render_to_nothing() {
        // CONTEXT.md keeps these apart in kind and has them agree on effect:
        // the Absence Marker is the absence of a value, and per ADR 0007 the
        // empty Sequence is a value holding no Atoms. Neither plans a Cell
        // write, and answering both here is what lets an `Encoding` that
        // exists place at least one Cell.
        assert_eq!(
            Encoding::render(&Value::Atom(Atom::Empty)),
            Ok(Rendered::Nothing)
        );
        assert_eq!(
            Encoding::render(&Value::Sequence(Sequence::empty())),
            Ok(Rendered::Nothing)
        );
    }

    #[test]
    fn a_scalar_and_a_sequence_reach_the_same_arm() {
        // ADR 0007 encodes a Sequence horizontally as ordinary Atoms with no
        // privileged literal-Sequence reading, so the only thing that differs
        // between the two widths by the time a destination is asked is how
        // many Cells came back.
        let scalar = Encoding::render(&Value::Atom(Atom::Number(0x0A)));
        let Ok(Rendered::Cells(scalar)) = scalar else {
            panic!("a Number places Cells");
        };
        assert_eq!(scalar.len(), 2);
        assert_eq!(scalar.to_string(), "0A");

        let wide = Encoding::render(&sequence([
            Atom::Number(0x0A),
            Atom::Number(0x0B),
            Atom::Number(0x0C),
        ]));
        let Ok(Rendered::Cells(wide)) = wide else {
            panic!("a Sequence places Cells");
        };
        assert_eq!(wide.len(), 6);
        assert_eq!(wide.to_string(), "0A0B0C");
    }

    #[test]
    fn every_value_the_language_holds_renders_to_cells() {
        // What leaves `render`'s refusal unreachable from a value: every Atom
        // a Sequence admits, and so every Sequence, spells itself in printable
        // ASCII.
        let mut atoms: Vec<Atom> = vec![Atom::Bang];
        atoms.extend((u8::MIN..=u8::MAX).map(Atom::Number));
        atoms.extend((0..=0x7F).map(|note| Atom::Note(Note::try_from(note).unwrap())));
        atoms.extend(
            Function::ALL
                .iter()
                .copied()
                .filter(|function| function.answers_value())
                .map(Atom::Function),
        );

        for atom in &atoms {
            assert!(
                matches!(
                    Encoding::render(&Value::Atom(*atom)),
                    Ok(Rendered::Cells(_))
                ),
                "{atom:?} did not become Source Cells",
            );
        }
        assert!(matches!(
            Encoding::render(&sequence(atoms)),
            Ok(Rendered::Cells(_))
        ));
    }

    #[test]
    fn text_outside_printable_ascii_becomes_no_cells_at_all() {
        // Refused whole: a literal that fails leaves nothing to place half of.
        for refused in ["\0", "\u{1f}", "\u{7f}", "é", "0Aé"] {
            assert!(
                matches!(
                    Encoding::literal(refused),
                    Err(RenderError::Unrepresentable(_))
                ),
                "{refused:?} became Source Cells",
            );
        }
    }

    #[test]
    fn the_error_carries_what_was_refused() {
        // A diagnostic names the text it refused, so the text has to survive
        // the refusal that rejected it.
        assert_eq!(
            Encoding::literal("é"),
            Err(RenderError::Unrepresentable("é".to_string()))
        );
    }

    #[test]
    fn a_literal_holds_the_same_rule_as_a_rendering() {
        // Bang cleanup writes Source content rather than an answer, and a test
        // states Source text for the same reason. A second constructor is not
        // a second rule: a Cell's definition still decides.
        assert_eq!(
            Encoding::literal("  ").map(|encoding| encoding.len()),
            Ok(2)
        );
        assert!(matches!(
            Encoding::literal("A\0"),
            Err(RenderError::Unrepresentable(_))
        ));
    }

    #[test]
    #[should_panic(expected = "an Encoding places at least one Cell")]
    fn a_literal_of_no_cells_is_a_caller_defect() {
        // `render` answers `Nothing` for the values that place no Cell, so an
        // empty literal is a caller that has not chosen a Cell count rather
        // than a value that plans no write.
        let _ = Encoding::literal("");
    }
}
