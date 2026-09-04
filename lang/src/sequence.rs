use std::fmt;
use std::slice;

use crate::{Atom, Error, SequenceError};

/// A flat ordered Sequence of Atoms, produced and consumed as one language
/// value.
///
/// Backed by a `Vec` rather than the `ArrayVec` an Expression's [`Atoms`]
/// uses. `Atoms` is bounded by `EXP_LEN` because one Expression is a bounded
/// run of Source Cells, so the parser knows its capacity up front. A Sequence
/// value's length is genuinely dynamic — a Number Range spans up to 256 Atoms
/// — and ADR 0007 constrains it where it becomes Source, at write time, with
/// the complete-fit Portal rule. Inventing a capacity constant here would cap
/// the value for a reason neither the language nor the Grid states.
///
/// Nesting is impossible by type: [`Atom`] has no Sequence-carrying variant,
/// so the flatness ADR 0007 requires needs no runtime flattening check. The
/// only runtime check is membership, and [`Sequence::new`] is the one place it
/// happens.
///
/// [`Atoms`]: crate::Atoms
#[derive(Clone, Debug, PartialEq)]
pub struct Sequence {
    atoms: Vec<Atom>,
}

impl Sequence {
    /// The one construction point every Sequence funnels through.
    ///
    /// Membership is checked here and nowhere else, so [`Sequence::empty`],
    /// [`Sequence::promote`], and every Sequence Function a later issue adds
    /// inherit exactly one rule and one diagnostic. An `Atom::Activation` is
    /// refused because a Self-Banging Function is a root-only Source effect
    /// rather than an operand, runtime value, or Sequence member; an
    /// `Atom::Empty` is refused because it is the absence marker the
    /// Interpreter answers with when an Expression leaves no value, not an
    /// Atom with a Source encoding.
    pub fn new(atoms: impl IntoIterator<Item = Atom>) -> Result<Self, Error> {
        let atoms: Vec<Atom> = atoms.into_iter().collect();

        for atom in &atoms {
            Self::check_member(*atom)?;
        }

        Ok(Self { atoms })
    }

    /// The empty Sequence, which is a legitimate value rather than an absent
    /// one: it encodes to the empty string and, per ADR 0007, plans no Cell
    /// writes.
    #[inline(always)]
    pub const fn empty() -> Self {
        Self { atoms: Vec::new() }
    }

    /// Promotes one Atom into a singleton Sequence.
    ///
    /// This is the only promotion, so an operand a Sequence Function widens
    /// diagnoses identically to a member supplied directly: a promoted
    /// Activation or Empty is refused by [`Sequence::new`] with the same
    /// diagnostic it would raise inside a longer Sequence.
    #[inline(always)]
    pub fn promote(atom: Atom) -> Result<Self, Error> {
        Self::new([atom])
    }

    #[inline(always)]
    pub fn len(&self) -> usize {
        self.atoms.len()
    }

    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.atoms.is_empty()
    }

    /// The members in order, for the structural Functions issue 03 adds.
    #[inline(always)]
    pub fn atoms(&self) -> &[Atom] {
        &self.atoms
    }

    #[inline(always)]
    pub fn iter(&self) -> slice::Iter<'_, Atom> {
        self.atoms.iter()
    }

    /// Exhaustive over `Atom` rather than admitting the remainder through a
    /// wildcard, so a new variant is classified here, by the compiler, instead
    /// of becoming a legal member by default.
    #[inline(always)]
    fn check_member(atom: Atom) -> Result<(), Error> {
        match atom {
            Atom::Activation(_) | Atom::Empty => Err(SequenceError::Member(atom.into()).into()),
            Atom::Bang | Atom::Char(_) | Atom::Function(_) | Atom::Note(_) | Atom::Number(_) => {
                Ok(())
            }
        }
    }
}

impl<'a> IntoIterator for &'a Sequence {
    type Item = &'a Atom;
    type IntoIter = slice::Iter<'a, Atom>;

    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl fmt::Display for Sequence {
    /// The concatenation of each member's complete `Atom` encoding, with no
    /// separators, so an encoded Sequence is exactly what the same Atoms
    /// written side by side would be. An empty Sequence renders as the empty
    /// string, which is what makes it plan no Cell writes without a special
    /// case in tick planning.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for atom in &self.atoms {
            write!(f, "{atom}")?;
        }
        Ok(())
    }
}

impl From<Sequence> for String {
    /// Delegates to `Display` so the two renderings can never drift apart, the
    /// way `Atom`'s does.
    #[inline(always)]
    fn from(sequence: Sequence) -> Self {
        sequence.to_string()
    }
}

/// What crosses Function evaluation.
///
/// The stack holds this rather than an `Atom` so a Sequence produced by one
/// Function can be consumed by another without first becoming Source writes.
/// That is the whole seam: a Sequence is a language value with a lifetime
/// inside one Expression, and only the Expression's ordinary result reaches a
/// Portal.
#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Atom(Atom),
    Sequence(Sequence),
}

impl From<Atom> for Value {
    #[inline(always)]
    fn from(atom: Atom) -> Self {
        Value::Atom(atom)
    }
}

impl From<Sequence> for Value {
    #[inline(always)]
    fn from(sequence: Sequence) -> Self {
        Value::Sequence(sequence)
    }
}

impl TryFrom<Value> for Sequence {
    type Error = Error;

    /// The seam a Sequence-shaped operand position pops through. It does not
    /// promote: promotion is a Function's decision — Concatenate promotes in
    /// issue 03, Select does not — so the conversion that merely requires a
    /// Sequence diagnoses instead of quietly widening one.
    #[inline(always)]
    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::Sequence(sequence) => Ok(sequence),
            Value::Atom(atom) => Err(SequenceError::ExpectedSequence(atom.into()).into()),
        }
    }
}

#[cfg(test)]
mod test {
    use super::{Sequence, Value};
    use crate::{
        Activation, Atom, Error, Function, Note, SequenceError, to_atom_note, to_atom_num,
    };

    fn note(value: u8) -> Atom {
        Atom::Note(Note::try_from(value).unwrap())
    }

    #[test]
    fn sequence_preserves_atom_order_and_type() {
        let members = [
            Atom::Number(0x0A),
            note(60),
            Atom::Bang,
            Atom::Char('z'),
            Atom::Function(Function::Add),
            Atom::Number(0x0A),
        ];

        let sequence = Sequence::new(members).unwrap();

        // Order, not a set: the repeated Number keeps both positions.
        assert_eq!(sequence.atoms(), members);
        assert_eq!(sequence.len(), 6);

        // A Note member stays a Note rather than collapsing to its Number.
        assert_eq!(sequence.atoms()[1], note(60));
        assert_ne!(sequence.atoms()[1], Atom::Number(60));
    }

    #[test]
    fn promotion_creates_a_singleton_sequence() {
        for atom in [
            Atom::Number(0),
            Atom::Number(0xFF),
            note(0x7F),
            Atom::Bang,
            Atom::Char('z'),
            Atom::Function(Function::RawPlay),
        ] {
            let sequence = Sequence::promote(atom).unwrap();

            assert_eq!(sequence.len(), 1, "{atom:?}");
            assert!(!sequence.is_empty(), "{atom:?}");
            assert_eq!(sequence.atoms(), [atom], "{atom:?}");
            assert_eq!(sequence.to_string(), atom.to_string(), "{atom:?}");
        }
    }

    #[test]
    fn nesting_is_impossible_because_no_atom_variant_carries_a_sequence() {
        // Flatness is a type property, not a runtime one. This match is
        // exhaustive over `Atom`, so a variant that carried a Sequence would
        // fail to compile here rather than needing a flattening pass at
        // construction.
        for atom in [
            Atom::Activation(Activation::North),
            Atom::Bang,
            Atom::Char('z'),
            Atom::Empty,
            Atom::Function(Function::Add),
            note(60),
            Atom::Number(0),
        ] {
            match atom {
                Atom::Activation(_)
                | Atom::Bang
                | Atom::Char(_)
                | Atom::Empty
                | Atom::Function(_)
                | Atom::Note(_)
                | Atom::Number(_) => {}
            }
        }
    }

    #[test]
    fn an_empty_sequence_has_no_members_and_encodes_to_nothing() {
        let sequence = Sequence::empty();

        assert!(sequence.is_empty());
        assert_eq!(sequence.len(), 0);
        assert_eq!(sequence.atoms(), []);
        assert_eq!(sequence.iter().next(), None);
        assert_eq!(sequence.to_string(), "");

        // The two ways of spelling emptiness are the same value.
        assert_eq!(Sequence::new([]).unwrap(), sequence);
    }

    #[test]
    fn encoding_concatenates_the_complete_encoding_of_every_number_member() {
        for number in 0..=u8::MAX {
            let atom = Atom::Number(number);
            let singleton = Sequence::promote(atom).unwrap();
            assert_eq!(singleton.to_string(), atom.to_string());

            let pair = Sequence::new([atom, Atom::Number(0)]).unwrap();
            assert_eq!(pair.to_string(), format!("{number:02X}00"));
            assert_eq!(pair.to_string().len(), 4);
        }
    }

    #[test]
    fn encoding_concatenates_the_complete_encoding_of_every_midi_note_member() {
        for value in 0x00..=0x7F {
            let atom = note(value);
            let singleton = Sequence::promote(atom).unwrap();
            assert_eq!(singleton.to_string(), atom.to_string());

            let pair = Sequence::new([atom, Atom::Number(0)]).unwrap();
            assert_eq!(pair.to_string(), format!("{atom}00"));
        }
    }

    #[test]
    fn sequence_encoding_is_ascii_and_round_trips_the_way_an_atom_does() {
        // `plan_tick` asserts a result is ASCII before writing it into the
        // Source, and re-parses the written Cells on the next Source Snapshot.
        for number in 0..=u8::MAX {
            let rendered = Sequence::promote(Atom::Number(number)).unwrap().to_string();
            assert!(
                rendered.is_ascii(),
                "Number({number}) rendered {rendered:?}"
            );
            assert_eq!(to_atom_num(&rendered).unwrap(), Atom::Number(number));
        }

        for value in 0x00..=0x7F {
            let rendered = Sequence::promote(note(value)).unwrap().to_string();
            assert!(rendered.is_ascii(), "Note({value}) rendered {rendered:?}");
            assert_eq!(to_atom_note(&rendered).unwrap(), note(value));
        }

        let mixed = Sequence::new([Atom::Number(0x3C), note(60), Atom::Bang]).unwrap();
        assert!(mixed.to_string().is_ascii());
        assert_eq!(mixed.to_string(), "3CC4**");
    }

    #[test]
    fn encoding_is_deterministic_for_the_same_members() {
        let members = [Atom::Number(1), note(60), Atom::Bang];
        let sequence = Sequence::new(members).unwrap();

        assert_eq!(
            sequence.to_string(),
            Sequence::new(members).unwrap().to_string()
        );
        assert_eq!(String::from(sequence.clone()), sequence.to_string());
    }

    #[test]
    fn bang_survives_construction_promotion_and_encoding() {
        // A Bang is a permitted member and keeps its own two-Cell encoding
        // rather than being folded into a neighbouring Atom.
        let promoted = Sequence::promote(Atom::Bang).unwrap();
        assert_eq!(promoted.atoms(), [Atom::Bang]);
        assert_eq!(promoted.to_string(), "**");

        let sequence = Sequence::new([Atom::Bang, Atom::Number(0), Atom::Bang]).unwrap();
        assert_eq!(sequence.atoms(), [Atom::Bang, Atom::Number(0), Atom::Bang]);
        assert_eq!(sequence.to_string(), "**00**");
        assert_eq!(sequence.len(), 3);
    }

    #[test]
    fn a_self_banging_function_is_rejected_as_a_member_and_through_promotion() {
        for activation in [
            Activation::North,
            Activation::South,
            Activation::West,
            Activation::East,
        ] {
            let atom = Atom::Activation(activation);

            for result in [
                Sequence::promote(atom),
                Sequence::new([atom]),
                Sequence::new([Atom::Number(0), atom, Atom::Number(1)]),
            ] {
                let error = result.unwrap_err();
                assert!(
                    matches!(&error, Error::Sequence(SequenceError::Member(found))
                        if found == activation.spelling()),
                    "{activation:?} gave {error:?}"
                );
            }
        }
    }

    #[test]
    fn the_absence_marker_is_rejected_as_a_member() {
        for result in [
            Sequence::promote(Atom::Empty),
            Sequence::new([Atom::Number(0), Atom::Empty]),
        ] {
            assert!(matches!(
                result.unwrap_err(),
                Error::Sequence(SequenceError::Member(found)) if found == "_"
            ));
        }
    }

    #[test]
    fn iteration_and_slice_access_report_the_same_members() {
        let sequence = Sequence::new([Atom::Number(1), Atom::Bang, note(60)]).unwrap();

        assert_eq!(
            sequence.iter().copied().collect::<Vec<_>>(),
            sequence.atoms()
        );
        assert_eq!(
            (&sequence).into_iter().copied().collect::<Vec<_>>(),
            sequence.atoms()
        );
        assert_eq!(sequence.iter().count(), sequence.len());
    }

    #[test]
    fn a_value_carries_either_one_atom_or_one_sequence() {
        let sequence = Sequence::new([Atom::Number(1), Atom::Number(2)]).unwrap();

        assert_eq!(Value::from(Atom::Bang), Value::Atom(Atom::Bang));
        assert_eq!(
            Value::from(sequence.clone()),
            Value::Sequence(sequence.clone())
        );

        assert_eq!(
            Sequence::try_from(Value::Sequence(sequence.clone())).unwrap(),
            sequence
        );
    }

    #[test]
    fn an_atom_where_a_sequence_is_required_diagnoses_without_promoting() {
        let error = Sequence::try_from(Value::Atom(Atom::Number(0x0A))).unwrap_err();

        assert!(matches!(
            error,
            Error::Sequence(SequenceError::ExpectedSequence(found)) if found == "0A"
        ));
    }

    #[test]
    fn each_sequence_diagnostic_is_distinguishable_by_variant() {
        // Item 8: Atom, Sequence, member, and incompatible-shape problems are
        // separate variants a caller can match, not one shared message.
        let diagnostics = [
            SequenceError::ExpectedAtom("0001".to_owned()),
            SequenceError::ExpectedSequence("0A".to_owned()),
            SequenceError::Member("^^".to_owned()),
            SequenceError::IncompatibleLengths { left: 2, right: 3 },
            SequenceError::EmptyNotAllowed,
        ];

        for (index, diagnostic) in diagnostics.iter().enumerate() {
            let matched = match diagnostic {
                SequenceError::ExpectedAtom(_) => 0,
                SequenceError::ExpectedSequence(_) => 1,
                SequenceError::Member(_) => 2,
                SequenceError::IncompatibleLengths { .. } => 3,
                SequenceError::EmptyNotAllowed => 4,
            };
            assert_eq!(matched, index, "{diagnostic}");
        }

        assert_eq!(
            SequenceError::ExpectedAtom("0001".to_owned()).to_string(),
            "expected an Atom, found the Sequence \"0001\""
        );
        assert_eq!(
            SequenceError::ExpectedSequence("0A".to_owned()).to_string(),
            "expected a Sequence, found \"0A\""
        );
        assert_eq!(
            SequenceError::Member("^^".to_owned()).to_string(),
            "\"^^\" cannot be a Sequence member"
        );
        assert_eq!(
            SequenceError::IncompatibleLengths { left: 2, right: 3 }.to_string(),
            "incompatible Sequence lengths 2 and 3"
        );
        assert_eq!(
            SequenceError::EmptyNotAllowed.to_string(),
            "expected a non-empty Sequence"
        );
    }
}
